// src/dynamic_pricing/types.rs - Type definitions for dynamic pricing

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
// use utoipa::ToSchema; // TODO (fix): Add back ToSchema when BigDecimal support is resolved
use uuid::Uuid;

use crate::price_oracle::types::PriceError;
use crate::oneinch::types::OneinchError;

/// Bridge quote request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeQuoteRequest {
    pub from_token: String,
    pub to_token: String,
    pub from_chain: String,
    pub to_chain: String,
    pub from_amount: BigDecimal,
    pub max_slippage: Option<f64>,
    pub user_id: Option<String>,
}

/// Bridge quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeQuote {
    pub quote_id: String,
    pub from_token: String,
    pub to_token: String,
    pub from_chain: String,
    pub to_chain: String,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub exchange_rate: ExchangeRate,
    pub fee_breakdown: FeeBreakdown,
    pub price_impact: PriceImpact,
    pub slippage_settings: SlippageSettings,
    pub estimated_execution_time: i64,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Exchange rate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub rate: BigDecimal,
    pub rate_source: String,
    pub confidence_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub volatility_indicator: f64,
}

/// Fee breakdown structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub base_fee: BigDecimal,
    pub gas_fee: BigDecimal,
    pub protocol_fee: BigDecimal,
    pub slippage_protection_fee: BigDecimal,
    pub total_fee_amount: BigDecimal,
    pub fee_percentage: f64,
    pub fee_currency: String,
}

/// Price impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpact {
    pub impact_percentage: f64,
    pub liquidity_assessment: LiquidityAssessment,
    pub market_depth: MarketDepth,
    pub impact_level: ImpactLevel,
    pub recommendations: Vec<String>,
}

/// Liquidity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityAssessment {
    pub liquidity_score: f64,
    pub available_liquidity: BigDecimal,
    pub liquidity_sources: Vec<String>,
    pub fragmentation_risk: f64,
}

/// Market depth information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub bid_depth: BigDecimal,
    pub ask_depth: BigDecimal,
    pub spread_percentage: f64,
    pub market_stability: f64,
}

/// Price impact level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Slippage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageSettings {
    pub max_slippage: f64,
    pub recommended_slippage: f64,
    pub dynamic_adjustment: bool,
    pub protection_level: SlippageProtectionLevel,
    pub timeout_minutes: i64,
}

/// Slippage protection level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlippageProtectionLevel {
    Basic,
    Standard,
    Advanced,
    Maximum,
}

/// Quote execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteExecutionParams {
    pub quote_id: String,
    pub user_id: String,
    pub execution_timeout: Option<i64>,
    pub custom_slippage: Option<f64>,
}

/// Quote validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteValidationResult {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub updated_quote: Option<BridgeQuote>,
}

/// Pricing health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingHealthStatus {
    pub overall_healthy: bool,
    pub oracle_healthy: bool,
    pub oneinch_healthy: bool,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Bridge parameters for pricing calculations
#[derive(Debug, Clone)]
pub struct BridgeParameters {
    pub from_chain: String,
    pub to_chain: String,
    pub bridge_type: BridgeType,
    pub security_level: SecurityLevel,
}

/// Bridge type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeType {
    Atomic,
    Optimistic,
    Canonical,
}

/// Security level for bridge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Standard,
    Enhanced,
    Maximum,
}

/// Dynamic pricing error types
#[derive(Debug, thiserror::Error)]
pub enum DynamicPricingError {
    #[error("Price oracle error: {0}")]
    OracleError(String),
    
    #[error("1inch service error: {0}")]
    OneinchError(String),
    
    #[error("Exchange rate calculation error: {0}")]
    ExchangeRateError(String),
    
    #[error("Fee calculation error: {0}")]
    FeeCalculationError(String),
    
    #[error("Price impact analysis error: {0}")]
    PriceImpactError(String),
    
    #[error("Slippage control error: {0}")]
    SlippageControlError(String),
    
    #[error("Quote validation error: {0}")]
    QuoteValidationError(String),
    
    #[error("Insufficient liquidity for amount: {0}")]
    InsufficientLiquidity(String),
    
    #[error("Market volatility too high: {0}")]
    HighVolatility(String),
    
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl From<PriceError> for DynamicPricingError {
    fn from(error: PriceError) -> Self {
        DynamicPricingError::OracleError(error.to_string())
    }
}

impl From<OneinchError> for DynamicPricingError {
    fn from(error: OneinchError) -> Self {
        DynamicPricingError::OneinchError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_quote_request_serialization() {
        let request = BridgeQuoteRequest {
            from_token: "ETH".to_string(),
            to_token: "NEAR".to_string(),
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            from_amount: BigDecimal::from(1),
            max_slippage: Some(0.5),
            user_id: Some("user123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ETH"));
        assert!(json.contains("NEAR"));
    }

    #[test]
    fn test_impact_level_variants() {
        let levels = vec![
            ImpactLevel::Low,
            ImpactLevel::Medium,
            ImpactLevel::High,
            ImpactLevel::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            assert!(!json.is_empty());
        }
    }
}