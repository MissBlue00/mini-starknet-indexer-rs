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

        // Create indexes for fast queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_contract_block ON events(contract_address, block_number)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)")
            .execute(&pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)")
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
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let normalized_address = Self::normalize_address(contract_address);
        // For now, use the existing get_events method and filter in memory
        // This can be optimized later with proper dynamic SQL queries
        let mut events = self.get_events(&normalized_address, event_types, from_block, to_block, limit * 2, offset).await?;
        
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
}
