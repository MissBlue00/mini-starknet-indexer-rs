use async_graphql::{Context, Object, FieldResult};
use std::sync::Arc;

use crate::database::{Database, DeploymentRecord};
use crate::deployment_service::{DeploymentService, validate_deployment_params};
use crate::graphql::types::{
    Deployment, DeploymentConnection, DeploymentEdge, PageInfo, 
    CreateDeploymentInput, UpdateDeploymentInput, DeploymentFilter, DeploymentStatus
};

#[derive(Default)]
pub struct DeploymentQueryRoot;

#[Object]
impl DeploymentQueryRoot {
    /// Get a single deployment by ID
    async fn deployment(&self, ctx: &Context<'_>, id: String) -> FieldResult<Option<Deployment>> {
        let database = ctx.data::<Arc<Database>>()?;
        
        match database.get_deployment(&id).await {
            Ok(Some(record)) => Ok(Some(convert_deployment_record_to_graphql(record))),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to fetch deployment: {}", e).into()),
        }
    }

    /// Get a list of deployments with optional filtering and pagination
    async fn deployments(
        &self,
        ctx: &Context<'_>,
        filter: Option<DeploymentFilter>,
        first: Option<i32>,
        after: Option<String>,
    ) -> FieldResult<DeploymentConnection> {
        let database = ctx.data::<Arc<Database>>()?;
        
        let limit = first.unwrap_or(20).min(100); // Max 100 items per page
        let offset = if let Some(cursor) = after {
            // Simple cursor-based pagination - in production you'd want more robust cursor handling
            cursor.parse::<i32>().unwrap_or(0)
        } else {
            0
        };

        let status = filter.as_ref().and_then(|f| f.status.map(|s| s.into()));
        let network = filter.as_ref().and_then(|f| f.network.as_deref());

        // Get deployments with one extra to check if there are more pages
        let records = database.get_deployments(status, network, limit + 1, offset).await
            .map_err(|e| format!("Failed to fetch deployments: {}", e))?;

        let has_next_page = records.len() > limit as usize;
        let deployments: Vec<DeploymentRecord> = records.into_iter().take(limit as usize).collect();

        let total_count = database.count_deployments(status, network).await
            .map_err(|e| format!("Failed to count deployments: {}", e))? as i32;

        let edges: Vec<DeploymentEdge> = deployments
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                let cursor = (offset + index as i32).to_string();
                DeploymentEdge {
                    node: convert_deployment_record_to_graphql(record),
                    cursor: cursor.clone(),
                }
            })
            .collect();

        let page_info = PageInfo {
            has_next_page,
            has_previous_page: offset > 0,
            start_cursor: edges.first().map(|e| e.cursor.clone()),
            end_cursor: edges.last().map(|e| e.cursor.clone()),
        };

        Ok(DeploymentConnection {
            edges,
            page_info,
            total_count,
        })
    }
}

#[derive(Default)]
pub struct DeploymentMutationRoot;

#[Object]
impl DeploymentMutationRoot {
    /// Create a new deployment (semi-mock implementation)
    async fn create_deployment(
        &self,
        ctx: &Context<'_>,
        input: CreateDeploymentInput,
    ) -> FieldResult<Deployment> {
        let database = ctx.data::<Arc<Database>>()?;
        
        // Validate input parameters
        validate_deployment_params(&input.name, &input.network)
            .map_err(|e| format!("Invalid deployment parameters: {}", e))?;
        
        // Create deployment service
        let deployment_service = DeploymentService::new(database.clone(), None);
        
        // Create the deployment using the service
        let deployment_record = deployment_service.create_deployment(
            input.name,
            input.description,
            input.network,
            input.contract_address,
            input.metadata,
        ).await.map_err(|e| format!("Failed to create deployment: {}", e))?;

        Ok(convert_deployment_record_to_graphql(deployment_record))
    }

    /// Update an existing deployment
    async fn update_deployment(
        &self,
        ctx: &Context<'_>,
        input: UpdateDeploymentInput,
    ) -> FieldResult<Option<Deployment>> {
        let database = ctx.data::<Arc<Database>>()?;
        
        let status = input.status.map(|s| s.into());
        let metadata = input.metadata.map(|v| v.to_string());
        
        database.update_deployment(
            &input.id,
            input.name.as_deref(),
            input.description.as_deref(),
            status,
            input.contract_address.as_deref(),
            metadata.as_deref(),
        ).await.map_err(|e| format!("Failed to update deployment: {}", e))?;

        // Return the updated deployment
        match database.get_deployment(&input.id).await {
            Ok(Some(record)) => Ok(Some(convert_deployment_record_to_graphql(record))),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to fetch updated deployment: {}", e).into()),
        }
    }

    /// Delete a deployment
    async fn delete_deployment(&self, ctx: &Context<'_>, id: String) -> FieldResult<bool> {
        let database = ctx.data::<Arc<Database>>()?;
        
        // Create deployment service and delete using the service
        let deployment_service = DeploymentService::new(database.clone(), None);
        deployment_service.delete_deployment(&id).await
            .map_err(|e| format!("Failed to delete deployment: {}", e))?;
        
        Ok(true)
    }
}

/// Helper function to convert database record to GraphQL type
fn convert_deployment_record_to_graphql(record: DeploymentRecord) -> Deployment {
    let metadata = record.metadata.and_then(|m| serde_json::from_str(&m).ok());
    
    Deployment {
        id: record.id,
        name: record.name,
        description: record.description,
        database_url: record.database_url,
        contract_address: record.contract_address,
        network: record.network,
        status: DeploymentStatus::from(record.status.as_str()),
        created_at: record.created_at.to_rfc3339(),
        updated_at: record.updated_at.to_rfc3339(),
        metadata,
    }
}
