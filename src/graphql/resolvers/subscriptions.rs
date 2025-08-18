use async_graphql::{Context, Subscription};
use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

use crate::graphql::types::Event;
use crate::starknet::{decode_event_using_abi, get_contract_abi_string, get_events, RpcContext};

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn event_stream(
        &self,
        ctx: &Context<'_>,
        contract_address: String,
        event_types: Option<Vec<String>>,
    ) -> BoxStream<'static, Event> {
        let rpc = ctx.data_unchecked::<RpcContext>().clone();
        let abi_str = get_contract_abi_string(&rpc, &contract_address).await.unwrap_or_else(|_| "[]".to_string());
        let abi_json: serde_json::Value = serde_json::from_str(&abi_str).unwrap_or(serde_json::Value::Array(vec![]));

        let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(3)))
            .then(move |_| {
                let rpc = rpc.clone();
                let contract_address = contract_address.clone();
                let event_types = event_types.clone();
                let abi_json = abi_json.clone();
                let mut last_tx: Option<String> = None;
                async move {
                    let raw = get_events(&rpc, &contract_address, Some("latest"), Some("latest"), 50, None).await.ok();
                    if let Some(raw) = raw {
                        let mut out: Vec<Event> = Vec::new();
                        if let Some(result) = raw.get("result") {
                            if let Some(events) = result.get("events").and_then(|v| v.as_array()) {
                                for (idx, ev) in events.iter().enumerate() {
                                    let (ev_type, decoded) = decode_event_using_abi(&abi_json, ev);
                                    if let Some(filter) = &event_types {
                                        if !filter.contains(&ev_type) { continue; }
                                    }
                                    let tx_hash = ev.get("transaction_hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    if Some(tx_hash.clone()) == last_tx { continue; }
                                    let id = format!("{}:{}", tx_hash, idx);
                                    let block_number = ev.get("block_number").and_then(|v| v.as_u64()).unwrap_or_default().to_string();
                                    let raw_data = ev.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                    let raw_keys = ev.get("keys").and_then(|v| v.as_array()).cloned().unwrap_or_default().into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                    out.push(Event {
                                        id,
                                        contract_address: contract_address.clone(),
                                        event_type: ev_type,
                                        block_number,
                                        transaction_hash: tx_hash.clone(),
                                        log_index: idx as i32,
                                        timestamp: "".to_string(),
                                        decoded_data: Some(crate::graphql::types::EventData { json: decoded.to_string() }),
                                        raw_data,
                                        raw_keys,
                                    });
                                    last_tx = Some(tx_hash);
                                }
                            }
                        }
                        return out.into_iter();
                    }
                    Vec::new().into_iter()
                }
            })
            .flat_map(stream::iter)
            .boxed();

        stream
    }
}

