use axum::{
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
struct MockResponse {
    status: String,
    data: String,
}

async fn root_handler() -> Json<MockResponse> {
    Json(MockResponse {
        status: "ok".to_string(),
        data: "mock".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Build our application with a route
    let app = Router::new()
        .route("/", get(root_handler));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}