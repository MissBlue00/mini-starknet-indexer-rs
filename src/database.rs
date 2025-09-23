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
    pub contract_address: Option<String>, // Legacy field - kept for backward compatibility
    pub network: String,
    pub status: String, // "active", "inactive", "error"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON metadata
}

#[derive(Debug, Clone)]
pub struct DeploymentContract {
    pub id: String,
    pub deployment_id: String,
    pub contract_address: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_block: Option<u64>,
    pub status: String, // "active", "inactive", "error"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON metadata for contract-specific config
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployment_contracts (
                id TEXT PRIMARY KEY,
                deployment_id TEXT NOT NULL,
                contract_address TEXT NOT NULL,
                name TEXT,
                description TEXT,
                start_block INTEGER,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (deployment_id) REFERENCES deployments (id) ON DELETE CASCADE
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
        
        // Create indexes for deployment_contracts table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployment_contracts_deployment_id ON deployment_contracts(deployment_id)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployment_contracts_contract_address ON deployment_contracts(contract_address)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_deployment_contracts_status ON deployment_contracts(status)")
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

    // Deployment Contract management methods
    pub async fn create_deployment_contract(&self, contract: &DeploymentContract) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO deployment_contracts (id, deployment_id, contract_address, name, description, start_block, status, created_at, updated_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&contract.id)
        .bind(&contract.deployment_id)
        .bind(&contract.contract_address)
        .bind(&contract.name)
        .bind(&contract.description)
        .bind(contract.start_block.map(|b| b as i64))
        .bind(&contract.status)
        .bind(contract.created_at.to_rfc3339())
        .bind(contract.updated_at.to_rfc3339())
        .bind(&contract.metadata)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_deployment_contracts(&self, deployment_id: &str) -> Result<Vec<DeploymentContract>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, deployment_id, contract_address, name, description, start_block, status, created_at, updated_at, metadata 
             FROM deployment_contracts WHERE deployment_id = ? ORDER BY created_at ASC"
        )
        .bind(deployment_id)
        .fetch_all(&self.pool)
        .await?;

        let mut contracts = Vec::new();
        for row in rows {
            contracts.push(DeploymentContract {
                id: row.get("id"),
                deployment_id: row.get("deployment_id"),
                contract_address: row.get("contract_address"),
                name: row.get("name"),
                description: row.get("description"),
                start_block: row.get::<Option<i64>, _>("start_block").map(|b| b as u64),
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

        Ok(contracts)
    }

    pub async fn get_deployment_contract(&self, contract_id: &str) -> Result<Option<DeploymentContract>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, deployment_id, contract_address, name, description, start_block, status, created_at, updated_at, metadata 
             FROM deployment_contracts WHERE id = ?"
        )
        .bind(contract_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DeploymentContract {
                id: row.get("id"),
                deployment_id: row.get("deployment_id"),
                contract_address: row.get("contract_address"),
                name: row.get("name"),
                description: row.get("description"),
                start_block: row.get::<Option<i64>, _>("start_block").map(|b| b as u64),
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

    pub async fn update_deployment_contract(&self, contract_id: &str, name: Option<&str>, description: Option<&str>, status: Option<&str>, start_block: Option<u64>, metadata: Option<&str>) -> Result<(), sqlx::Error> {
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
        if let Some(m) = metadata {
            updates.push("metadata = ?");
            values.push(m);
        }
        
        if updates.is_empty() && start_block.is_none() {
            return Ok(()); // Nothing to update
        }
        
        if let Some(_sb) = start_block {
            updates.push("start_block = ?");
        }
        
        updates.push("updated_at = ?");
        let now_str = now.to_rfc3339();
        
        let query = format!("UPDATE deployment_contracts SET {} WHERE id = ?", updates.join(", "));
        
        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }
        if let Some(sb) = start_block {
            sql_query = sql_query.bind(sb as i64);
        }
        sql_query = sql_query.bind(&now_str).bind(contract_id);
        
        sql_query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_deployment_contract(&self, contract_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM deployment_contracts WHERE id = ?")
            .bind(contract_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_contracts_by_deployment(&self, deployment_id: &str) -> Result<Vec<DeploymentContract>, sqlx::Error> {
        self.get_deployment_contracts(deployment_id).await
    }

    pub async fn count_deployment_contracts(&self, deployment_id: &str) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deployment_contracts WHERE deployment_id = ?")
            .bind(deployment_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
