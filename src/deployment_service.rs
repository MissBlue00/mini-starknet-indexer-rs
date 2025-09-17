use std::sync::Arc;
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;
use tokio::fs;

use crate::database::{Database, DeploymentRecord};

/// Semi-mock deployment service for managing deployment databases
pub struct DeploymentService {
    main_database: Arc<Database>,
    deployments_base_path: String,
}

impl DeploymentService {
    pub fn new(main_database: Arc<Database>, deployments_base_path: Option<String>) -> Self {
        let base_path = deployments_base_path.unwrap_or_else(|| "deployments".to_string());
        
        Self {
            main_database,
            deployments_base_path: base_path,
        }
    }

    /// Create a new deployment with its own database
    pub async fn create_deployment(
        &self,
        name: String,
        description: Option<String>,
        network: String,
        contract_address: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<DeploymentRecord, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        
        // Create deployments directory if it doesn't exist
        fs::create_dir_all(&self.deployments_base_path).await?;
        
        // Generate database file path
        let db_filename = format!("{}_{}.db", 
            name.replace(" ", "_").to_lowercase(), 
            network.to_lowercase()
        );
        let db_path = Path::new(&self.deployments_base_path).join(&db_filename);
        let database_url = format!("sqlite:{}", db_path.to_string_lossy());
        
        // Create the deployment database
        let deployment_db = Database::new(&database_url).await?;
        
        // Initialize with some mock data if contract address is provided
        if let Some(contract_addr) = &contract_address {
            self.initialize_deployment_database(&deployment_db, contract_addr).await?;
        }
        
        let deployment_record = DeploymentRecord {
            id: id.clone(),
            name,
            description,
            database_url,
            contract_address,
            network,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
            metadata: metadata.map(|v| v.to_string()),
        };

        // Save deployment record to main database
        self.main_database.create_deployment(&deployment_record).await?;

        println!("✅ Created deployment '{}' with database: {}", deployment_record.name, deployment_record.database_url);

        Ok(deployment_record)
    }

    /// Initialize a deployment database with mock data
    async fn initialize_deployment_database(
        &self,
        deployment_db: &Database,
        contract_address: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Set initial indexer state
        deployment_db.update_indexer_state(contract_address, 0).await?;
        
        println!("🔧 Initialized deployment database for contract: {}", contract_address);
        Ok(())
    }

    /// Get a deployment database connection
    #[allow(dead_code)]
    pub async fn get_deployment_database(
        &self,
        deployment_id: &str,
    ) -> Result<Option<Database>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(deployment) = self.main_database.get_deployment(deployment_id).await? {
            let db = Database::new(&deployment.database_url).await?;
            Ok(Some(db))
        } else {
            Ok(None)
        }
    }

    /// Update deployment status
    #[allow(dead_code)]
    pub async fn update_deployment_status(
        &self,
        deployment_id: &str,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.main_database.update_deployment(deployment_id, None, None, Some(status), None, None).await?;
        Ok(())
    }

    /// Delete a deployment and its database file
    pub async fn delete_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get deployment info first
        if let Some(deployment) = self.main_database.get_deployment(deployment_id).await? {
            // Extract file path from database URL
            if let Some(file_path) = deployment.database_url.strip_prefix("sqlite:") {
                // Try to delete the database file
                if Path::new(file_path).exists() {
                    match fs::remove_file(file_path).await {
                        Ok(_) => println!("🗑️  Deleted database file: {}", file_path),
                        Err(e) => println!("⚠️  Failed to delete database file {}: {}", file_path, e),
                    }
                }
            }
        }

        // Remove from main database
        self.main_database.delete_deployment(deployment_id).await?;
        
        println!("🗑️  Deleted deployment: {}", deployment_id);
        Ok(())
    }

    /// List all deployments
    #[allow(dead_code)]
    pub async fn list_deployments(
        &self,
        status: Option<&str>,
        network: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<DeploymentRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let deployments = self.main_database.get_deployments(status, network, limit, offset).await?;
        Ok(deployments)
    }

    /// Get deployment stats
    #[allow(dead_code)]
    pub async fn get_deployment_stats(
        &self,
        deployment_id: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(deployment) = self.main_database.get_deployment(deployment_id).await? {
            if let Some(contract_address) = &deployment.contract_address {
                if let Some(deployment_db) = self.get_deployment_database(deployment_id).await? {
                    let stats = deployment_db.get_indexer_stats(contract_address).await?;
                    return Ok(Some(stats));
                }
            }
            
            // Return basic stats if no contract address
            Ok(Some(serde_json::json!({
                "deployment_id": deployment_id,
                "name": deployment.name,
                "network": deployment.network,
                "status": deployment.status,
                "created_at": deployment.created_at.to_rfc3339(),
                "contract_address": deployment.contract_address,
                "total_events": 0
            })))
        } else {
            Ok(None)
        }
    }
}

/// Helper function to validate deployment parameters
pub fn validate_deployment_params(
    name: &str,
    network: &str,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Deployment name cannot be empty".to_string());
    }
    
    if name.len() > 100 {
        return Err("Deployment name cannot exceed 100 characters".to_string());
    }
    
    let valid_networks = ["mainnet", "testnet", "devnet", "local"];
    if !valid_networks.contains(&network.to_lowercase().as_str()) {
        return Err(format!("Invalid network '{}'. Must be one of: {}", network, valid_networks.join(", ")));
    }
    
    Ok(())
}
