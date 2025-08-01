use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
    extract::Path,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::env;
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



async fn get_contract_abi_handler(Path(contract_address): Path<String>) -> Result<String, StatusCode> {
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://starknet-mainnet.public.blastapi.io".to_string());
    
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
        let address = env::var("CONTRACT_ADDRESS").unwrap_or_else(|_| "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d".to_string());
        let chunk_size = 10;
        (address, chunk_size)
    };
    
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://starknet-mainnet.public.blastapi.io".to_string());
    
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
        Ok(abi_json_str) => {
            // First, parse the full RPC response from get_contract_abi_handler
            let full_abi_rpc_response: serde_json::Value = match serde_json::from_str(&abi_json_str) {
                Ok(val) => val,
                Err(_) => {
                    // If parsing fails, return original response with an error message
                    return serde_json::to_string_pretty(&serde_json::json!({
                        "error": "Failed to parse ABI RPC response",
                        "original_response": response,
                        "raw_abi_response": abi_json_str
                    })).unwrap();
                }
            };

            // Now, extract the 'abi' field which is a string, and parse it again into an actual array
            let parsed_abi_array: Option<serde_json::Value> = if let Some(result) = full_abi_rpc_response.get("result") {
                if let Some(abi_str_value) = result.get("abi") {
                    if let Some(abi_str) = abi_str_value.as_str() {
                        serde_json::from_str(abi_str).ok()
                    } else { None }
                } else { None }
            } else { None };

            let abi_for_decoding = parsed_abi_array.as_ref().unwrap_or(&serde_json::Value::Null);

            // Extract events from the response
            let mut decoded_events = Vec::new();
            let mut continuation_key = None;
            
            if let Some(result) = response.get("result") {
                // Extract continuation key if present
                if let Some(continuation) = result.get("continuation_token") {
                    continuation_key = Some(continuation.clone());
                }
                
                if let Some(events_array) = result.get("events").and_then(|e| e.as_array()) {
                    for event in events_array {
                        if let (Some(data), Some(keys), Some(block_number), Some(tx_hash)) = (
                            event.get("data"), event.get("keys"), event.get("block_number"), event.get("transaction_hash")
                        ) {
                            let decoded_event = decode_single_event(
                                data.as_array().unwrap_or(&Vec::new()),
                                keys.as_array().unwrap_or(&Vec::new()),
                                block_number.as_u64().unwrap_or(0),
                                tx_hash.as_str().unwrap_or(""),
                                abi_for_decoding
                            );
                            decoded_events.push(decoded_event);
                        }
                    }
                }
            }
            
            // Build response with decoded events and continuation key
            let mut response_json = serde_json::Map::new();
            response_json.insert("decoded_events".to_string(), serde_json::Value::Array(decoded_events));
            
            if let Some(continuation) = continuation_key {
                response_json.insert("continuation_token".to_string(), continuation);
            }
            
            serde_json::to_string_pretty(&serde_json::Value::Object(response_json)).unwrap()
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
    
    // Only add event_type if we found a real event name from ABI
    if event_name != "Unknown" {
        decoded_data.insert("event_type".to_string(), serde_json::Value::String(event_name));
    }
    decoded_data.insert("block_number".to_string(), serde_json::Value::Number(serde_json::Number::from(block_number)));
    decoded_data.insert("transaction_hash".to_string(), serde_json::Value::String(transaction_hash.to_string()));
    
    // Map data to field names from ABI - only use actual ABI field names
    for (index, value) in data.iter().enumerate() {
        if index < field_names.len() {
            decoded_data.insert(field_names[index].clone(), value.clone());
        }
        // Don't add fallback param_X names - only use actual ABI field names
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
                        
                        // Extract just the event name (last part after the last "::")
                        let event_name = name.split("::").last().unwrap_or(name).to_string();
                        
                        // Return the first event we find (Transfer, Swap, etc.)
                        return (event_name, field_names);
                    }
                }
            }
        }
    }
    
    // No fallback - only use actual ABI field names
    ("Unknown".to_string(), Vec::new())
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Build our application with routes
    let app = Router::new()
        .route("/", post(fetch_starknet_events_handler))
        .route("/test", get(test_json_handler))
        .route("/get-abi/:contract_address", get(get_contract_abi_handler));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}