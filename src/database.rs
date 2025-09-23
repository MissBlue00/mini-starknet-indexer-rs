use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use chrono::{DateTime, Utc};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub id: String,
    pub contract_address: String,
    pub event_type: String,
    pub block_number: u64,
    pub transaction_hash: String,
    pub log_index: i32,
    pub timestamp: DateTime<Utc>,
    pub decoded_data: Option<String>,
    pub raw_data: String,
    pub raw_keys: String,
}

#[derive(Debug, Clone)]
pub struct IndexerState {
    #[allow(dead_code)]
    pub id: i32,
    #[allow(dead_code)] 
    pub contract_address: String,
    pub last_synced_block: u64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DeploymentRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub database_url: String,
    pub contract_address: Option<String>,
    pub network: String,
    pub status: String, // "active", "inactive", "error"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON metadata
}

#[derive(Debug, Clone)]
pub struct ApiCallRecord {
    pub id: String,
    pub deployment_id: Option<String>,
    pub user_id: Option<String>,
    pub endpoint: String,
    pub method: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContractQueryRecord {
    pub id: String,
    pub api_call_id: String,
    pub contract_address: String,
    pub query_type: String,
    pub timestamp: DateTime<Utc>,
    pub cost_usdc: f64,
}

#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: String,
    pub deployment_id: String,
    pub key_hash: String, // Hashed version of the API key for storage
    pub name: String,
    pub description: Option<String>,
    pub permissions: String, // JSON string of allowed operations
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub fn normalize_address(address: &str) -> String {
        if !address.starts_with("0x") {
            return address.to_string();
        }
        
        let hex = &address[2..];
        let trimmed = hex.trim_start_matches('0');
        let hex_part = if trimmed.is_empty() { "0" } else { trimmed };
        let padded = format!("{:0>64}", hex_part);
        format!("0x{}", padded)
    }

    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Parse the database URL and create connection options that will create the file if it doesn't exist
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options).await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                contract_address TEXT NOT NULL,
                event_type TEXT NOT NULL,
                block_number INTEGER NOT NULL,
                transaction_hash TEXT NOT NULL,
                log_index INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                decoded_data TEXT,
                raw_data TEXT NOT NULL,
                raw_keys TEXT NOT NULL
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS indexer_state (
                id INTEGER PRIMARY KEY,
                contract_address TEXT UNIQUE NOT NULL,
                last_synced_block INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                database_url TEXT NOT NULL,
                contract_address TEXT,
                network TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            )
            "#
        ).execute(&pool).await?;

        // API usage tracking tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_calls (
                id TEXT PRIMARY KEY,
                deployment_id TEXT,
                user_id TEXT,
                endpoint TEXT NOT NULL,
                method TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                duration_ms INTEGER,
                status_code INTEGER,
                metadata TEXT
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contract_queries (
                id TEXT PRIMARY KEY,
                api_call_id TEXT NOT NULL,
                contract_address TEXT NOT NULL,
                query_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                cost_usdc REAL NOT NULL DEFAULT 0.001,
                FOREIGN KEY (api_call_id) REFERENCES api_calls(id)
            )
            "#
        ).execute(&pool).await?;

        // API keys table for deployment authentication
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                deployment_id TEXT NOT NULL,
                key_hash TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                permissions TEXT NOT NULL DEFAULT '{"read": true, "write": false}',
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_used TEXT,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                FOREIGN KEY (deployment_id) REFERENCES deployments(id) ON DELETE CASCADE
            )
            "#
        ).execute(&pool).await?;

        // Create indexes for fast queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_contract_block ON events(contract_address, block_number)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)")
            .execute(&pool).await?;
            
        // Create indexes for deployments table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployments_network ON deployments(network)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployments_contract_address ON deployments(contract_address)")
            .execute(&pool).await?;
            
        // Create indexes for API keys table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_deployment_id ON api_keys(deployment_id)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active)")
            .execute(&pool).await?;
        
        // Create indexes for API usage tracking
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_calls_deployment_id ON api_calls(deployment_id)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_calls_timestamp ON api_calls(timestamp)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_contract_queries_api_call_id ON contract_queries(api_call_id)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_contract_queries_contract_address ON contract_queries(contract_address)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_contract_queries_timestamp ON contract_queries(timestamp)")
            .execute(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn insert_events(&self, events: &[EventRecord]) -> Result<(), sqlx::Error> {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;
        
        for event in events {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO events 
                (id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&event.id)
            .bind(&event.contract_address)
            .bind(&event.event_type)
            .bind(event.block_number as i64)
            .bind(&event.transaction_hash)
            .bind(event.log_index)
            .bind(event.timestamp.to_rfc3339())
            .bind(&event.decoded_data)
            .bind(&event.raw_data)
            .bind(&event.raw_keys)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_events(
        &self,
        contract_address: &str,
        event_types: Option<&[String]>,
        from_block: Option<u64>,
        to_block: Option<u64>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        // Use a simpler approach with separate queries for different cases
        let rows = match (event_types, from_block, to_block) {
            // No filters except contract address
            (None, None, None) => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? 
                     ORDER BY block_number DESC, log_index DESC LIMIT ? OFFSET ?"
                )
                .bind(&normalized_address)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
            }
            // Only block range filter
            (None, Some(from), Some(to)) => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number >= ? AND block_number <= ? 
                     ORDER BY block_number DESC, log_index DESC LIMIT ? OFFSET ?"
                )
                .bind(&normalized_address)
                .bind(from as i64)
                .bind(to as i64)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
            }
            // Only from block
            (None, Some(from), None) => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number >= ? 
                     ORDER BY block_number DESC, log_index DESC LIMIT ? OFFSET ?"
                )
                .bind(&normalized_address)
                .bind(from as i64)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
            }
            // Only to block
            (None, None, Some(to)) => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number <= ? 
                     ORDER BY block_number DESC, log_index DESC LIMIT ? OFFSET ?"
                )
                .bind(&normalized_address)
                .bind(to as i64)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?
            }
            // For now, handle event type filtering in memory - we can optimize this later
            _ => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? 
                     ORDER BY block_number DESC, log_index DESC"
                )
                .bind(&normalized_address)
                .fetch_all(&self.pool)
                .await?
            }
        };
        
        let mut events = Vec::new();
        for row in rows.into_iter().take(limit as usize).skip(offset as usize) {
            let event_type: String = row.get("event_type");
            
            // Filter by event types if specified
            if let Some(filter_types) = event_types {
                if !filter_types.contains(&event_type) {
                    continue;
                }
            }
            
            events.push(EventRecord {
                id: row.get("id"),
                contract_address: row.get("contract_address"),
                event_type,
                block_number: row.get::<i64, _>("block_number") as u64,
                transaction_hash: row.get("transaction_hash"),
                log_index: row.get("log_index"),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                    .unwrap()
                    .with_timezone(&Utc),
                decoded_data: row.get("decoded_data"),
                raw_data: row.get("raw_data"),
                raw_keys: row.get("raw_keys"),
            });
        }
        
        Ok(events)
    }

    pub async fn get_events_with_ordering(
        &self,
        contract_address: &str,
        event_types: Option<&[String]>,
        from_block: Option<u64>,
        to_block: Option<u64>,
        limit: i32,
        offset: i32,
        order_by: Option<crate::graphql::types::EventOrderBy>,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        
        // Determine the ORDER BY clause based on the order_by parameter
        let order_clause = match order_by {
            Some(crate::graphql::types::EventOrderBy::BlockNumberDesc) | None => "ORDER BY block_number DESC, log_index DESC",
            Some(crate::graphql::types::EventOrderBy::BlockNumberAsc) => "ORDER BY block_number ASC, log_index ASC",
            Some(crate::graphql::types::EventOrderBy::TimestampDesc) => "ORDER BY timestamp DESC, log_index DESC",
            Some(crate::graphql::types::EventOrderBy::TimestampAsc) => "ORDER BY timestamp ASC, log_index ASC",
        };
        
        // Use a simpler approach with separate queries for different cases
        let rows = match (event_types, from_block, to_block) {
            // No filters except contract address
            (None, None, None) => {
                let query = format!(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? 
                     {} LIMIT ? OFFSET ?", order_clause
                );
                sqlx::query(&query)
                    .bind(&normalized_address)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            // Only block range filter
            (None, Some(from), Some(to)) => {
                let query = format!(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number >= ? AND block_number <= ? 
                     {} LIMIT ? OFFSET ?", order_clause
                );
                sqlx::query(&query)
                    .bind(&normalized_address)
                    .bind(from as i64)
                    .bind(to as i64)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            // Only from block
            (None, Some(from), None) => {
                let query = format!(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number >= ? 
                     {} LIMIT ? OFFSET ?", order_clause
                );
                sqlx::query(&query)
                    .bind(&normalized_address)
                    .bind(from as i64)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            // Only to block
            (None, None, Some(to)) => {
                let query = format!(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? AND block_number <= ? 
                     {} LIMIT ? OFFSET ?", order_clause
                );
                sqlx::query(&query)
                    .bind(&normalized_address)
                    .bind(to as i64)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            // For now, handle event type filtering in memory - we can optimize this later
            _ => {
                let query = format!(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? 
                     {}", order_clause
                );
                sqlx::query(&query)
                    .bind(&normalized_address)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        
        let mut events = Vec::new();
        for row in rows.into_iter().take(limit as usize).skip(offset as usize) {
            let event_type: String = row.get("event_type");
            
            // Filter by event types if specified
            if let Some(filter_types) = event_types {
                if !filter_types.contains(&event_type) {
                    continue;
                }
            }
            
            events.push(EventRecord {
                id: row.get("id"),
                contract_address: row.get("contract_address"),
                event_type,
                block_number: row.get::<i64, _>("block_number") as u64,
                transaction_hash: row.get("transaction_hash"),
                log_index: row.get("log_index"),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                    .unwrap()
                    .with_timezone(&Utc),
                decoded_data: row.get("decoded_data"),
                raw_data: row.get("raw_data"),
                raw_keys: row.get("raw_keys"),
            });
        }
        
        Ok(events)
    }

    pub async fn get_indexer_state(&self, contract_address: &str) -> Result<Option<IndexerState>, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        let row = sqlx::query(
            "SELECT id, contract_address, last_synced_block, updated_at FROM indexer_state WHERE contract_address = ?"
        )
        .bind(&normalized_address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(IndexerState {
                id: row.get("id"),
                contract_address: row.get("contract_address"),
                last_synced_block: row.get::<i64, _>("last_synced_block") as u64,
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_indexer_state(&self, contract_address: &str, last_synced_block: u64) -> Result<(), sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO indexer_state (contract_address, last_synced_block, updated_at)
            VALUES (?, ?, ?)
            "#
        )
        .bind(&normalized_address)
        .bind(last_synced_block as i64)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn count_events(&self, contract_address: &str, event_types: Option<&[String]>) -> Result<i64, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        match event_types {
            None => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = ?")
                    .bind(&normalized_address)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count)
            }
            Some(types) if types.is_empty() => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = ?")
                    .bind(&normalized_address)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count)
            }
            Some(types) => {
                // For now, use a simple approach - get all events and count in memory
                // In production, you'd want to optimize this with proper SQL IN clauses
                let events = self.get_events(&normalized_address, Some(types), None, None, i32::MAX, 0).await?;
                Ok(events.len() as i64)
            }
        }
    }

    pub async fn get_events_with_advanced_filters(
        &self,
        contract_address: &str,
        event_types: Option<&[String]>,
        event_keys: Option<&[String]>,
        from_block: Option<u64>,
        to_block: Option<u64>,
        from_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        to_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        transaction_hash: Option<&str>,
        limit: i32,
        offset: i32,
        order_by: Option<crate::graphql::types::EventOrderBy>,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        // For now, use the existing get_events method and filter in memory
        // This can be optimized later with proper dynamic SQL queries
        let mut events = self.get_events_with_ordering(&normalized_address, event_types, from_block, to_block, limit * 2, offset, order_by).await?;
        
        // Apply additional filters in memory
        events.retain(|event| {
            // Filter by event keys if specified
            if let Some(filter_keys) = event_keys {
                let keys: Vec<String> = serde_json::from_str(&event.raw_keys).unwrap_or_default();
                let has_matching_key = filter_keys.iter().any(|filter_key| {
                    keys.iter().any(|key| key.contains(filter_key))
                });
                if !has_matching_key {
                    return false;
                }
            }

            // Filter by timestamp if specified
            if let Some(from_ts) = from_timestamp {
                if event.timestamp < from_ts {
                    return false;
                }
            }
            if let Some(to_ts) = to_timestamp {
                if event.timestamp > to_ts {
                    return false;
                }
            }

            // Filter by transaction hash if specified
            if let Some(tx_hash) = transaction_hash {
                if event.transaction_hash != tx_hash {
                    return false;
                }
            }

            true
        });

        // Apply limit after filtering
        events.truncate(limit as usize);
        
        Ok(events)
    }

    pub async fn get_indexer_stats(&self, contract_address: &str) -> Result<serde_json::Value, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        // Get total events count
        let total_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = ?")
            .bind(&normalized_address)
            .fetch_one(&self.pool)
            .await?;

        // Get events by type
        let event_types = sqlx::query(
            "SELECT event_type, COUNT(*) as count FROM events WHERE contract_address = ? GROUP BY event_type ORDER BY count DESC"
        )
        .bind(&normalized_address)
        .fetch_all(&self.pool)
        .await?;

        let mut type_stats = serde_json::Map::new();
        for row in event_types {
            let event_type: String = row.get("event_type");
            let count: i64 = row.get("count");
            type_stats.insert(event_type, serde_json::Value::Number(count.into()));
        }

        // Get block range
        let block_range = sqlx::query(
            "SELECT MIN(block_number) as min_block, MAX(block_number) as max_block FROM events WHERE contract_address = ?"
        )
        .bind(&normalized_address)
        .fetch_one(&self.pool)
        .await?;

        let min_block: Option<i64> = block_range.get("min_block");
        let max_block: Option<i64> = block_range.get("max_block");

        // Get time range
        let time_range = sqlx::query(
            "SELECT MIN(timestamp) as min_time, MAX(timestamp) as max_time FROM events WHERE contract_address = ?"
        )
        .bind(&normalized_address)
        .fetch_one(&self.pool)
        .await?;

        let min_time: Option<String> = time_range.get("min_time");
        let max_time: Option<String> = time_range.get("max_time");

        Ok(serde_json::json!({
            "contract_address": normalized_address,
            "total_events": total_events,
            "event_types": type_stats,
            "block_range": {
                "min": min_block,
                "max": max_block
            },
            "time_range": {
                "min": min_time,
                "max": max_time
            }
        }))
    }

    pub async fn get_all_contract_addresses(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT DISTINCT contract_address FROM events ORDER BY contract_address"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let addresses: Vec<String> = rows.into_iter()
            .map(|row| row.get("contract_address"))
            .collect();
        
        Ok(addresses)
    }

    #[allow(dead_code)]
    pub async fn get_events_from_multiple_contracts(
        &self,
        contract_addresses: &[String],
        event_types: Option<&[String]>,
        event_keys: Option<&[String]>,
        from_block: Option<u64>,
        to_block: Option<u64>,
        from_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        to_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        transaction_hash: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let mut all_events = Vec::new();
        
        for contract_address in contract_addresses {
            let events = self.get_events_with_advanced_filters(
                contract_address,
                event_types,
                event_keys,
                from_block,
                to_block,
                from_timestamp,
                to_timestamp,
                transaction_hash,
                limit,
                offset,
                None, // Default ordering
            ).await?;
            
            all_events.extend(events);
        }
        
        // Sort by block number and log index (newest first)
        all_events.sort_by(|a, b| {
            b.block_number.cmp(&a.block_number)
                .then(b.log_index.cmp(&a.log_index))
        });
        
        // Apply limit to the combined results
        all_events.truncate(limit as usize);
        
        Ok(all_events)
    }

    // Deployment management methods
    pub async fn create_deployment(&self, deployment: &DeploymentRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO deployments (id, name, description, database_url, contract_address, network, status, created_at, updated_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&deployment.id)
        .bind(&deployment.name)
        .bind(&deployment.description)
        .bind(&deployment.database_url)
        .bind(&deployment.contract_address)
        .bind(&deployment.network)
        .bind(&deployment.status)
        .bind(deployment.created_at.to_rfc3339())
        .bind(deployment.updated_at.to_rfc3339())
        .bind(&deployment.metadata)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_deployment(&self, id: &str) -> Result<Option<DeploymentRecord>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, description, database_url, contract_address, network, status, created_at, updated_at, metadata 
             FROM deployments WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DeploymentRecord {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                database_url: row.get("database_url"),
                contract_address: row.get("contract_address"),
                network: row.get("network"),
                status: row.get("status"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                metadata: row.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_deployments(
        &self,
        status: Option<&str>,
        network: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<DeploymentRecord>, sqlx::Error> {
        let mut query = "SELECT id, name, description, database_url, contract_address, network, status, created_at, updated_at, metadata FROM deployments".to_string();
        let mut conditions = Vec::new();
        
        if status.is_some() {
            conditions.push("status = ?");
        }
        if network.is_some() {
            conditions.push("network = ?");
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        
        let mut sql_query = sqlx::query(&query);
        
        if let Some(s) = status {
            sql_query = sql_query.bind(s);
        }
        if let Some(n) = network {
            sql_query = sql_query.bind(n);
        }
        
        sql_query = sql_query.bind(limit as i64).bind(offset as i64);
        
        let rows = sql_query.fetch_all(&self.pool).await?;
        
        let mut deployments = Vec::new();
        for row in rows {
            deployments.push(DeploymentRecord {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                database_url: row.get("database_url"),
                contract_address: row.get("contract_address"),
                network: row.get("network"),
                status: row.get("status"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                metadata: row.get("metadata"),
            });
        }
        
        Ok(deployments)
    }

    pub async fn update_deployment(&self, id: &str, name: Option<&str>, description: Option<&str>, status: Option<&str>, contract_address: Option<&str>, metadata: Option<&str>) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        let mut updates = Vec::new();
        let mut values: Vec<&str> = Vec::new();
        
        if let Some(n) = name {
            updates.push("name = ?");
            values.push(n);
        }
        if let Some(d) = description {
            updates.push("description = ?");
            values.push(d);
        }
        if let Some(s) = status {
            updates.push("status = ?");
            values.push(s);
        }
        if let Some(c) = contract_address {
            updates.push("contract_address = ?");
            values.push(c);
        }
        if let Some(m) = metadata {
            updates.push("metadata = ?");
            values.push(m);
        }
        
        if updates.is_empty() {
            return Ok(()); // Nothing to update
        }
        
        updates.push("updated_at = ?");
        let now_str = now.to_rfc3339();
        
        let query = format!("UPDATE deployments SET {} WHERE id = ?", updates.join(", "));
        
        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }
        sql_query = sql_query.bind(&now_str).bind(id);
        
        sql_query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_deployment(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM deployments WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_deployments(&self, status: Option<&str>, network: Option<&str>) -> Result<i64, sqlx::Error> {
        let mut query = "SELECT COUNT(*) FROM deployments".to_string();
        let mut conditions = Vec::new();
        
        if status.is_some() {
            conditions.push("status = ?");
        }
        if network.is_some() {
            conditions.push("network = ?");
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        let mut sql_query = sqlx::query_scalar(&query);
        
        if let Some(s) = status {
            sql_query = sql_query.bind(s);
        }
        if let Some(n) = network {
            sql_query = sql_query.bind(n);
        }
        
        let count: i64 = sql_query.fetch_one(&self.pool).await?;
        Ok(count)
    }

    // API Call and Contract Query tracking methods
    
    pub async fn insert_api_call(&self, api_call: &ApiCallRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO api_calls (id, deployment_id, user_id, endpoint, method, timestamp, duration_ms, status_code, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&api_call.id)
        .bind(&api_call.deployment_id)
        .bind(&api_call.user_id)
        .bind(&api_call.endpoint)
        .bind(&api_call.method)
        .bind(api_call.timestamp.to_rfc3339())
        .bind(api_call.duration_ms)
        .bind(api_call.status_code)
        .bind(&api_call.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn insert_contract_query(&self, contract_query: &ContractQueryRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO contract_queries (id, api_call_id, contract_address, query_type, timestamp, cost_usdc)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&contract_query.id)
        .bind(&contract_query.api_call_id)
        .bind(&contract_query.contract_address)
        .bind(&contract_query.query_type)
        .bind(contract_query.timestamp.to_rfc3339())
        .bind(contract_query.cost_usdc)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_api_call_usage_stats(
        &self,
        deployment_id: Option<&str>,
        user_id: Option<&str>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let mut query = String::from(
            "SELECT 
                ac.id as api_call_id,
                ac.endpoint,
                ac.method,
                ac.timestamp,
                ac.duration_ms,
                ac.status_code,
                COUNT(cq.id) as contract_count,
                SUM(cq.cost_usdc) as total_cost_usdc
            FROM api_calls ac
            LEFT JOIN contract_queries cq ON ac.id = cq.api_call_id
            WHERE 1=1"
        );

        let mut conditions = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(dep_id) = deployment_id {
            conditions.push("ac.deployment_id = ?");
            values.push(dep_id.to_string());
        }

        if let Some(uid) = user_id {
            conditions.push("ac.user_id = ?");
            values.push(uid.to_string());
        }

        if let Some(from) = from_date {
            conditions.push("ac.timestamp >= ?");
            values.push(from.to_rfc3339());
        }

        if let Some(to) = to_date {
            conditions.push("ac.timestamp <= ?");
            values.push(to.to_rfc3339());
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" GROUP BY ac.id, ac.endpoint, ac.method, ac.timestamp, ac.duration_ms, ac.status_code");
        query.push_str(" ORDER BY ac.timestamp DESC");

        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(serde_json::json!({
                "api_call_id": row.get::<String, _>("api_call_id"),
                "endpoint": row.get::<String, _>("endpoint"),
                "method": row.get::<String, _>("method"),
                "timestamp": row.get::<String, _>("timestamp"),
                "duration_ms": row.get::<Option<i64>, _>("duration_ms"),
                "status_code": row.get::<Option<i32>, _>("status_code"),
                "contract_count": row.get::<i64, _>("contract_count"),
                "total_cost_usdc": row.get::<Option<f64>, _>("total_cost_usdc").unwrap_or(0.0)
            }));
        }

        Ok(stats)
    }

    pub async fn get_contract_usage_stats(
        &self,
        contract_address: Option<&str>,
        deployment_id: Option<&str>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let mut query = String::from(
            "SELECT 
                cq.contract_address,
                cq.query_type,
                COUNT(cq.id) as query_count,
                SUM(cq.cost_usdc) as total_cost_usdc,
                ac.deployment_id
            FROM contract_queries cq
            LEFT JOIN api_calls ac ON cq.api_call_id = ac.id
            WHERE 1=1"
        );

        let mut conditions = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(contract) = contract_address {
            conditions.push("cq.contract_address = ?");
            values.push(contract.to_string());
        }

        if let Some(dep_id) = deployment_id {
            conditions.push("ac.deployment_id = ?");
            values.push(dep_id.to_string());
        }

        if let Some(from) = from_date {
            conditions.push("cq.timestamp >= ?");
            values.push(from.to_rfc3339());
        }

        if let Some(to) = to_date {
            conditions.push("cq.timestamp <= ?");
            values.push(to.to_rfc3339());
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" GROUP BY cq.contract_address, cq.query_type, ac.deployment_id");
        query.push_str(" ORDER BY total_cost_usdc DESC");

        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(serde_json::json!({
                "contract_address": row.get::<String, _>("contract_address"),
                "query_type": row.get::<String, _>("query_type"),
                "query_count": row.get::<i64, _>("query_count"),
                "total_cost_usdc": row.get::<f64, _>("total_cost_usdc"),
                "deployment_id": row.get::<Option<String>, _>("deployment_id")
            }));
        }

        Ok(stats)
    }

    // API Key Management Methods
    
    /// Create a new API key for a deployment
    pub async fn create_api_key(&self, api_key: &ApiKeyRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, deployment_id, key_hash, name, description, permissions, is_active, last_used, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&api_key.id)
        .bind(&api_key.deployment_id)
        .bind(&api_key.key_hash)
        .bind(&api_key.name)
        .bind(&api_key.description)
        .bind(&api_key.permissions)
        .bind(api_key.is_active)
        .bind(api_key.last_used.map(|dt| dt.to_rfc3339()))
        .bind(api_key.created_at.to_rfc3339())
        .bind(api_key.expires_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get an API key by its hash
    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, deployment_id, key_hash, name, description, permissions, is_active, last_used, created_at, expires_at
            FROM api_keys
            WHERE key_hash = ? AND is_active = 1
            "#
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ApiKeyRecord {
                id: row.get("id"),
                deployment_id: row.get("deployment_id"),
                key_hash: row.get("key_hash"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: row.get("permissions"),
                is_active: row.get("is_active"),
                last_used: row.get::<Option<String>, _>("last_used")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                expires_at: row.get::<Option<String>, _>("expires_at")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all API keys for a deployment
    pub async fn get_api_keys_for_deployment(&self, deployment_id: &str) -> Result<Vec<ApiKeyRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, deployment_id, key_hash, name, description, permissions, is_active, last_used, created_at, expires_at
            FROM api_keys
            WHERE deployment_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(deployment_id)
        .fetch_all(&self.pool)
        .await?;

        let mut api_keys = Vec::new();
        for row in rows {
            api_keys.push(ApiKeyRecord {
                id: row.get("id"),
                deployment_id: row.get("deployment_id"),
                key_hash: row.get("key_hash"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: row.get("permissions"),
                is_active: row.get("is_active"),
                last_used: row.get::<Option<String>, _>("last_used")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                expires_at: row.get::<Option<String>, _>("expires_at")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            });
        }

        Ok(api_keys)
    }

    /// Update API key last used timestamp
    pub async fn update_api_key_last_used(&self, api_key_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used = ?
            WHERE id = ?
            "#
        )
        .bind(Utc::now().to_rfc3339())
        .bind(api_key_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Deactivate an API key
    pub async fn deactivate_api_key(&self, api_key_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = 0
            WHERE id = ?
            "#
        )
        .bind(api_key_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete an API key
    pub async fn delete_api_key(&self, api_key_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM api_keys
            WHERE id = ?
            "#
        )
        .bind(api_key_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get API key by ID (without hash for display purposes)
    pub async fn get_api_key_by_id(&self, api_key_id: &str) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, deployment_id, key_hash, name, description, permissions, is_active, last_used, created_at, expires_at
            FROM api_keys
            WHERE id = ?
            "#
        )
        .bind(api_key_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ApiKeyRecord {
                id: row.get("id"),
                deployment_id: row.get("deployment_id"),
                key_hash: row.get("key_hash"),
                name: row.get("name"),
                description: row.get("description"),
                permissions: row.get("permissions"),
                is_active: row.get("is_active"),
                last_used: row.get::<Option<String>, _>("last_used")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                expires_at: row.get::<Option<String>, _>("expires_at")
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            }))
        } else {
            Ok(None)
        }
    }
}
