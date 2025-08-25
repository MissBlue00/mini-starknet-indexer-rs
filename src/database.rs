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
        // Use a simpler approach with separate queries for different cases
        let rows = match (event_types, from_block, to_block) {
            // No filters except contract address
            (None, None, None) => {
                sqlx::query(
                    "SELECT id, contract_address, event_type, block_number, transaction_hash, log_index, timestamp, decoded_data, raw_data, raw_keys 
                     FROM events WHERE contract_address = ? 
                     ORDER BY block_number DESC, log_index DESC LIMIT ? OFFSET ?"
                )
                .bind(contract_address)
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
                .bind(contract_address)
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
                .bind(contract_address)
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
                .bind(contract_address)
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
                .bind(contract_address)
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
        let row = sqlx::query(
            "SELECT id, contract_address, last_synced_block, updated_at FROM indexer_state WHERE contract_address = ?"
        )
        .bind(contract_address)
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
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO indexer_state (contract_address, last_synced_block, updated_at)
            VALUES (?, ?, ?)
            "#
        )
        .bind(contract_address)
        .bind(last_synced_block as i64)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn count_events(&self, contract_address: &str, event_types: Option<&[String]>) -> Result<i64, sqlx::Error> {
        match event_types {
            None => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = ?")
                    .bind(contract_address)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count)
            }
            Some(types) if types.is_empty() => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_address = ?")
                    .bind(contract_address)
                    .fetch_one(&self.pool)
                    .await?;
                Ok(count)
            }
            Some(types) => {
                // For now, use a simple approach - get all events and count in memory
                // In production, you'd want to optimize this with proper SQL IN clauses
                let events = self.get_events(contract_address, Some(types), None, None, i32::MAX, 0).await?;
                Ok(events.len() as i64)
            }
        }
    }
}
