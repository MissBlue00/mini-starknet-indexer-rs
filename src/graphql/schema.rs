use async_graphql::{EmptyMutation, MergedObject, Schema};

use crate::graphql::resolvers::contracts::ContractQueryRoot;
use crate::graphql::resolvers::events::EventQueryRoot;
use crate::graphql::resolvers::subscriptions::SubscriptionRoot;
use crate::starknet::RpcContext;

#[derive(MergedObject, Default)]
pub struct QueryRoot(EventQueryRoot, ContractQueryRoot);

pub type AppSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

pub fn build_schema(rpc: RpcContext) -> AppSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, SubscriptionRoot)
        .data(rpc)
        .finish()
}

