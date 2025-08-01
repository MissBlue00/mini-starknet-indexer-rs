use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
    extract::Path,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use reqwest::Client;

#[derive(Serialize, Deserialize)]
struct MockResponse {
    status: String,
    data: String,
}

// Starknet RPC request structures
#[derive(Serialize, Deserialize)]
struct StarknetEventFilter {
    address: String,
    chunk_size: u32,
}

#[derive(Serialize, Deserialize)]
struct StarknetRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<StarknetEventFilter>,
    id: u32,
}

// Starknet RPC response structures
#[derive(Serialize, Deserialize, Debug)]
struct StarknetEvent {
    #[serde(rename = "from_address")]
    from_address: String,
    #[serde(rename = "keys")]
    keys: Vec<String>,
    #[serde(rename = "data")]
    data: Vec<String>,
    #[serde(rename = "block_number")]
    block_number: u64,
    #[serde(rename = "transaction_hash")]
    transaction_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct StarknetEventsResult {
    events: Vec<StarknetEvent>,
    #[serde(rename = "page_number")]
    page_number: u32,
    #[serde(rename = "is_last_page")]
    is_last_page: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct StarknetRpcResponse<T> {
    jsonrpc: String,
    id: u32,
    result: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct StarknetRpcError {
    jsonrpc: String,
    id: u32,
    error: serde_json::Value,
}

// ABI and Event structures
#[derive(Serialize, Deserialize, Debug)]
struct ContractAbi {
    #[serde(rename = "type")]
    abi_type: String,
    name: String,
    inputs: Vec<AbiInput>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AbiInput {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContractClass {
    #[serde(rename = "abi")]
    abi: Vec<ContractAbi>,
    #[serde(rename = "entry_points_by_type")]
    entry_points_by_type: serde_json::Value,
    #[serde(rename = "program")]
    program: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct DecodedEvent {
    event_type: String,
    decoded_data: serde_json::Value,
    block_number: u64,
    transaction_hash: String,
}

async fn root_handler() -> Json<MockResponse> {
    Json(MockResponse {
        status: "ok".to_string(),
        data: "mock".to_string(),
    })
}

async fn get_contract_abi_handler(Path(contract_address): Path<String>) -> Result<String, StatusCode> {
    let rpc_url = "https://starknet-mainnet.public.blastapi.io";
    
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getClassAt",
        "params": [
            "pending",
            contract_address
        ],
        "id": 1
    });

    let client = Client::new();
    
    match client
        .post(rpc_url)
        .json(&rpc_request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        // Return the raw JSON response
                        Ok(text)
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            } else {
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn test_json_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Test endpoint working",
        "status": "success"
    }))
}

async fn fetch_starknet_events_handler(
    request: Option<Json<StarknetEventFilter>>,
) -> Result<String, (StatusCode, String)> {
    // Use provided values or defaults
    let (address, chunk_size) = if let Some(Json(req)) = request {
        (req.address, req.chunk_size)
    } else {
        // Use default values if no JSON body provided
        let address = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"; // USDC contract
        let chunk_size = 10;
        (address.to_string(), chunk_size)
    };
    
    let rpc_url = "https://starknet-mainnet.public.blastapi.io";
    
    // Get events from Starknet RPC
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "starknet_getEvents",
        "params": [{
            "from_block": "latest",
            "to_block": "latest",
            "address": address,
            "chunk_size": chunk_size
        }],
        "id": 1
    });

    let client = Client::new();
    
    match client
        .post(rpc_url)
        .json(&rpc_request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        // Try to decode events using ABI
                        let decoded_response = decode_events_with_abi(&json_response, &address).await;
                        Ok(decoded_response)
                    }
                    Err(e) => {
                        Ok(format!("Error: Failed to parse response - {}", e))
                    }
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Ok(format!("Error: RPC request failed with status {} - {}", status, error_text))
            }
        }
        Err(e) => {
            Ok(format!("Error: Network error - {}", e))
        }
    }
}

async fn decode_events_with_abi(response: &serde_json::Value, contract_address: &str) -> String {
    // Get the ABI for the contract
    let abi_response = get_contract_abi_handler(Path(contract_address.to_string())).await;
    
    match abi_response {
        Ok(abi_json) => {
            // Parse the ABI JSON
            match serde_json::from_str::<serde_json::Value>(&abi_json) {
                Ok(abi_value) => {
                    // Extract events from the response
                    if let Some(result) = response.get("result") {
                        if let Some(events) = result.get("events") {
                            if let Some(events_array) = events.as_array() {
                                let mut decoded_events = Vec::new();
                                
                                for event in events_array {
                                    if let (Some(data), Some(keys), Some(block_number), Some(tx_hash)) = (
                                        event.get("data"),
                                        event.get("keys"),
                                        event.get("block_number"),
                                        event.get("transaction_hash")
                                    ) {
                                        // Try to decode based on event signature
                                        let decoded_event = decode_single_event(
                                            data.as_array().unwrap_or(&Vec::new()),
                                            keys.as_array().unwrap_or(&Vec::new()),
                                            block_number.as_u64().unwrap_or(0),
                                            tx_hash.as_str().unwrap_or(""),
                                            &abi_value
                                        );
                                        
                                        decoded_events.push(decoded_event);
                                    }
                                }
                                
                                return serde_json::to_string_pretty(&serde_json::json!({
                                    "original_response": response,
                                    "decoded_events": decoded_events
                                })).unwrap();
                            }
                        }
                    }
                    
                    // Fallback to original response if decoding fails
                    serde_json::to_string_pretty(response).unwrap()
                }
                Err(_) => serde_json::to_string_pretty(response).unwrap()
            }
        }
        Err(_) => serde_json::to_string_pretty(response).unwrap()
    }
}

fn decode_single_event(
    data: &[serde_json::Value],
    keys: &[serde_json::Value],
    block_number: u64,
    transaction_hash: &str,
    abi: &serde_json::Value
) -> serde_json::Value {
    let mut decoded_data = serde_json::Map::new();
    
    // Try to find the event name and structure from ABI based on the first key (event signature)
    let (event_name, field_names) = if let Some(first_key) = keys.first() {
        if let Some(key_str) = first_key.as_str() {
            find_event_info_from_abi(key_str, abi)
        } else {
            ("Unknown".to_string(), Vec::new())
        }
    } else {
        ("Unknown".to_string(), Vec::new())
    };
    
    decoded_data.insert("event_type".to_string(), serde_json::Value::String(event_name));
    decoded_data.insert("block_number".to_string(), serde_json::Value::Number(serde_json::Number::from(block_number)));
    decoded_data.insert("transaction_hash".to_string(), serde_json::Value::String(transaction_hash.to_string()));
    
    // Map data to field names from ABI
    for (index, value) in data.iter().enumerate() {
        let field_name = if index < field_names.len() {
            field_names[index].clone()
        } else {
            format!("param_{}", index)
        };
        decoded_data.insert(field_name, value.clone());
    }
    
    serde_json::Value::Object(decoded_data)
}

fn find_event_info_from_abi(_event_signature: &str, abi: &serde_json::Value) -> (String, Vec<String>) {
    // Look for events in the ABI - the ABI is directly an array, not nested under "result"
    if let Some(abi_array) = abi.as_array() {
        for abi_item in abi_array {
            if let Some(item_type) = abi_item.get("type").and_then(|t| t.as_str()) {
                if item_type == "event" {
                    if let Some(name) = abi_item.get("name").and_then(|n| n.as_str()) {
                        // Extract field names from the event members
                        let mut field_names = Vec::new();
                        if let Some(members) = abi_item.get("members").and_then(|m| m.as_array()) {
                            for member in members {
                                if let Some(member_name) = member.get("name").and_then(|n| n.as_str()) {
                                    field_names.push(member_name.to_string());
                                }
                            }
                        }
                        
                        // For now, return the first Transfer event we find
                        if name.contains("Transfer") {
                            return (name.to_string(), field_names);
                        }
                    }
                }
            }
        }
    }
    
    // Fallback for Transfer event
    ("Transfer".to_string(), vec!["from".to_string(), "to".to_string(), "value".to_string()])
}

#[tokio::main]
async fn main() {
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/test", get(test_json_handler))
        .route("/fetch-events", post(fetch_starknet_events_handler))
        .route("/get-abi/:contract_address", get(get_contract_abi_handler));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}