use async_graphql::{Context, Object, Result as GqlResult};
use serde_json::Value;

use crate::graphql::types::{Contract, EventSchema, EventInput};
use crate::graphql::deployment_context::DeploymentContext;
use crate::starknet::{get_contract_abi_string, RpcContext};

/// Deployment-specific contract query root
#[derive(Default)]
pub struct DeploymentContractQueryRoot;

#[Object]
impl DeploymentContractQueryRoot {
    /// Get a contract by address (only if it belongs to this deployment)
    async fn contract(&self, ctx: &Context<'_>, address: String) -> GqlResult<Option<Contract>> {
        let deployment_context = ctx.data::<DeploymentContext>()?;
        let rpc = ctx.data::<RpcContext>()?.clone();
        
        // Check if this contract belongs to the deployment
        if !deployment_context.is_contract_in_deployment(&address).await
            .map_err(|e| format!("Failed to check contract ownership: {}", e))? {
            return Ok(None);
        }

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

    /// Get all contracts for this deployment
    async fn contracts(&self, ctx: &Context<'_>) -> GqlResult<Vec<Contract>> {
        let deployment_context = ctx.data::<DeploymentContext>()?;
        let rpc = ctx.data::<RpcContext>()?.clone();
        let database = deployment_context.get_database();
        
        // Get contract addresses for this deployment
        let addresses = deployment_context.get_deployment_contract_addresses().await
            .map_err(|e| format!("Failed to get deployment contracts: {}", e))?;
        
        let mut contracts = Vec::new();
        
        for addr in addresses {
            // Get contract stats from database
            let stats = database.get_indexer_stats(&addr).await
                .map_err(|e| format!("Database error for {}: {}", addr, e))?;
            
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

    /// Get deployment information
    async fn deployment_info(&self, ctx: &Context<'_>) -> GqlResult<DeploymentInfo> {
        let deployment_context = ctx.data::<DeploymentContext>()?;
        let database = deployment_context.get_database();
        
        let contract_addresses = deployment_context.get_deployment_contract_addresses().await
            .map_err(|e| format!("Failed to get deployment contracts: {}", e))?;
            
        let mut total_events = 0i64;
        for addr in &contract_addresses {
            let count = database.count_events(addr, None).await
                .map_err(|e| format!("Failed to count events for {}: {}", addr, e))?;
            total_events += count;
        }
        
        Ok(DeploymentInfo {
            id: deployment_context.deployment.id.clone(),
            name: deployment_context.deployment.name.clone(),
            description: deployment_context.deployment.description.clone(),
            network: deployment_context.deployment.network.clone(),
            contract_count: contract_addresses.len() as i32,
            total_events: total_events as i32,
            status: deployment_context.deployment.status.clone(),
            created_at: deployment_context.deployment.created_at.to_rfc3339(),
        })
    }
}

/// Deployment information type
#[derive(async_graphql::SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub network: String,
    pub contract_count: i32,
    pub total_events: i32,
    pub status: String,
    pub created_at: String,
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
