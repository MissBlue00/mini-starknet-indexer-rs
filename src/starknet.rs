use reqwest::Client;
use serde_json::Value;
use std::env;

#[derive(Clone)]
pub struct RpcContext {
    pub rpc_url: String,
    pub http: Client,
}

impl RpcContext {
    pub fn from_env() -> Self {
        let rpc_url = env::var("RPC_URL")
            .unwrap_or_else(|_| "https://starknet-mainnet.public.blastapi.io".to_string());
        Self {
            rpc_url,
            http: Client::new(),
        }
    }
}

pub async fn rpc_call(ctx: &RpcContext, payload: &Value) -> Result<Value, String> {
    let max_retries = 3;
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        let res = ctx
            .http
            .post(&ctx.rpc_url)
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("network error: {}", e))?;
        
        let status = res.status();
        let body_text = res.text().await.map_err(|e| format!("body error: {}", e))?;
        
        // Check if we got a rate limit error
        if status == 429 {
            if attempt <= max_retries {
                let delay = std::cmp::min(2u64.pow(attempt as u32), 30); // Exponential backoff, max 30 seconds
                eprintln!("⚠️  Rate limited (attempt {}/{}), waiting {} seconds...", attempt, max_retries, delay);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                continue;
            } else {
                return Err(format!("rpc status {} after {} retries: {}", status, max_retries, body_text));
            }
        }
        
        if !status.is_success() {
            return Err(format!("rpc status {}: {}", status, body_text));
        }
        
        return serde_json::from_str(&body_text).map_err(|e| format!("json parse error: {} | body={} ", e, body_text));
    }
}

pub async fn get_contract_class(ctx: &RpcContext, address: &str) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getClassAt",
        "params": ["pending", address],
        "id": 1
    });
    rpc_call(ctx, &payload).await
}

pub async fn get_contract_abi_string(ctx: &RpcContext, address: &str) -> Result<String, String> {
    let class = get_contract_class(ctx, address).await?;
    let abi_str = class
        .get("result")
        .and_then(|r| r.get("abi"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| "missing abi in class".to_string())?;
    Ok(abi_str.to_string())
}

pub async fn get_current_block_number(ctx: &RpcContext) -> Result<u64, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_blockNumber",
        "params": [],
        "id": 1
    });
    
    let response = rpc_call(ctx, &payload).await?;
    response
        .get("result")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "Failed to parse block number".to_string())
}



pub async fn get_events(
    ctx: &RpcContext,
    address: &str,
    from_block: Option<&str>,
    to_block: Option<&str>,
    chunk_size: u32,
    continuation: Option<&str>,
) -> Result<Value, String> {
    let mut filter = serde_json::json!({
        "address": address,
        "chunk_size": chunk_size,
    });
    if let Some(f) = from_block { 
        filter["from_block"] = if f == "latest" || f == "pending" {
            serde_json::Value::String(f.to_string())
        } else {
            serde_json::json!({"block_number": f.parse::<u64>().unwrap_or(0)})
        };
    }
    if let Some(t) = to_block { 
        filter["to_block"] = if t == "latest" || t == "pending" {
            serde_json::Value::String(t.to_string())
        } else {
            serde_json::json!({"block_number": t.parse::<u64>().unwrap_or(0)})
        };
    }
    if let Some(c) = continuation { filter["continuation_token"] = serde_json::Value::String(c.to_string()); }

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getEvents",
        "params": [filter],
        "id": 1
    });
    rpc_call(ctx, &payload).await
}

#[allow(dead_code)]
pub async fn get_block_with_tx_hashes_by_number(ctx: &RpcContext, block_number: u64) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getBlockWithTxHashes",
        "params": [{"block_number": block_number}],
        "id": 1
    });
    rpc_call(ctx, &payload).await
}

#[allow(dead_code)]
pub async fn get_transaction_by_hash(ctx: &RpcContext, tx_hash: &str) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getTransactionByHash",
        "params": [tx_hash],
        "id": 1
    });
    rpc_call(ctx, &payload).await
}

pub fn decode_event_using_abi(abi_json: &serde_json::Value, event: &serde_json::Value) -> (String, serde_json::Value) {
    let keys = event.get("keys").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let data = event.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    if let Some(arr) = abi_json.as_array() {
        // Look for all event definitions - both direct events and nested ones
        for item in arr {
            if item.get("type").and_then(|v| v.as_str()) == Some("event") {
                let full_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let name = full_name.split("::").last().unwrap_or(full_name).to_string();
                
                // Check if this is a simple struct event
                if item.get("kind").and_then(|k| k.as_str()) == Some("struct") {
                    let members = item
                        .get("members")
                        .and_then(|m| m.as_array())
                        .map(|members_array| {
                            members_array.iter()
                                .filter_map(|member| {
                                    let member_name = member.get("name")?.as_str()?;
                                    let member_type = member.get("type")?.as_str()?;
                                    let is_key = member.get("kind")
                                        .and_then(|k| k.as_str())
                                        .map(|k| k == "key")
                                        .unwrap_or(false);
                                    Some((member_name.to_string(), member_type.to_string(), is_key))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    // For struct events, we need to account for nested event selectors
                    // In enum events, the keys array typically contains:
                    // [main_event_selector, variant_selector, ...actual_field_values]
                    let mut decoded = serde_json::Map::new();
                    let mut key_index = if keys.len() > members.len() + 1 { 2 } else { 1 }; // Skip event selectors
                    let mut data_index = 0;

                    // Decode members according to their types
                    for (member_name, member_type, is_key) in &members {
                        let decoded_value = if *is_key {
                            // This field comes from keys array
                            if let Some(key_val) = keys.get(key_index) {
                                key_index += 1;
                                decode_cairo_value(key_val, member_type)
                            } else {
                                serde_json::Value::Null
                            }
                        } else {
                            // This field comes from data array
                            if let Some(data_val) = data.get(data_index) {
                                data_index += 1;
                                decode_cairo_value(data_val, member_type)
                            } else {
                                serde_json::Value::Null
                            }
                        };
                        
                        decoded.insert(member_name.clone(), decoded_value);
                    }

                    // Return if we have any members to decode
                    if !members.is_empty() {
                        // Include raw keys and data for debugging
                        decoded.insert("_keys".to_string(), serde_json::Value::Array(keys.clone()));
                        decoded.insert("_raw_data".to_string(), serde_json::Value::Array(data.clone()));
                        
                        return (name, serde_json::Value::Object(decoded));
                    }
                }
            }
        }
    }
    
    // Fallback: return raw data with field names if possible
    let mut decoded = serde_json::Map::new();
    for (idx, val) in data.iter().enumerate() {
        decoded.insert(format!("field_{}", idx), val.clone());
    }
    decoded.insert("_keys".to_string(), serde_json::Value::Array(keys.clone()));
    decoded.insert("_raw_data".to_string(), serde_json::Value::Array(data.clone()));
    
    ("Unknown".to_string(), serde_json::Value::Object(decoded))
}

fn decode_cairo_value(value: &serde_json::Value, cairo_type: &str) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            match cairo_type {
                "felt252" | "felt" => {
                    // Try to decode felt as different formats
                    if let Ok(num) = s.parse::<u64>() {
                        // If it's a reasonable number, show both hex and decimal
                        serde_json::json!({
                            "hex": s,
                            "decimal": num.to_string(),
                            "type": "felt252"
                        })
                    } else {
                        // Keep as hex string
                        serde_json::json!({
                            "hex": s,
                            "type": "felt252"
                        })
                    }
                },
                "u8" | "u16" | "u32" | "u64" | "u128" | "u256" => {
                    // Parse as number if possible
                    if let Ok(num) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                        serde_json::json!({
                            "value": num,
                            "hex": s,
                            "type": cairo_type
                        })
                    } else if let Ok(num) = s.parse::<u64>() {
                        serde_json::json!({
                            "value": num,
                            "type": cairo_type
                        })
                    } else {
                        serde_json::json!({
                            "raw": s,
                            "type": cairo_type
                        })
                    }
                },
                "ContractAddress" | "contract_address" => {
                    serde_json::json!({
                        "address": s,
                        "type": "ContractAddress"
                    })
                },
                _ => {
                    // Default: return the raw value with type info
                    serde_json::json!({
                        "raw": s,
                        "type": cairo_type
                    })
                }
            }
        },
        _ => value.clone()
    }
}

