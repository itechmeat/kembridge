// src/oneinch/slippage.rs - Slippage protection for 1inch Fusion+ integration

use super::types::*;
use crate::constants::*;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Slippage protection service
pub struct SlippageProtection {
    max_slippage: f64,
    price_impact_threshold: f64,
}

impl SlippageProtection {
    /// Create new SlippageProtection instance
    pub fn new() -> Self {
        Self {
            max_slippage: ONEINCH_MAX_SLIPPAGE,
            price_impact_threshold: 2.0, // 2% price impact threshold
        }
    }

    /// Create with custom settings
    pub fn with_settings(max_slippage: f64, price_impact_threshold: f64) -> Self {
        Self {
            max_slippage,
            price_impact_threshold,
        }
    }

    /// Validate slippage parameters
    pub fn validate_slippage(&self, slippage: f64) -> Result<(), OneinchError> {
        if slippage < ONEINCH_MIN_SLIPPAGE {
            return Err(OneinchError::InvalidParams(
                format!("Slippage too low: {}% < {}%", slippage, ONEINCH_MIN_SLIPPAGE)
            ));
        }

        if slippage > self.max_slippage {
            return Err(OneinchError::SlippageTooHigh {
                actual: slippage,
                max: self.max_slippage,
            });
        }

        Ok(())
    }

    /// Calculate optimal slippage based on market conditions
    pub fn calculate_optimal_slippage(&self, params: &SlippageCalculationParams) -> SlippageRecommendation {
        let mut base_slippage = ONEINCH_DEFAULT_SLIPPAGE;

        // Adjust for volatility
        if params.volatility > 5.0 {
            base_slippage += 0.5; // Add 0.5% for high volatility
        } else if params.volatility > 2.0 {
            base_slippage += 0.2; // Add 0.2% for medium volatility
        }

        // Adjust for liquidity
        if params.liquidity_score < 0.3 {
            base_slippage += 1.0; // Add 1% for low liquidity
        } else if params.liquidity_score < 0.6 {
            base_slippage += 0.3; // Add 0.3% for medium liquidity
        }

        // Adjust for trade size
        if params.trade_size_ratio > 0.1 {
            base_slippage += 0.5; // Add 0.5% for large trades
        } else if params.trade_size_ratio > 0.05 {
            base_slippage += 0.2; // Add 0.2% for medium trades
        }

        // Adjust for market conditions
        match params.market_condition {
            MarketCondition::Bull => base_slippage += 0.1,
            MarketCondition::Bear => base_slippage += 0.3,
            MarketCondition::Sideways => {}, // No adjustment
            MarketCondition::HighVolatility => base_slippage += 0.5,
        }

        // Ensure we don't exceed limits
        let recommended_slippage = base_slippage.min(self.max_slippage).max(ONEINCH_MIN_SLIPPAGE);

        let confidence = self.calculate_confidence(&params);
        let warning = self.get_slippage_warning(recommended_slippage, &params);

        SlippageRecommendation {
            recommended_slippage,
            confidence,
            reasoning: self.get_reasoning(base_slippage, recommended_slippage, &params),
            warning,
        }
    }

    /// Calculate confidence score for slippage recommendation
    fn calculate_confidence(&self, params: &SlippageCalculationParams) -> f64 {
        let mut confidence: f64 = 1.0;

        // Reduce confidence for high volatility
        if params.volatility > 5.0 {
            confidence -= 0.3;
        } else if params.volatility > 2.0 {
            confidence -= 0.1;
        }

        // Reduce confidence for low liquidity
        if params.liquidity_score < 0.3 {
            confidence -= 0.4;
        } else if params.liquidity_score < 0.6 {
            confidence -= 0.2;
        }

        // Reduce confidence for large trades
        if params.trade_size_ratio > 0.1 {
            confidence -= 0.2;
        }

        confidence.max(0.1) // Minimum 10% confidence
    }

    /// Get reasoning for slippage recommendation
    fn get_reasoning(&self, base_slippage: f64, final_slippage: f64, params: &SlippageCalculationParams) -> String {
        let mut reasons = Vec::new();

        if params.volatility > 5.0 {
            reasons.push("High market volatility detected".to_string());
        } else if params.volatility > 2.0 {
            reasons.push("Medium market volatility".to_string());
        }

        if params.liquidity_score < 0.3 {
            reasons.push("Low liquidity for this pair".to_string());
        } else if params.liquidity_score < 0.6 {
            reasons.push("Medium liquidity".to_string());
        }

        if params.trade_size_ratio > 0.1 {
            reasons.push("Large trade size relative to liquidity".to_string());
        }

        match params.market_condition {
            MarketCondition::HighVolatility => reasons.push("High volatility market conditions".to_string()),
            MarketCondition::Bear => reasons.push("Bearish market conditions".to_string()),
            _ => {}
        }

        if final_slippage != base_slippage {
            if final_slippage == self.max_slippage {
                reasons.push("Capped at maximum allowed slippage".to_string());
            } else if final_slippage == ONEINCH_MIN_SLIPPAGE {
                reasons.push("Set to minimum allowed slippage".to_string());
            }
        }

        if reasons.is_empty() {
            "Standard market conditions".to_string()
        } else {
            reasons.join(", ")
        }
    }

    /// Get warning message if needed
    fn get_slippage_warning(&self, slippage: f64, params: &SlippageCalculationParams) -> Option<String> {
        if slippage > 3.0 {
            Some("High slippage tolerance may result in significant price impact".to_string())
        } else if params.liquidity_score < 0.2 {
            Some("Very low liquidity detected - consider splitting large orders".to_string())
        } else if params.trade_size_ratio > 0.15 {
            Some("Large trade size may cause significant market impact".to_string())
        } else {
            None
        }
    }

    /// Check if actual slippage is acceptable
    pub fn check_actual_slippage(
        &self,
        expected_amount: &BigDecimal,
        actual_amount: &BigDecimal,
        max_allowed_slippage: f64,
    ) -> Result<SlippageAnalysis, OneinchError> {
        let slippage_amount = expected_amount - actual_amount;
        let slippage_percent = if expected_amount > &BigDecimal::from(0) {
            (&slippage_amount / expected_amount) * BigDecimal::from(100)
        } else {
            BigDecimal::from(0)
        };

        let slippage_float = slippage_percent.to_string().parse::<f64>()
            .unwrap_or(0.0);

        let analysis = SlippageAnalysis {
            expected_amount: expected_amount.clone(),
            actual_amount: actual_amount.clone(),
            slippage_amount: slippage_amount.clone(),
            slippage_percent: slippage_percent.clone(),
            is_acceptable: slippage_float <= max_allowed_slippage,
            severity: self.get_slippage_severity(slippage_float),
        };

        if !analysis.is_acceptable {
            warn!(
                "Excessive slippage detected: {}% > {}%",
                slippage_float, max_allowed_slippage
            );
            return Err(OneinchError::SlippageTooHigh {
                actual: slippage_float,
                max: max_allowed_slippage,
            });
        }

        debug!(
            "Slippage analysis: expected: {}, actual: {}, slippage: {}%",
            expected_amount, actual_amount, slippage_float
        );

        Ok(analysis)
    }

    /// Get slippage severity level
    fn get_slippage_severity(&self, slippage: f64) -> SlippageSeverity {
        if slippage < 0.5 {
            SlippageSeverity::Low
        } else if slippage < 1.0 {
            SlippageSeverity::Medium
        } else if slippage < 2.0 {
            SlippageSeverity::High
        } else {
            SlippageSeverity::Critical
        }
    }

    /// Create slippage configuration
    pub fn create_config(&self, slippage: f64, deadline_minutes: u64) -> SlippageConfig {
        SlippageConfig {
            max_slippage: slippage,
            min_return_amount: BigDecimal::from(0), // Will be calculated based on quote
            price_impact_threshold: self.price_impact_threshold,
            deadline: (Utc::now() + Duration::minutes(deadline_minutes as i64)).timestamp() as u64,
        }
    }

    /// Update minimum return amount based on quote
    pub fn update_min_return_amount(&self, config: &mut SlippageConfig, quote: &FusionQuote) {
        let multiplier_percent = 100.0 - config.max_slippage;
        let slippage_multiplier = BigDecimal::from(multiplier_percent as i32) / BigDecimal::from(100);
        config.min_return_amount = &quote.to_amount * slippage_multiplier;
    }

    /// Check if deadline is still valid
    pub fn is_deadline_valid(&self, config: &SlippageConfig) -> bool {
        Utc::now().timestamp() < config.deadline as i64
    }

    /// Get adaptive slippage for different trading pairs
    pub fn get_adaptive_slippage(&self, pair_type: TradingPairType) -> f64 {
        match pair_type {
            TradingPairType::StablecoinToStablecoin => ONEINCH_MIN_SLIPPAGE,
            TradingPairType::MajorToMajor => ONEINCH_DEFAULT_SLIPPAGE,
            TradingPairType::MajorToMinor => ONEINCH_DEFAULT_SLIPPAGE + 0.3,
            TradingPairType::MinorToMinor => ONEINCH_DEFAULT_SLIPPAGE + 0.5,
            TradingPairType::ExoticPair => ONEINCH_DEFAULT_SLIPPAGE + 1.0,
        }
    }
}

impl Default for SlippageProtection {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for slippage calculation
#[derive(Debug, Clone)]
pub struct SlippageCalculationParams {
    pub volatility: f64,           // Market volatility percentage
    pub liquidity_score: f64,      // 0.0 to 1.0
    pub trade_size_ratio: f64,     // Trade size / total liquidity
    pub market_condition: MarketCondition,
}

/// Market condition types
#[derive(Debug, Clone)]
pub enum MarketCondition {
    Bull,
    Bear,
    Sideways,
    HighVolatility,
}

/// Slippage recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageRecommendation {
    pub recommended_slippage: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub warning: Option<String>,
}

/// Slippage analysis result
#[derive(Debug, Clone)]
pub struct SlippageAnalysis {
    pub expected_amount: BigDecimal,
    pub actual_amount: BigDecimal,
    pub slippage_amount: BigDecimal,
    pub slippage_percent: BigDecimal,
    pub is_acceptable: bool,
    pub severity: SlippageSeverity,
}

/// Slippage severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum SlippageSeverity {
    Low,      // < 0.5%
    Medium,   // 0.5% - 1.0%
    High,     // 1.0% - 2.0%
    Critical, // > 2.0%
}

/// Trading pair types for adaptive slippage
#[derive(Debug, Clone)]
pub enum TradingPairType {
    StablecoinToStablecoin,
    MajorToMajor,
    MajorToMinor,
    MinorToMinor,
    ExoticPair,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slippage_validation() {
        let protection = SlippageProtection::new();
        
        // Valid slippage
        assert!(protection.validate_slippage(ONEINCH_DEFAULT_SLIPPAGE).is_ok());
        
        // Too low slippage
        assert!(protection.validate_slippage(0.05).is_err());
        
        // Too high slippage
        assert!(protection.validate_slippage(10.0).is_err());
    }

    #[test]
    fn test_slippage_severity() {
        let protection = SlippageProtection::new();
        
        assert_eq!(protection.get_slippage_severity(0.3), SlippageSeverity::Low);
        assert_eq!(protection.get_slippage_severity(0.8), SlippageSeverity::Medium);
        assert_eq!(protection.get_slippage_severity(1.5), SlippageSeverity::High);
        assert_eq!(protection.get_slippage_severity(3.0), SlippageSeverity::Critical);
    }

    #[test]
    fn test_adaptive_slippage() {
        let protection = SlippageProtection::new();
        
        assert_eq!(protection.get_adaptive_slippage(TradingPairType::StablecoinToStablecoin), ONEINCH_MIN_SLIPPAGE);
        assert_eq!(protection.get_adaptive_slippage(TradingPairType::MajorToMajor), ONEINCH_DEFAULT_SLIPPAGE);
        assert!(protection.get_adaptive_slippage(TradingPairType::ExoticPair) > ONEINCH_DEFAULT_SLIPPAGE);
    }

    #[test]
    fn test_optimal_slippage_calculation() {
        let protection = SlippageProtection::new();
        
        let params = SlippageCalculationParams {
            volatility: 1.0,
            liquidity_score: 0.8,
            trade_size_ratio: 0.02,
            market_condition: MarketCondition::Sideways,
        };
        
        let recommendation = protection.calculate_optimal_slippage(&params);
        assert!(recommendation.recommended_slippage >= ONEINCH_MIN_SLIPPAGE);
        assert!(recommendation.recommended_slippage <= ONEINCH_MAX_SLIPPAGE);
        assert!(recommendation.confidence > 0.0 && recommendation.confidence <= 1.0);
    }

    #[test]
    fn test_slippage_analysis() {
        let protection = SlippageProtection::new();
        
        let expected = BigDecimal::from(1000);
        let actual = BigDecimal::from(995); // 0.5% slippage
        
        let analysis = protection.check_actual_slippage(&expected, &actual, 1.0).unwrap();
        
        assert!(analysis.is_acceptable);
        assert_eq!(analysis.severity, SlippageSeverity::Medium);
        let expected_slippage = BigDecimal::from(1) / BigDecimal::from(2);
        assert_eq!(analysis.slippage_percent, expected_slippage);
    }

    #[test]
    fn test_config_creation() {
        let protection = SlippageProtection::new();
        
        let config = protection.create_config(1.0, 5);
        assert_eq!(config.max_slippage, 1.0);
        assert!(config.deadline > Utc::now().timestamp() as u64);
    }

    #[test]
    fn test_deadline_validation() {
        let protection = SlippageProtection::new();
        
        let mut config = protection.create_config(1.0, 5);
        assert!(protection.is_deadline_valid(&config));
        
        // Set deadline in the past
        config.deadline = (Utc::now() - Duration::minutes(1)).timestamp() as u64;
        assert!(!protection.is_deadline_valid(&config));
    }
}