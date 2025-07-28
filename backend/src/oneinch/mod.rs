// src/oneinch/mod.rs - 1inch Fusion+ Integration

pub mod adapter;
pub mod client;
pub mod types;
pub mod quote_engine;
pub mod slippage;
pub mod routing;
pub mod bridge_integration;
pub mod price_comparison;
pub mod validation;
pub mod monitoring;
pub mod fusion_plus;

pub use adapter::OneinchAdapter;
pub use client::FusionClient;
pub use types::*;
pub use quote_engine::QuoteEngine;
pub use slippage::SlippageProtection;
pub use routing::RoutingEngine;
pub use bridge_integration::{OneinchBridgeIntegration};
pub use price_comparison::{PriceComparisonService, OptimalQuoteWithComparison, PriceComparisonError};
pub use validation::{ApiKeyValidator, OneinchApiKeyValidator, SelfValidating};
pub use monitoring::{PerformanceMonitor, InMemoryPerformanceMonitor, Monitorable};
pub use fusion_plus::*;

use crate::constants::*;
use std::sync::Arc;
use tracing::{info, warn};

/// Main service for 1inch Fusion+ integration
pub struct OneinchService {
    pub adapter: Arc<OneinchAdapter>,
    pub quote_engine: Arc<QuoteEngine>,
    pub slippage_protection: Arc<SlippageProtection>,
    pub routing_engine: Arc<RoutingEngine>,
    pub price_comparison: Option<Arc<PriceComparisonService>>,
    pub performance_monitor: Option<Arc<dyn PerformanceMonitor>>,
    pub fusion_plus_client: Option<Arc<FusionPlusClient>>,
}

impl OneinchService {
    /// Create new OneinchService instance with API key validation
    pub fn new(api_key: String, chain_id: u64) -> Self {
        // Validate chain ID is supported
        if !Self::is_supported_chain_static(chain_id) {
            warn!("⚠️  Chain ID {} is not officially supported by 1inch", chain_id);
        }

        let fusion_client = Arc::new(FusionClient::new(api_key, chain_id));
        let adapter = Arc::new(OneinchAdapter::new(fusion_client.clone()));
        let quote_engine = Arc::new(QuoteEngine::new(fusion_client.clone()));
        let slippage_protection = Arc::new(SlippageProtection::new());
        let routing_engine = Arc::new(RoutingEngine::new(fusion_client.clone()));

        info!("✅ Created OneinchService for chain {} with honest API integration", chain_id);

        Self {
            adapter,
            quote_engine,
            slippage_protection,
            routing_engine,
            price_comparison: None, // Will be set when price oracle is available
            performance_monitor: Some(Arc::new(InMemoryPerformanceMonitor::new(1000))), // Default monitor
            fusion_plus_client: None, // Will be set when cross-chain functionality is needed
        }
    }

    /// Set price oracle for comparison features
    pub fn with_price_oracle(mut self, price_oracle: Arc<crate::price_oracle::PriceOracleService>) -> Self {
        let price_comparison = Arc::new(PriceComparisonService::new(
            Arc::new(self.clone_without_comparison()),
            price_oracle,
        ));
        self.price_comparison = Some(price_comparison);
        self
    }

    /// Set custom performance monitor
    pub fn with_performance_monitor(mut self, monitor: Arc<dyn PerformanceMonitor>) -> Self {
        self.performance_monitor = Some(monitor);
        self
    }

    /// Disable performance monitoring
    pub fn without_performance_monitor(mut self) -> Self {
        self.performance_monitor = None;
        self
    }

    /// Enable Fusion+ cross-chain functionality
    pub fn with_fusion_plus(mut self, api_key: Option<String>) -> Self {
        let fusion_client = match api_key {
            Some(key) => FusionPlusClient::with_api_key(key),
            None => match FusionPlusClient::new() {
                Ok(client) => client,
                Err(e) => {
                    warn!("⚠️  Failed to create Fusion+ client: {}", e);
                    return self;
                }
            }
        };
        
        self.fusion_plus_client = Some(Arc::new(fusion_client));
        info!("✅ Fusion+ cross-chain functionality enabled");
        self
    }

    /// Check if Fusion+ is available
    pub fn has_fusion_plus(&self) -> bool {
        self.fusion_plus_client.is_some()
    }

    /// Clone without price comparison to avoid circular reference
    fn clone_without_comparison(&self) -> OneinchService {
        OneinchService {
            adapter: self.adapter.clone(),
            quote_engine: self.quote_engine.clone(),
            slippage_protection: self.slippage_protection.clone(),
            routing_engine: self.routing_engine.clone(),
            price_comparison: None,
            performance_monitor: self.performance_monitor.clone(),
            fusion_plus_client: self.fusion_plus_client.clone(),
        }
    }

    /// Get quote for token swap
    pub async fn get_quote(&self, params: &QuoteParams) -> Result<FusionQuote, OneinchError> {
        self.quote_engine.get_quote(params).await
    }

    /// Execute swap order
    pub async fn execute_swap(&self, params: &SwapParams) -> Result<SwapResult, OneinchError> {
        self.adapter.execute_swap(params).await
    }

    /// Get order status
    pub async fn get_order_status(&self, order_hash: &str) -> Result<OrderStatus, OneinchError> {
        self.adapter.get_order_status(order_hash).await
    }

    /// Find optimal route using intelligent routing
    pub async fn find_optimal_route(&self, params: &routing::RouteSearchParams) -> Result<routing::OptimalRoute, OneinchError> {
        self.routing_engine.find_optimal_route(params).await
    }

    /// Get multiple route options for comparison
    pub async fn get_route_comparison(&self, params: &QuoteParams, criteria: &routing::OptimizationCriteria) -> Result<Vec<routing::ScoredRoute>, OneinchError> {
        let route_params = routing::RouteSearchParams {
            from_token: params.from_token.clone(),
            to_token: params.to_token.clone(),
            amount: params.amount.clone(),
            from_address: params.from_address.clone(),
            optimization_criteria: criteria.clone(),
            risk_tolerance: routing::RiskTolerance::Moderate, // Default
            include_gas_optimization: true,
            max_slippage: params.slippage,
        };

        let optimal_route = self.routing_engine.find_optimal_route(&route_params).await?;
        Ok(optimal_route.alternative_routes)
    }

    /// Get optimal quote with oracle comparison - REQUIRES price oracle!
    pub async fn get_optimal_quote_with_oracle(&self, params: &QuoteParams) -> Result<OptimalQuoteWithComparison, OneinchError> {
        if let Some(price_comparison) = &self.price_comparison {
            price_comparison.get_optimal_quote_with_comparison(params)
                .await
                .map_err(|e| OneinchError::OperationFailed(format!("Price comparison failed: {}", e)))
        } else {
            // NO FALLBACK! Oracle comparison requires oracle - fail honestly
            Err(OneinchError::OperationFailed(
                "❌ Price oracle not available - cannot provide oracle comparison. Configure price oracle first!".to_string()
            ))
        }
    }

    /// Find oracle-optimized quote (best quote based on oracle comparison)
    pub async fn get_oracle_optimized_quote(&self, params: &QuoteParams) -> Result<OptimalQuoteWithComparison, OneinchError> {
        if let Some(price_comparison) = &self.price_comparison {
            price_comparison.find_oracle_optimized_quote(params)
                .await
                .map_err(|e| OneinchError::OperationFailed(format!("Oracle optimization failed: {}", e)))
        } else {
            Err(OneinchError::OperationFailed("Price oracle not available for optimization".to_string()))
        }
    }

    /// Check if price oracle comparison is available
    pub fn has_price_oracle(&self) -> bool {
        self.price_comparison.is_some()
    }

    /// Get cross-chain quote using Fusion+
    pub async fn get_cross_chain_quote(&self, request: FusionPlusQuoteRequest) -> Result<FusionPlusQuoteResponse, OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.get_cross_chain_quote(request).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot provide cross-chain quotes. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Build cross-chain order from quote
    pub async fn build_cross_chain_order(&self, quote: &FusionPlusQuoteResponse, preset: &str, wallet_address: &str) -> Result<FusionPlusBuildOrderResponse, OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.build_order(quote, preset, wallet_address).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot build cross-chain orders. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Submit cross-chain order
    pub async fn submit_cross_chain_order(&self, request: FusionPlusSubmitOrderRequest) -> Result<(), OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.submit_order(request).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot submit cross-chain orders. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Get active cross-chain orders
    pub async fn get_active_cross_chain_orders(&self, request: FusionPlusActiveOrdersRequest) -> Result<FusionPlusActiveOrdersResponse, OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.get_active_orders(request).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot get active cross-chain orders. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Get cross-chain order by hash
    pub async fn get_cross_chain_order_by_hash(&self, order_hash: &str) -> Result<ActiveOrder, OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.get_order_by_hash(order_hash).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot get cross-chain order details. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Get escrow factory address for chain
    pub async fn get_escrow_factory(&self, chain_id: u64) -> Result<String, OneinchError> {
        if let Some(fusion_client) = &self.fusion_plus_client {
            fusion_client.get_escrow_factory(chain_id).await
        } else {
            Err(OneinchError::OperationFailed(
                "❌ Fusion+ not enabled - cannot get escrow factory address. Enable Fusion+ first!".to_string()
            ))
        }
    }

    /// Validate 1inch API key by making a test request
    pub async fn validate_api_key(&self) -> Result<bool, OneinchError> {
        self.adapter.client.validate_api_key().await
    }

    /// Get real-time liquidity information for token pair
    pub async fn get_liquidity_info(&self, from_token: &str, to_token: &str) -> Result<serde_json::Value, OneinchError> {
        self.adapter.client.get_liquidity_info(from_token, to_token).await
    }

    /// Perform comprehensive health check of 1inch integration
    pub async fn comprehensive_health_check(&self) -> Result<serde_json::Value, OneinchError> {
        info!("🔍 Performing comprehensive 1inch health check...");

        let mut results = serde_json::json!({
            "chain_id": self.adapter.client.get_chain_id(),
            "chain_supported": self.is_supported_chain(self.adapter.client.get_chain_id()),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Test API connectivity
        match self.adapter.client.health_check().await {
            Ok(healthy) => {
                results["api_connectivity"] = serde_json::json!({
                    "status": if healthy { "healthy" } else { "unhealthy" },
                    "accessible": healthy
                });
            },
            Err(e) => {
                results["api_connectivity"] = serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }

        // Test API key validation
        match self.validate_api_key().await {
            Ok(valid) => {
                results["api_key"] = serde_json::json!({
                    "status": if valid { "valid" } else { "invalid" },
                    "authenticated": valid
                });
            },
            Err(e) => {
                results["api_key"] = serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }

        // Test token listing
        match self.adapter.client.get_tokens().await {
            Ok(tokens) => {
                results["tokens"] = serde_json::json!({
                    "status": "available",
                    "count": tokens.len(),
                    "sample": tokens.iter().take(3).collect::<Vec<_>>()
                });
            },
            Err(e) => {
                results["tokens"] = serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }

        // Test Fusion+ if available
        if let Some(fusion_client) = &self.fusion_plus_client {
            match fusion_client.health_check().await {
                Ok(healthy) => {
                    results["fusion_plus"] = serde_json::json!({
                        "status": if healthy { "healthy" } else { "unhealthy" },
                        "cross_chain_available": healthy
                    });
                },
                Err(e) => {
                    results["fusion_plus"] = serde_json::json!({
                        "status": "error",
                        "error": e.to_string()
                    });
                }
            }
        } else {
            results["fusion_plus"] = serde_json::json!({
                "status": "disabled",
                "cross_chain_available": false
            });
        }

        info!("✅ 1inch health check completed");
        Ok(results)
    }

    /// Check if 1inch is available for given chain
    pub fn is_supported_chain(&self, chain_id: u64) -> bool {
        Self::is_supported_chain_static(chain_id)
    }

    /// Static method to check if chain is supported (for use in constructor)
    fn is_supported_chain_static(chain_id: u64) -> bool {
        matches!(
            chain_id,
            ONEINCH_ETHEREUM_CHAIN_ID
                | ONEINCH_BSC_CHAIN_ID
                | ONEINCH_POLYGON_CHAIN_ID
                | ONEINCH_AVALANCHE_CHAIN_ID
                | ONEINCH_ARBITRUM_CHAIN_ID
                | ONEINCH_OPTIMISM_CHAIN_ID
                | ONEINCH_SEPOLIA_CHAIN_ID
                | ONEINCH_BSC_TESTNET_CHAIN_ID
                | ONEINCH_POLYGON_MUMBAI_CHAIN_ID
        )
    }

    /// Get supported chains list
    pub fn get_supported_chains() -> Vec<u64> {
        vec![
            ONEINCH_ETHEREUM_CHAIN_ID,
            ONEINCH_BSC_CHAIN_ID,
            ONEINCH_POLYGON_CHAIN_ID,
            ONEINCH_AVALANCHE_CHAIN_ID,
            ONEINCH_ARBITRUM_CHAIN_ID,
            ONEINCH_OPTIMISM_CHAIN_ID,
            ONEINCH_SEPOLIA_CHAIN_ID,
            ONEINCH_BSC_TESTNET_CHAIN_ID,
            ONEINCH_POLYGON_MUMBAI_CHAIN_ID,
        ]
    }
}

impl Monitorable for OneinchService {
    fn get_monitor(&self) -> Option<Arc<dyn PerformanceMonitor>> {
        self.performance_monitor.clone()
    }
}

impl SelfValidating for OneinchService {
    type ValidationError = OneinchError;

    fn validate_configuration(&self) -> Result<(), Self::ValidationError> {
        // Check adapter configuration
        if let Err(e) = self.adapter.client.validate_configuration() {
            return Err(e);
        }

        // Check that all components are initialized
        if self.adapter.client.get_chain_id() == 0 {
            return Err(OneinchError::InvalidParams("Chain ID not set".to_string()));
        }

        info!("✅ OneinchService configuration is valid");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_chains() {
        let service = OneinchService::new("test_key".to_string(), ONEINCH_ETHEREUM_CHAIN_ID);
        
        assert!(service.is_supported_chain(ONEINCH_ETHEREUM_CHAIN_ID));
        assert!(service.is_supported_chain(ONEINCH_BSC_CHAIN_ID));
        assert!(service.is_supported_chain(ONEINCH_POLYGON_CHAIN_ID));
        assert!(service.is_supported_chain(ONEINCH_SEPOLIA_CHAIN_ID));
        
        // Test unsupported chain (NEAR)
        assert!(!service.is_supported_chain(1234567890));
    }

    #[test]
    fn test_supported_chains_list() {
        let chains = OneinchService::get_supported_chains();
        assert!(chains.len() >= 6); // At least 6 main chains
        assert!(chains.contains(&ONEINCH_ETHEREUM_CHAIN_ID));
        assert!(chains.contains(&ONEINCH_BSC_CHAIN_ID));
    }
}