use async_graphql::{MergedObject, Schema};
use std::sync::Arc;

use crate::database::Database;
use crate::graphql::resolvers::contracts::ContractQueryRoot;
use crate::graphql::resolvers::events::EventQueryRoot;
use crate::graphql::resolvers::deployments::{DeploymentQueryRoot, DeploymentMutationRoot, DeploymentContractQueryRoot, DeploymentContractMutationRoot};
use crate::graphql::resolvers::subscriptions::SubscriptionRoot;
use crate::starknet::RpcContext;
use crate::realtime::RealtimeEventManager;

#[derive(MergedObject, Default)]
pub struct QueryRoot(EventQueryRoot, ContractQueryRoot, DeploymentQueryRoot, DeploymentContractQueryRoot);

#[derive(MergedObject, Default)]
pub struct MutationRoot(DeploymentMutationRoot, DeploymentContractMutationRoot);

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn build_schema(rpc: RpcContext, database: Arc<Database>, realtime_manager: Arc<RealtimeEventManager>) -> AppSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), SubscriptionRoot)
        .data(rpc)
        .data(database)
        .data(realtime_manager)
        .finish()
}

