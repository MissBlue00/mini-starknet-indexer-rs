use async_graphql::{Context, Object, Result as GqlResult};
use std::sync::Arc;

use crate::database::Database;
use crate::graphql::types::{Event, EventConnection, EventData, EventEdge, PageInfo, AdvancedEventQueryArgs, ContractEvents, MultiContractEventsConnection, EventField};

#[derive(Default)]
pub struct EventQueryRoot;

fn convert_decoded_data_to_fields(decoded_json: &str) -> Option<Vec<EventField>> {
    if let Ok(decoded) = serde_json::from_str::<serde_json::Value>(decoded_json) {
        if let Some(obj) = decoded.as_object() {
            let mut fields = Vec::new();
            
            // Check if this is the old format with only _keys (backward compatibility)
            if obj.len() == 1 && obj.contains_key("_keys") {
                if let Some(keys_array) = obj.get("_keys").and_then(|k| k.as_array()) {
                    // For events like U8Event, the structure is typically:
                    // [event_selector, variant_selector, actual_value]
                    if keys_array.len() >= 3 {
                        // Extract the actual value (last element in most cases)
                        if let Some(value_key) = keys_array.last() {
                            if let Some(value_str) = value_key.as_str() {
                                // Try to decode the hex value
                                let decoded_value = if let Ok(num) = u64::from_str_radix(value_str.trim_start_matches("0x"), 16) {
                                    serde_json::json!({
                                        "value": num,
                                        "hex": value_str,
                                        "type": "auto_detected"
                                    })
                                } else {
                                    serde_json::json!({
                                        "raw": value_str,
                                        "type": "felt252"
                                    })
                                };
                                
                                fields.push(EventField {
                                    name: "value".to_string(),
                                    value: value_str.to_string(),
                                    decoded_value: Some(decoded_value),
                                    field_type: "auto_detected".to_string(),
                                });
                            }
                        }
                    }
                }
            } else {
                // New format - process normally
                for (key, value) in obj {
                    // Skip internal fields that start with underscore
                    if key.starts_with('_') {
                        continue;
                    }
                    
                    let (field_type, decoded_value, raw_value) = match value {
                        serde_json::Value::Object(nested) => {
                            // Handle structured decoded values from our improved decoder
                            if let Some(type_str) = nested.get("type").and_then(|t| t.as_str()) {
                                let decoded_val = if nested.contains_key("value") {
                                    nested.get("value").cloned()
                                } else if nested.contains_key("decimal") {
                                    nested.get("decimal").cloned()
                                } else if nested.contains_key("address") {
                                    nested.get("address").cloned()
                                } else {
                                    Some(value.clone())
                                };
                                
                                let raw_val = if let Some(hex) = nested.get("hex") {
                                    hex.as_str().unwrap_or("").to_string()
                                } else if let Some(raw) = nested.get("raw") {
                                    raw.as_str().unwrap_or("").to_string()
                                } else {
                                    value.to_string()
                                };
                                
                                (type_str.to_string(), decoded_val, raw_val)
                            } else {
                                // Fallback for unstructured objects
                                ("object".to_string(), Some(value.clone()), value.to_string())
                            }
                        },
                        _ => {
                            // Simple value
                            ("unknown".to_string(), Some(value.clone()), value.to_string())
                        }
                    };
                    
                    fields.push(EventField {
                        name: key.clone(),
                        value: raw_value,
                        decoded_value,
                        field_type,
                    });
                }
            }
            
            Some(fields)
        } else {
            None
        }
    } else {
        None
    }
}

#[Object]
impl EventQueryRoot {
    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "contractAddress")] contract_address: String,
        #[graphql(name = "fromBlock")] from_block: Option<String>,
        #[graphql(name = "toBlock")] to_block: Option<String>,
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>,
        #[graphql(name = "eventKeys")] event_keys: Option<Vec<String>>,
        #[graphql(name = "fromTimestamp")] from_timestamp: Option<String>,
        #[graphql(name = "toTimestamp")] to_timestamp: Option<String>,
        #[graphql(name = "transactionHash")] transaction_hash: Option<String>,
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

        // Parse timestamp range
        let from_timestamp_dt = from_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let to_timestamp_dt = to_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // Query events from database with advanced filters
        let db_events = database.get_events_with_advanced_filters(
            &contract_address,
            event_types.as_ref().map(|v| v.as_slice()),
            event_keys.as_ref().map(|v| v.as_slice()),
            from_block_num,
            to_block_num,
            from_timestamp_dt,
            to_timestamp_dt,
            transaction_hash.as_deref(),
            limit,
            offset,
        ).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        // Get total count for pagination (simplified for now)
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
                    json: json.clone(),
                    fields: convert_decoded_data_to_fields(json),
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

    async fn events_advanced(
        &self,
        ctx: &Context<'_>,
        args: AdvancedEventQueryArgs,
    ) -> GqlResult<EventConnection> {
        let database = ctx.data::<Arc<Database>>()?.clone();
        
        // Extract filters
        let filters = args.filters.unwrap_or_default();
        let pagination = args.pagination.unwrap_or_default();
        
        let limit = pagination.first.unwrap_or(10).clamp(1, 100);
        let offset = pagination.after.as_ref()
            .and_then(|cursor| cursor.parse::<i32>().ok())
            .unwrap_or(0);

        // Parse block range
        let (from_block_num, to_block_num) = if let Some(block_range) = filters.block_range {
            (
                block_range.from_block.as_ref().and_then(|s| s.parse::<u64>().ok()),
                block_range.to_block.as_ref().and_then(|s| s.parse::<u64>().ok())
            )
        } else {
            (None, None)
        };

        // Parse timestamp range
        let (from_timestamp_dt, to_timestamp_dt) = if let Some(time_range) = filters.time_range {
            (
                time_range.from_timestamp.as_ref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                time_range.to_timestamp.as_ref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            )
        } else {
            (None, None)
        };

        // Query events from database with advanced filters
        let db_events = database.get_events_with_advanced_filters(
            &args.contract_address,
            filters.event_types.as_ref().map(|v| v.as_slice()),
            filters.event_keys.as_ref().map(|v| v.as_slice()),
            from_block_num,
            to_block_num,
            from_timestamp_dt,
            to_timestamp_dt,
            filters.transaction_hash.as_deref(),
            limit,
            offset,
        ).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        // Get total count for pagination (simplified for now)
        let total_count = database.count_events(
            &args.contract_address,
            filters.event_types.as_ref().map(|v| v.as_slice()),
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
                    json: json.clone(),
                    fields: convert_decoded_data_to_fields(json),
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

    async fn indexer_stats(
        &self,
        ctx: &Context<'_>,
        contract_address: String,
    ) -> GqlResult<serde_json::Value> {
        let database = ctx.data::<Arc<Database>>()?.clone();
        
        database.get_indexer_stats(&contract_address)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))
    }

    async fn events_multi_contract(
        &self,
        ctx: &Context<'_>,
        contract_addresses: Vec<String>,
        #[graphql(name = "fromBlock")] from_block: Option<String>,
        #[graphql(name = "toBlock")] to_block: Option<String>,
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>,
        #[graphql(name = "eventKeys")] event_keys: Option<Vec<String>>,
        #[graphql(name = "fromTimestamp")] from_timestamp: Option<String>,
        #[graphql(name = "toTimestamp")] to_timestamp: Option<String>,
        #[graphql(name = "transactionHash")] transaction_hash: Option<String>,
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

        // Parse timestamp range
        let from_timestamp_dt = from_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let to_timestamp_dt = to_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // Query events from all contracts
        let db_events = database.get_events_from_multiple_contracts(
            &contract_addresses,
            event_types.as_ref().map(|v| v.as_slice()),
            event_keys.as_ref().map(|v| v.as_slice()),
            from_block_num,
            to_block_num,
            from_timestamp_dt,
            to_timestamp_dt,
            transaction_hash.as_deref(),
            limit,
            offset,
        ).await.map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        // Calculate total count across all contracts
        let mut total_count: i64 = 0;
        for contract_address in &contract_addresses {
            let count = database.count_events(
                contract_address,
                event_types.as_ref().map(|v| v.as_slice()),
            ).await.map_err(|e| async_graphql::Error::new(format!("Database error for contract {}: {}", contract_address, e)))?;
            total_count += count;
        }

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
                    json: json.clone(),
                    fields: convert_decoded_data_to_fields(json),
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

        let has_next_page = (offset + limit) < (total_count as i32);
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
            total_count: total_count as i32
        })
    }

    async fn events_by_contract(
        &self,
        ctx: &Context<'_>,
        contract_addresses: Vec<String>,
        #[graphql(name = "fromBlock")] from_block: Option<String>,
        #[graphql(name = "toBlock")] to_block: Option<String>,
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>,
        #[graphql(name = "eventKeys")] event_keys: Option<Vec<String>>,
        #[graphql(name = "fromTimestamp")] from_timestamp: Option<String>,
        #[graphql(name = "toTimestamp")] to_timestamp: Option<String>,
        #[graphql(name = "transactionHash")] transaction_hash: Option<String>,
        first: Option<i32>,
        after: Option<String>,
    ) -> GqlResult<MultiContractEventsConnection> {
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

        // Parse timestamp range
        let from_timestamp_dt = from_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let to_timestamp_dt = to_timestamp.as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let mut contract_events: Vec<ContractEvents> = Vec::new();
        let mut total_events: i32 = 0;

        // Query events for each contract separately
        for contract_address in &contract_addresses {
            let db_events = database.get_events_with_advanced_filters(
                contract_address,
                event_types.as_ref().map(|v| v.as_slice()),
                event_keys.as_ref().map(|v| v.as_slice()),
                from_block_num,
                to_block_num,
                from_timestamp_dt,
                to_timestamp_dt,
                transaction_hash.as_deref(),
                limit,
                offset,
            ).await.map_err(|e| async_graphql::Error::new(format!("Database error for contract {}: {}", contract_address, e)))?;

            // Get total count for this contract
            let contract_total_count = database.count_events(
                contract_address,
                event_types.as_ref().map(|v| v.as_slice()),
            ).await.map_err(|e| async_graphql::Error::new(format!("Database error for contract {}: {}", contract_address, e)))? as i32;

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
                        json: json.clone(),
                        fields: convert_decoded_data_to_fields(json),
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

            let has_next_page = (offset + limit) < contract_total_count;
            let has_previous_page = offset > 0;
            
            let page_info = PageInfo {
                has_next_page,
                has_previous_page,
                start_cursor: edges.first().map(|e| e.cursor.clone()),
                end_cursor: edges.last().map(|e| e.cursor.clone()),
            };

            let event_connection = EventConnection { 
                edges, 
                page_info, 
                total_count: contract_total_count
            };

            contract_events.push(ContractEvents {
                contract_address: contract_address.clone(),
                events: event_connection,
            });

            total_events += contract_total_count;
        }

        Ok(MultiContractEventsConnection {
            contracts: contract_events,
            total_contracts: contract_addresses.len() as i32,
            total_events,
        })
    }
}

