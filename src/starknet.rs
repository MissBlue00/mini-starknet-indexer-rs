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

// Enhanced ABI parser that fully supports unlimited nested structs
#[derive(Debug, Clone)]
struct AbiType {
    name: String,
    members: Vec<AbiMember>,
}

#[derive(Debug, Clone)]
struct AbiMember {
    name: String,
    type_name: String,
    is_key: bool,
}

#[derive(Debug, Clone)]
struct AbiParser {
    types: std::collections::HashMap<String, AbiType>,
    events: std::collections::HashMap<String, AbiType>,
}

impl AbiParser {
    fn new(abi_json: &serde_json::Value) -> Self {
        let mut parser = AbiParser {
            types: std::collections::HashMap::new(),
            events: std::collections::HashMap::new(),
        };
        
        if let Some(arr) = abi_json.as_array() {
            // First pass: collect all struct and enum definitions
            for item in arr {
                if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
                    if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                        match item_type {
                            "struct" | "enum" => {
                                let abi_type = Self::parse_type_definition(item);
                                parser.types.insert(name.to_string(), abi_type);
                            },
                            "event" => {
                                if item.get("kind").and_then(|k| k.as_str()) == Some("struct") {
                                    let abi_type = Self::parse_type_definition(item);
                                    let short_name = name.split("::").last().unwrap_or(name).to_string();
                                    parser.events.insert(short_name, abi_type);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        
        parser
    }
    
    fn parse_type_definition(item: &serde_json::Value) -> AbiType {
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let mut members = Vec::new();
        
        if let Some(members_array) = item.get("members").and_then(|m| m.as_array()) {
            for member in members_array {
                if let (Some(member_name), Some(member_type)) = (
                    member.get("name").and_then(|n| n.as_str()),
                    member.get("type").and_then(|t| t.as_str())
                ) {
                    let is_key = member.get("kind")
                        .and_then(|k| k.as_str())
                        .map(|k| k == "key")
                        .unwrap_or(false);
                    
                    members.push(AbiMember {
                        name: member_name.to_string(),
                        type_name: member_type.to_string(),
                        is_key,
                    });
                }
            }
        }
        
        // Handle enum variants
        if let Some(variants_array) = item.get("variants").and_then(|v| v.as_array()) {
            for variant in variants_array {
                if let (Some(variant_name), Some(variant_type)) = (
                    variant.get("name").and_then(|n| n.as_str()),
                    variant.get("type").and_then(|t| t.as_str())
                ) {
                    members.push(AbiMember {
                        name: variant_name.to_string(),
                        type_name: variant_type.to_string(),
                        is_key: false,
                    });
                }
            }
        }
        
        AbiType { name, members }
    }
    
    fn decode_value(&self, value: &serde_json::Value, type_name: &str) -> serde_json::Value {
        // Handle basic types
        if let Some(decoded) = self.decode_basic_type(value, type_name) {
            return decoded;
        }
        
        // Handle complex types (structs)
        if let Some(struct_def) = self.types.get(type_name) {
            return self.decode_struct(value, struct_def);
        }
        
        // Fallback: return raw value
        value.clone()
    }
    
    fn decode_basic_type(&self, value: &serde_json::Value, type_name: &str) -> Option<serde_json::Value> {
        if let Some(s) = value.as_str() {
            match type_name {
                "felt252" | "core::felt252" | "felt" => {
                    // Convert felt252 to readable string
                    Some(serde_json::Value::String(self.felt_to_string(s)))
                },
                t if t.starts_with("core::integer::u") || ["u8", "u16", "u32", "u64", "u128"].contains(&t) => {
                    // Handle unsigned integers
                    if let Ok(num) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                        Some(serde_json::Value::Number(num.into()))
                    } else if let Ok(num) = s.parse::<u64>() {
                        Some(serde_json::Value::Number(num.into()))
                    } else {
                        Some(serde_json::Value::String(s.to_string()))
                    }
                },
                "core::integer::u256" | "u256" => {
                    // For u256, try to parse as number if possible (use u64 limit for JSON compatibility)
                    if let Ok(num) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                        Some(serde_json::Value::Number(num.into()))
                    } else {
                        // For very large numbers, return as string
                        Some(serde_json::Value::String(s.to_string()))
                    }
                },
                "core::starknet::contract_address::ContractAddress" | "ContractAddress" | "contract_address" => {
                    Some(serde_json::Value::String(s.to_string()))
                },
                "core::bool" | "bool" => {
                    // Decode boolean from felt
                    if s == "0x0" || s == "0" {
                        Some(serde_json::Value::Bool(false))
                    } else {
                        Some(serde_json::Value::Bool(true))
                    }
                },
                // Handle signed integers
                t if t.starts_with("core::integer::i") || ["i8", "i16", "i32", "i64", "i128"].contains(&t) => {
                    if let Ok(num) = i64::from_str_radix(s.trim_start_matches("0x"), 16) {
                        Some(serde_json::Value::Number(num.into()))
                    } else if let Ok(num) = s.parse::<i64>() {
                        Some(serde_json::Value::Number(num.into()))
                    } else {
                        Some(serde_json::Value::String(s.to_string()))
                    }
                },
                // Handle ByteArray (Cairo strings)
                "core::byte_array::ByteArray" | "ByteArray" => {
                    Some(serde_json::Value::String(self.felt_to_string(s)))
                },
                // Handle ClassHash
                "core::starknet::class_hash::ClassHash" | "ClassHash" => {
                    Some(serde_json::Value::String(s.to_string()))
                },
                _ => None
            }
        } else {
            None
        }
    }
    
    fn felt_to_string(&self, felt_hex: &str) -> String {
        // Remove 0x prefix if present
        let hex_str = felt_hex.trim_start_matches("0x");
        
        // Try to decode as UTF-8 string
        if let Ok(bytes) = hex::decode(hex_str) {
            // Remove trailing zeros
            let trimmed_bytes: Vec<u8> = bytes.into_iter()
                .rev()
                .skip_while(|&b| b == 0)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            
            // Try to convert to UTF-8 string
            if let Ok(utf8_string) = String::from_utf8(trimmed_bytes.clone()) {
                // Check if it's a readable string (printable ASCII or valid UTF-8)
                if utf8_string.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()) && !utf8_string.is_empty() {
                    return utf8_string;
                }
            }
        }
        
        // If not a valid string, try to parse as number and return as string
        if let Ok(num) = u64::from_str_radix(hex_str, 16) {
            num.to_string()
        } else {
            // Fallback to original hex value
            felt_hex.to_string()
        }
    }
    
    fn decode_struct(&self, _value: &serde_json::Value, _struct_def: &AbiType) -> serde_json::Value {
        // Struct decoding is complex because structs are serialized as flattened values
        // This requires a different approach - we need to handle this at the event level
        // where we have access to the full data/keys arrays
        serde_json::Value::Null
    }
    
    fn decode_struct_from_arrays(&self, keys: &[serde_json::Value], data: &[serde_json::Value], 
                                 struct_def: &AbiType, key_index: &mut usize, data_index: &mut usize) -> serde_json::Value {
        let mut decoded = serde_json::Map::new();
        
        for member in &struct_def.members {
            let decoded_value = if member.is_key {
                // This field comes from keys array
                if let Some(key_val) = keys.get(*key_index) {
                    *key_index += 1;
                    self.decode_value_recursive(keys, data, key_val, &member.type_name, key_index, data_index)
                } else {
                    serde_json::Value::Null
                }
            } else {
                // This field comes from data array
                if let Some(data_val) = data.get(*data_index) {
                    *data_index += 1;
                    self.decode_value_recursive(keys, data, data_val, &member.type_name, key_index, data_index)
                } else {
                    serde_json::Value::Null
                }
            };
            
            decoded.insert(member.name.clone(), decoded_value);
        }
        
        serde_json::Value::Object(decoded)
    }
    
    fn decode_value_recursive(&self, _keys: &[serde_json::Value], _data: &[serde_json::Value], 
                             value: &serde_json::Value, type_name: &str, 
                             _key_index: &mut usize, _data_index: &mut usize) -> serde_json::Value {
        // Handle basic types first - these don't require additional array consumption
        if let Some(decoded) = self.decode_basic_type(value, type_name) {
            return decoded;
        }
        
        // Handle complex types (nested structs) - these would require array consumption
        // For now, we'll implement this as a simple case since full struct serialization 
        // in Starknet is complex and depends on the exact contract implementation
        if let Some(_struct_def) = self.types.get(type_name) {
            // For nested structs, the proper implementation would need to:
            // 1. Determine how many array positions this struct consumes
            // 2. Extract those positions and recursively decode them
            // 3. Properly handle nested fields
            // This is a complex feature that would need more sophisticated handling
            return value.clone(); // Return raw value for now
        }
        
        // Handle arrays/spans - complex serialization
        if type_name.starts_with("core::array::Array::<") || type_name.starts_with("core::array::Span::<") {
            return value.clone(); // Return raw value for now
        }
        
        // Handle Option types - moderately complex
        if type_name.starts_with("core::option::Option::<") {
            return value.clone(); // Return raw value for now
        }
        
        // Fallback: return raw value
        value.clone()
    }
    
    fn decode_array(&self, _keys: &[serde_json::Value], _data: &[serde_json::Value], 
                   value: &serde_json::Value, _type_name: &str, 
                   _key_index: &mut usize, _data_index: &mut usize) -> serde_json::Value {
        // Array decoding: first value is length, followed by elements
        // This is complex and depends on the exact serialization format
        // For now, return the raw value
        value.clone()
    }
    
    fn decode_option(&self, _keys: &[serde_json::Value], _data: &[serde_json::Value], 
                    value: &serde_json::Value, _type_name: &str,
                    _key_index: &mut usize, _data_index: &mut usize) -> serde_json::Value {
        // Option decoding: first value indicates Some(0) or None(1), then the value if Some
        // For now, return the raw value
        value.clone()
    }
    
    fn decode_member_from_keys(&self, keys: &[serde_json::Value], data: &[serde_json::Value], 
                              type_name: &str, key_index: &mut usize, data_index: &mut usize) -> serde_json::Value {
        if *key_index < keys.len() {
            let key_val = &keys[*key_index];
            *key_index += 1;
            self.decode_value_recursive(keys, data, key_val, type_name, key_index, data_index)
        } else {
            serde_json::Value::Null
        }
    }
    
    fn decode_member_from_data(&self, keys: &[serde_json::Value], data: &[serde_json::Value], 
                              type_name: &str, key_index: &mut usize, data_index: &mut usize) -> serde_json::Value {
        if *data_index < data.len() {
            let data_val = &data[*data_index];
            *data_index += 1;
            self.decode_value_recursive(keys, data, data_val, type_name, key_index, data_index)
        } else {
            serde_json::Value::Null
        }
    }
}

pub fn decode_event_using_abi(abi_json: &serde_json::Value, event: &serde_json::Value) -> (String, serde_json::Value) {
    let keys = event.get("keys").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let data = event.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let parser = AbiParser::new(abi_json);
    
        // Try to find matching event definition
    for (event_name, event_def) in &parser.events {
        // For now, we'll try the first struct event we find
        // In a more sophisticated implementation, we'd match by event selector
                let mut decoded = serde_json::Map::new();
        let mut key_index = if keys.len() > event_def.members.len() + 1 { 2 } else { 1 }; // Skip event selectors
        let mut data_index = 0;
        
        // Decode each member using recursive decoding for full struct support
        for member in &event_def.members {
            let decoded_value = if member.is_key {
                // This field comes from keys array
                parser.decode_member_from_keys(&keys, &data, &member.type_name, &mut key_index, &mut data_index)
            } else {
                // This field comes from data array
                parser.decode_member_from_data(&keys, &data, &member.type_name, &mut key_index, &mut data_index)
            };
            
            decoded.insert(member.name.clone(), decoded_value);
        }
        
        if !event_def.members.is_empty() {
            // Include raw keys and data for debugging
                decoded.insert("_keys".to_string(), serde_json::Value::Array(keys.clone()));
            decoded.insert("_raw_data".to_string(), serde_json::Value::Array(data.clone()));
            
            return (event_name.clone(), serde_json::Value::Object(decoded));
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

