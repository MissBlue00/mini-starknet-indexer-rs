use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Json,
};
use async_graphql::http::GraphiQLSource;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::database::Database;
use crate::starknet::RpcContext;
use crate::realtime::RealtimeEventManager;
use crate::graphql::deployment_context::DeploymentContext;
use crate::graphql::deployment_schema::{build_deployment_schema, DeploymentSchema};

/// Cache for deployment-specific GraphQL schemas
pub type SchemaCache = Arc<RwLock<HashMap<String, DeploymentSchema>>>;

/// Create a new schema cache
pub fn create_schema_cache() -> SchemaCache {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Get or create a deployment-specific GraphQL schema
pub async fn get_deployment_schema(
    deployment_id: &str,
    database: Arc<Database>,
    rpc: RpcContext,
    realtime_manager: Arc<RealtimeEventManager>,
    cache: SchemaCache,
) -> Result<DeploymentSchema, StatusCode> {
    // Check cache first
    {
        let cache_read = cache.read().await;
        if let Some(schema) = cache_read.get(deployment_id) {
            return Ok(schema.clone());
        }
    }
    
    // Get the deployment from the database
    let deployment = match database.get_deployment(deployment_id).await {
        Ok(Some(deployment)) => deployment,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Create deployment context
    let deployment_context = DeploymentContext::new(deployment, database);
    
    // Build deployment-specific schema
    let schema = build_deployment_schema(deployment_context, rpc, realtime_manager);
    
    // Cache the schema
    {
        let mut cache_write = cache.write().await;
        cache_write.insert(deployment_id.to_string(), schema.clone());
    }
    
    Ok(schema)
}

/// Handler for deployment-specific GraphQL queries
pub async fn deployment_graphql_post_handler(
    Path(deployment_id): Path<String>,
    State((database, rpc, realtime_manager, cache)): State<(Arc<Database>, RpcContext, Arc<RealtimeEventManager>, SchemaCache)>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let schema = match get_deployment_schema(&deployment_id, database, rpc, realtime_manager, cache).await {
        Ok(schema) => schema,
        Err(status) => return Err(status),
    };
    
    // Parse the GraphQL request
    let graphql_request: async_graphql::Request = match serde_json::from_value(request) {
        Ok(req) => req,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    // Execute the GraphQL request
    let response = schema.execute(graphql_request).await;
    
    // Convert response to JSON
    let json_response = serde_json::to_value(response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json_response))
}

/// Handler for deployment-specific GraphiQL interface
pub async fn deployment_graphiql_handler(
    Path(deployment_id): Path<String>,
) -> Html<String> {
    Html(GraphiQLSource::build()
        .endpoint(&format!("/deployment/{}/graphql", deployment_id))
        .subscription_endpoint(&format!("ws://localhost:3000/deployment/{}/ws", deployment_id))
        .finish())
}


/// Handler to list all deployments with their GraphQL endpoints
pub async fn list_deployment_endpoints(
    State((database, _rpc, _realtime_manager, _cache)): State<(Arc<Database>, RpcContext, Arc<RealtimeEventManager>, SchemaCache)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let deployments = database.get_deployments(None, None, 100, 0).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let endpoints: Vec<serde_json::Value> = deployments.into_iter().map(|deployment| {
        serde_json::json!({
            "id": deployment.id,
            "name": deployment.name,
            "description": deployment.description,
            "network": deployment.network,
            "status": deployment.status,
            "endpoints": {
                "graphql": format!("/deployment/{}/graphql", deployment.id),
                "graphiql": format!("/deployment/{}/graphiql", deployment.id),
                "websocket": format!("/deployment/{}/ws", deployment.id)
            }
        })
    }).collect();
    
    Ok(Json(serde_json::json!({
        "deployments": endpoints,
        "total_count": endpoints.len()
    })))
}
