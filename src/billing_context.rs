use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::billing::BillingService;

pub struct BillingContext {
    pub api_call_id: String,
    pub deployment_id: Option<String>,
    pub user_id: Option<String>,
    pub start_time: Instant,
    pub billing_service: Arc<BillingService>,
}

impl BillingContext {
    pub fn new(
        deployment_id: Option<String>,
        user_id: Option<String>,
        endpoint: String,
        method: String,
        billing_service: Arc<BillingService>,
    ) -> Self {
        let api_call_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        // Start tracking the API call asynchronously
        let service = billing_service.clone();
        let dep_id = deployment_id.clone();
        let usr_id = user_id.clone();
        tokio::spawn(async move {
            if let Err(e) = service.start_api_call(
                dep_id,
                usr_id,
                endpoint,
                method,
                None,
            ).await {
                eprintln!("Failed to start tracking API call: {}", e);
            }
        });

        Self {
            api_call_id,
            deployment_id,
            user_id,
            start_time,
            billing_service,
        }
    }

    pub async fn track_contract_query(
        &self,
        contract_address: String,
        query_type: String,
        cost_usdc: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        self.billing_service.track_contract_query(
            &self.api_call_id,
            contract_address,
            query_type,
            cost_usdc,
        ).await
    }

    pub async fn track_multiple_contract_queries(
        &self,
        contract_addresses: Vec<String>,
        query_type: String,
        cost_per_contract: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        self.billing_service.track_multiple_contract_queries(
            &self.api_call_id,
            contract_addresses,
            query_type,
            cost_per_contract,
        ).await
    }

    pub async fn complete_api_call(&self, status_code: i32) -> Result<(), sqlx::Error> {
        let duration_ms = self.start_time.elapsed().as_millis() as i64;
        self.billing_service.complete_api_call(
            &self.api_call_id,
            duration_ms,
            status_code,
        ).await
    }

    pub fn get_api_call_id(&self) -> &str {
        &self.api_call_id
    }
}

impl Drop for BillingContext {
    fn drop(&mut self) {
        // Complete the API call tracking when the context is dropped
        let api_call_id = self.api_call_id.clone();
        let duration_ms = self.start_time.elapsed().as_millis() as i64;
        let billing_service = self.billing_service.clone();

        tokio::spawn(async move {
            if let Err(e) = billing_service.complete_api_call(
                &api_call_id,
                duration_ms,
                200, // Default success status
            ).await {
                eprintln!("Failed to complete API call tracking: {}", e);
            }
        });
    }
}
