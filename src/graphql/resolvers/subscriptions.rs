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
    async fn event_stream(
        &self,
        ctx: &Context<'_>,
        contract_address: String,
        event_types: Option<Vec<String>>,
        event_keys: Option<Vec<String>>,
    ) -> Result<BoxStream<'static, Event>, async_graphql::Error> {
        let realtime_manager = ctx.data_unchecked::<Arc<RealtimeEventManager>>();
        
        let filter = SubscriptionFilter {
            contract_address,
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

        // Note: In a production system, you might want to clean up the subscription
        // when the stream ends, but for now we'll let it persist
        Ok(stream)
    }

    async fn event_stream_realtime(
        &self,
        ctx: &Context<'_>,
        contract_address: String,
        event_types: Option<Vec<String>>,
        event_keys: Option<Vec<String>>,
    ) -> Result<BoxStream<'static, Event>, async_graphql::Error> {
        // This is an alias for the main event_stream for backward compatibility
        self.event_stream(ctx, contract_address, event_types, event_keys).await
    }
}

