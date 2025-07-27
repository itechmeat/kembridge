// src/price_oracle/types.rs - Price Oracle type definitions
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal as Decimal;
use std::fmt;

/// Raw price data from a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub confidence: f64,
    pub volume_24h: Option<Decimal>,
    pub change_24h: Option<f64>,
}

/// Aggregated price from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPrice {
    pub symbol: String,
    pub price: Decimal,
    pub sources: Vec<String>,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
    pub price_variance: f64,
    pub volume_24h: Option<Decimal>,
    pub change_24h: Option<f64>,
}

/// Price provider trait for different data sources
#[async_trait::async_trait]
pub trait PriceProvider {
    /// Get price for a single symbol
    async fn get_price(&self, symbol: &str) -> Result<PriceData, PriceError>;
    
    /// Get prices for multiple symbols (batch request)
    async fn get_multiple_prices(&self, symbols: &[&str]) -> Result<Vec<PriceData>, PriceError>;
    
    /// Get provider name
    fn provider_name(&self) -> &str;
    
    /// Check if provider is available
    fn is_available(&self) -> bool;
    
    /// Get supported symbols
    fn get_supported_symbols(&self) -> Vec<String>;
}

/// Price-related errors
#[derive(Debug, thiserror::Error)]
pub enum PriceError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Price not found for symbol: {0}")]
    PriceNotFound(String),
    
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),
    
    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),
    
    #[error("Rate limit exceeded for provider: {0}")]
    RateLimitExceeded(String),
    
    #[error("Stale price data: {0}")]
    StalePrice(String),
    
    #[error("Price validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Price validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub adjusted_confidence: f64,
}

/// Price aggregation method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationMethod {
    WeightedAverage,
    MedianPrice,
    HighestConfidence,
    MostRecentPrice,
}

/// Price cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub primary_ttl: u64,        // TTL for fresh prices (seconds)
    pub fallback_ttl: u64,       // TTL for fallback prices (seconds)
    pub max_staleness: u64,      // Max age for stale prices (seconds)
    pub prefix: String,          // Redis key prefix
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            primary_ttl: 60,           // 1 minute
            fallback_ttl: 86400,       // 24 hours
            max_staleness: 300,        // 5 minutes
            prefix: "price".to_string(),
        }
    }
}

/// Supported trading pairs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradingPair {
    EthUsd,
    NearUsd,
    BtcUsd,
    UsdtUsd,
    UsdcUsd,
}

impl TradingPair {
    /// Convert to symbol string
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TradingPair::EthUsd => "ETH/USD",
            TradingPair::NearUsd => "NEAR/USD",
            TradingPair::BtcUsd => "BTC/USD",
            TradingPair::UsdtUsd => "USDT/USD",
            TradingPair::UsdcUsd => "USDC/USD",
        }
    }
    
    /// Parse from symbol string
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol.to_uppercase().as_str() {
            "ETH/USD" | "ETHUSD" | "ETH" => Some(TradingPair::EthUsd),
            "NEAR/USD" | "NEARUSD" | "NEAR" => Some(TradingPair::NearUsd),
            "BTC/USD" | "BTCUSD" | "BTC" => Some(TradingPair::BtcUsd),
            "USDT/USD" | "USDTUSD" | "USDT" => Some(TradingPair::UsdtUsd),
            "USDC/USD" | "USDCUSD" | "USDC" => Some(TradingPair::UsdcUsd),
            _ => None,
        }
    }
    
    /// Get all supported pairs
    pub fn all() -> Vec<Self> {
        vec![
            TradingPair::EthUsd,
            TradingPair::NearUsd,
            TradingPair::BtcUsd,
            TradingPair::UsdtUsd,
            TradingPair::UsdcUsd,
        ]
    }
}

impl fmt::Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_symbol())
    }
}

/// Price quote for swap operations
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

/// Market data for a trading pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: Decimal,
    pub volume_24h: Decimal,
    pub change_24h: f64,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub market_cap: Option<Decimal>,
    pub circulating_supply: Option<Decimal>,
    pub last_updated: DateTime<Utc>,
}

/// Price alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub symbol: String,
    pub target_price: Decimal,
    pub condition: AlertCondition,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub triggered_at: Option<DateTime<Utc>>,
}

/// Alert condition types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    Change(f64), // Percentage change
}

/// Price history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryEntry {
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub volume: Option<Decimal>,
}

/// Price statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceStatistics {
    pub symbol: String,
    pub current_price: Decimal,
    pub average_price_1h: Decimal,
    pub average_price_24h: Decimal,
    pub price_variance: f64,
    pub volatility: f64,
    pub trend: PriceTrend,
    pub last_updated: DateTime<Utc>,
}

/// Price trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PriceTrend {
    Up,
    Down,
    Stable,
}

impl PriceData {
    /// Create new price data
    pub fn new(
        symbol: String,
        price: Decimal,
        source: String,
        confidence: f64,
    ) -> Self {
        Self {
            symbol,
            price,
            timestamp: Utc::now(),
            source,
            confidence,
            volume_24h: None,
            change_24h: None,
        }
    }
    
    /// Check if price data is stale
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.timestamp);
        age.num_seconds() > max_age_seconds as i64
    }
    
    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().signed_duration_since(self.timestamp).num_seconds()
    }
}

impl AggregatedPrice {
    /// Create new aggregated price
    pub fn new(
        symbol: String,
        price: Decimal,
        sources: Vec<String>,
        confidence: f64,
    ) -> Self {
        Self {
            symbol,
            price,
            sources,
            confidence,
            last_updated: Utc::now(),
            price_variance: 0.0,
            volume_24h: None,
            change_24h: None,
        }
    }
    
    /// Check if aggregated price is stale
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.last_updated);
        age.num_seconds() > max_age_seconds as i64
    }
}