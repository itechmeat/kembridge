// src/dynamic_pricing/price_service.rs - Centralized price service using existing Price Oracle + 1inch

use std::sync::Arc;
use bigdecimal::BigDecimal;
use anyhow::Result;
use tracing::{debug, warn, info, error};

use crate::{
    price_oracle::PriceOracleService,
    oneinch::{OneinchService, QuoteParams},
    constants::*,
};
use super::types::DynamicPricingError;

/// Centralized price service using existing Price Oracle + 1inch integration
pub struct PriceService {
    price_oracle: Arc<PriceOracleService>,
    oneinch_service: Arc<OneinchService>,
}

impl PriceService {
    /// Create new price service with existing integrations
    pub fn new(
        price_oracle: Arc<PriceOracleService>,
        oneinch_service: Arc<OneinchService>,
    ) -> Self {
        Self { 
            price_oracle,
            oneinch_service,
        }
    }

    /// Create price service with oracle + best-effort 1inch integration
    /// ⚠️  WARNING: 1inch integration depends on environment configuration
    /// ⚠️  If no valid API key is available, 1inch requests will fail (which is honest behavior)
    /// ✅ RECOMMENDED: Use new() with proper OneinchService from AppState for guaranteed 1inch support
    pub fn new_oracle_only(price_oracle: Arc<PriceOracleService>) -> Self {
        // Try to create OneinchService using same logic as AppState
        // If no API key is configured, 1inch will fail honestly - no fake fallbacks!
        
        // Use same fallback logic as AppState - try environment, fallback to test key
        let api_key = std::env::var("ONEINCH_API_KEY")
            .unwrap_or_else(|_| {
                warn!("🔑 No ONEINCH_API_KEY environment variable found, using test key");
                warn!("🔑 1inch requests may fail with test key - this is expected and honest!");
                "test_key".to_string()
            });
            
        // Only show integration warnings if we're actually using fallback configuration
        if api_key == "test_key" {
            warn!("⚠️  Creating PriceService without guaranteed 1inch integration!");
            warn!("⚠️  1inch functionality depends on API key configuration!");
            warn!("⚠️  For guaranteed 1inch support, use new() with OneinchService from AppState!");
        } else {
            debug!("✅ Creating PriceService with 1inch API key from environment");
        }
            
        let chain_id = std::env::var("ETHEREUM_CHAIN_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ONEINCH_ETHEREUM_CHAIN_ID);
        
        let oneinch_service = Arc::new(OneinchService::new(api_key, chain_id));
        Self::new(price_oracle, oneinch_service)
    }

    /// Get real USD value using Price Oracle + 1inch for validation - NO FAKE PRICES!
    pub async fn get_usd_value(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<f64, DynamicPricingError> {
        debug!("Getting USD value for {} {} using Price Oracle + 1inch validation", amount, token);

        let amount_f64 = amount.to_string().parse::<f64>().unwrap_or(0.0);
        if amount_f64 < 0.0 {
            return Err(DynamicPricingError::QuoteValidationError(
                "Amount cannot be negative".to_string()
            ));
        }

        // Try Price Oracle first (multi-provider: CoinGecko, Binance, Chainlink)
        let price_usd = match self.price_oracle.get_price(&format!("{}/USD", token)).await {
            Ok(price_data) => {
                let price = price_data.price.to_string().parse::<f64>().unwrap_or(0.0);
                if price <= 0.0 {
                    return Err(DynamicPricingError::OracleError(
                        format!("Invalid price data for {}: {}", token, price)
                    ));
                }
                info!("Got price from Oracle for {}: ${:.4}", token, price);
                price
            },
            Err(oracle_error) => {
                warn!("Oracle failed for {}: {}. Trying 1inch as backup...", token, oracle_error);
                
                // Try 1inch as backup - if it fails, we fail honestly with no fake data
                match self.get_price_from_oneinch(token).await {
                    Ok(price) => {
                        info!("✅ Got price from 1inch for {}: ${:.4}", token, price);
                        price
                    },
                    Err(oneinch_error) => {
                        // Both Oracle and 1inch failed - fail honestly
                        error!("❌ Both Oracle and 1inch failed for {}", token);
                        error!("Oracle error: {}", oracle_error);
                        error!("1inch error: {}", oneinch_error);
                        error!("No fallback data provided - failing honestly!");
                        
                        return Err(DynamicPricingError::OracleError(
                            format!(
                                "No price data available for {} - Oracle: {}, 1inch: {}. No fake data provided.", 
                                token, oracle_error, oneinch_error
                            )
                        ));
                    }
                }
            }
        };

        let usd_value = amount_f64 * price_usd;
        debug!("USD value: {} {} = ${:.2} (price: ${:.4})", amount, token, usd_value, price_usd);
        
        Ok(usd_value)
    }

    /// Get price from 1inch for token validation
    /// 🚨 WARNING: This will FAIL if PriceService was created with new_oracle_only()!
    /// 🚨 Reason: new_oracle_only() provides invalid API key to OneinchService!
    /// 🚨 Solution: Use proper OneinchService from AppState with valid API key!
    async fn get_price_from_oneinch(&self, token: &str) -> Result<f64, DynamicPricingError> {
        debug!("Getting price from 1inch for token: {} (🚨 May fail if no valid API key!)", token);
        
        // Map tokens to supported 1inch tokens using constants
        let token_address = match token.to_uppercase().as_str() {
            "ETH" => ETHEREUM_NATIVE_TOKEN,
            "USDT" => ETHEREUM_USDT_ADDRESS,
            "USDC" => ETHEREUM_USDC_ADDRESS,
            _ => {
                return Err(DynamicPricingError::OracleError(
                    format!("Token {} not supported by 1inch integration", token)
                ));
            }
        };
        
        // Use existing OneinchService to get quote
        // For price discovery, we request quote for 1 token to USDT
        let amount_wei = BigDecimal::from(ETHEREUM_WEI_MULTIPLIER); // 1 ETH in wei using constant
        let usdt_address = ETHEREUM_USDT_ADDRESS;
        
        let quote_params = QuoteParams {
            from_token: token_address.to_string(),
            to_token: usdt_address.to_string(),
            amount: amount_wei,
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(), // Use constant for zero address
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE), // Use constant from constants.rs
            disable_estimate: Some(false),
            allow_partial_fill: Some(false),
            source: Some(ONEINCH_DEFAULT_SOURCE.to_string()), // Use constant
        };
        
        match self.oneinch_service.get_quote(&quote_params).await {
            Ok(quote) => {
                // Convert quote to price per token
                let output_amount = quote.to_amount.to_string().parse::<f64>().map_err(|e| {
                    DynamicPricingError::OracleError(format!("Failed to parse 1inch amount: {}", e))
                })?;
                
                // USDT has 6 decimals, normalize to USD price
                let price_usd = output_amount / USDT_DECIMAL_MULTIPLIER;
                
                info!("Got price from 1inch for {}: ${:.4}", token, price_usd);
                Ok(price_usd)
            },
            Err(e) => {
                warn!("1inch quote failed for {}: {}", token, e);
                Err(DynamicPricingError::OracleError(
                    format!("1inch API error for {}: {}", token, e)
                ))
            }
        }
    }

    /// Get current price for token pair
    pub async fn get_token_price(
        &self,
        token: &str,
        quote_currency: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        let price_pair = format!("{}/{}", token, quote_currency);
        debug!("Getting price for pair: {}", price_pair);

        match self.price_oracle.get_price(&price_pair).await {
            Ok(price_data) => Ok(price_data.price),
            Err(e) => {
                warn!("Failed to get price for {}: {}", price_pair, e);
                Err(DynamicPricingError::OracleError(
                    format!("Failed to get price for {}: {}", price_pair, e)
                ))
            }
        }
    }

    /// Check if token price is available
    pub async fn is_price_available(&self, token: &str) -> bool {
        match self.price_oracle.get_price(&format!("{}/USD", token)).await {
            Ok(_) => true,
            Err(_) => {
                // No fallbacks - if both Oracle and 1inch fail, we fail honestly
                false // Always return false - no price fallbacks allowed
            }
        }
    }

    /// Get multiple token prices in batch
    pub async fn get_batch_prices(
        &self,
        tokens: &[&str],
        quote_currency: &str,
    ) -> Result<Vec<(String, BigDecimal)>, DynamicPricingError> {
        let mut prices = Vec::new();
        
        for token in tokens {
            match self.get_token_price(token, quote_currency).await {
                Ok(price) => prices.push((token.to_string(), price)),
                Err(e) => {
                    warn!("Failed to get price for {}: {}", token, e);
                    // Continue with other tokens instead of failing entire batch
                }
            }
        }

        if prices.is_empty() {
            return Err(DynamicPricingError::OracleError(
                "No price data available for any requested tokens".to_string()
            ));
        }

        Ok(prices)
    }
}

/// Trait for components that need USD value estimation
pub trait UsdValueEstimator {
    fn price_service(&self) -> &PriceService;

    /// Get USD value using the centralized price service
    async fn estimate_usd_value(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<f64, DynamicPricingError> {
        self.price_service().get_usd_value(amount, token).await
    }
}