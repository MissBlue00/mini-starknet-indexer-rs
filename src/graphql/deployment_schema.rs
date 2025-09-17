use async_graphql::{MergedObject, Schema};
use std::sync::Arc;
use crate::graphql::deployment_context::DeploymentContext;
use crate::graphql::resolvers::deployment_events::DeploymentEventQueryRoot;
use crate::graphql::resolvers::deployment_contracts::DeploymentContractQueryRoot;
use crate::graphql::resolvers::subscriptions::SubscriptionRoot;
use crate::starknet::RpcContext;
use crate::realtime::RealtimeEventManager;

/// Deployment-specific query root that merges all deployment-specific resolvers
#[derive(MergedObject, Default)]
pub struct DeploymentQueryRoot(DeploymentEventQueryRoot, DeploymentContractQueryRoot);

/// Deployment-specific GraphQL schema type
pub type DeploymentSchema = Schema<DeploymentQueryRoot, async_graphql::EmptyMutation, SubscriptionRoot>;

/// Build a deployment-specific GraphQL schema
pub fn build_deployment_schema(
    deployment_context: DeploymentContext,
    rpc: RpcContext,
    realtime_manager: Arc<RealtimeEventManager>
) -> DeploymentSchema {
    Schema::build(
        DeploymentQueryRoot::default(),
        async_graphql::EmptyMutation,
        SubscriptionRoot
    )
    .data(deployment_context)
    .data(rpc)
    .data(realtime_manager)
    .finish()
}
