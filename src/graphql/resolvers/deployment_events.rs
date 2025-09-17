use async_graphql::{Context, Object, Result as GqlResult, InputObject};

use crate::database::EventRecord;
use crate::graphql::types::{Event, EventConnection, EventEdge, PageInfo, EventOrderBy};
use crate::graphql::deployment_context::DeploymentContext;

/// Input type for deployment-specific event queries
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentEventFilter {
    pub event_types: Option<Vec<String>>,
    pub event_keys: Option<Vec<String>>,
    pub from_block: Option<String>,
    pub to_block: Option<String>,
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
    pub transaction_hash: Option<String>,
}

/// Deployment-specific event query root
#[derive(Default)]
pub struct DeploymentEventQueryRoot;

#[Object]
impl DeploymentEventQueryRoot {
    /// Get events for this specific deployment
    async fn events(
        &self,
        ctx: &Context<'_>,
        filter: Option<DeploymentEventFilter>,
        first: Option<i32>,
        after: Option<String>,
        order_by: Option<EventOrderBy>,
    ) -> GqlResult<EventConnection> {
        let deployment_context = ctx.data::<DeploymentContext>()?;
        let database = deployment_context.get_database();
        
        // Get all contract addresses for this deployment
        let contract_addresses = deployment_context.get_deployment_contract_addresses().await
            .map_err(|e| format!("Failed to get deployment contracts: {}", e))?;
            
        if contract_addresses.is_empty() {
            return Ok(EventConnection {
                edges: vec![],
                page_info: PageInfo {
                    has_next_page: false,
                    has_previous_page: false,
                    start_cursor: None,
                    end_cursor: None,
                },
                total_count: 0,
            });
        }

        let limit = first.unwrap_or(20).min(100);
        let offset = after.as_ref()
            .and_then(|cursor| cursor.parse::<i32>().ok())
            .unwrap_or(0);

        // Parse filters
        let (event_types, event_keys, from_block, to_block, from_timestamp, to_timestamp, transaction_hash) = 
            if let Some(f) = &filter {
                (
                    f.event_types.as_deref(),
                    f.event_keys.as_deref(),
                    f.from_block.as_deref().and_then(|s| s.parse().ok()),
                    f.to_block.as_deref().and_then(|s| s.parse().ok()),
                    f.from_timestamp.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                    f.to_timestamp.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                    f.transaction_hash.as_deref(),
                )
            } else {
                (None, None, None, None, None, None, None)
            };

        // Get events from all contracts in this deployment
        let mut all_events = Vec::new();
        let mut total_count = 0;

        for contract_address in &contract_addresses {
            let events = database.get_events_with_advanced_filters(
                contract_address,
                event_types,
                event_keys,
                from_block,
                to_block,
                from_timestamp,
                to_timestamp,
                transaction_hash,
                limit + 1, // Get one extra to check for next page
                offset,
                order_by,
            ).await.map_err(|e| format!("Failed to fetch events: {}", e))?;

            let count = database.count_events(contract_address, event_types).await
                .map_err(|e| format!("Failed to count events: {}", e))?;

            all_events.extend(events);
            total_count += count;
        }

        // Sort all events by block number and log index (newest first by default)
        all_events.sort_by(|a, b| match order_by.unwrap_or(EventOrderBy::BlockNumberDesc) {
            EventOrderBy::BlockNumberDesc => b.block_number.cmp(&a.block_number)
                .then(b.log_index.cmp(&a.log_index)),
            EventOrderBy::BlockNumberAsc => a.block_number.cmp(&b.block_number)
                .then(a.log_index.cmp(&b.log_index)),
            EventOrderBy::TimestampDesc => b.timestamp.cmp(&a.timestamp)
                .then(b.log_index.cmp(&a.log_index)),
            EventOrderBy::TimestampAsc => a.timestamp.cmp(&b.timestamp)
                .then(a.log_index.cmp(&b.log_index)),
        });

        let has_next_page = all_events.len() > limit as usize;
        let events: Vec<EventRecord> = all_events.into_iter().take(limit as usize).collect();

        let edges: Vec<EventEdge> = events
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                let cursor = (offset + index as i32).to_string();
                EventEdge {
                    node: convert_event_record_to_graphql(record),
                    cursor: cursor.clone(),
                }
            })
            .collect();

        let page_info = PageInfo {
            has_next_page,
            has_previous_page: offset > 0,
            start_cursor: edges.first().map(|e| e.cursor.clone()),
            end_cursor: edges.last().map(|e| e.cursor.clone()),
        };

        Ok(EventConnection {
            edges,
            page_info,
            total_count: total_count as i32,
        })
    }

    /// Get a single event by ID (only if it belongs to this deployment)
    async fn event(&self, ctx: &Context<'_>, id: String) -> GqlResult<Option<Event>> {
        let deployment_context = ctx.data::<DeploymentContext>()?;
        let database = deployment_context.get_database();
        
        // Parse the event ID to extract contract address and check if it belongs to this deployment
        // Event IDs are typically in format: contract_address:block_number:transaction_hash:log_index
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() < 2 {
            return Ok(None);
        }
        
        let contract_address = parts[0];
        
        // Check if this contract belongs to the deployment
        if !deployment_context.is_contract_in_deployment(contract_address).await
            .map_err(|e| format!("Failed to check contract ownership: {}", e))? {
            return Ok(None);
        }

        // Get all events for this contract and find the one with matching ID
        let events = database.get_events(contract_address, None, None, None, 1000, 0).await
            .map_err(|e| format!("Failed to fetch events: {}", e))?;
            
        for event in events {
            if event.id == id {
                return Ok(Some(convert_event_record_to_graphql(event)));
            }
        }
        
        Ok(None)
    }
}

/// Helper function to convert database record to GraphQL type
fn convert_event_record_to_graphql(record: EventRecord) -> Event {
    let data = record.decoded_data.and_then(|d| serde_json::from_str(&d).ok());
    let raw_data: Vec<String> = serde_json::from_str(&record.raw_data).unwrap_or_default();
    let raw_keys: Vec<String> = serde_json::from_str(&record.raw_keys).unwrap_or_default();

    Event {
        id: record.id,
        contract_address: record.contract_address,
        event_type: record.event_type,
        block_number: record.block_number.to_string(),
        transaction_hash: record.transaction_hash,
        log_index: record.log_index,
        timestamp: record.timestamp.to_rfc3339(),
        data,
        raw_data,
        raw_keys,
    }
}
