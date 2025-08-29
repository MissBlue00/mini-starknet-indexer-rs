use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::graphql::types::Event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    pub contract_address: String,
    pub event_types: Option<Vec<String>>,
    pub event_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    #[allow(dead_code)]
    pub id: String,
    pub filter: SubscriptionFilter,
    pub sender: broadcast::Sender<Event>,
}

#[derive(Clone)]
pub struct RealtimeEventManager {
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    event_sender: broadcast::Sender<Event>,
}

impl RealtimeEventManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub async fn subscribe(&self, filter: SubscriptionFilter) -> (String, broadcast::Receiver<Event>) {
        let subscription_id = Uuid::new_v4().to_string();
        let (sender, receiver) = broadcast::channel(100);
        
        let subscription = Subscription {
            id: subscription_id.clone(),
            filter: filter.clone(),
            sender,
        };

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        (subscription_id, receiver)
    }

    #[allow(dead_code)]
    pub async fn unsubscribe(&self, subscription_id: &str) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id).is_some()
    }

    pub async fn broadcast_event(&self, event: Event) {
        // Send to all subscribers that match the filter
        let subscriptions = self.subscriptions.read().await;
        let mut matched_subscribers = Vec::new();

        for subscription in subscriptions.values() {
            if self.matches_filter(&event, &subscription.filter) {
                matched_subscribers.push(subscription.sender.clone());
            }
        }

        // Broadcast to matched subscribers
        for sender in matched_subscribers {
            let _ = sender.send(event.clone());
        }

        // Also broadcast to the main event channel for any other listeners
        let _ = self.event_sender.send(event);
    }

    fn matches_filter(&self, event: &Event, filter: &SubscriptionFilter) -> bool {
        // Check contract address
        if event.contract_address != filter.contract_address {
            return false;
        }

        // Check event types if specified
        if let Some(ref event_types) = filter.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        // Check event keys if specified
        if let Some(ref event_keys) = filter.event_keys {
            let event_keys_set: std::collections::HashSet<_> = event_keys.iter().collect();
            let raw_keys_set: std::collections::HashSet<_> = event.raw_keys.iter().collect();
            
            if event_keys_set.is_disjoint(&raw_keys_set) {
                return false;
            }
        }

        true
    }

    #[allow(dead_code)]
    pub async fn get_subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    #[allow(dead_code)]
    pub async fn list_subscriptions(&self) -> Vec<SubscriptionFilter> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values().map(|s| s.filter.clone()).collect()
    }
}

impl Default for RealtimeEventManager {
    fn default() -> Self {
        Self::new()
    }
}
