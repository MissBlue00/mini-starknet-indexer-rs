use crate::database::{Database, EventRecord};
use crate::starknet::{get_events, get_contract_abi_string, decode_event_using_abi, get_current_block_number, RpcContext};
use serde_json::Value;
use chrono::Utc;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;

pub struct BlockchainIndexer {
    database: Arc<Database>,
    rpc: RpcContext,
    contract_address: String,
    chunk_size: u64,
}

impl BlockchainIndexer {
    pub fn new(database: Arc<Database>, rpc: RpcContext, contract_address: String) -> Self {
        Self {
            database,
            rpc,
            contract_address,
            chunk_size: 2000, // Process 2000 blocks at a time
        }
    }

    pub async fn start_syncing(&self) {
        println!("🚀 Starting blockchain indexer for contract: {}", self.contract_address);
        
        // Get current network status
        let current_block = match get_current_block_number(&self.rpc).await {
            Ok(block) => block,
            Err(e) => {
                eprintln!("❌ Failed to get current block number: {}", e);
                return;
            }
        };

        // Get last synced block
        let last_synced = match self.database.get_indexer_state(&self.contract_address).await {
            Ok(Some(state)) => state.last_synced_block,
            Ok(None) => 0,
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
        let chunk_size = self.chunk_size;

        // Start continuous sync task immediately for real-time monitoring
        let continuous_sync_task = {
            let database_clone = database.clone();
            let rpc_clone = rpc.clone();
            let contract_address_clone = contract_address.clone();
            
            tokio::spawn(async move {
                let indexer = BlockchainIndexer {
                    database: database_clone,
                    rpc: rpc_clone,
                    contract_address: contract_address_clone,
                    chunk_size,
                };
                indexer.continuous_sync().await;
            })
        };

        // Create a new indexer instance for historical sync
        let historical_indexer = BlockchainIndexer {
            database,
            rpc,
            contract_address,
            chunk_size,
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
        println!("📚 Starting historical data sync...");
        
        // Get the last synced block for this contract
        let last_synced = match self.database.get_indexer_state(&self.contract_address).await? {
            Some(state) => state.last_synced_block,
            None => {
                println!("🆕 New contract - starting from block 0");
                0
            }
        };

        // Get current block number
        let current_block = get_current_block_number(&self.rpc).await
            .map_err(|e| format!("Failed to get current block: {}", e))?;

        println!("📊 Syncing from block {} to {} (total: {} blocks)", 
                last_synced, current_block, current_block - last_synced);

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
            let to_block = std::cmp::min(from_block + self.chunk_size, current_block);
            
            println!("🔄 Processing blocks {} to {} ({:.1}%)", 
                    from_block, to_block,
                    ((from_block as f64 - last_synced as f64) / (current_block as f64 - last_synced as f64)) * 100.0);

            match self.sync_block_range(from_block, to_block, &abi_json).await {
                Ok(events_count) => {
                    total_events += events_count;
                    println!("   ✅ Found {} events in this chunk", events_count);
                    
                    // Update indexer state
                    self.database.update_indexer_state(&self.contract_address, to_block).await?;
                }
                Err(e) => {
                    eprintln!("   ❌ Error processing chunk: {}", e);
                    // Continue with next chunk instead of failing completely
                }
            }

            from_block = to_block + 1;
            
            // Small delay to avoid rate limiting
            sleep(Duration::from_millis(100)).await;
        }

        println!("🎉 Historical sync complete! Indexed {} total events", total_events);
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
                                    println!("   📦 Processed {} new blocks with events found", blocks_synced);
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
            
            // Sleep for 2 seconds, but account for processing time
            let elapsed = start_time.elapsed();
            let sleep_duration = Duration::from_secs(2).saturating_sub(elapsed);
            if sleep_duration > Duration::from_millis(100) {
                sleep(sleep_duration).await;
            }
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
            println!("🎉 FOUND {} NEW EVENTS in blocks {} to {} - updating database!", events_count, last_synced + 1, current_block);
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
        
        // Query events from RPC
        let raw = get_events(
            &self.rpc,
            &self.contract_address,
            Some(&from_block.to_string()),
            Some(&to_block.to_string()),
            1000, // Max events per request
            None, // No continuation for chunk processing
        ).await.map_err(|e| format!("RPC error: {}", e))?;

        let mut events = Vec::new();

        if let Some(result) = raw.get("result") {
            if let Some(events_array) = result.get("events").and_then(|v| v.as_array()) {
                for (idx, ev) in events_array.iter().enumerate() {
                    let (event_type, decoded) = decode_event_using_abi(abi_json, ev);
                    
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
                        contract_address: self.contract_address.clone(),
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
        }

        Ok(events.len())
    }
}

pub async fn start_background_indexer(
    database: Arc<Database>,
    rpc: RpcContext,
    contract_address: String,
) {
    let indexer = BlockchainIndexer::new(database, rpc, contract_address);
    indexer.start_syncing().await;
}
