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
    let res = ctx
        .http
        .post(&ctx.rpc_url)
        .json(payload)
        .send()
        .await
        .map_err(|e| format!("network error: {}", e))?;
    let status = res.status();
    let body_text = res.text().await.map_err(|e| format!("body error: {}", e))?;
    if !status.is_success() {
        return Err(format!("rpc status {}: {}", status, body_text));
    }
    serde_json::from_str(&body_text).map_err(|e| format!("json parse error: {} | body={} ", e, body_text))
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
    if let Some(f) = from_block { filter["from_block"] = serde_json::Value::String(f.to_string()); }
    if let Some(t) = to_block { filter["to_block"] = serde_json::Value::String(t.to_string()); }
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

    // Find first event in ABI and map members by order (best-effort)
    if let Some(arr) = abi_json.as_array() {
        for item in arr {
            if item.get("type").and_then(|v| v.as_str()) == Some("event") {
                let full_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let name = full_name.split("::").last().unwrap_or(full_name).to_string();
                let members: Vec<String> = item
                    .get("members")
                    .and_then(|m| m.as_array())
                    .map(|a| a.iter().filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let mut decoded = serde_json::Map::new();
                for (idx, member_name) in members.iter().enumerate() {
                    if let Some(val) = data.get(idx) { decoded.insert(member_name.clone(), val.clone()); }
                }
                // include keys for reference
                decoded.insert("_keys".to_string(), serde_json::Value::Array(keys.clone()));
                return (name, serde_json::Value::Object(decoded));
            }
        }
    }
    ("Unknown".to_string(), serde_json::json!({"_keys": keys, "_data": data}))
}

