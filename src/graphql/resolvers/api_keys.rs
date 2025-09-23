use async_graphql::{Context, Object, FieldResult};
use std::sync::Arc;
use base64::Engine;

use crate::database::{Database, ApiKeyRecord};
use crate::deployment_service::DeploymentService;
use crate::graphql::types::{
    ApiKey, ApiKeyConnection, ApiKeyEdge, PageInfo,
    CreateApiKeyInput, CreateApiKeyResult, UpdateApiKeyInput
};

#[derive(Default)]
pub struct ApiKeyQueryRoot;

#[Object]
impl ApiKeyQueryRoot {
    /// Get API keys for a specific deployment
    async fn api_keys(
        &self,
        ctx: &Context<'_>,
        deployment_id: String,
        first: Option<i32>,
        after: Option<String>,
    ) -> FieldResult<ApiKeyConnection> {
        let database = ctx.data::<Arc<Database>>()?;
        let deployment_service = DeploymentService::new(database.clone(), None);
        
        let api_keys = deployment_service.get_deployment_api_keys(&deployment_id).await
            .map_err(|e| format!("Failed to fetch API keys: {}", e))?;
        
        // Convert database records to GraphQL types
        let mut edges = Vec::new();
        for (index, api_key_record) in api_keys.iter().enumerate() {
            let cursor = base64::engine::general_purpose::STANDARD.encode(format!("{}", index));
            let api_key = convert_api_key_record_to_graphql(api_key_record.clone());
            edges.push(ApiKeyEdge {
                node: api_key,
                cursor,
            });
        }
        
        // Apply pagination
        let start_index = if let Some(after_cursor) = after {
            let decoded = base64::engine::general_purpose::STANDARD.decode(&after_cursor).unwrap_or_default();
            let index_str = String::from_utf8(decoded).unwrap_or_default();
            index_str.parse::<usize>().unwrap_or(0) + 1
        } else {
            0
        };
        
        let limit = first.unwrap_or(10) as usize;
        let end_index = std::cmp::min(start_index + limit, edges.len());
        
        let paginated_edges = if start_index < edges.len() {
            edges[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };
        
        let page_info = PageInfo {
            has_next_page: end_index < edges.len(),
            has_previous_page: start_index > 0,
            start_cursor: paginated_edges.first().map(|e| e.cursor.clone()),
            end_cursor: paginated_edges.last().map(|e| e.cursor.clone()),
        };
        
        Ok(ApiKeyConnection {
            edges: paginated_edges,
            page_info,
            total_count: api_keys.len() as i32,
        })
    }
    
    /// Get a single API key by ID
    async fn api_key(&self, ctx: &Context<'_>, id: String) -> FieldResult<Option<ApiKey>> {
        let database = ctx.data::<Arc<Database>>()?;
        
        match database.get_api_key_by_id(&id).await {
            Ok(Some(api_key_record)) => Ok(Some(convert_api_key_record_to_graphql(api_key_record))),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to fetch API key: {}", e).into()),
        }
    }
}

#[derive(Default)]
pub struct ApiKeyMutationRoot;

#[Object]
impl ApiKeyMutationRoot {
    /// Create a new API key for a deployment
    async fn create_api_key(
        &self,
        ctx: &Context<'_>,
        input: CreateApiKeyInput,
    ) -> FieldResult<CreateApiKeyResult> {
        let database = ctx.data::<Arc<Database>>()?;
        let deployment_service = DeploymentService::new(database.clone(), None);
        
        // Verify deployment exists
        match database.get_deployment(&input.deployment_id).await {
            Ok(Some(_)) => {},
            Ok(None) => return Err("Deployment not found".into()),
            Err(e) => return Err(format!("Failed to verify deployment: {}", e).into()),
        }
        
        // Create the API key
        let (api_key, api_key_record) = deployment_service.create_deployment_api_key(
            &input.deployment_id,
            input.name,
            input.description,
            input.permissions,
        ).await.map_err(|e| format!("Failed to create API key: {}", e))?;
        
        Ok(CreateApiKeyResult {
            api_key,
            api_key_record: convert_api_key_record_to_graphql(api_key_record),
        })
    }
    
    /// Update an API key
    async fn update_api_key(
        &self,
        ctx: &Context<'_>,
        input: UpdateApiKeyInput,
    ) -> FieldResult<ApiKey> {
        let database = ctx.data::<Arc<Database>>()?;
        
        // Get existing API key
        let mut api_key_record = match database.get_api_key_by_id(&input.id).await {
            Ok(Some(record)) => record,
            Ok(None) => return Err("API key not found".into()),
            Err(e) => return Err(format!("Failed to fetch API key: {}", e).into()),
        };
        
        // Update fields if provided
        if let Some(name) = input.name {
            api_key_record.name = name;
        }
        if let Some(description) = input.description {
            api_key_record.description = Some(description);
        }
        if let Some(permissions) = input.permissions {
            api_key_record.permissions = permissions.to_string();
        }
        if let Some(is_active) = input.is_active {
            api_key_record.is_active = is_active;
        }
        
        // For now, we'll just return the updated record
        // In a full implementation, you'd update the database record here
        Ok(convert_api_key_record_to_graphql(api_key_record))
    }
    
    /// Deactivate an API key
    async fn deactivate_api_key(
        &self,
        ctx: &Context<'_>,
        api_key_id: String,
    ) -> FieldResult<bool> {
        let database = ctx.data::<Arc<Database>>()?;
        let deployment_service = DeploymentService::new(database.clone(), None);
        
        deployment_service.deactivate_api_key(&api_key_id).await
            .map_err(|e| format!("Failed to deactivate API key: {}", e))?;
        
        Ok(true)
    }
    
    /// Delete an API key
    async fn delete_api_key(
        &self,
        ctx: &Context<'_>,
        api_key_id: String,
    ) -> FieldResult<bool> {
        let database = ctx.data::<Arc<Database>>()?;
        let deployment_service = DeploymentService::new(database.clone(), None);
        
        deployment_service.delete_api_key(&api_key_id).await
            .map_err(|e| format!("Failed to delete API key: {}", e))?;
        
        Ok(true)
    }
}

/// Convert database API key record to GraphQL type
fn convert_api_key_record_to_graphql(api_key_record: ApiKeyRecord) -> ApiKey {
    let permissions: serde_json::Value = serde_json::from_str(&api_key_record.permissions)
        .unwrap_or_else(|_| serde_json::json!({"read": true, "write": false}));
    
    ApiKey {
        id: api_key_record.id,
        deployment_id: api_key_record.deployment_id,
        name: api_key_record.name,
        description: api_key_record.description,
        permissions,
        is_active: api_key_record.is_active,
        last_used: api_key_record.last_used.map(|dt| dt.to_rfc3339()),
        created_at: api_key_record.created_at.to_rfc3339(),
        expires_at: api_key_record.expires_at.map(|dt| dt.to_rfc3339()),
    }
}
