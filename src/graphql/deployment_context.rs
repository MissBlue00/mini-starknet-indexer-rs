use std::sync::Arc;
use crate::database::{Database, DeploymentRecord};

/// Context for deployment-specific GraphQL operations
#[derive(Clone)]
pub struct DeploymentContext {
    pub deployment: DeploymentRecord,
    pub database: Arc<Database>,
}

impl DeploymentContext {
    pub fn new(deployment: DeploymentRecord, database: Arc<Database>) -> Self {
        Self { deployment, database }
    }

    /// Get all contract addresses associated with this deployment
    pub async fn get_deployment_contract_addresses(&self) -> Result<Vec<String>, sqlx::Error> {
        // For now, if the deployment has a specific contract address, return that
        // Otherwise, return all contract addresses (this can be enhanced later)
        if let Some(contract_address) = &self.deployment.contract_address {
            Ok(vec![contract_address.clone()])
        } else {
            // Get all contract addresses from the database
            self.database.get_all_contract_addresses().await
        }
    }

    /// Check if a contract address belongs to this deployment
    pub async fn is_contract_in_deployment(&self, contract_address: &str) -> Result<bool, sqlx::Error> {
        let deployment_addresses = self.get_deployment_contract_addresses().await?;
        let normalized_address = Database::normalize_address(contract_address);
        Ok(deployment_addresses.contains(&normalized_address))
    }

    /// Get deployment-specific database connection
    /// This allows for future enhancement where each deployment could have its own database
    pub fn get_database(&self) -> Arc<Database> {
        // For now, use the main database, but in the future this could connect to deployment-specific databases
        self.database.clone()
    }
}
