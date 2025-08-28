use axum::{
    routing::{get, post, get_service, post_service},
    Router,
    Json,
    http::StatusCode,
    extract::Path,
    response::Html,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::env;
use reqwest::Client;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use clap::Parser;
use url::Url;

mod graphql;
mod starknet;
mod database;
mod indexer;

#[derive(Parser, Debug)]
#[command(name = "mini-starknet-indexer", version, about = "Mini Starknet Indexer with REST and GraphQL APIs", long_about = None)]
struct CliArgs {
    #[arg(long, value_name = "URL", value_parser = parse_url, help = "RPC URL for Starknet JSON-RPC (overrides RPC_URL env)")]
    rpc_url: Option<String>,

    #[arg(long, value_name = "ADDRESS", value_parser = parse_contract_address, help = "Default contract address for REST fetch (overrides CONTRACT_ADDRESS env)")]
    contract_address: Option<String>,

    #[arg(long, value_name = "BLOCK", help = "Start indexing from this block number (overrides START_BLOCK env)")]
    start_block: Option<u64>,

    #[arg(long, value_name = "SIZE", default_value = "2000", help = "Number of blocks to process in each chunk")]
    chunk_size: Option<u64>,

    #[arg(long, value_name = "SECONDS", default_value = "2", help = "Interval between sync checks in seconds")]
    sync_interval: Option<u64>,

    #[arg(long, value_name = "KEYS", help = "Comma-separated list of event keys to filter for")]
    event_keys: Option<String>,

    #[arg(long, value_name = "TYPES", help = "Comma-separated list of event types to filter for")]
    event_types: Option<String>,

    #[arg(long, help = "Enable batch processing for better performance")]
    batch_mode: bool,

    #[arg(long, value_name = "RETRIES", default_value = "3", help = "Number of retries for failed RPC calls")]
    max_retries: Option<u32>,
}

fn parse_url(s: &str) -> Result<String, String> {
    Url::parse(s)
        .map(|_| s.to_string())
        .map_err(|e| format!("invalid URL: {}", e))
}

fn parse_contract_address(s: &str) -> Result<String, String> {
    if !s.starts_with("0x") {
        return Err("contract address must start with 0x".to_string());
    }
    let hex = &s[2..];
    if hex.is_empty() {
        return Err("contract address hex part is empty".to_string());
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("contract address must be hexadecimal".to_string());
    }
    
    // Normalize the address by removing leading zeros and ensuring consistent format
    let normalized = normalize_starknet_address(s);
    Ok(normalized)
}

fn normalize_starknet_address(address: &str) -> String {
    // Remove 0x prefix
    let hex = &address[2..];
    
    // Remove leading zeros
    let trimmed = hex.trim_start_matches('0');
    
    // If all zeros were removed, keep at least one zero
    let hex_part = if trimmed.is_empty() { "0" } else { trimmed };
    
    // Ensure the address is 64 characters (32 bytes) by padding with leading zeros
    let padded = format!("{:0>64}", hex_part);
    
    // Return with 0x prefix
    format!("0x{}", padded)
}

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
    // Parse CLI args and override env if provided
    let cli = CliArgs::parse();
    if let Some(url) = cli.rpc_url.as_deref() {
        env::set_var("RPC_URL", url);
    }
    if let Some(addr) = cli.contract_address.as_deref() {
        env::set_var("CONTRACT_ADDRESS", addr);
    }
    
    // Create indexer configuration from CLI args
    let mut indexer_config = crate::indexer::IndexerConfig::default();
    
    // Override with CLI values if provided
    if let Some(start_block) = cli.start_block {
        indexer_config.start_block = Some(start_block);
        println!("🔧 Using start block: {}", start_block);
    }
    if let Some(chunk_size) = cli.chunk_size {
        indexer_config.chunk_size = chunk_size;
        println!("🔧 Using chunk size: {}", chunk_size);
    }
    if let Some(sync_interval) = cli.sync_interval {
        indexer_config.sync_interval = sync_interval;
        println!("🔧 Using sync interval: {}s", sync_interval);
    }
    if let Some(event_keys) = cli.event_keys {
        indexer_config.event_keys = Some(event_keys.split(',').map(|s| s.trim().to_string()).collect());
        println!("🔧 Using event keys filter: {:?}", indexer_config.event_keys);
    }
    if let Some(event_types) = cli.event_types {
        indexer_config.event_types = Some(event_types.split(',').map(|s| s.trim().to_string()).collect());
        println!("🔧 Using event types filter: {:?}", indexer_config.event_types);
    }
    if cli.batch_mode {
        indexer_config.batch_mode = true;
        println!("🔧 Batch mode enabled");
    }
    if let Some(max_retries) = cli.max_retries {
        indexer_config.max_retries = max_retries;
        println!("🔧 Using max retries: {}", max_retries);
    }
    
    // Initialize database
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:events.db".to_string());
    let database = std::sync::Arc::new(
        crate::database::Database::new(&database_url)
            .await
            .expect("Failed to initialize database")
    );
    
    // Build GraphQL schema with database
    let rpc = crate::starknet::RpcContext::from_env();
    let schema = crate::graphql::schema::build_schema(rpc.clone(), database.clone());

    // Build our application with routes
    let app = Router::new()
        .route("/", post(fetch_starknet_events_handler))
        .route("/test", get(test_json_handler))
        .route("/get-abi/:contract_address", get(get_contract_abi_handler))
        .route("/sync-status", get(sync_status_handler))
        .route("/stats/:contract_address", get(indexer_stats_handler))
        // GraphQL: POST for queries/mutations, GET for GraphiQL interface, separate WS endpoint for subscriptions
        .route("/graphql", post_service(GraphQL::new(schema.clone())))
        .route("/graphql", get(graphiql_handler))
        .route("/ws", get_service(GraphQLSubscription::new(schema.clone())))
        // GraphiQL UI (alternative endpoint)
        .route("/graphiql", get(graphiql_handler))
        .with_state((database.clone(), rpc.clone()));

    // Start background indexer and server concurrently
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🌐 Starting GraphQL server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Start background indexer for default contract if specified
    let indexer_handle = if let Ok(contract_address) = env::var("CONTRACT_ADDRESS") {
        println!("🚀 Starting background indexer for contract: {}", contract_address);
        let indexer_database = database.clone();
        let indexer_rpc = rpc.clone();
        let indexer_contract = contract_address.clone();
        let indexer_config_clone = indexer_config.clone();
        
        Some(tokio::spawn(async move {
            crate::indexer::start_background_indexer(
                indexer_database,
                indexer_rpc,
                indexer_contract,
                Some(indexer_config_clone),
            ).await;
        }))
    } else {
        println!("ℹ️  No CONTRACT_ADDRESS env var set - background indexer not started");
        println!("   GraphQL queries will work but may be slower without pre-indexed data");
        None
    };

    println!("✅ All services started successfully!");
    println!("   📊 GraphQL Playground: http://localhost:3000/graphql");
    println!("   🔍 GraphiQL Interface: http://localhost:3000/graphiql");
    println!("   📈 Sync Status API: http://localhost:3000/sync-status");

    // Wait for either service to complete (they should run indefinitely)
    if let Some(indexer) = indexer_handle {
        tokio::select! {
            _ = server_handle => println!("🛑 GraphQL server stopped"),
            _ = indexer => println!("🛑 Background indexer stopped"),
        }
    } else {
        server_handle.await.unwrap();
    }
}

async fn sync_status_handler(
    axum::extract::State((database, rpc)): axum::extract::State<(std::sync::Arc<crate::database::Database>, crate::starknet::RpcContext)>
) -> Json<serde_json::Value> {
    use serde_json::json;
    
    // Get contract address from env
    let contract_address = match std::env::var("CONTRACT_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => {
            return Json(json!({
                "status": "error",
                "message": "No CONTRACT_ADDRESS configured"
            }));
        }
    };

    // Get current block from network
    let current_block = match crate::starknet::get_current_block_number(&rpc).await {
        Ok(block) => block,
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to get current block: {}", e)
            }));
        }
    };

    // Get indexer state
    let indexer_state = match database.get_indexer_state(&contract_address).await {
        Ok(Some(state)) => state,
        Ok(None) => {
            return Json(json!({
                "status": "not_started",
                "current_block": current_block,
                "last_synced_block": 0,
                "blocks_behind": current_block,
                "message": "Indexer not started yet"
            }));
        }
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": format!("Database error: {}", e)
            }));
        }
    };

    let blocks_behind = current_block.saturating_sub(indexer_state.last_synced_block);
    let sync_percentage = if current_block > 0 {
        (indexer_state.last_synced_block as f64 / current_block as f64) * 100.0
    } else {
        100.0
    };

    let status = if blocks_behind > 100 {
        "out_of_sync"
    } else if blocks_behind > 10 {
        "catching_up"
    } else if blocks_behind > 0 {
        "nearly_synced"
    } else {
        "fully_synced"
    };

    Json(json!({
        "status": status,
        "current_block": current_block,
        "last_synced_block": indexer_state.last_synced_block,
        "blocks_behind": blocks_behind,
        "sync_percentage": format!("{:.2}%", sync_percentage),
        "contract_address": contract_address,
        "last_updated": indexer_state.updated_at.to_rfc3339()
    }))
}

async fn indexer_stats_handler(
    axum::extract::State((database, _rpc)): axum::extract::State<(std::sync::Arc<crate::database::Database>, crate::starknet::RpcContext)>,
    Path(contract_address): Path<String>
) -> Json<serde_json::Value> {
    use serde_json::json;

    match database.get_indexer_stats(&contract_address).await {
        Ok(stats) => Json(stats),
        Err(e) => Json(json!({
            "error": format!("Failed to get indexer stats: {}", e)
        }))
    }
}

async fn graphiql_handler() -> Html<String> {
    // For local dev: ws://
    Html(GraphiQLSource::build().endpoint("/graphql").subscription_endpoint("ws://localhost:3000/ws").finish())
}