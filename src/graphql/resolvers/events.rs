use async_graphql::{Context, Object, Result as GqlResult};
use serde_json::Value;

use crate::starknet::{decode_event_using_abi, get_events, get_contract_abi_string, RpcContext};
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
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>, // currently best-effort filter post-decode
        #[graphql(name = "fromAddress")] _from_address: Option<String>,
        #[graphql(name = "toAddress")] _to_address: Option<String>,
        #[graphql(name = "transactionHash")] _transaction_hash: Option<String>,
        first: Option<i32>,
        after: Option<String>,
    ) -> GqlResult<EventConnection> {
        let rpc = ctx.data::<RpcContext>()?.clone();

        let first = first.unwrap_or(10).clamp(1, 100);
        let continuation = after.as_deref();

        // If no range and not paginating, default to latest
        let (use_from_block, use_to_block) = if from_block.is_none() && to_block.is_none() && after.is_none() {
            (Some("latest".to_string()), Some("latest".to_string()))
        } else {
            (from_block.clone(), to_block.clone())
        };

        let raw = get_events(
            &rpc,
            &contract_address,
            use_from_block.as_deref(),
            use_to_block.as_deref(),
            first as u32,
            continuation,
        ).await.map_err(|e| async_graphql::Error::new(e))?;

        // Fetch ABI
        let abi_str = get_contract_abi_string(&rpc, &contract_address)
            .await
            .unwrap_or_else(|_| "[]".to_string());
        let abi_json: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));

        let mut edges: Vec<EventEdge> = Vec::new();
        let mut end_cursor: Option<String> = None;

        if let Some(result) = raw.get("result") {
            // We'll compute total_count from edges after filtering
            if let Some(events) = result.get("events").and_then(|v| v.as_array()) {
                for (idx, ev) in events.iter().enumerate() {
                    let (ev_type, decoded) = decode_event_using_abi(&abi_json, ev);
                    if let Some(filter_types) = &event_types {
                        if !filter_types.contains(&ev_type) { continue; }
                    }
                    let id = format!("{}:{}", ev.get("transaction_hash").and_then(|v| v.as_str()).unwrap_or(""), idx);
                    let block_number = ev.get("block_number").and_then(|v| v.as_u64()).unwrap_or_default().to_string();
                    let tx_hash = ev.get("transaction_hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let raw_data = ev.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    let raw_keys = ev.get("keys").and_then(|v| v.as_array()).cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    let event = Event {
                        id: id.clone(),
                        contract_address: contract_address.clone(),
                        event_type: ev_type,
                        block_number,
                        transaction_hash: tx_hash,
                        log_index: idx as i32,
                        timestamp: "".to_string(),
                        decoded_data: Some(EventData { json: decoded.to_string() }),
                        raw_data,
                        raw_keys,
                    };
                    edges.push(EventEdge { node: event, cursor: id });
                }
            }
            end_cursor = result.get("continuation_token").and_then(|v| v.as_str()).map(|s| s.to_string());
        }

        let total_count = edges.len() as i32;

        let page_info = PageInfo {
            has_next_page: end_cursor.is_some(),
            has_previous_page: after.is_some(),
            start_cursor: edges.first().map(|e| e.cursor.clone()),
            end_cursor,
        };

        Ok(EventConnection { edges, page_info, total_count })
    }
}

