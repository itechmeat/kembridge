// src/price_oracle/providers/oneinch.rs - 1inch Price Provider integration

use async_trait::async_trait;
use bigdecimal::{BigDecimal as Decimal, FromPrimitive};
use chrono::Utc;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::constants::{*, ONEINCH_TEST_FROM_ADDRESS};
use crate::oneinch::{types::QuoteParams, OneinchService};
use crate::price_oracle::types::{PriceData, PriceError, PriceProvider, TradingPair};
use crate::utils::token_mapping::symbol_to_token_address;

/// 1inch provider implementation for real market prices
pub struct OneinchPriceProvider {
    oneinch_service: Arc<OneinchService>,
    config: Arc<AppConfig>,
}

impl OneinchPriceProvider {
    /// Create new 1inch price provider
    pub fn new(oneinch_service: Arc<OneinchService>, config: Arc<AppConfig>) -> Self {
        info!("OneinchPriceProvider initialized");
        Self {
            oneinch_service,
            config,
        }
    }

    /// Get price from 1inch by getting a quote to USDT
    async fn get_oneinch_price(&self, pair: TradingPair) -> Result<PriceData, PriceError> {
        debug!("Getting 1inch price for pair: {:?}", pair);

        let (from_token, to_token) = match pair {
            TradingPair::EthUsd => ("ETH", "USDT"),
            TradingPair::NearUsd => {
                // NEAR is not supported on 1inch (not an EVM token)
                return Err(PriceError::ProviderUnavailable("1inch does not support NEAR token (non-EVM)".to_string()));
            },
            TradingPair::BtcUsd => ("WBTC", "USDT"),
            TradingPair::UsdtUsd => return Ok(self.create_stable_price("USDT")),
            TradingPair::UsdcUsd => ("USDC", "USDT"),
        };

        // For stablecoins, return 1.0
        if from_token == to_token {
            return Ok(self.create_stable_price(from_token));
        }

        // Use 1 token as base amount for price discovery
        let amount = match from_token {
            "ETH" => Decimal::from_str("1000000000000000000")
                .map_err(|e| PriceError::ValidationFailed(format!("Invalid ETH amount: {}", e)))?, // 1 ETH in wei
            "WBTC" => Decimal::from_str("100000000")
                .map_err(|e| PriceError::ValidationFailed(format!("Invalid WBTC amount: {}", e)))?, // 1 WBTC in satoshi
            "NEAR" => Decimal::from_str("1000000000000000000000000")
                .map_err(|e| PriceError::ValidationFailed(format!("Invalid NEAR amount: {}", e)))?, // 1 NEAR in yocto
            "USDC" => Decimal::from_str("1000000")
                .map_err(|e| PriceError::ValidationFailed(format!("Invalid USDC amount: {}", e)))?, // 1 USDC (6 decimals)
            _ => Decimal::from(1),
        };

        // Convert symbols to contract addresses for 1inch API
        let from_token_address = symbol_to_token_address(from_token)
            .map_err(|e| PriceError::ValidationFailed(format!("Invalid from_token symbol {}: {}", from_token, e)))?;
        let to_token_address = symbol_to_token_address(to_token)
            .map_err(|e| PriceError::ValidationFailed(format!("Invalid to_token symbol {}: {}", to_token, e)))?;

        debug!("Converted tokens: {} -> {}, {} -> {}", from_token, from_token_address, to_token, to_token_address);

        let quote_params = QuoteParams {
            from_token: from_token_address,
            to_token: to_token_address,
            amount,
            from_address: ONEINCH_TEST_FROM_ADDRESS.to_string(),
            slippage: Some(0.5),
            disable_estimate: Some(true),
            allow_partial_fill: Some(false),
            source: None,
        };

        match self.oneinch_service.get_quote(&quote_params).await {
            Ok(quote) => {
                // Calculate price accounting for token decimals
                // 1inch returns USDT in 6 decimals, but we sent ETH in 18 decimals
                let price = match pair {
                    TradingPair::EthUsd => {
                        // Convert: (USDT amount in 6 decimals) / (ETH amount in 18 decimals) * 10^12
                        let usdt_decimals_factor = Decimal::from(1000000); // 10^6 for USDT
                        let eth_decimals_factor = Decimal::from_str("1000000000000000000").unwrap(); // 10^18 for ETH
                        
                        (&quote.to_amount * &eth_decimals_factor) / (&quote.from_amount * &usdt_decimals_factor)
                    },
                    TradingPair::BtcUsd => {
                        // Convert: (USDT amount in 6 decimals) / (WBTC amount in 8 decimals) * 10^(-2)
                        let usdt_decimals_factor = Decimal::from(1000000); // 10^6 for USDT  
                        let wbtc_decimals_factor = Decimal::from(100000000); // 10^8 for WBTC
                        
                        (&quote.to_amount * &wbtc_decimals_factor) / (&quote.from_amount * &usdt_decimals_factor)
                    },
                    TradingPair::UsdcUsd => {
                        // Both USDC and USDT have 6 decimals, so simple division
                        &quote.to_amount / &quote.from_amount
                    },
                    _ => {
                        // Fallback: simple division (may not be accurate for all pairs)
                        &quote.to_amount / &quote.from_amount
                    }
                };

                Ok(PriceData {
                    symbol: pair.to_symbol().to_string(),
                    price,
                    timestamp: Utc::now(),
                    source: "1inch".to_string(),
                    confidence: 0.95, // 1inch aggregates multiple DEXs, high confidence
                    volume_24h: None,
                    change_24h: None,
                })
            }
            Err(e) => {
                error!("Failed to get 1inch quote for {}: {}", pair.to_symbol(), e);
                Err(PriceError::ProviderUnavailable(format!(
                    "1inch quote failed: {}",
                    e
                )))
            }
        }
    }

    /// Create stable price data for stablecoins
    fn create_stable_price(&self, symbol: &str) -> PriceData {
        PriceData {
            symbol: format!("{}/USD", symbol),
            price: Decimal::from(1),
            timestamp: Utc::now(),
            source: "1inch-stable".to_string(),
            confidence: 0.99,
            volume_24h: None,
            change_24h: None,
        }
    }
}

#[async_trait]
impl PriceProvider for OneinchPriceProvider {
    async fn get_price(&self, symbol: &str) -> Result<PriceData, PriceError> {
        let pair = TradingPair::from_symbol(symbol)
            .ok_or_else(|| PriceError::InvalidSymbol(symbol.to_string()))?;

        info!("Getting 1inch price for {}", symbol);

        match self.get_oneinch_price(pair).await {
            Ok(price_data) => {
                info!(
                    "Successfully got 1inch price for {}: ${}",
                    symbol, price_data.price
                );
                Ok(price_data)
            }
            Err(e) => {
                error!("Failed to get 1inch price for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<PriceData>, PriceError> {
        info!(
            "Getting multiple 1inch prices for {} symbols",
            symbols.len()
        );

        let mut results = Vec::new();

        for &symbol in symbols {
            match self.get_price(symbol).await {
                Ok(price_data) => results.push(price_data),
                Err(e) => {
                    warn!("Failed to get price for {} from 1inch: {}", symbol, e);
                    // Continue with other symbols instead of failing the entire batch
                }
            }
        }

        if results.is_empty() {
            return Err(PriceError::ProviderUnavailable(
                "No prices available from 1inch".to_string(),
            ));
        }

        Ok(results)
    }

    fn provider_name(&self) -> &str {
        "1inch"
    }

    fn is_available(&self) -> bool {
        true // 1inch is always available if properly configured
    }

    fn get_supported_symbols(&self) -> Vec<String> {
        vec![
            "ETH/USD".to_string(),
            "NEAR/USD".to_string(),
            "BTC/USD".to_string(),
            "USDT/USD".to_string(),
            "USDC/USD".to_string(),
        ]
    }
}
