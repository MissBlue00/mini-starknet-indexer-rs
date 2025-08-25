use async_graphql::{Context, Object, Result as GqlResult};
use std::sync::Arc;

use crate::database::Database;
use crate::graphql::types::{Event, EventConnection, EventData, EventEdge, PageInfo};

#[derive(Default)]
pub struct EventQueryRoot;

#[Object]
impl EventQueryRoot {
    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "contractAddress")] contract_address: String,
        #[graphql(name = "fromBlock")] from_block: Option<String>,
        #[graphql(name = "toBlock")] to_block: Option<String>,
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>,
        #[graphql(name = "fromAddress")] _from_address: Option<String>,
        #[graphql(name = "toAddress")] _to_address: Option<String>,
        #[graphql(name = "transactionHash")] _transaction_hash: Option<String>,
        first: Option<i32>,
        after: Option<String>,
    ) -> GqlResult<EventConnection> {
        let database = ctx.data::<Arc<Database>>()?.clone();
        let limit = first.unwrap_or(10).clamp(1, 100);
        
        // Parse pagination - offset from cursor or default to 0
        let offset = after.as_ref()
            .and_then(|cursor| cursor.parse::<i32>().ok())
            .unwrap_or(0);

        // Parse block range
        let from_block_num = from_block.as_ref()
            .and_then(|s| s.parse::<u64>().ok());
        let to_block_num = to_block.as_ref()
            .and_then(|s| s.parse::<u64>().ok());

        // Query events from database
        let db_events = database.get_events(
            &contract_address,
            event_types.as_ref().map(|v| v.as_slice()),
            from_block_num,
            to_block_num,
            limit,
            offset,
        ).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        // Get total count for pagination
        let total_count = database.count_events(
            &contract_address,
            event_types.as_ref().map(|v| v.as_slice()),
        ).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))? as i32;

        let mut edges: Vec<EventEdge> = Vec::new();
        
        for (idx, db_event) in db_events.iter().enumerate() {
            // Parse raw data back to vec
            let raw_data: Vec<String> = serde_json::from_str(&db_event.raw_data)
                .unwrap_or_default();
            let raw_keys: Vec<String> = serde_json::from_str(&db_event.raw_keys)
                .unwrap_or_default();

            let event = Event {
                id: db_event.id.clone(),
                contract_address: db_event.contract_address.clone(),
                event_type: db_event.event_type.clone(),
                block_number: db_event.block_number.to_string(),
                transaction_hash: db_event.transaction_hash.clone(),
                log_index: db_event.log_index,
                timestamp: db_event.timestamp.to_rfc3339(),
                decoded_data: db_event.decoded_data.as_ref().map(|json| EventData { 
                    json: json.clone() 
                }),
                raw_data,
                raw_keys,
            };
            
            let cursor = (offset + idx as i32 + limit).to_string();
            edges.push(EventEdge { 
                node: event, 
                cursor: cursor.clone(),
            });
        }

        let has_next_page = (offset + limit) < total_count;
        let has_previous_page = offset > 0;
        
        let page_info = PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor: edges.first().map(|e| e.cursor.clone()),
            end_cursor: edges.last().map(|e| e.cursor.clone()),
        };

        Ok(EventConnection { 
            edges, 
            page_info, 
            total_count 
        })
    }
}

