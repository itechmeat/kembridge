// src/price_oracle/providers/coingecko.rs - CoinGecko API integration
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

/// CoinGecko API response for simple price endpoint
#[derive(Debug, Deserialize)]
struct CoinGeckoSimplePrice {
    #[serde(flatten)]
    prices: HashMap<String, f64>,
}

/// CoinGecko API response for coins endpoint
#[derive(Debug, Deserialize)]
struct CoinGeckoMarketData {
    id: String,
    symbol: String,
    name: String,
    current_price: f64,
    market_cap: Option<f64>,
    total_volume: Option<f64>,
    price_change_percentage_24h: Option<f64>,
    last_updated: String,
}

/// CoinGecko provider implementation
pub struct CoinGeckoProvider {
    client: Client,
    config: Arc<AppConfig>,
    coin_mappings: HashMap<TradingPair, String>,
    is_available: bool,
    api_key: Option<String>,
}

impl CoinGeckoProvider {
    /// Create new CoinGecko provider
    pub async fn new(config: Arc<AppConfig>) -> Result<Self, PriceError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_CLIENT_TIMEOUT))
            .user_agent("KEMBridge/1.0")
            .build()
            .map_err(|e| PriceError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
        
        // CoinGecko coin ID mappings
        let mut coin_mappings = HashMap::new();
        coin_mappings.insert(TradingPair::EthUsd, "ethereum".to_string());
        coin_mappings.insert(TradingPair::NearUsd, "near".to_string());
        coin_mappings.insert(TradingPair::BtcUsd, "bitcoin".to_string());
        coin_mappings.insert(TradingPair::UsdtUsd, "tether".to_string());
        coin_mappings.insert(TradingPair::UsdcUsd, "usd-coin".to_string());
        
        // Get API key from config (optional, for higher rate limits)
        let api_key = std::env::var("COINGECKO_API_KEY").ok();
        
        let provider = Self {
            client,
            config,
            coin_mappings,
            is_available: true,
            api_key,
        };
        
        info!("CoinGeckoProvider initialized with {} coin mappings", provider.coin_mappings.len());
        Ok(provider)
    }
    
    /// Get price from CoinGecko API
    async fn get_coingecko_price(&self, pair: TradingPair) -> Result<PriceData, PriceError> {
        let coin_id = self.coin_mappings.get(&pair)
            .ok_or_else(|| PriceError::InvalidSymbol(pair.to_symbol().to_string()))?;
        
        let url = if let Some(ref _api_key) = self.api_key {
            format!("{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", COINGECKO_PRO_API_BASE, coin_id)
        } else {
            format!("{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", COINGECKO_API_BASE, coin_id)
        };
        
        let mut request = self.client.get(&url);
        
        // Add API key header if available
        if let Some(ref api_key) = self.api_key {
            request = request.header("X-CG-Pro-API-Key", api_key);
        }
        
        let response = request.send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 429 {
                return Err(PriceError::RateLimitExceeded("CoinGecko".to_string()));
            }
            return Err(PriceError::ProviderUnavailable(
                format!("CoinGecko API error: {}", response.status())
            ));
        }
        
        let price_data: HashMap<String, serde_json::Value> = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse CoinGecko response: {}", e)))?;
        
        let coin_data = price_data.get(coin_id)
            .ok_or_else(|| PriceError::PriceNotFound(coin_id.clone()))?;
        
        let price = coin_data.get("usd")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| PriceError::InvalidPriceData("Missing USD price".to_string()))?;
        
        let change_24h = coin_data.get("usd_24h_change")
            .and_then(|v| v.as_f64());
        
        Ok(PriceData {
            symbol: pair.to_symbol().to_string(),
            price: Decimal::from_f64(price)
                .ok_or_else(|| PriceError::InvalidPriceData("Invalid price value".to_string()))?,
            timestamp: Utc::now(),
            source: "coingecko".to_string(),
            confidence: PRICE_CONFIDENCE_COINGECKO,
            volume_24h: None,
            change_24h,
        })
    }
    
    /// Get multiple prices using batch endpoint
    async fn get_multiple_coingecko_prices(&self, pairs: &[TradingPair]) -> Result<Vec<PriceData>, PriceError> {
        let coin_ids: Vec<String> = pairs.iter()
            .filter_map(|pair| self.coin_mappings.get(pair))
            .cloned()
            .collect();
        
        if coin_ids.is_empty() {
            return Err(PriceError::InvalidSymbol("No valid coin IDs".to_string()));
        }
        
        let ids_param = coin_ids.join(",");
        let url = if let Some(ref _api_key) = self.api_key {
            format!("{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", COINGECKO_PRO_API_BASE, ids_param)
        } else {
            format!("{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", COINGECKO_API_BASE, ids_param)
        };
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.api_key {
            request = request.header("X-CG-Pro-API-Key", api_key);
        }
        
        let response = request.send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            if response.status().as_u16() == 429 {
                return Err(PriceError::RateLimitExceeded("CoinGecko".to_string()));
            }
            return Err(PriceError::ProviderUnavailable(
                format!("CoinGecko API error: {}", response.status())
            ));
        }
        
        let price_data: HashMap<String, serde_json::Value> = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse CoinGecko response: {}", e)))?;
        
        let mut results = Vec::new();
        
        for pair in pairs {
            if let Some(coin_id) = self.coin_mappings.get(pair) {
                if let Some(coin_data) = price_data.get(coin_id) {
                    if let Some(price) = coin_data.get("usd").and_then(|v| v.as_f64()) {
                        let change_24h = coin_data.get("usd_24h_change")
                            .and_then(|v| v.as_f64());
                        
                        if let Some(price_decimal) = Decimal::from_f64(price) {
                            results.push(PriceData {
                                symbol: pair.to_symbol().to_string(),
                                price: price_decimal,
                                timestamp: Utc::now(),
                                source: "coingecko".to_string(),
                                confidence: PRICE_CONFIDENCE_COINGECKO,
                                volume_24h: None,
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
    
    /// Get detailed market data
    #[allow(dead_code)]
    async fn get_market_data(&self, pair: TradingPair) -> Result<CoinGeckoMarketData, PriceError> {
        let coin_id = self.coin_mappings.get(&pair)
            .ok_or_else(|| PriceError::InvalidSymbol(pair.to_symbol().to_string()))?;
        
        let url = if let Some(ref _api_key) = self.api_key {
            format!("{}/coins/{}?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false", COINGECKO_PRO_API_BASE, coin_id)
        } else {
            format!("{}/coins/{}?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false", COINGECKO_API_BASE, coin_id)
        };
        
        let mut request = self.client.get(&url);
        
        if let Some(ref api_key) = self.api_key {
            request = request.header("X-CG-Pro-API-Key", api_key);
        }
        
        let response = request.send().await
            .map_err(|e| PriceError::Network(e))?;
        
        if !response.status().is_success() {
            return Err(PriceError::ProviderUnavailable(
                format!("CoinGecko API error: {}", response.status())
            ));
        }
        
        let market_data: CoinGeckoMarketData = response.json().await
            .map_err(|e| PriceError::InvalidPriceData(format!("Failed to parse market data: {}", e)))?;
        
        Ok(market_data)
    }
    
    /// Check if provider is healthy
    pub async fn health_check(&self) -> bool {
        let url = if self.api_key.is_some() {
            &format!("{}/ping", COINGECKO_PRO_API_BASE)
        } else {
            &format!("{}/ping", COINGECKO_API_BASE)
        };
        
        match self.client.get(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait]
impl PriceProvider for CoinGeckoProvider {
    async fn get_price(&self, symbol: &str) -> Result<PriceData, PriceError> {
        let pair = TradingPair::from_symbol(symbol)
            .ok_or_else(|| PriceError::InvalidSymbol(symbol.to_string()))?;
        
        info!("Getting CoinGecko price for {}", symbol);
        
        match self.get_coingecko_price(pair).await {
            Ok(price_data) => {
                info!("Successfully got CoinGecko price for {}: ${}", symbol, price_data.price);
                Ok(price_data)
            }
            Err(e) => {
                error!("Failed to get CoinGecko price for {}: {}", symbol, e);
                Err(e)
            }
        }
    }
    
    async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<PriceData>, PriceError> {
        info!("Getting multiple CoinGecko prices for {} symbols", symbols.len());
        
        let pairs: Vec<TradingPair> = symbols.iter()
            .filter_map(|&symbol| TradingPair::from_symbol(symbol))
            .collect();
        
        if pairs.is_empty() {
            return Err(PriceError::InvalidSymbol("No valid symbols".to_string()));
        }
        
        match self.get_multiple_coingecko_prices(&pairs).await {
            Ok(prices) => {
                info!("Successfully got {} prices from CoinGecko", prices.len());
                Ok(prices)
            }
            Err(e) => {
                error!("Failed to get multiple prices from CoinGecko: {}", e);
                Err(e)
            }
        }
    }
    
    fn provider_name(&self) -> &str {
        "coingecko"
    }
    
    fn is_available(&self) -> bool {
        self.is_available
    }
    
    fn get_supported_symbols(&self) -> Vec<String> {
        self.coin_mappings.keys()
            .map(|pair| pair.to_symbol().to_string())
            .collect()
    }
}