use async_graphql::{SimpleObject, InputObject, Enum};
use serde_json;

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct Event {
    pub id: String,
    pub contract_address: String,
    pub event_type: String,
    pub block_number: String,
    pub transaction_hash: String,
    pub log_index: i32,
    pub timestamp: String,
    pub data: Option<serde_json::Value>, // Flattened data structure
    pub raw_data: Vec<String>,
    pub raw_keys: Vec<String>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EventConnection {
    pub edges: Vec<EventEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EventEdge {
    pub node: Event,
    pub cursor: String,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Default)]
pub enum EventOrderBy {
    #[default]
    BlockNumberDesc, // Latest to oldest (default)
    BlockNumberAsc,  // Oldest to latest
    TimestampDesc,   // Latest to oldest by timestamp
    TimestampAsc,    // Oldest to latest by timestamp
}

// Simple subscription event for real-time updates
#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct SubscriptionEvent {
    pub id: String,
    pub contract_address: String,
    pub event_type: String,
    pub block_number: String,
    pub data: Option<serde_json::Value>,
}

// Optional: Keep some legacy types for backward compatibility if needed
#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct Block {
    pub number: String,
    pub hash: String,
    pub timestamp: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct Transaction {
    pub hash: String,
    pub block_number: String,
    pub from: String,
    pub to: String,
    pub value: String,
}

// Legacy types for backward compatibility with contracts resolver
#[derive(SimpleObject, Clone)]
pub struct Contract {
    pub address: String,
    pub name: Option<String>,
    pub abi: Option<String>,
    pub events: Vec<EventSchema>,
    pub verified: bool,
}

#[derive(SimpleObject, Clone)]
pub struct EventInput {
    pub name: String,
    pub r#type: String,
    pub indexed: bool,
}

#[derive(SimpleObject, Clone)]
pub struct EventSchema {
    pub name: String,
    pub r#type: String,
    pub inputs: Vec<EventInput>,
    pub anonymous: bool,
}

// Deployment types
#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub database_url: String,
    pub contract_address: Option<String>, // Legacy field for backward compatibility
    pub network: String,
    pub status: DeploymentStatus,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
    pub contracts: Option<Vec<DeploymentContract>>, // New multi-contract support
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentContract {
    pub id: String,
    pub deployment_id: String,
    pub contract_address: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_block: Option<String>,
    pub status: DeploymentContractStatus,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum DeploymentContractStatus {
    Active,
    Inactive,
    Error,
}

impl From<&str> for DeploymentContractStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => DeploymentContractStatus::Active,
            "inactive" => DeploymentContractStatus::Inactive,
            "error" => DeploymentContractStatus::Error,
            _ => DeploymentContractStatus::Inactive,
        }
    }
}

impl From<DeploymentContractStatus> for &'static str {
    fn from(status: DeploymentContractStatus) -> Self {
        match status {
            DeploymentContractStatus::Active => "active",
            DeploymentContractStatus::Inactive => "inactive",
            DeploymentContractStatus::Error => "error",
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum DeploymentStatus {
    Active,
    Inactive,
    Error,
}

impl From<&str> for DeploymentStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => DeploymentStatus::Active,
            "inactive" => DeploymentStatus::Inactive,
            "error" => DeploymentStatus::Error,
            _ => DeploymentStatus::Inactive,
        }
    }
}

impl From<DeploymentStatus> for &'static str {
    fn from(status: DeploymentStatus) -> Self {
        match status {
            DeploymentStatus::Active => "active",
            DeploymentStatus::Inactive => "inactive",
            DeploymentStatus::Error => "error",
        }
    }
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentConnection {
    pub edges: Vec<DeploymentEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentEdge {
    pub node: Deployment,
    pub cursor: String,
}

// Input types for deployment mutations
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateDeploymentInput {
    pub name: String,
    pub description: Option<String>,
    pub network: String,
    pub contract_address: Option<String>, // Legacy field for backward compatibility
    pub contracts: Option<Vec<CreateDeploymentContractInput>>, // New multi-contract support
    pub metadata: Option<serde_json::Value>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateDeploymentContractInput {
    pub contract_address: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_block: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct UpdateDeploymentInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<DeploymentStatus>,
    pub contract_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DeploymentFilter {
    pub status: Option<DeploymentStatus>,
    pub network: Option<String>,
}

// Input types for deployment contract management
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct AddDeploymentContractInput {
    pub deployment_id: String,
    pub contract_address: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_block: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// API Key types
#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct ApiKey {
    pub id: String,
    pub deployment_id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub is_active: bool,
    pub last_used: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    // Note: We never return the actual key hash for security
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct UpdateDeploymentContractInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<DeploymentContractStatus>,
    pub start_block: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateApiKeyInput {
    pub deployment_id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct UpdateApiKeyInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateApiKeyResult {
    pub api_key: String,
    pub api_key_record: ApiKey,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ApiKeyConnection {
    pub edges: Vec<ApiKeyEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct ApiKeyEdge {
    pub node: ApiKey,
    pub cursor: String,
}
