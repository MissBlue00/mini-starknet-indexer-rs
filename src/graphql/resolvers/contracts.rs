use async_graphql::{Context, Object, Result as GqlResult};
use serde_json::Value;
use std::sync::Arc;

use crate::graphql::types::{Contract, EventInput, EventSchema};
use crate::starknet::{get_contract_abi_string, RpcContext};
use crate::billing::BillingService;
use crate::billing_context::BillingContext;

#[derive(Default)]
pub struct ContractQueryRoot;

#[Object]
impl ContractQueryRoot {
    async fn contract(&self, ctx: &Context<'_>, address: String) -> GqlResult<Option<Contract>> {
        let rpc = ctx.data::<RpcContext>()?.clone();
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Start tracking this API call
        let billing_context = BillingContext::new(
            None, // deployment_id
            None, // user_id
            "/graphql".to_string(),
            "POST".to_string(),
            billing_service.clone(),
        );
        
        let abi_str = match get_contract_abi_string(&rpc, &address).await {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        let abi_val: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));
        let events = parse_event_schemas(&abi_val);
        
        // Track contract query
        if let Err(e) = billing_context.track_contract_query(
            address.clone(),
            "contract_query".to_string(),
            Some(0.001),
        ).await {
            eprintln!("Failed to track contract query: {}", e);
        }
        
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
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Start tracking this API call
        let billing_context = BillingContext::new(
            None, // deployment_id
            None, // user_id
            "/graphql".to_string(),
            "POST".to_string(),
            billing_service.clone(),
        );
        
        let addresses_clone = addresses.clone();
        let mut out = Vec::new();
        for addr in addresses {
            if let Ok(abi_str) = get_contract_abi_string(&rpc, &addr).await {
                let abi_val: Value = serde_json::from_str(&abi_str).unwrap_or(Value::Array(vec![]));
                let events = parse_event_schemas(&abi_val);
                out.push(Contract { address: addr.clone(), abi: Some(abi_str), events, name: None, verified: true });
            }
        }
        
        // Track multiple contract queries
        if let Err(e) = billing_context.track_multiple_contract_queries(
            addresses_clone,
            "contracts_query".to_string(),
            Some(0.001),
        ).await {
            eprintln!("Failed to track contract queries: {}", e);
        }
        
        Ok(out)
    }

    /// Get all deployed contracts (unique contract addresses from events)
    async fn deployments(&self, ctx: &Context<'_>, first: Option<i32>, after: Option<String>) -> GqlResult<Vec<Contract>> {
        let database = ctx.data::<std::sync::Arc<crate::database::Database>>()?.clone();
        let rpc = ctx.data::<RpcContext>()?.clone();
        
        // Get unique contract addresses from the database
        let addresses = database.get_all_contract_addresses().await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        let limit = first.unwrap_or(20).clamp(1, 100) as usize;
        let offset = after.as_ref()
            .and_then(|cursor| cursor.parse::<usize>().ok())
            .unwrap_or(0);
        
        let mut contracts = Vec::new();
        for addr in addresses.into_iter().skip(offset).take(limit) {
            // Get contract stats from database
            let stats = database.get_indexer_stats(&addr).await
                .map_err(|e| async_graphql::Error::new(format!("Database error for {}: {}", addr, e)))?;
            
            // Try to get ABI and events
            let (abi_str, events) = match get_contract_abi_string(&rpc, &addr).await {
                Ok(abi) => {
                    let abi_val: Value = serde_json::from_str(&abi).unwrap_or(Value::Array(vec![]));
                    let events = parse_event_schemas(&abi_val);
                    (Some(abi), events)
                },
                Err(_) => (None, vec![])
            };
            
            // Extract contract name from stats or use a default
            let name = stats.get("contract_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            let verified = abi_str.is_some();
            contracts.push(Contract {
                address: addr,
                name,
                abi: abi_str,
                events,
                verified,
            });
        }
        
        Ok(contracts)
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


