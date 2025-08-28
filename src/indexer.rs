use crate::database::{Database, EventRecord};
use crate::starknet::{get_events, get_contract_abi_string, decode_event_using_abi, get_current_block_number, RpcContext};
use crate::realtime::RealtimeEventManager;
use crate::graphql::types::Event;
use serde_json::Value;
use chrono::Utc;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;

#[derive(Clone)]
pub struct ContractConfig {
    pub address: String,
    pub start_block: Option<u64>,
}

#[derive(Clone)]
pub struct IndexerConfig {
    pub start_block: Option<u64>,
    pub chunk_size: u64,
    pub sync_interval: u64,
    pub event_keys: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
    pub batch_mode: bool,
    pub max_retries: u32,
    pub allow_list: Option<Vec<String>>, // Added for multi-contract indexing
    pub contract_configs: Option<Vec<ContractConfig>>, // Per-contract configuration
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            start_block: None,
            chunk_size: 2000,
            sync_interval: 2,
            event_keys: None,
            event_types: None,
            batch_mode: false,
            max_retries: 3,
            allow_list: None,
            contract_configs: None,
        }
    }
}

pub struct BlockchainIndexer {
    database: Arc<Database>,
    rpc: RpcContext,
    contract_address: String,
    config: IndexerConfig,
    realtime_manager: Option<Arc<RealtimeEventManager>>,
}

// New struct for handling multiple contracts
pub struct MultiContractIndexer {
    database: Arc<Database>,
    rpc: RpcContext,
    config: IndexerConfig,
    realtime_manager: Option<Arc<RealtimeEventManager>>,
}

impl MultiContractIndexer {
    pub fn new(database: Arc<Database>, rpc: RpcContext, config: IndexerConfig, realtime_manager: Option<Arc<RealtimeEventManager>>) -> Self {
        Self {
            database,
            rpc,
            config,
            realtime_manager,
        }
    }

    pub async fn start_syncing_all(&self) {
        if let Some(allow_list) = &self.config.allow_list {
            println!("🚀 Starting multi-contract indexer for {} contracts", allow_list.len());
            println!("⏱️  Staggering startup to avoid RPC rate limits...");
            
            // Start individual indexers for each contract with staggered delays
            let mut handles = Vec::new();
            
            for (index, contract_address) in allow_list.iter().enumerate() {
                let database = self.database.clone();
                let rpc = self.rpc.clone();
                let config = self.config.clone();
                let contract = contract_address.clone();
                let realtime_manager = self.realtime_manager.clone();
                
                // Stagger startup by 2 seconds per contract to avoid rate limits
                let delay_seconds = index as u64 * 2;
                
                let handle = tokio::spawn(async move {
                    // Wait before starting this indexer
                    if delay_seconds > 0 {
                        println!("⏳ Starting indexer for {} in {} seconds...", contract, delay_seconds);
                        tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
                    }
                    
                    let indexer = BlockchainIndexer::new(database, rpc, contract, Some(config), realtime_manager);
                    indexer.start_syncing().await;
                });
                
                handles.push(handle);
            }
            
            // Wait for all indexers to complete (they should run indefinitely)
            for handle in handles {
                if let Err(e) = handle.await {
                    eprintln!("❌ Indexer task failed: {}", e);
                }
            }
        } else {
            eprintln!("❌ No allow list configured for multi-contract indexer");
        }
    }
}

impl BlockchainIndexer {
    pub fn new(database: Arc<Database>, rpc: RpcContext, contract_address: String, config: Option<IndexerConfig>, realtime_manager: Option<Arc<RealtimeEventManager>>) -> Self {
        Self {
            database,
            rpc,
            contract_address,
            config: config.unwrap_or_default(),
            realtime_manager,
        }
    }

    pub async fn start_syncing(&self) {
        println!("🚀 Starting blockchain indexer for contract: {}", self.contract_address);
        println!("   📊 Configuration: chunk_size={}, sync_interval={}s, batch_mode={}", 
                self.config.chunk_size, self.config.sync_interval, self.config.batch_mode);
        
        // Get current network status
        let current_block = match get_current_block_number(&self.rpc).await {
            Ok(block) => block,
            Err(e) => {
                eprintln!("❌ Failed to get current block number: {}", e);
                return;
            }
        };

        // Get contract-specific start block or use global start block
        let contract_start_block = if let Some(contract_configs) = &self.config.contract_configs {
            contract_configs.iter()
                .find(|c| c.address == self.contract_address)
                .and_then(|c| c.start_block)
                .or(self.config.start_block)
        } else {
            self.config.start_block
        };

        // Get last synced block or use configured start block
        let last_synced = match self.database.get_indexer_state(&self.contract_address).await {
            Ok(Some(state)) => {
                // If a start block is configured and it's higher than the last synced block, use the start block
                if let Some(start_block) = contract_start_block {
                    if start_block > state.last_synced_block {
                        println!("🔄 Using configured start block {} (higher than last synced {})", start_block, state.last_synced_block);
                        start_block
                    } else {
                        state.last_synced_block
                    }
                } else {
                    state.last_synced_block
                }
            }
            Ok(None) => {
                // If no state exists, use configured start block or default to 0
                let start_block = contract_start_block.unwrap_or(0);
                println!("🆕 New contract - starting from block {}", start_block);
                start_block
            }
            Err(e) => {
                eprintln!("❌ Failed to get indexer state: {}", e);
                return;
            }
        };

        let blocks_behind = current_block.saturating_sub(last_synced);
        
        if blocks_behind > 100 {
            println!("⚠️  INDEXER STATUS: OUT OF SYNC - {} blocks behind", blocks_behind);
            println!("   Syncing from block {} to {} (this may take a while...)", last_synced, current_block);
        } else if blocks_behind > 10 {
            println!("⚠️  INDEXER STATUS: CATCHING UP - {} blocks behind", blocks_behind);
        } else {
            println!("✅ INDEXER STATUS: FULLY SYNCED - only {} blocks behind", blocks_behind);
        }

        // Clone the necessary data for the spawned tasks
        let database = self.database.clone();
        let rpc = self.rpc.clone();
        let contract_address = self.contract_address.clone();
        let config = self.config.clone();

        // Start continuous sync task immediately for real-time monitoring
                let continuous_sync_task = {
            let database_clone = database.clone();
            let rpc_clone = rpc.clone();
            let contract_address_clone = contract_address.clone();
            let config_clone = config.clone();
            let realtime_manager_clone = self.realtime_manager.clone();
            
            tokio::spawn(async move {
                let indexer = BlockchainIndexer {
                    database: database_clone,
                    rpc: rpc_clone,
                    contract_address: contract_address_clone,
                    config: config_clone,
                    realtime_manager: realtime_manager_clone,
                };
                indexer.continuous_sync().await;
            })
        };

        // Create a new indexer instance for historical sync
        let historical_indexer = BlockchainIndexer {
            database,
            rpc,
            contract_address,
            config,
            realtime_manager: self.realtime_manager.clone(),
        };

        // Run historical sync
        if let Err(e) = historical_indexer.sync_historical_data().await {
            eprintln!("❌ Error during historical sync: {}", e);
        }

        // Historical sync is complete, but continuous sync should keep running
        println!("🎉 Historical sync complete! Continuous monitoring will continue...");
        
        // Wait for continuous sync (should run forever)
        if let Err(e) = continuous_sync_task.await {
            eprintln!("❌ Continuous sync task failed: {}", e);
        }
    }

    async fn sync_historical_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("📚 Starting historical data sync for contract: {}", self.contract_address);
        
        // Get contract-specific start block or use global start block
        let contract_start_block = if let Some(contract_configs) = &self.config.contract_configs {
            contract_configs.iter()
                .find(|c| c.address == self.contract_address)
                .and_then(|c| c.start_block)
                .or(self.config.start_block)
        } else {
            self.config.start_block
        };

        // Get the last synced block for this contract
        let last_synced = match self.database.get_indexer_state(&self.contract_address).await? {
            Some(state) => {
                // If a start block is configured and it's higher than the last synced block, use the start block
                if let Some(start_block) = contract_start_block {
                    if start_block > state.last_synced_block {
                        println!("🔄 Using configured start block {} (higher than last synced {})", start_block, state.last_synced_block);
                        start_block
                    } else {
                        state.last_synced_block
                    }
                } else {
                    state.last_synced_block
                }
            }
            None => {
                // If no state exists, use configured start block or default to 0
                let start_block = contract_start_block.unwrap_or(0);
                println!("🆕 New contract - starting from block {}", start_block);
                start_block
            }
        };

        // Get current block number
        let current_block = get_current_block_number(&self.rpc).await
            .map_err(|e| format!("Failed to get current block: {}", e))?;

        println!("📊 Scanning blocks {} to {} for events from contract {} (total: {} blocks)", 
                last_synced, current_block, self.contract_address, current_block - last_synced);

        if last_synced >= current_block {
            println!("✅ Already up to date!");
            return Ok(());
        }

        // Fetch contract ABI once
        let abi_str = get_contract_abi_string(&self.rpc, &self.contract_address)
            .await
            .unwrap_or_else(|_| "[]".to_string());
        let abi_json: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));

        // Process in chunks
        let mut from_block = last_synced;
        let mut total_events = 0;

        while from_block < current_block {
            let to_block = std::cmp::min(from_block + self.config.chunk_size, current_block);
            
            println!("🔄 Scanning blocks {} to {} for contract events ({:.1}%)", 
                    from_block, to_block,
                    ((from_block as f64 - last_synced as f64) / (current_block as f64 - last_synced as f64)) * 100.0);

            match self.sync_block_range(from_block, to_block, &abi_json).await {
                Ok(events_count) => {
                    total_events += events_count;
                    if events_count > 0 {
                        println!("   ✅ Found {} events from contract in this chunk", events_count);
                    } else {
                        println!("   ℹ️  No events from contract in this chunk");
                    }
                    
                    // Update indexer state
                    self.database.update_indexer_state(&self.contract_address, to_block).await?;
                }
                Err(e) => {
                    eprintln!("   ❌ Error processing chunk: {}", e);
                    // Continue with next chunk instead of failing completely
                }
            }

            from_block = to_block + 1;
            
            // Longer delay to avoid rate limiting
            sleep(Duration::from_millis(500)).await;
        }

        if total_events > 0 {
            println!("🎉 Historical sync complete! Found and indexed {} events from contract", total_events);
        } else {
            println!("🎉 Historical sync complete! No events found from contract in scanned blocks");
        }
        Ok(())
    }

    async fn continuous_sync(&self) {
        println!("🔄 Starting continuous sync (checking every 2 seconds)...");
        let mut last_status_update = Instant::now();
        
        loop {
            let start_time = Instant::now();
            
            match self.sync_latest_blocks().await {
                Ok(blocks_synced) => {
                    // Show status update every 60 seconds or when blocks are synced
                    if last_status_update.elapsed() >= Duration::from_secs(60) || blocks_synced > 0 {
                        if let Ok(current_block) = get_current_block_number(&self.rpc).await {
                            if let Ok(Some(state)) = self.database.get_indexer_state(&self.contract_address).await {
                                let blocks_behind = current_block.saturating_sub(state.last_synced_block);
                                
                                if blocks_behind > 100 {
                                    println!("⚠️  INDEXER STATUS: OUT OF SYNC - {} blocks behind (syncing...)", blocks_behind);
                                } else if blocks_behind > 10 {
                                    println!("⚠️  INDEXER STATUS: CATCHING UP - {} blocks behind", blocks_behind);
                                } else if blocks_behind > 0 {
                                    println!("🔄 INDEXER STATUS: NEARLY SYNCED - {} blocks behind", blocks_behind);
                                } else {
                                    println!("✅ INDEXER STATUS: FULLY SYNCED - up to date!");
                                }
                                
                                if blocks_synced > 0 {
                                    println!("   📦 Processed {} new blocks for contract events", blocks_synced);
                                }
                            }
                        }
                        last_status_update = Instant::now();
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error in continuous sync: {}", e);
                }
            }
            
            // Sleep for sync interval, but account for processing time
            let elapsed = start_time.elapsed();
            let sleep_duration = Duration::from_secs(self.config.sync_interval).saturating_sub(elapsed);
            if sleep_duration > Duration::from_millis(100) {
                sleep(sleep_duration).await;
            }
            
            // Add a small delay to prevent overwhelming the RPC endpoint
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    async fn sync_latest_blocks(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Get current state
        let last_synced = match self.database.get_indexer_state(&self.contract_address).await? {
            Some(state) => state.last_synced_block,
            None => return Ok(0), // Should not happen after historical sync
        };

        let current_block = get_current_block_number(&self.rpc).await
            .map_err(|e| format!("Failed to get current block: {}", e))?;

        if current_block <= last_synced {
            return Ok(0); // No new blocks
        }

        let blocks_to_sync = current_block - last_synced;

        // Fetch ABI
        let abi_str = get_contract_abi_string(&self.rpc, &self.contract_address)
            .await
            .unwrap_or_else(|_| "[]".to_string());
        let abi_json: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));

        // Sync new blocks
        let events_count = self.sync_block_range(last_synced + 1, current_block, &abi_json).await?;
        
        if events_count > 0 {
            println!("🎉 FOUND {} NEW EVENTS from contract in blocks {} to {} - updating database!", events_count, last_synced + 1, current_block);
        }

        // Update state
        self.database.update_indexer_state(&self.contract_address, current_block).await?;
        
        Ok(blocks_to_sync)
    }

    async fn sync_block_range(
        &self, 
        from_block: u64, 
        to_block: u64, 
        abi_json: &Value
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        
        // Query events from RPC with retry mechanism
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;
        let mut raw = None;
        
        while attempts < max_attempts {
            match get_events(
                &self.rpc,
                &self.contract_address,
                Some(&from_block.to_string()),
                Some(&to_block.to_string()),
                1000, // Max events per request
                None, // No continuation for chunk processing
            ).await {
                Ok(result) => {
                    raw = Some(result);
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    if attempts < max_attempts {
                        println!("   ⚠️  RPC error (attempt {}/{}): {}. Retrying in 2 seconds...", attempts, max_attempts, e);
                        sleep(Duration::from_secs(2)).await;
                    } else {
                        return Err(format!("RPC error after {} attempts: {}", max_attempts, e).into());
                    }
                }
            }
        }
        
        let raw = raw.ok_or("Failed to get events after all retries")?;

        let mut events = Vec::new();

        if let Some(result) = raw.get("result") {
            if let Some(events_array) = result.get("events").and_then(|v| v.as_array()) {
                for (idx, ev) in events_array.iter().enumerate() {
                    let (event_type, decoded) = decode_event_using_abi(abi_json, ev);
                    
                    // Apply event type filter if configured
                    if let Some(filter_types) = &self.config.event_types {
                        if !filter_types.contains(&event_type) {
                            continue;
                        }
                    }
                    
                    // Apply event keys filter if configured
                    if let Some(filter_keys) = &self.config.event_keys {
                        let keys = ev.get("keys").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                        let keys_str: Vec<String> = keys.iter()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                            .collect();
                        
                        let has_matching_key = filter_keys.iter().any(|filter_key| {
                            keys_str.iter().any(|key| key.contains(filter_key))
                        });
                        if !has_matching_key {
                            continue;
                        }
                    }
                    
                    let tx_hash = ev.get("transaction_hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    let block_number = ev.get("block_number")
                        .and_then(|v| v.as_u64())
                        .unwrap_or_default();
                    
                    let raw_data = ev.get("data")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    
                    let raw_keys = ev.get("keys")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    let event_record = EventRecord {
                        id: format!("{}:{}", tx_hash, idx),
                        contract_address: crate::database::Database::normalize_address(&self.contract_address),
                        event_type,
                        block_number,
                        transaction_hash: tx_hash,
                        log_index: idx as i32,
                        timestamp: Utc::now(), // In production, get actual block timestamp
                        decoded_data: Some(decoded.to_string()),
                        raw_data: serde_json::to_string(&raw_data)?,
                        raw_keys: serde_json::to_string(&raw_keys)?,
                    };

                    events.push(event_record);
                }
            }
        }

        // Insert events into database
        if !events.is_empty() {
            self.database.insert_events(&events).await?;
            
            // Broadcast events to real-time subscribers
            if let Some(realtime_manager) = &self.realtime_manager {
                for event_record in &events {
                    // Convert EventRecord to GraphQL Event type for broadcasting
                    let graphql_event = Event {
                        id: event_record.id.clone(),
                        contract_address: event_record.contract_address.clone(),
                        event_type: event_record.event_type.clone(),
                        block_number: event_record.block_number.to_string(),
                        transaction_hash: event_record.transaction_hash.clone(),
                        log_index: event_record.log_index,
                        timestamp: event_record.timestamp.to_rfc3339(),
                        decoded_data: event_record.decoded_data.as_ref().map(|data| crate::graphql::types::EventData { 
                            json: data.clone(),
                            fields: None, // Will be computed when requested through GraphQL
                        }),
                        raw_data: serde_json::from_str(&event_record.raw_data).unwrap_or_default(),
                        raw_keys: serde_json::from_str(&event_record.raw_keys).unwrap_or_default(),
                    };
                    
                    realtime_manager.broadcast_event(graphql_event).await;
                }
            }
        }

        Ok(events.len())
    }
}

// Single contract background indexer (kept for REST API compatibility)
#[allow(dead_code)]
pub async fn start_background_indexer(
    database: Arc<Database>,
    rpc: RpcContext,
    contract_address: String,
    config: Option<IndexerConfig>,
    realtime_manager: Option<Arc<RealtimeEventManager>>,
) {
    let indexer = BlockchainIndexer::new(database, rpc, contract_address, config, realtime_manager);
    indexer.start_syncing().await;
}

pub async fn start_multi_contract_background_indexer(
    database: Arc<Database>,
    rpc: RpcContext,
    config: IndexerConfig,
    realtime_manager: Option<Arc<RealtimeEventManager>>,
) {
    let indexer = MultiContractIndexer::new(database, rpc, config, realtime_manager);
    indexer.start_syncing_all().await;
}
