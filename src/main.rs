use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode
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
struct StarknetRpcResponse {
    jsonrpc: String,
    id: u32,
    result: StarknetEventsResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct StarknetRpcError {
    jsonrpc: String,
    id: u32,
    error: serde_json::Value,
}

async fn root_handler() -> Json<MockResponse> {
    Json(MockResponse {
        status: "ok".to_string(),
        data: "mock".to_string(),
    })
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
    
    // According to Starknet RPC spec, the method should be "starknet_getEvents"
    // but let's try the correct parameter format
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
                        Ok(serde_json::to_string_pretty(&json_response).unwrap())
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

#[tokio::main]
async fn main() {
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/test", get(test_json_handler))
        .route("/fetch-events", post(fetch_starknet_events_handler));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}