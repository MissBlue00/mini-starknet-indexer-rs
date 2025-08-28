use async_graphql::{InputObject, SimpleObject};


#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct EventEdge {
    pub node: Event,
    pub cursor: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct EventConnection {
    pub edges: Vec<EventEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct EventFieldValue {
    pub r#type: String,
    pub value: String,
    pub decoded_value: Option<String>,
}

// Clean, lean event data structure like TheGraph protocol
// This will be flattened as a JSON object directly

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
    pub data: Option<serde_json::Value>,
    pub raw_data: Vec<String>,
    pub raw_keys: Vec<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct Contract {
    pub address: String,
    pub abi: String,
    pub events: Vec<EventSchema>,
    pub name: Option<String>,
    pub verified: bool,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct EventSchema {
    pub name: String,
    pub r#type: String,
    pub inputs: Vec<EventInput>,
    pub anonymous: bool,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct EventInput {
    pub name: String,
    pub r#type: String,
    pub indexed: bool,
}

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

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct EventQueryArgs {
    pub contract_address: String,
    pub from_block: Option<String>,
    pub to_block: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub event_keys: Option<Vec<String>>,
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
    pub transaction_hash: Option<String>,
    pub first: Option<i32>,
    pub after: Option<String>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvancedEventQueryArgs {
    pub contract_address: String,
    pub filters: Option<EventFilters>,
    pub pagination: Option<PaginationArgs>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "camelCase")]
pub struct EventFilters {
    pub block_range: Option<BlockRangeFilter>,
    pub time_range: Option<TimeRangeFilter>,
    pub event_types: Option<Vec<String>>,
    pub event_keys: Option<Vec<String>>,
    pub transaction_hash: Option<String>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockRangeFilter {
    pub from_block: Option<String>,
    pub to_block: Option<String>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "camelCase")]
pub struct TimeRangeFilter {
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
}

#[derive(InputObject, Default, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct PaginationArgs {
    pub first: Option<i32>,
    pub after: Option<String>,
    pub order_by: Option<EventOrderBy>,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Default)]
pub enum EventOrderBy {
    #[default]
    BlockNumberDesc, // Latest to oldest (default)
    BlockNumberAsc,  // Oldest to latest
    TimestampDesc,   // Latest to oldest by timestamp
    TimestampAsc,    // Oldest to latest by timestamp
}

#[derive(Clone, SimpleObject)]
pub struct ContractEvents {
    pub contract_address: String,
    pub events: EventConnection,
}

#[derive(Clone, SimpleObject)]
pub struct MultiContractEventsConnection {
    pub contracts: Vec<ContractEvents>,
    pub total_contracts: i32,
    pub total_events: i32,
}

