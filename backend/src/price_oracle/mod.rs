// src/price_oracle/mod.rs - Price Oracle service with multiple providers
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use anyhow::Result;
use tracing::{info, error, warn};
use redis::aio::ConnectionManager;
use crate::config::AppConfig;
use crate::constants::*;

pub mod providers;
pub mod aggregator;
pub mod validator;
pub mod cache;
pub mod types;
pub mod simple_types;

pub use types::*;
pub use providers::*;
pub use aggregator::*;
pub use validator::*;
pub use cache::*;
pub use simple_types::*;

/// Main price oracle service that coordinates all price providers
pub struct PriceOracleService {
    providers: Vec<Arc<dyn PriceProvider + Send + Sync>>,
    aggregator: PriceAggregator,
    validator: PriceValidator,
    cache: PriceCache,
    config: Arc<AppConfig>,
    quick_oracle: QuickPriceOracleService,
}

impl PriceOracleService {
    /// Create new price oracle service with all providers
    pub async fn new(
        redis_manager: ConnectionManager,
        config: Arc<AppConfig>,
    ) -> Result<Self> {
        let mut providers: Vec<Arc<dyn PriceProvider + Send + Sync>> = Vec::new();
        
        // Initialize Chainlink provider (primary)
        let chainlink = Arc::new(ChainlinkProvider::new(config.clone()).await?);
        providers.push(chainlink);
        
        // Initialize CoinGecko provider (secondary)
        let coingecko = Arc::new(CoinGeckoProvider::new(config.clone()).await?);
        providers.push(coingecko);
        
        // Initialize Binance provider (tertiary)
        let binance = Arc::new(BinanceProvider::new(config.clone()).await?);
        providers.push(binance);
        
        let aggregator = PriceAggregator::new();
        let validator = PriceValidator::new(config.clone());
        let cache = PriceCache::new(redis_manager);
        let quick_oracle = QuickPriceOracleService::new();
        
        let service = Self {
            providers,
            aggregator,
            validator,
            cache,
            config,
            quick_oracle,
        };
        
        info!("PriceOracleService initialized with {} providers", service.providers.len());
        Ok(service)
    }
    
    /// Get price for a single symbol with fallback strategy
    pub async fn get_price(&self, symbol: &str) -> Result<AggregatedPrice> {
        info!("Getting price for symbol: {}", symbol);
        
        // First, try to get from cache
        if let Ok(cached_price) = self.cache.get_price(symbol).await {
            if !self.is_price_stale(&cached_price) {
                info!("Returning cached price for {}", symbol);
                return Ok(cached_price);
            }
        }
        
        // Collect prices from all available providers
        let mut prices = Vec::new();
        for provider in &self.providers {
            if !provider.is_available() {
                warn!("Provider {} is not available", provider.provider_name());
                continue;
            }
            
            match provider.get_price(symbol).await {
                Ok(price_data) => {
                    if let Ok(validated_price) = self.validator.validate_price(&price_data) {
                        prices.push(validated_price);
                    }
                }
                Err(e) => {
                    warn!("Provider {} failed to get price for {}: {}", 
                        provider.provider_name(), symbol, e);
                }
            }
        }
        
        if prices.is_empty() {
            return Err(anyhow::anyhow!(
                "No price data available for symbol: {}. All {} providers failed or unavailable.", 
                symbol, 
                self.providers.len()
            ));
        }
        
        // Aggregate prices from multiple sources
        let aggregated_price = self.aggregator.aggregate_prices(symbol, &prices);
        
        // Cache the result
        if let Err(e) = self.cache.set_price(&aggregated_price).await {
            warn!("Failed to cache price for {}: {}", symbol, e);
        }
        
        info!("Successfully got aggregated price for {}: ${}", 
            symbol, aggregated_price.price);
        
        Ok(aggregated_price)
    }
    
    /// Get multiple prices with batch optimization
    pub async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<AggregatedPrice>> {
        info!("Getting prices for {} symbols", symbols.len());
        
        let mut results = Vec::new();
        
        // First, try to get cached prices
        let mut uncached_symbols = Vec::new();
        for symbol in symbols {
            if let Ok(cached_price) = self.cache.get_price(symbol).await {
                if !self.is_price_stale(&cached_price) {
                    results.push(cached_price);
                    continue;
                }
            }
            uncached_symbols.push(*symbol);
        }
        
        if uncached_symbols.is_empty() {
            return Ok(results);
        }
        
        // Batch request to all providers for uncached symbols
        let mut provider_results = Vec::new();
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }
            
            match provider.get_multiple_prices(&uncached_symbols).await {
                Ok(prices) => {
                    for price in prices {
                        if let Ok(validated_price) = self.validator.validate_price(&price) {
                            provider_results.push(validated_price);
                        }
                    }
                }
                Err(e) => {
                    warn!("Provider {} failed batch request: {}", 
                        provider.provider_name(), e);
                }
            }
        }
        
        // Aggregate prices by symbol
        let mut symbol_groups = std::collections::HashMap::new();
        for price in provider_results {
            symbol_groups.entry(price.symbol.clone())
                .or_insert_with(Vec::new)
                .push(price);
        }
        
        for symbol in uncached_symbols {
            if let Some(symbol_prices) = symbol_groups.get(symbol) {
                let aggregated_price = self.aggregator.aggregate_prices(symbol, symbol_prices);
                
                // Cache the result
                if let Err(e) = self.cache.set_price(&aggregated_price).await {
                    warn!("Failed to cache price for {}: {}", symbol, e);
                }
                
                results.push(aggregated_price);
            }
        }
        
        Ok(results)
    }
    
    /// Get supported symbols
    pub fn get_supported_symbols(&self) -> Vec<String> {
        self.quick_oracle.get_supported_symbols()
    }
    
    /// Get real price data (quick implementation)
    pub async fn get_real_price(&self, symbol: &str) -> Result<SimpleAggregatedPrice, Box<dyn std::error::Error + Send + Sync>> {
        self.quick_oracle.get_real_price(symbol).await
    }
    
    /// Get multiple real prices (quick implementation)
    pub async fn get_multiple_real_prices(&self, symbols: &[&str]) -> Result<Vec<SimpleAggregatedPrice>, Box<dyn std::error::Error + Send + Sync>> {
        self.quick_oracle.get_multiple_real_prices(symbols).await
    }
    
    /// Check if price is stale based on timestamp
    fn is_price_stale(&self, price: &AggregatedPrice) -> bool {
        let staleness_threshold = Duration::from_secs(CACHE_TTL_SHORT); // 5 minutes
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(price.last_updated);
        
        age > chrono::Duration::from_std(staleness_threshold).unwrap()
    }
    
    /// Start background price updates
    pub async fn start_background_updates(&self) {
        info!("Starting background price updates");
        
        let supported_symbols = self.get_supported_symbols();
        let symbols_refs: Vec<&str> = supported_symbols.iter().map(|s| s.as_str()).collect();
        
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            match self.get_multiple_prices(&symbols_refs).await {
                Ok(prices) => {
                    info!("Background update completed for {} symbols", prices.len());
                }
                Err(e) => {
                    error!("Background price update failed: {}", e);
                }
            }
        }
    }
    
    /// Get provider health status
    pub async fn get_provider_health(&self) -> Vec<ProviderHealth> {
        let mut health_status = Vec::new();
        
        for provider in &self.providers {
            let health = ProviderHealth {
                name: provider.provider_name().to_string(),
                is_available: provider.is_available(),
                last_successful_request: chrono::Utc::now(), // TODO (feat): Track real timestamps (P4.2)
                error_rate: 0.0, // TODO (feat): Calculate real error rate (P4.2)
                average_latency: Duration::from_millis(100), // TODO (feat): Track real latency (P4.2)
            };
            health_status.push(health);
        }
        
        health_status
    }
}

/// Provider health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub name: String,
    pub is_available: bool,
    pub last_successful_request: chrono::DateTime<chrono::Utc>,
    pub error_rate: f64,
    pub average_latency: Duration,
}