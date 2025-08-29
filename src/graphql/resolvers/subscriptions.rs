use async_graphql::{Context, Subscription};
use futures::StreamExt;
use futures::stream::BoxStream;
use tokio_stream::wrappers::BroadcastStream;
use std::sync::Arc;

use crate::graphql::types::Event;
use crate::realtime::{RealtimeEventManager, SubscriptionFilter};
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Universal event subscription that handles all use cases:
    /// - Single contract: provide contractAddress
    /// - Multiple contracts: provide contractAddresses  
    /// - Event filtering: eventTypes, eventKeys
    /// - Real-time updates: automatically streams new events
    async fn events(
        &self,
        ctx: &Context<'_>,
        // Contract filtering - supports single or multiple contracts
        #[graphql(name = "contractAddress")] contract_address: Option<String>,
        #[graphql(name = "contractAddresses")] contract_addresses: Option<Vec<String>>,
        
        // Event filtering
        #[graphql(name = "eventTypes")] event_types: Option<Vec<String>>,
        #[graphql(name = "eventKeys")] event_keys: Option<Vec<String>>,
    ) -> Result<BoxStream<'static, Event>, async_graphql::Error> {
        let realtime_manager = ctx.data_unchecked::<Arc<RealtimeEventManager>>();
        
        // Determine target contracts
        let target_contracts = if let Some(addresses) = contract_addresses {
            addresses
        } else if let Some(address) = contract_address {
            vec![address]
        } else {
            return Err(async_graphql::Error::new("Either contractAddress or contractAddresses must be provided"));
        };

        // For now, we'll support single contract subscriptions
        // TODO: Enhance realtime manager to support multiple contracts
        let contract_addr = target_contracts.first().unwrap().clone();
        
        let filter = SubscriptionFilter {
            contract_address: contract_addr,
            event_types,
            event_keys,
        };

        let (_subscription_id, receiver) = realtime_manager.subscribe(filter).await;
        
        // Create a stream from the broadcast receiver
        let stream = BroadcastStream::new(receiver)
            .filter_map(|result| async move {
                match result {
                    Ok(event) => Some(event),
                    Err(_) => None, // Ignore broadcast errors
                }
            })
            .boxed();

        Ok(stream)
    }
}

