use async_graphql::{Context, Object, Result as GqlResult};
use serde_json::Value;

use crate::graphql::types::{Contract, EventInput, EventSchema};
use crate::starknet::{get_contract_abi_string, RpcContext};

#[derive(Default)]
pub struct ContractQueryRoot;

#[Object]
impl ContractQueryRoot {
    async fn contract(&self, ctx: &Context<'_>, address: String) -> GqlResult<Option<Contract>> {
        let rpc = ctx.data::<RpcContext>()?.clone();
        let abi_str = match get_contract_abi_string(&rpc, &address).await {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        let abi_val: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));
        let events = parse_event_schemas(&abi_val);
        Ok(Some(Contract {
            address,
            abi: Some(abi_str),
            events,
            name: None,
            verified: true,
        }))
    }

    async fn contracts(&self, ctx: &Context<'_>, addresses: Vec<String>) -> GqlResult<Vec<Contract>> {
        let rpc = ctx.data::<RpcContext>()?.clone();
        let mut out = Vec::new();
        for addr in addresses {
            if let Ok(abi_str) = get_contract_abi_string(&rpc, &addr).await {
                let abi_val: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));
                let events = parse_event_schemas(&abi_val);
                out.push(Contract { address: addr, abi: Some(abi_str), events, name: None, verified: true });
            }
        }
        Ok(out)
    }
}

fn parse_event_schemas(abi: &Value) -> Vec<EventSchema> {
    let mut result = Vec::new();
    if let Some(arr) = abi.as_array() {
        for item in arr {
            if item.get("type").and_then(|v| v.as_str()) == Some("event") {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").split("::").last().unwrap_or("").to_string();
                let inputs: Vec<EventInput> = item
                    .get("members")
                    .and_then(|m| m.as_array())
                    .map(|a| a.iter().map(|m| EventInput {
                        name: m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        r#type: m.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        indexed: m.get("kind").and_then(|v| v.as_str()).map(|k| k == "key").unwrap_or(false),
                    }).collect()).unwrap_or_default();
                result.push(EventSchema { name, r#type: "event".to_string(), inputs, anonymous: false });
            }
        }
    }
    result
}

