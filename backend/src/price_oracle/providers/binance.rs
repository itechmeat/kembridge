// src/price_oracle/providers/binance.rs - Binance API integration
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use bigdecimal::{BigDecimal as Decimal, FromPrimitive};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use crate::config::AppConfig;
use crate::constants::*;
use crate::price_oracle::types::{PriceProvider, PriceData, PriceError, TradingPair};

/// Binance API 24hr ticker response
#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
    volume: String,
    count: u64,
}

/// Binance API price response
#[derive(Debug, Deserialize)]
struct BinancePrice {
    symbol: String,
    price: String,
}

/// Binance provider implementation
pub struct BinanceProvider {
    client: Client,
    config: Arc<AppConfig>,
    symbol_mappings: HashMap<TradingPair, String>,
    is_available: bool,
}

impl BinanceProvider {
    /// Create new Binance provider
    pub async fn new(config: Arc<AppConfig>) -> Result<Self, PriceError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_CLIENT_TIMEOUT))
            .user_agent("KEMBridge/1.0")
            .build()
            .map_err(|e| PriceError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
        
        // Binance symbol mappings
        let mut symbol_mappings = HashMap::new();
        symbol_mappings.insert(TradingPair::EthUsd, "ETHUSDT".to_string());
        symbol_mappings.insert(TradingPair::BtcUsd, "BTCUSDT".to_string());
        symbol_mappings.insert(TradingPair::UsdtUsd, "USDTUSD".to_string());
        symbol_mappings.insert(TradingPair::UsdcUsd, "USDCUSDT".to_string());
        // Note: NEAR is not typically available on Binance, so we'll handle it separately
        
        let provider = Self {
            client,
            config,
            symbol_mappings,
            is_available: true,
        };
        
        info!("BinanceProvider initialized with {} symbol mappings", provider.symbol_mappings.len());
        Ok(provider)
    }
    
    /// Get price from Binance API
    async fn get_binance_price(&self, pair: TradingPair) -> Result<PriceData, PriceError> {
        let binance_symbol = self.symbol_mappings.get(&pair)
            .ok_or_else(|| PriceError::InvalidSymbol(pair.to_symbol().to_string()))?;
        
        // Use ticker endpoint for more detailed data
        let url = format!("{}/ticker/24hr?symbol={}", BINANCE_API_BASE, binance_symbol);
        
        let response = self.client.get(&url).send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 429 {
                return Err(PriceError::RateLimitExceeded("Binance".to_string()));
            }
            return Err(PriceError::ProviderUnavailable(
                format!("Binance API error: {}", response.status())
            ));
        }
        
        let ticker: BinanceTicker = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse Binance response: {}", e)))?;
        
        let price = ticker.last_price.parse::<f64>()
            .map_err(|e| PriceError::InvalidPriceData(format!("Invalid price format: {}", e)))?;
        
        let change_24h = ticker.price_change_percent.parse::<f64>().ok();
        
        let volume_24h = ticker.volume.parse::<f64>().ok()
            .and_then(|v| Decimal::from_f64(v));
        
        Ok(PriceData {
            symbol: pair.to_symbol().to_string(),
            price: Decimal::from_f64(price)
                .ok_or_else(|| PriceError::InvalidPriceData("Invalid price value".to_string()))?,
            timestamp: Utc::now(),
            source: "binance".to_string(),
            confidence: PRICE_CONFIDENCE_BINANCE,
            volume_24h,
            change_24h,
        })
    }
    
    /// Get multiple prices using batch endpoint
    async fn get_multiple_binance_prices(&self, pairs: &[TradingPair]) -> Result<Vec<PriceData>, PriceError> {
        let binance_symbols: Vec<String> = pairs.iter()
            .filter_map(|pair| self.symbol_mappings.get(pair))
            .cloned()
            .collect();
        
        if binance_symbols.is_empty() {
            return Err(PriceError::InvalidSymbol("No valid Binance symbols".to_string()));
        }
        
        // Use ticker endpoint for all symbols
        let url = format!("{}/ticker/24hr", BINANCE_API_BASE);
        
        let response = self.client.get(&url).send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 429 {
                return Err(PriceError::RateLimitExceeded("Binance".to_string()));
            }
            return Err(PriceError::ProviderUnavailable(
                format!("Binance API error: {}", response.status())
            ));
        }
        
        let tickers: Vec<BinanceTicker> = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse Binance response: {}", e)))?;
        
        let mut results = Vec::new();
        
        for pair in pairs {
            if let Some(binance_symbol) = self.symbol_mappings.get(pair) {
                if let Some(ticker) = tickers.iter().find(|t| t.symbol == *binance_symbol) {
                    if let Ok(price) = ticker.last_price.parse::<f64>() {
                        let change_24h = ticker.price_change_percent.parse::<f64>().ok();
                        let volume_24h = ticker.volume.parse::<f64>().ok()
                            .and_then(|v| Decimal::from_f64(v));
                        
                        if let Some(price_decimal) = Decimal::from_f64(price) {
                            results.push(PriceData {
                                symbol: pair.to_symbol().to_string(),
                                price: price_decimal,
                                timestamp: Utc::now(),
                                source: "binance".to_string(),
                                confidence: PRICE_CONFIDENCE_BINANCE,
                                volume_24h,
                                change_24h,
                            });
                        }
                    }
                }
            }
        }
        
        if results.is_empty() {
            return Err(PriceError::PriceNotFound("No valid prices found".to_string()));
        }
        
        Ok(results)
    }
    
    /// Get simple price (faster endpoint)
    async fn get_simple_price(&self, pair: TradingPair) -> Result<PriceData, PriceError> {
        let binance_symbol = self.symbol_mappings.get(&pair)
            .ok_or_else(|| PriceError::InvalidSymbol(pair.to_symbol().to_string()))?;
        
        let url = format!("{}/ticker/price?symbol={}", BINANCE_API_BASE, binance_symbol);
        
        let response = self.client.get(&url).send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            return Err(PriceError::ProviderUnavailable(
                format!("Binance API error: {}", response.status())
            ));
        }
        
        let price_data: BinancePrice = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse Binance response: {}", e)))?;
        
        let price = price_data.price.parse::<f64>()
            .map_err(|e| PriceError::InvalidPriceData(format!("Invalid price format: {}", e)))?;
        
        Ok(PriceData {
            symbol: pair.to_symbol().to_string(),
            price: Decimal::from_f64(price)
                .ok_or_else(|| PriceError::InvalidPriceData("Invalid price value".to_string()))?,
            timestamp: Utc::now(),
            source: "binance".to_string(),
            confidence: PRICE_CONFIDENCE_BINANCE,
            volume_24h: None,
            change_24h: None,
        })
    }
    
    /// Check if provider is healthy
    pub async fn health_check(&self) -> bool {
        let url = &format!("{}/ping", BINANCE_API_BASE);
        
        match self.client.get(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    /// Get server time (for synchronization)
    pub async fn get_server_time(&self) -> Result<DateTime<Utc>, PriceError> {
        let url = &format!("{}/time", BINANCE_API_BASE);
        
        let response = self.client.get(url).send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            return Err(PriceError::ProviderUnavailable(
                format!("Binance API error: {}", response.status())
            ));
        }
        
        let time_data: serde_json::Value = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse time response: {}", e)))?;
        
        let server_time = time_data.get("serverTime")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| PriceError::InvalidPriceData("Missing server time".to_string()))?;
        
        let datetime = DateTime::from_timestamp(server_time as i64 / 1000, 0)
            .ok_or_else(|| PriceError::InvalidPriceData("Invalid timestamp".to_string()))?;
        
        Ok(datetime)
    }
}

#[async_trait]
impl PriceProvider for BinanceProvider {
    async fn get_price(&self, symbol: &str) -> Result<PriceData, PriceError> {
        let pair = TradingPair::from_symbol(symbol)
            .ok_or_else(|| PriceError::InvalidSymbol(symbol.to_string()))?;
        
        // Special handling for NEAR since it's not on Binance
        if pair == TradingPair::NearUsd {
            return Err(PriceError::InvalidSymbol("NEAR not available on Binance".to_string()));
        }
        
        info!("Getting Binance price for {}", symbol);
        
        match self.get_binance_price(pair).await {
            Ok(price_data) => {
                info!("Successfully got Binance price for {}: ${}", symbol, price_data.price);
                Ok(price_data)
            }
            Err(e) => {
                error!("Failed to get Binance price for {}: {}", symbol, e);
                Err(e)
            }
        }
    }
    
    async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<PriceData>, PriceError> {
        info!("Getting multiple Binance prices for {} symbols", symbols.len());
        
        let pairs: Vec<TradingPair> = symbols.iter()
            .filter_map(|&symbol| TradingPair::from_symbol(symbol))
            .filter(|pair| *pair != TradingPair::NearUsd) // Filter out NEAR
            .collect();
        
        if pairs.is_empty() {
            return Err(PriceError::InvalidSymbol("No valid Binance symbols".to_string()));
        }
        
        match self.get_multiple_binance_prices(&pairs).await {
            Ok(prices) => {
                info!("Successfully got {} prices from Binance", prices.len());
                Ok(prices)
            }
            Err(e) => {
                error!("Failed to get multiple prices from Binance: {}", e);
                Err(e)
            }
        }
    }
    
    fn provider_name(&self) -> &str {
        "binance"
    }
    
    fn is_available(&self) -> bool {
        self.is_available
    }
    
    fn get_supported_symbols(&self) -> Vec<String> {
        self.symbol_mappings.keys()
            .map(|pair| pair.to_symbol().to_string())
            .collect()
    }
}

/// Binance exchange info for symbol validation
#[derive(Debug, Deserialize)]
pub struct BinanceExchangeInfo {
    pub symbols: Vec<BinanceSymbolInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSymbolInfo {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
}