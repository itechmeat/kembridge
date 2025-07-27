// src/oneinch/mod.rs - 1inch Fusion+ Integration

pub mod adapter;
pub mod client;
pub mod types;
pub mod quote_engine;
pub mod slippage;
pub mod routing;
pub mod bridge_integration;
pub mod price_comparison;

pub use adapter::OneinchAdapter;
pub use client::FusionClient;
pub use types::*;
pub use quote_engine::QuoteEngine;
pub use slippage::SlippageProtection;
pub use routing::RoutingEngine;
pub use bridge_integration::{OneinchBridgeIntegration, OptimizedBridgeSwapResult, OneinchBridgeError};
pub use price_comparison::{PriceComparisonService, OptimalQuoteWithComparison, PriceComparisonError};

use crate::constants::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main service for 1inch Fusion+ integration
pub struct OneinchService {
    pub adapter: Arc<OneinchAdapter>,
    pub quote_engine: Arc<QuoteEngine>,
    pub slippage_protection: Arc<SlippageProtection>,
    pub routing_engine: Arc<RoutingEngine>,
    pub price_comparison: Option<Arc<PriceComparisonService>>,
}

impl OneinchService {
    /// Create new OneinchService instance
    pub fn new(api_key: String, chain_id: u64) -> Self {
        let fusion_client = Arc::new(FusionClient::new(api_key, chain_id));
        let adapter = Arc::new(OneinchAdapter::new(fusion_client.clone()));
        let quote_engine = Arc::new(QuoteEngine::new(fusion_client.clone()));
        let slippage_protection = Arc::new(SlippageProtection::new());
        let routing_engine = Arc::new(RoutingEngine::new(fusion_client.clone()));

        Self {
            adapter,
            quote_engine,
            slippage_protection,
            routing_engine,
            price_comparison: None, // Will be set when price oracle is available
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

    /// Clone without price comparison to avoid circular reference
    fn clone_without_comparison(&self) -> OneinchService {
        OneinchService {
            adapter: self.adapter.clone(),
            quote_engine: self.quote_engine.clone(),
            slippage_protection: self.slippage_protection.clone(),
            routing_engine: self.routing_engine.clone(),
            price_comparison: None,
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

    /// Get optimal quote with oracle comparison (if price oracle is available)
    pub async fn get_optimal_quote_with_oracle(&self, params: &QuoteParams) -> Result<OptimalQuoteWithComparison, OneinchError> {
        if let Some(price_comparison) = &self.price_comparison {
            price_comparison.get_optimal_quote_with_comparison(params)
                .await
                .map_err(|e| OneinchError::OperationFailed(format!("Price comparison failed: {}", e)))
        } else {
            // Fallback to regular quote if no oracle available
            let quote = self.get_quote(params).await?;
            Err(OneinchError::OperationFailed("Price oracle not available for comparison".to_string()))
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

    /// Check if 1inch is available for given chain
    pub fn is_supported_chain(&self, chain_id: u64) -> bool {
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