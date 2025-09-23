use async_graphql::{Context, Object, Result as GqlResult};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::billing::BillingService;

#[derive(Default)]
pub struct BillingQueryRoot;

#[Object]
impl BillingQueryRoot {
    /// Get API usage statistics
    async fn api_usage_stats(
        &self,
        ctx: &Context<'_>,
        deployment_id: Option<String>,
        user_id: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> GqlResult<Vec<serde_json::Value>> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Parse dates
        let from_date_dt = from_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let to_date_dt = to_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let stats = billing_service
            .get_api_usage_stats(
                deployment_id.as_deref(),
                user_id.as_deref(),
                from_date_dt,
                to_date_dt,
            )
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to get API usage stats: {}", e)))?;
        
        Ok(stats)
    }

    /// Get contract usage statistics
    async fn contract_usage_stats(
        &self,
        ctx: &Context<'_>,
        contract_address: Option<String>,
        deployment_id: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> GqlResult<Vec<serde_json::Value>> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Parse dates
        let from_date_dt = from_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let to_date_dt = to_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let stats = billing_service
            .get_contract_usage_stats(
                contract_address.as_deref(),
                deployment_id.as_deref(),
                from_date_dt,
                to_date_dt,
            )
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to get contract usage stats: {}", e)))?;
        
        Ok(stats)
    }

    /// Calculate total cost for a deployment or user
    async fn total_cost(
        &self,
        ctx: &Context<'_>,
        deployment_id: Option<String>,
        user_id: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> GqlResult<f64> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Parse dates
        let from_date_dt = from_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let to_date_dt = to_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let total_cost = billing_service
            .calculate_total_cost(
                deployment_id.as_deref(),
                user_id.as_deref(),
                from_date_dt,
                to_date_dt,
            )
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to calculate total cost: {}", e)))?;
        
        Ok(total_cost)
    }

    /// Get billing summary for a deployment
    async fn billing_summary(
        &self,
        ctx: &Context<'_>,
        deployment_id: String,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> GqlResult<serde_json::Value> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Parse dates
        let from_date_dt = from_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let to_date_dt = to_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let summary = billing_service
            .get_billing_summary(&deployment_id, from_date_dt, to_date_dt)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to get billing summary: {}", e)))?;
        
        Ok(summary)
    }

    /// Get CPU usage statistics
    async fn cpu_usage_stats(
        &self,
        ctx: &Context<'_>,
        deployment_id: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
    ) -> GqlResult<Vec<serde_json::Value>> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        // Parse dates
        let from_date_dt = from_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let to_date_dt = to_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let stats = billing_service
            .get_cpu_usage_stats(
                deployment_id.as_deref(),
                from_date_dt,
                to_date_dt,
            )
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to get CPU usage stats: {}", e)))?;
        
        Ok(stats)
    }

    /// Get CPU pricing tiers
    async fn cpu_pricing_tiers(
        &self,
        ctx: &Context<'_>,
        active_only: Option<bool>,
    ) -> GqlResult<Vec<serde_json::Value>> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        let tiers = billing_service
            .get_cpu_pricing_tiers(active_only.unwrap_or(true))
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to get CPU pricing tiers: {}", e)))?;
        
        let tier_values: Vec<serde_json::Value> = tiers
            .into_iter()
            .map(|tier| serde_json::json!({
                "id": tier.id,
                "name": tier.name,
                "cpu_units_per_request": tier.cpu_units_per_request,
                "price_per_cpu_unit_usdc": tier.price_per_cpu_unit_usdc,
                "minimum_charge_usdc": tier.minimum_charge_usdc,
                "is_active": tier.is_active,
                "created_at": tier.created_at.to_rfc3339(),
                "updated_at": tier.updated_at.to_rfc3339()
            }))
            .collect();
        
        Ok(tier_values)
    }

    /// Create a new CPU pricing tier
    async fn create_cpu_pricing_tier(
        &self,
        ctx: &Context<'_>,
        name: String,
        cpu_units_per_request: i32,
        price_per_cpu_unit_usdc: f64,
        minimum_charge_usdc: f64,
    ) -> GqlResult<String> {
        let billing_service = ctx.data::<Arc<BillingService>>()?.clone();
        
        let tier_id = billing_service
            .create_cpu_pricing_tier(
                name,
                cpu_units_per_request,
                price_per_cpu_unit_usdc,
                minimum_charge_usdc,
            )
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to create CPU pricing tier: {}", e)))?;
        
        Ok(tier_id)
    }
}
