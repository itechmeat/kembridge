// src/price_oracle/cache.rs - Redis caching for price data
use std::time::Duration;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal as Decimal;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use crate::price_oracle::types::{PriceData, AggregatedPrice, PriceError, CacheConfig};

/// Price cache using Redis
pub struct PriceCache {
    redis: ConnectionManager,
    config: CacheConfig,
}

impl PriceCache {
    /// Create new price cache
    pub fn new(redis: ConnectionManager) -> Self {
        Self {
            redis,
            config: CacheConfig::default(),
        }
    }
    
    /// Create cache with custom configuration
    pub fn with_config(redis: ConnectionManager, config: CacheConfig) -> Self {
        Self {
            redis,
            config,
        }
    }
    
    /// Cache aggregated price with TTL
    pub async fn set_price(&self, price: &AggregatedPrice) -> Result<(), PriceError> {
        let key = format!("{}:primary:{}", self.config.prefix, price.symbol);
        let fallback_key = format!("{}:fallback:{}", self.config.prefix, price.symbol);
        
        // Serialize price data
        let serialized = serde_json::to_string(price)
            .map_err(|e| PriceError::CacheError(format!("Failed to serialize price: {}", e)))?;
        
        let mut conn = self.redis.clone();
        
        // Set primary cache with short TTL
        let _: () = conn.set_ex(&key, &serialized, self.config.primary_ttl)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to set primary cache: {}", e)))?;
        
        // Set fallback cache with long TTL
        let _: () = conn.set_ex(&fallback_key, &serialized, self.config.fallback_ttl)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to set fallback cache: {}", e)))?;
        
        info!("Cached price for {} with primary TTL {}s and fallback TTL {}s", 
            price.symbol, self.config.primary_ttl, self.config.fallback_ttl);
        
        Ok(())
    }
    
    /// Get price from primary cache
    pub async fn get_price(&self, symbol: &str) -> Result<AggregatedPrice, PriceError> {
        let key = format!("{}:primary:{}", self.config.prefix, symbol);
        let mut conn = self.redis.clone();
        
        let cached_data: Option<String> = conn.get(&key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get from cache: {}", e)))?;
        
        if let Some(data) = cached_data {
            let price: AggregatedPrice = serde_json::from_str(&data)
                .map_err(|e| PriceError::CacheError(format!("Failed to deserialize price: {}", e)))?;
            
            info!("Retrieved price for {} from primary cache", symbol);
            Ok(price)
        } else {
            Err(PriceError::CacheError(format!("No cached price for {}", symbol)))
        }
    }
    
    /// Get price from fallback cache (for emergency situations)
    pub async fn get_fallback_price(&self, symbol: &str) -> Result<AggregatedPrice, PriceError> {
        let key = format!("{}:fallback:{}", self.config.prefix, symbol);
        let mut conn = self.redis.clone();
        
        let cached_data: Option<String> = conn.get(&key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get from fallback cache: {}", e)))?;
        
        if let Some(data) = cached_data {
            let price: AggregatedPrice = serde_json::from_str(&data)
                .map_err(|e| PriceError::CacheError(format!("Failed to deserialize fallback price: {}", e)))?;
            
            warn!("Retrieved price for {} from fallback cache", symbol);
            Ok(price)
        } else {
            Err(PriceError::CacheError(format!("No fallback price for {}", symbol)))
        }
    }
    
    /// Cache individual price data from provider
    pub async fn cache_provider_price(&self, provider: &str, price: &PriceData) -> Result<(), PriceError> {
        let key = format!("{}:provider:{}:{}", self.config.prefix, provider, price.symbol);
        
        let serialized = serde_json::to_string(price)
            .map_err(|e| PriceError::CacheError(format!("Failed to serialize provider price: {}", e)))?;
        
        let mut conn = self.redis.clone();
        
        // Cache provider-specific price with shorter TTL
        let _: () = conn.set_ex(&key, &serialized, self.config.primary_ttl / 2)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to cache provider price: {}", e)))?;
        
        info!("Cached provider price for {} from {}", price.symbol, provider);
        Ok(())
    }
    
    /// Get cached price from specific provider
    pub async fn get_provider_price(&self, provider: &str, symbol: &str) -> Result<PriceData, PriceError> {
        let key = format!("{}:provider:{}:{}", self.config.prefix, provider, symbol);
        let mut conn = self.redis.clone();
        
        let cached_data: Option<String> = conn.get(&key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get provider price: {}", e)))?;
        
        if let Some(data) = cached_data {
            let price: PriceData = serde_json::from_str(&data)
                .map_err(|e| PriceError::CacheError(format!("Failed to deserialize provider price: {}", e)))?;
            
            Ok(price)
        } else {
            Err(PriceError::CacheError(format!("No cached price for {} from {}", symbol, provider)))
        }
    }
    
    /// Get multiple prices from cache
    pub async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<AggregatedPrice>, PriceError> {
        let mut results = Vec::new();
        
        for &symbol in symbols {
            if let Ok(price) = self.get_price(symbol).await {
                results.push(price);
            }
        }
        
        Ok(results)
    }
    
    /// Cache price quote for swap operations
    pub async fn cache_price_quote(&self, from_token: &str, to_token: &str, quote: &PriceQuote) -> Result<(), PriceError> {
        let key = format!("{}:quote:{}:{}", self.config.prefix, from_token, to_token);
        
        let serialized = serde_json::to_string(quote)
            .map_err(|e| PriceError::CacheError(format!("Failed to serialize quote: {}", e)))?;
        
        let mut conn = self.redis.clone();
        
        // Cache quote with very short TTL (30 seconds)
        let _: () = conn.set_ex(&key, &serialized, 30)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to cache quote: {}", e)))?;
        
        info!("Cached price quote for {} -> {}", from_token, to_token);
        Ok(())
    }
    
    /// Get cached price quote
    pub async fn get_price_quote(&self, from_token: &str, to_token: &str) -> Result<PriceQuote, PriceError> {
        let key = format!("{}:quote:{}:{}", self.config.prefix, from_token, to_token);
        let mut conn = self.redis.clone();
        
        let cached_data: Option<String> = conn.get(&key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get quote: {}", e)))?;
        
        if let Some(data) = cached_data {
            let quote: PriceQuote = serde_json::from_str(&data)
                .map_err(|e| PriceError::CacheError(format!("Failed to deserialize quote: {}", e)))?;
            
            // Check if quote is still valid
            if Utc::now() > quote.expires_at {
                return Err(PriceError::CacheError("Quote has expired".to_string()));
            }
            
            Ok(quote)
        } else {
            Err(PriceError::CacheError(format!("No cached quote for {} -> {}", from_token, to_token)))
        }
    }
    
    /// Delete price from cache
    pub async fn delete_price(&self, symbol: &str) -> Result<(), PriceError> {
        let primary_key = format!("{}:primary:{}", self.config.prefix, symbol);
        let fallback_key = format!("{}:fallback:{}", self.config.prefix, symbol);
        
        let mut conn = self.redis.clone();
        
        let _: () = conn.del(&primary_key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to delete primary cache: {}", e)))?;
        
        let _: () = conn.del(&fallback_key)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to delete fallback cache: {}", e)))?;
        
        info!("Deleted cached price for {}", symbol);
        Ok(())
    }
    
    /// Clear all price caches
    pub async fn clear_all(&self) -> Result<(), PriceError> {
        let pattern = format!("{}:*", self.config.prefix);
        let mut conn = self.redis.clone();
        
        // Get all keys matching the pattern
        let keys: Vec<String> = conn.keys(&pattern)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get keys: {}", e)))?;
        
        if !keys.is_empty() {
            let _: () = conn.del(&keys)
                .await
                .map_err(|e| PriceError::CacheError(format!("Failed to delete keys: {}", e)))?;
        }
        
        info!("Cleared all price caches");
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats, PriceError> {
        let pattern = format!("{}:*", self.config.prefix);
        let mut conn = self.redis.clone();
        
        let keys: Vec<String> = conn.keys(&pattern)
            .await
            .map_err(|e| PriceError::CacheError(format!("Failed to get keys: {}", e)))?;
        
        let mut primary_count = 0;
        let mut fallback_count = 0;
        let mut provider_count = 0;
        let mut quote_count = 0;
        
        for key in &keys {
            if key.contains(":primary:") {
                primary_count += 1;
            } else if key.contains(":fallback:") {
                fallback_count += 1;
            } else if key.contains(":provider:") {
                provider_count += 1;
            } else if key.contains(":quote:") {
                quote_count += 1;
            }
        }
        
        Ok(CacheStats {
            total_keys: keys.len(),
            primary_prices: primary_count,
            fallback_prices: fallback_count,
            provider_prices: provider_count,
            quote_prices: quote_count,
        })
    }
    
    /// Set cache configuration
    pub fn set_config(&mut self, config: CacheConfig) {
        self.config = config;
    }
    
    /// Get cache configuration
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: usize,
    pub primary_prices: usize,
    pub fallback_prices: usize,
    pub provider_prices: usize,
    pub quote_prices: usize,
}

/// Price quote for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub exchange_rate: Decimal,
    pub price_impact: f64,
    pub sources: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confidence: f64,
}

impl PriceQuote {
    /// Create new price quote
    pub fn new(
        from_token: String,
        to_token: String,
        from_amount: Decimal,
        to_amount: Decimal,
        sources: Vec<String>,
        confidence: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            from_token,
            to_token,
            from_amount: from_amount.clone(),
            to_amount: to_amount.clone(),
            exchange_rate: &to_amount / &from_amount,
            price_impact: 0.0, // Calculate based on liquidity
            sources,
            timestamp: now,
            expires_at: now + chrono::Duration::seconds(30),
            confidence,
        }
    }
    
    /// Check if quote is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() <= self.expires_at
    }
    
    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().signed_duration_since(self.timestamp).num_seconds()
    }
}

/// Cache warming service
pub struct CacheWarmer {
    cache: PriceCache,
    symbols: Vec<String>,
}

impl CacheWarmer {
    /// Create new cache warmer
    pub fn new(cache: PriceCache, symbols: Vec<String>) -> Self {
        Self {
            cache,
            symbols,
        }
    }
    
    /// Warm cache with prices
    pub async fn warm_cache(&self, prices: &[AggregatedPrice]) -> Result<usize, PriceError> {
        let mut cached_count = 0;
        
        for price in prices {
            if let Ok(()) = self.cache.set_price(price).await {
                cached_count += 1;
            }
        }
        
        info!("Warmed cache with {} prices", cached_count);
        Ok(cached_count)
    }
    
    /// Start background cache warming
    pub async fn start_background_warming(&self, interval: Duration) {
        let mut interval_timer = tokio::time::interval(interval);
        
        loop {
            interval_timer.tick().await;
            
            // This would integrate with the price oracle service
            // to fetch fresh prices and warm the cache
            info!("Background cache warming cycle");
        }
    }
}

/// Cache health checker
pub struct CacheHealthChecker {
    cache: PriceCache,
}

impl CacheHealthChecker {
    /// Create new health checker
    pub fn new(cache: PriceCache) -> Self {
        Self { cache }
    }
    
    /// Check cache health
    pub async fn check_health(&self) -> CacheHealth {
        let mut health = CacheHealth::default();
        
        // Test basic connectivity
        match self.cache.get_stats().await {
            Ok(stats) => {
                health.is_connected = true;
                health.total_keys = stats.total_keys;
                health.primary_cache_hits = stats.primary_prices;
                health.fallback_cache_hits = stats.fallback_prices;
            }
            Err(e) => {
                health.is_connected = false;
                health.last_error = Some(e.to_string());
            }
        }
        
        health
    }
}

/// Cache health status
#[derive(Debug, Clone, Default)]
pub struct CacheHealth {
    pub is_connected: bool,
    pub total_keys: usize,
    pub primary_cache_hits: usize,
    pub fallback_cache_hits: usize,
    pub last_error: Option<String>,
    pub last_check: DateTime<Utc>,
}

impl CacheHealth {
    /// Create new health status
    pub fn new() -> Self {
        Self {
            is_connected: false,
            total_keys: 0,
            primary_cache_hits: 0,
            fallback_cache_hits: 0,
            last_error: None,
            last_check: Utc::now(),
        }
    }
}