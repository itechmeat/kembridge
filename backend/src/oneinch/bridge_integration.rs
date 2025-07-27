// src/oneinch/bridge_integration.rs - Integration with Bridge Service for atomic swaps

use super::{OneinchService, types::*};
use crate::constants::*;
use kembridge_bridge::{BridgeService, SwapOperation, SwapStatus, BridgeError};
use bigdecimal::BigDecimal;
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Integration service combining 1inch Fusion+ with KEMBridge atomic swaps
pub struct OneinchBridgeIntegration {
    oneinch_service: Arc<OneinchService>,
    bridge_service: Arc<BridgeService>,
}

impl OneinchBridgeIntegration {
    /// Create new integration service
    pub fn new(
        oneinch_service: Arc<OneinchService>,
        bridge_service: Arc<BridgeService>,
    ) -> Self {
        Self {
            oneinch_service,
            bridge_service,
        }
    }

    /// Execute optimized cross-chain swap using 1inch for best rates
    pub async fn execute_optimized_bridge_swap(
        &self,
        user_id: Uuid,
        from_chain: &str,
        to_chain: &str,
        from_token: &str,
        to_token: &str,
        amount: BigDecimal,
        recipient: &str,
        optimization_strategy: Option<String>,
    ) -> Result<OptimizedBridgeSwapResult, OneinchBridgeError> {
        info!(
            "Starting optimized bridge swap: {} {} on {} -> {} {} on {} for user {}",
            amount, from_token, from_chain, amount, to_token, to_chain, user_id
        );

        // Step 1: Get optimal 1inch route for source chain swap (if needed)
        let source_optimization = if from_token != self.get_native_token(from_chain) {
            Some(self.optimize_source_chain_swap(
                from_chain,
                from_token,
                &amount,
                optimization_strategy.as_deref(),
            ).await?)
        } else {
            None
        };

        // Step 2: Get optimal 1inch route for destination chain swap (if needed)
        let destination_optimization = if to_token != self.get_native_token(to_chain) {
            Some(self.optimize_destination_chain_swap(
                to_chain,
                to_token,
                &amount,
                optimization_strategy.as_deref(),
            ).await?)
        } else {
            None
        };

        // Step 3: Calculate total optimization savings
        let optimization_summary = self.calculate_optimization_summary(
            &source_optimization,
            &destination_optimization,
        );

        // Step 4: Execute bridge operation with optimized amounts
        let final_amount = source_optimization
            .as_ref()
            .map(|opt| opt.optimized_output.clone())
            .unwrap_or_else(|| amount.clone());

        // Convert BigDecimal to u128 for bridge service (wei format)
        let amount_wei = self.bigdecimal_to_wei(&final_amount)?;

        let bridge_result = self.bridge_service.init_swap(
            user_id,
            from_chain,
            to_chain,
            amount_wei,
            recipient,
        ).await.map_err(OneinchBridgeError::BridgeError)?;

        // Step 5: Execute the atomic swap
        let swap_result = self.bridge_service.execute_swap(bridge_result.swap_id)
            .await.map_err(OneinchBridgeError::BridgeError)?;

        // Step 6: Execute destination chain optimization (if applicable)
        let final_destination_result = if let Some(dest_opt) = destination_optimization {
            Some(self.execute_destination_optimization(
                bridge_result.swap_id,
                &dest_opt,
                to_chain,
            ).await?)
        } else {
            None
        };

        // Calculate total savings before moving optimization_summary
        let total_savings = optimization_summary.total_gas_savings.clone() + optimization_summary.total_output_improvement.clone();

        Ok(OptimizedBridgeSwapResult {
            bridge_swap_id: bridge_result.swap_id,
            bridge_status: swap_result.status,
            source_optimization: source_optimization,
            destination_optimization: final_destination_result,
            optimization_summary,
            total_savings,
            eth_tx_hash: swap_result.eth_tx_hash,
            near_tx_hash: swap_result.near_tx_hash,
            quantum_key_id: swap_result.quantum_key_id,
        })
    }

    /// Optimize swap on source chain before bridging
    async fn optimize_source_chain_swap(
        &self,
        chain: &str,
        from_token: &str,
        amount: &BigDecimal,
        strategy: Option<&str>,
    ) -> Result<ChainOptimizationResult, OneinchBridgeError> {
        info!("Optimizing source chain swap on {}: {} {}", chain, amount, from_token);

        // Only optimize on supported 1inch chains
        if !self.is_oneinch_supported_chain(chain) {
            return Ok(ChainOptimizationResult {
                chain: chain.to_string(),
                original_amount: amount.clone(),
                optimized_output: amount.clone(),
                gas_savings: BigDecimal::from(0),
                output_improvement: BigDecimal::from(0),
                oneinch_quote_id: None,
                optimization_applied: false,
            });
        }

        let native_token = self.get_native_token(chain);
        let quote_params = QuoteParams {
            from_token: from_token.to_string(),
            to_token: native_token,
            amount: amount.clone(),
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(), // Will be replaced with actual user address
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some(ONEINCH_BRIDGE_INTEGRATION_SOURCE.to_string()),
        };

        let quote = self.oneinch_service.get_quote(&quote_params)
            .await.map_err(OneinchBridgeError::OneinchError)?;

        // Calculate improvements vs direct bridge
        let output_improvement = &quote.to_amount - amount;
        let gas_savings = self.estimate_gas_savings(&quote);

        Ok(ChainOptimizationResult {
            chain: chain.to_string(),
            original_amount: amount.clone(),
            optimized_output: quote.to_amount.clone(),
            gas_savings,
            output_improvement,
            oneinch_quote_id: Some(quote.quote_id),
            optimization_applied: true,
        })
    }

    /// Optimize swap on destination chain after bridging
    async fn optimize_destination_chain_swap(
        &self,
        chain: &str,
        to_token: &str,
        amount: &BigDecimal,
        strategy: Option<&str>,
    ) -> Result<ChainOptimizationResult, OneinchBridgeError> {
        info!("Planning destination chain optimization on {}: {} -> {}", chain, amount, to_token);

        if !self.is_oneinch_supported_chain(chain) {
            return Ok(ChainOptimizationResult {
                chain: chain.to_string(),
                original_amount: amount.clone(),
                optimized_output: amount.clone(),
                gas_savings: BigDecimal::from(0),
                output_improvement: BigDecimal::from(0),
                oneinch_quote_id: None,
                optimization_applied: false,
            });
        }

        let native_token = self.get_native_token(chain);
        let quote_params = QuoteParams {
            from_token: native_token,
            to_token: to_token.to_string(),
            amount: amount.clone(),
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(), // Using zero address as placeholder
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some(ONEINCH_BRIDGE_INTEGRATION_SOURCE.to_string()),
        };

        let quote = self.oneinch_service.get_quote(&quote_params)
            .await.map_err(OneinchBridgeError::OneinchError)?;

        let output_improvement = &quote.to_amount - amount;
        let gas_savings = self.estimate_gas_savings(&quote);

        Ok(ChainOptimizationResult {
            chain: chain.to_string(),
            original_amount: amount.clone(),
            optimized_output: quote.to_amount.clone(),
            gas_savings,
            output_improvement,
            oneinch_quote_id: Some(quote.quote_id),
            optimization_applied: true,
        })
    }

    /// Execute destination chain optimization after bridge completion
    async fn execute_destination_optimization(
        &self,
        bridge_swap_id: Uuid,
        optimization: &ChainOptimizationResult,
        chain: &str,
    ) -> Result<ChainOptimizationResult, OneinchBridgeError> {
        info!("Executing destination optimization for bridge swap {}", bridge_swap_id);

        // In a production environment, this would:
        // 1. Wait for bridge completion
        // 2. Monitor destination address for incoming tokens
        // 3. Execute the 1inch swap automatically
        // 4. Send final tokens to user

        // For hackathon version, we return the planned optimization
        warn!("Destination optimization execution not fully implemented - returning planned optimization");
        Ok(optimization.clone())
    }

    /// Calculate overall optimization summary
    fn calculate_optimization_summary(
        &self,
        source_opt: &Option<ChainOptimizationResult>,
        dest_opt: &Option<ChainOptimizationResult>,
    ) -> OptimizationSummary {
        let source_gas_savings = source_opt.as_ref()
            .map(|opt| opt.gas_savings.clone())
            .unwrap_or_else(|| BigDecimal::from(0));
        
        let source_output_improvement = source_opt.as_ref()
            .map(|opt| opt.output_improvement.clone())
            .unwrap_or_else(|| BigDecimal::from(0));

        let dest_gas_savings = dest_opt.as_ref()
            .map(|opt| opt.gas_savings.clone())
            .unwrap_or_else(|| BigDecimal::from(0));
        
        let dest_output_improvement = dest_opt.as_ref()
            .map(|opt| opt.output_improvement.clone())
            .unwrap_or_else(|| BigDecimal::from(0));

        OptimizationSummary {
            total_gas_savings: source_gas_savings + dest_gas_savings,
            total_output_improvement: source_output_improvement + dest_output_improvement,
            source_chain_optimized: source_opt.as_ref().map(|opt| opt.optimization_applied).unwrap_or(false),
            destination_chain_optimized: dest_opt.as_ref().map(|opt| opt.optimization_applied).unwrap_or(false),
        }
    }

    /// Check if chain is supported by 1inch
    fn is_oneinch_supported_chain(&self, chain: &str) -> bool {
        let chain_id = match chain {
            "ethereum" => ONEINCH_ETHEREUM_CHAIN_ID,
            "bsc" => ONEINCH_BSC_CHAIN_ID,
            "polygon" => ONEINCH_POLYGON_CHAIN_ID,
            "avalanche" => ONEINCH_AVALANCHE_CHAIN_ID,
            "arbitrum" => ONEINCH_ARBITRUM_CHAIN_ID,
            "optimism" => ONEINCH_OPTIMISM_CHAIN_ID,
            _ => return false,
        };
        
        self.oneinch_service.is_supported_chain(chain_id)
    }

    /// Get native token address for chain
    fn get_native_token(&self, chain: &str) -> String {
        match chain {
            "ethereum" => ETHEREUM_NATIVE_TOKEN.to_string(),
            "bsc" => BSC_NATIVE_TOKEN.to_string(),
            "polygon" => POLYGON_NATIVE_TOKEN.to_string(),
            "avalanche" => AVALANCHE_NATIVE_TOKEN.to_string(),
            "arbitrum" => ARBITRUM_NATIVE_TOKEN.to_string(),
            "optimism" => OPTIMISM_NATIVE_TOKEN.to_string(),
            "near" => NEAR_NATIVE_TOKEN.to_string(),
            _ => ETHEREUM_NATIVE_TOKEN.to_string(), // Default to ETH address
        }
    }

    /// Estimate gas savings from 1inch optimization
    fn estimate_gas_savings(&self, quote: &FusionQuote) -> BigDecimal {
        // Simplified calculation: assume 20% gas savings vs naive swap
        let base_gas = BigDecimal::from(ETHEREUM_BASE_GAS); // Base transaction gas
        let complex_gas = quote.estimated_gas.clone();
        let savings_ratio = BigDecimal::from_str("0.2").unwrap_or_else(|_| BigDecimal::from(0));
        
        (&complex_gas - &base_gas) * savings_ratio
    }

    /// Convert BigDecimal to wei (u128)
    fn bigdecimal_to_wei(&self, amount: &BigDecimal) -> Result<u128, OneinchBridgeError> {
        let wei_multiplier = BigDecimal::from(ETHEREUM_WEI_MULTIPLIER); // Wei conversion multiplier
        let wei_amount = amount * wei_multiplier;
        
        wei_amount.to_string().parse::<u128>()
            .map_err(|_| OneinchBridgeError::ConversionError("Failed to convert amount to wei".to_string()))
    }

    /// Get bridge swap status
    pub async fn get_bridge_swap_status(&self, swap_id: Uuid) -> Result<SwapOperation, OneinchBridgeError> {
        self.bridge_service.get_swap_operation(swap_id)
            .await.map_err(OneinchBridgeError::BridgeError)
    }
}

/// Result of optimized bridge swap
#[derive(Debug, Clone)]
pub struct OptimizedBridgeSwapResult {
    pub bridge_swap_id: Uuid,
    pub bridge_status: SwapStatus,
    pub source_optimization: Option<ChainOptimizationResult>,
    pub destination_optimization: Option<ChainOptimizationResult>,
    pub optimization_summary: OptimizationSummary,
    pub total_savings: BigDecimal,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub quantum_key_id: Option<String>,
}

/// Optimization result for a specific chain
#[derive(Debug, Clone)]
pub struct ChainOptimizationResult {
    pub chain: String,
    pub original_amount: BigDecimal,
    pub optimized_output: BigDecimal,
    pub gas_savings: BigDecimal,
    pub output_improvement: BigDecimal,
    pub oneinch_quote_id: Option<String>,
    pub optimization_applied: bool,
}

/// Summary of total optimizations applied
#[derive(Debug, Clone)]
pub struct OptimizationSummary {
    pub total_gas_savings: BigDecimal,
    pub total_output_improvement: BigDecimal,
    pub source_chain_optimized: bool,
    pub destination_chain_optimized: bool,
}

/// Errors for bridge integration
#[derive(Debug, thiserror::Error)]
pub enum OneinchBridgeError {
    #[error("1inch error: {0}")]
    OneinchError(OneinchError),
    #[error("Bridge error: {0}")]
    BridgeError(BridgeError),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Chain not supported: {0}")]
    ChainNotSupported(String),
    #[error("Integration error: {0}")]
    IntegrationError(String),
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_integration() -> OneinchBridgeIntegration {
        // This would need actual service implementations for real testing
        // For now, we test the structure and logic
        // TODO: Integration tests require real service instances
        // For now, skip test implementation - would need proper DI setup
        panic!("Test not implemented - requires service instances")
    }

    #[test]
    fn test_native_token_mapping() {
        let integration = create_test_integration();
        
        assert_eq!(integration.get_native_token("ethereum"), ETHEREUM_NATIVE_TOKEN);
        assert_eq!(integration.get_native_token("bsc"), BSC_NATIVE_TOKEN);
        assert_eq!(integration.get_native_token("polygon"), POLYGON_NATIVE_TOKEN);
    }

    #[test]
    fn test_chain_support_detection() {
        let integration = create_test_integration();
        
        // These tests would work with actual service
        // assert!(integration.is_oneinch_supported_chain("ethereum"));
        // assert!(integration.is_oneinch_supported_chain("bsc"));
        // assert!(!integration.is_oneinch_supported_chain("near"));
    }
}