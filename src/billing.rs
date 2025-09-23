use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::database::{Database, ApiCallRecord, ContractQueryRecord};

pub struct BillingService {
    database: Arc<Database>,
}

impl BillingService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Start tracking an API call and return the API call ID
    pub async fn start_api_call(
        &self,
        deployment_id: Option<String>,
        user_id: Option<String>,
        endpoint: String,
        method: String,
        metadata: Option<String>,
    ) -> Result<String, sqlx::Error> {
        let api_call_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let api_call = ApiCallRecord {
            id: api_call_id.clone(),
            deployment_id,
            user_id,
            endpoint,
            method,
            timestamp: now,
            duration_ms: None, // Will be updated when call completes
            status_code: None, // Will be updated when call completes
            metadata,
        };

        self.database.insert_api_call(&api_call).await?;
        Ok(api_call_id)
    }

    /// Complete an API call with duration and status code
    pub async fn complete_api_call(
        &self,
        api_call_id: &str,
        duration_ms: i64,
        status_code: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE api_calls SET duration_ms = ?, status_code = ? WHERE id = ?"
        )
        .bind(duration_ms)
        .bind(status_code)
        .bind(api_call_id)
        .execute(&self.database.pool)
        .await?;

        Ok(())
    }

    /// Track a contract query within an API call
    pub async fn track_contract_query(
        &self,
        api_call_id: &str,
        contract_address: String,
        query_type: String,
        cost_usdc: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        let query_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let contract_query = ContractQueryRecord {
            id: query_id,
            api_call_id: api_call_id.to_string(),
            contract_address: Database::normalize_address(&contract_address),
            query_type,
            timestamp: now,
            cost_usdc: cost_usdc.unwrap_or(0.001), // Default cost per contract query
        };

        self.database.insert_contract_query(&contract_query).await?;
        Ok(())
    }

    /// Track multiple contract queries in a single API call
    pub async fn track_multiple_contract_queries(
        &self,
        api_call_id: &str,
        contract_addresses: Vec<String>,
        query_type: String,
        cost_per_contract: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        let cost = cost_per_contract.unwrap_or(0.001);
        
        for contract_address in contract_addresses {
            self.track_contract_query(
                api_call_id,
                contract_address,
                query_type.clone(),
                Some(cost),
            ).await?;
        }

        Ok(())
    }

    /// Get usage statistics for API calls
    pub async fn get_api_usage_stats(
        &self,
        deployment_id: Option<&str>,
        user_id: Option<&str>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        self.database.get_api_call_usage_stats(
            deployment_id,
            user_id,
            from_date,
            to_date,
        ).await
    }

    /// Get usage statistics for contract queries
    pub async fn get_contract_usage_stats(
        &self,
        contract_address: Option<&str>,
        deployment_id: Option<&str>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        self.database.get_contract_usage_stats(
            contract_address,
            deployment_id,
            from_date,
            to_date,
        ).await
    }

    /// Calculate total cost for a deployment or user
    pub async fn calculate_total_cost(
        &self,
        deployment_id: Option<&str>,
        user_id: Option<&str>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<f64, sqlx::Error> {
        let stats = self.get_api_usage_stats(deployment_id, user_id, from_date, to_date).await?;
        
        let total_cost: f64 = stats
            .iter()
            .map(|stat| stat.get("total_cost_usdc").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum();

        Ok(total_cost)
    }

    /// Get billing summary for a deployment
    pub async fn get_billing_summary(
        &self,
        deployment_id: &str,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let api_stats = self.get_api_usage_stats(Some(deployment_id), None, from_date, to_date).await?;
        let contract_stats = self.get_contract_usage_stats(None, Some(deployment_id), from_date, to_date).await?;
        let total_cost = self.calculate_total_cost(Some(deployment_id), None, from_date, to_date).await?;

        let total_api_calls = api_stats.len() as i64;
        let total_contract_queries: i64 = api_stats
            .iter()
            .map(|stat| stat.get("contract_count").and_then(|v| v.as_i64()).unwrap_or(0))
            .sum();

        Ok(serde_json::json!({
            "deployment_id": deployment_id,
            "period": {
                "from": from_date.map(|d| d.to_rfc3339()),
                "to": to_date.map(|d| d.to_rfc3339())
            },
            "summary": {
                "total_api_calls": total_api_calls,
                "total_contract_queries": total_contract_queries,
                "total_cost_usdc": total_cost
            },
            "api_calls": api_stats,
            "contract_usage": contract_stats
        }))
    }
}
