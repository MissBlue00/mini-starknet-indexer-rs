use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};
use serde_json;

use crate::database::{Database, ApiKeyRecord};

/// API key service for managing deployment API keys
pub struct ApiKeyService {
    pub database: Arc<Database>,
}

impl ApiKeyService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Generate a new API key for a deployment
    pub async fn create_api_key(
        &self,
        deployment_id: &str,
        name: String,
        description: Option<String>,
        permissions: Option<serde_json::Value>,
    ) -> Result<(String, ApiKeyRecord), Box<dyn std::error::Error + Send + Sync>> {
        // Generate a secure API key
        let api_key = self.generate_api_key();
        
        // Hash the API key for storage
        let key_hash = self.hash_api_key(&api_key);
        
        // Default permissions if none provided
        let permissions_json = permissions
            .unwrap_or_else(|| serde_json::json!({"read": true, "write": false}));
        
        let now = Utc::now();
        let api_key_record = ApiKeyRecord {
            id: Uuid::new_v4().to_string(),
            deployment_id: deployment_id.to_string(),
            key_hash,
            name,
            description,
            permissions: permissions_json.to_string(),
            is_active: true,
            last_used: None,
            created_at: now,
            expires_at: None,
        };

        // Save to database
        self.database.create_api_key(&api_key_record).await?;

        Ok((api_key, api_key_record))
    }

    /// Validate an API key and return the associated deployment ID
    pub async fn validate_api_key(
        &self,
        api_key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let key_hash = self.hash_api_key(api_key);
        
        if let Some(api_key_record) = self.database.get_api_key_by_hash(&key_hash).await? {
            // Check if key is active and not expired
            if !api_key_record.is_active {
                return Ok(None);
            }
            
            if let Some(expires_at) = api_key_record.expires_at {
                if Utc::now() > expires_at {
                    return Ok(None);
                }
            }
            
            // Update last used timestamp
            let _ = self.database.update_api_key_last_used(&api_key_record.id).await;
            
            Ok(Some(api_key_record.deployment_id))
        } else {
            Ok(None)
        }
    }

    /// Get all API keys for a deployment (without the actual keys)
    pub async fn get_deployment_api_keys(
        &self,
        deployment_id: &str,
    ) -> Result<Vec<ApiKeyRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let api_keys = self.database.get_api_keys_for_deployment(deployment_id).await?;
        Ok(api_keys)
    }

    /// Deactivate an API key
    pub async fn deactivate_api_key(
        &self,
        api_key_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.database.deactivate_api_key(api_key_id).await?;
        Ok(())
    }

    /// Delete an API key
    pub async fn delete_api_key(
        &self,
        api_key_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.database.delete_api_key(api_key_id).await?;
        Ok(())
    }

    /// Generate a secure API key
    fn generate_api_key(&self) -> String {
        // Generate a random UUID and encode it with additional entropy
        let uuid = Uuid::new_v4();
        let timestamp = Utc::now().timestamp_millis();
        
        // Create a prefix for easy identification
        let prefix = "sk_";
        
        // Combine UUID and timestamp for additional entropy
        let combined = format!("{}{}{}", uuid, timestamp, Uuid::new_v4());
        
        // Hash the combined string and take first 32 characters
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);
        
        format!("{}{}", prefix, &hash_hex[..32])
    }

    /// Hash an API key for secure storage
    pub fn hash_api_key(&self, api_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    /// Parse API key permissions
    pub fn parse_permissions(permissions_json: &str) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(permissions_json)
    }

    /// Check if API key has specific permission
    pub fn has_permission(
        permissions: &serde_json::Value,
        permission: &str,
    ) -> bool {
        permissions
            .get(permission)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[tokio::test]
    async fn test_api_key_generation_and_validation() {
        // Create a test database
        let database = Arc::new(Database::new("sqlite::memory:").await.unwrap());
        let api_key_service = ApiKeyService::new(database);

        // Create a test deployment ID
        let deployment_id = "test-deployment-123";

        // Create an API key
        let (api_key, api_key_record) = api_key_service
            .create_api_key(
                deployment_id,
                "Test Key".to_string(),
                Some("Test description".to_string()),
                None,
            )
            .await
            .unwrap();

        // Validate the API key
        let validated_deployment_id = api_key_service
            .validate_api_key(&api_key)
            .await
            .unwrap();

        assert_eq!(validated_deployment_id, Some(deployment_id.to_string()));
        assert_eq!(api_key_record.deployment_id, deployment_id);
        assert!(api_key_record.is_active);
    }

    #[tokio::test]
    async fn test_api_key_generation_format() {
        // Create a dummy database for the service (not actually used in this test)
        let database = Arc::new(Database::new("sqlite::memory:").await.unwrap());
        let api_key_service = ApiKeyService::new(database);

        let api_key = api_key_service.generate_api_key();
        
        // Should start with sk_ prefix
        assert!(api_key.starts_with("sk_"));
        
        // Should be 35 characters total (3 for prefix + 32 for hash)
        assert_eq!(api_key.len(), 35);
    }

    #[test]
    fn test_permissions_parsing() {
        let permissions_json = r#"{"read": true, "write": false}"#;
        let permissions = ApiKeyService::parse_permissions(permissions_json).unwrap();
        
        assert!(ApiKeyService::has_permission(&permissions, "read"));
        assert!(!ApiKeyService::has_permission(&permissions, "write"));
        assert!(!ApiKeyService::has_permission(&permissions, "admin"));
    }
}
