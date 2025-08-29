use async_graphql::SimpleObject;
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
