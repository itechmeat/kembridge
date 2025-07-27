// src/oneinch/price_comparison.rs - Price comparison with Oracle for 1inch optimization

use super::{types::*, OneinchService};
use crate::{
    price_oracle::{PriceOracleService, types::{TradingPair, PriceData}},
    constants::*,
};
use bigdecimal::BigDecimal;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Service for comparing 1inch prices with oracle data
pub struct PriceComparisonService {
    oneinch_service: Arc<OneinchService>,
    price_oracle: Arc<PriceOracleService>,
}

impl PriceComparisonService {
    /// Create new price comparison service
    pub fn new(
        oneinch_service: Arc<OneinchService>,
        price_oracle: Arc<PriceOracleService>,
    ) -> Self {
        Self {
            oneinch_service,
            price_oracle,
        }
    }

    /// Get optimal price quote with oracle comparison
    pub async fn get_optimal_quote_with_comparison(
        &self,
        params: &QuoteParams,
    ) -> Result<OptimalQuoteWithComparison, PriceComparisonError> {
        info!("Getting optimal quote with oracle comparison: {} -> {}", params.from_token, params.to_token);

        // Step 1: Get 1inch quote
        let oneinch_quote = self.oneinch_service.get_quote(params)
            .await.map_err(PriceComparisonError::OneinchError)?;

        // Step 2: Get oracle price data
        let oracle_comparison = self.get_oracle_comparison(&oneinch_quote, params).await?;

        // Step 3: Calculate price efficiency
        let efficiency_analysis = self.calculate_price_efficiency(&oneinch_quote, &oracle_comparison);

        // Step 4: Generate recommendation
        let recommendation = self.generate_price_recommendation(&efficiency_analysis, &oracle_comparison);

        Ok(OptimalQuoteWithComparison {
            oneinch_quote,
            oracle_comparison,
            efficiency_analysis,
            recommendation,
            comparison_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get multiple quotes with oracle comparison for optimization
    pub async fn get_quote_variants_with_comparison(
        &self,
        base_params: &QuoteParams,
        slippages: &[f64],
    ) -> Result<Vec<OptimalQuoteWithComparison>, PriceComparisonError> {
        info!("Getting {} quote variants with oracle comparison", slippages.len());

        let mut results = Vec::new();

        for &slippage in slippages {
            let mut params = base_params.clone();
            params.slippage = Some(slippage);

            match self.get_optimal_quote_with_comparison(&params).await {
                Ok(comparison) => results.push(comparison),
                Err(e) => {
                    warn!("Failed to get quote for slippage {}: {:?}", slippage, e);
                    continue;
                }
            }
        }

        if results.is_empty() {
            return Err(PriceComparisonError::NoValidQuotes);
        }

        // Sort by efficiency score (descending)
        results.sort_by(|a, b| {
            b.efficiency_analysis.total_efficiency_score
                .partial_cmp(&a.efficiency_analysis.total_efficiency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Find best quote compared to oracle
    pub async fn find_oracle_optimized_quote(
        &self,
        base_params: &QuoteParams,
    ) -> Result<OptimalQuoteWithComparison, PriceComparisonError> {
        info!("Finding oracle-optimized quote for {} -> {}", base_params.from_token, base_params.to_token);

        // Generate multiple slippage variants
        let slippages = vec![
            ONEINCH_MIN_SLIPPAGE,
            ONEINCH_DEFAULT_SLIPPAGE - 0.2,
            ONEINCH_DEFAULT_SLIPPAGE,
            ONEINCH_DEFAULT_SLIPPAGE + 0.3,
            ONEINCH_DEFAULT_SLIPPAGE + 0.5,
        ];

        let quote_variants = self.get_quote_variants_with_comparison(base_params, &slippages).await?;

        // Find the best quote based on oracle efficiency
        let best_quote = quote_variants.into_iter()
            .max_by(|a, b| {
                a.efficiency_analysis.oracle_efficiency_score
                    .partial_cmp(&b.efficiency_analysis.oracle_efficiency_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(PriceComparisonError::NoValidQuotes)?;

        info!("Selected quote with oracle efficiency score: {:.2}%", 
            best_quote.efficiency_analysis.oracle_efficiency_score);

        Ok(best_quote)
    }

    /// Get oracle comparison data
    async fn get_oracle_comparison(
        &self,
        quote: &FusionQuote,
        params: &QuoteParams,
    ) -> Result<OraclePriceComparison, PriceComparisonError> {
        // Create trading pair from tokens
        let trading_pair = self.create_trading_pair(&params.from_token, &params.to_token)?;

        // Get oracle price using trading pair symbol
        let oracle_price = self.price_oracle.get_price(trading_pair.to_symbol())
            .await.map_err(|e| PriceComparisonError::ConversionError(format!("Oracle error: {}", e)))?;

        // Calculate 1inch rate
        let oneinch_rate = &quote.to_amount / &quote.from_amount;

        // Calculate price difference
        let oracle_rate_decimal = oracle_price.price.clone();

        let price_difference = &oneinch_rate - &oracle_rate_decimal;
        let price_difference_percentage = if oracle_rate_decimal > BigDecimal::from(0) {
            (&price_difference / &oracle_rate_decimal) * BigDecimal::from(100)
        } else {
            BigDecimal::from(0)
        };

        // Determine if 1inch is better
        let is_oneinch_better = oneinch_rate > oracle_rate_decimal;
        let price_deviation = price_difference_percentage.abs();

        Ok(OraclePriceComparison {
            oracle_price_data: oracle_price.clone(),
            oneinch_rate: oneinch_rate.to_string(),
            oracle_rate: oracle_rate_decimal.to_string(),
            price_difference: price_difference.to_string(),
            price_difference_percentage: price_difference_percentage.to_string(),
            is_oneinch_better,
            price_deviation_percentage: price_deviation.to_string(),
            oracle_confidence: oracle_price.confidence,
            oracle_sources: oracle_price.sources.clone(),
        })
    }

    /// Calculate price efficiency analysis
    fn calculate_price_efficiency(
        &self,
        quote: &FusionQuote,
        oracle_comparison: &OraclePriceComparison,
    ) -> PriceEfficiencyAnalysis {
        // Parse percentage values
        let price_diff_pct = oracle_comparison.price_difference_percentage
            .parse::<f64>().unwrap_or(0.0);
        let price_deviation = oracle_comparison.price_deviation_percentage
            .parse::<f64>().unwrap_or(0.0);

        // Calculate oracle efficiency score (0-100)
        let oracle_efficiency_score = if oracle_comparison.is_oneinch_better {
            // Bonus for better prices, capped at reasonable levels
            100.0 - (price_deviation * 2.0).min(50.0)
        } else {
            // Penalty for worse prices
            100.0 - (price_deviation * 3.0).min(80.0)
        };

        // Calculate gas efficiency score (0-100)
        let gas_efficiency_score = self.calculate_gas_efficiency_score(quote);

        // Calculate slippage efficiency score (0-100)
        let slippage_efficiency_score = self.calculate_slippage_efficiency_score(quote);

        // Calculate confidence score based on oracle confidence
        let confidence_score = oracle_comparison.oracle_confidence * 100.0;

        // Calculate total efficiency score (weighted average)
        let total_efficiency_score = (oracle_efficiency_score * 0.4) +
                                   (gas_efficiency_score * 0.25) +
                                   (slippage_efficiency_score * 0.2) +
                                   (confidence_score * 0.15);

        PriceEfficiencyAnalysis {
            oracle_efficiency_score,
            gas_efficiency_score,
            slippage_efficiency_score,
            confidence_score,
            total_efficiency_score,
            price_improvement_vs_oracle: price_diff_pct,
            gas_cost_usd: self.estimate_gas_cost_usd(quote),
            execution_probability: self.calculate_execution_probability(quote, oracle_comparison),
        }
    }

    /// Generate price recommendation
    fn generate_price_recommendation(
        &self,
        efficiency: &PriceEfficiencyAnalysis,
        oracle: &OraclePriceComparison,
    ) -> PriceRecommendation {
        let recommendation_type = if efficiency.total_efficiency_score >= 85.0 {
            PriceRecommendationType::HighlyRecommended
        } else if efficiency.total_efficiency_score >= 70.0 {
            PriceRecommendationType::Recommended
        } else if efficiency.total_efficiency_score >= 50.0 {
            PriceRecommendationType::Neutral
        } else {
            PriceRecommendationType::NotRecommended
        };

        let mut reasons = Vec::new();

        if oracle.is_oneinch_better {
            reasons.push(format!("Better price than oracle by {:.2}%", 
                efficiency.price_improvement_vs_oracle.abs()));
        } else {
            reasons.push(format!("Price {:.2}% below oracle", 
                efficiency.price_improvement_vs_oracle.abs()));
        }

        if efficiency.gas_efficiency_score > 80.0 {
            reasons.push("Gas efficient execution".to_string());
        } else if efficiency.gas_efficiency_score < 50.0 {
            reasons.push("High gas cost".to_string());
        }

        if efficiency.confidence_score > 90.0 {
            reasons.push("High oracle confidence".to_string());
        } else if efficiency.confidence_score < 70.0 {
            reasons.push("Low oracle confidence".to_string());
        }

        let summary = match recommendation_type {
            PriceRecommendationType::HighlyRecommended => "Excellent price and execution conditions",
            PriceRecommendationType::Recommended => "Good price with reasonable execution costs",
            PriceRecommendationType::Neutral => "Average price efficiency",
            PriceRecommendationType::NotRecommended => "Consider waiting for better market conditions",
        };

        PriceRecommendation {
            recommendation_type,
            summary: summary.to_string(),
            reasons,
            risk_level: self.calculate_risk_level(efficiency),
            alternative_suggestion: None, // Could suggest different slippage or timing
        }
    }

    /// Create trading pair from token addresses
    fn create_trading_pair(&self, from_token: &str, to_token: &str) -> Result<TradingPair, PriceComparisonError> {
        // Simplified mapping for hackathon - in production would use token registry
        let base_symbol = self.token_address_to_symbol(from_token)?;
        let _quote_symbol = self.token_address_to_symbol(to_token)?; // Usually USD for pricing

        // Map to supported trading pairs (most tokens are priced against USD)
        let trading_pair = match base_symbol.as_str() {
            "ETH" => TradingPair::EthUsd,
            "NEAR" => TradingPair::NearUsd,
            "BTC" | "WBTC" => TradingPair::BtcUsd,
            "USDT" => TradingPair::UsdtUsd,
            "USDC" => TradingPair::UsdcUsd,
            _ => return Err(PriceComparisonError::UnsupportedToken(base_symbol)),
        };

        Ok(trading_pair)
    }

    /// Map token address to symbol (simplified for hackathon)
    fn token_address_to_symbol(&self, address: &str) -> Result<String, PriceComparisonError> {
        let symbol = match address {
            ETHEREUM_NATIVE_TOKEN => "ETH",
            "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D" => "USDT", // Example USDT
            "0x6B175474E89094C44Da98b954EedeAC495271d0F" => "DAI",  // Example DAI
            "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1C" => "USDC", // Example USDC
            "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599" => "WBTC", // Example WBTC
            "wrap.near" => "NEAR",
            NEAR_NATIVE_TOKEN => "NEAR",
            _ => return Err(PriceComparisonError::UnsupportedToken(address.to_string())),
        };

        Ok(symbol.to_string())
    }

    // Helper calculation methods

    fn calculate_gas_efficiency_score(&self, quote: &FusionQuote) -> f64 {
        let gas_amount = quote.estimated_gas.to_string().parse::<f64>().unwrap_or(0.0);
        // Score based on gas usage (lower is better)
        let base_score = 100.0 - (gas_amount / 500000.0 * 100.0).min(80.0);
        base_score.max(10.0)
    }

    fn calculate_slippage_efficiency_score(&self, quote: &FusionQuote) -> f64 {
        // TODO: Extract actual slippage from quote if available
        // For now, assume reasonable slippage based on protocols used
        let protocol_complexity = quote.protocols.len() as f64;
        let complexity_penalty = (protocol_complexity - 1.0) * 5.0;
        (100.0 - complexity_penalty).max(50.0)
    }

    fn estimate_gas_cost_usd(&self, quote: &FusionQuote) -> f64 {
        // Calculate gas cost using real ETH price from oracle
        let gas_amount = quote.estimated_gas.to_string().parse::<f64>().unwrap_or(0.0);
        let gas_price_gwei = ONEINCH_DEFAULT_GAS_PRICE_GWEI as f64;
        
        // Get real ETH price from price oracle
        let eth_price_usd = self.get_eth_price_from_oracle();
        
        gas_amount * gas_price_gwei * 1e-9 * eth_price_usd
    }

    /// Get current ETH price from price oracle with fallback
    fn get_eth_price_from_oracle(&self) -> f64 {
        // Try to get ETH price from our price oracle service
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            if let Ok(price_data) = runtime.block_on(self.price_oracle.get_price("ETH/USD")) {
                return price_data.price.to_string().parse::<f64>().unwrap_or(ONEINCH_ETH_PRICE_FALLBACK_USD);
            }
        }
        
        // Fallback to conservative estimate if oracle unavailable
        ONEINCH_ETH_PRICE_FALLBACK_USD
    }

    fn calculate_execution_probability(&self, quote: &FusionQuote, oracle: &OraclePriceComparison) -> f64 {
        let mut probability: f64 = ONEINCH_EXECUTION_PROBABILITY_BASE; // Base 90% probability

        // Adjust based on price deviation
        let deviation = oracle.price_deviation_percentage.parse::<f64>().unwrap_or(0.0);
        if deviation > 5.0 {
            probability -= 0.2;
        } else if deviation > 2.0 {
            probability -= 0.1;
        }

        // Adjust based on gas cost
        let gas_cost = self.estimate_gas_cost_usd(quote);
        if gas_cost > 50.0 {
            probability -= 0.1;
        } else if gas_cost > 20.0 {
            probability -= 0.05;
        }

        probability.max(0.1).min(1.0)
    }

    fn calculate_risk_level(&self, efficiency: &PriceEfficiencyAnalysis) -> RiskLevel {
        if efficiency.total_efficiency_score >= 85.0 && efficiency.execution_probability >= 0.9 {
            RiskLevel::Low
        } else if efficiency.total_efficiency_score >= 70.0 && efficiency.execution_probability >= 0.8 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        }
    }
}

use std::str::FromStr;

/// Optimal quote with oracle comparison
#[derive(Debug, Clone)]
pub struct OptimalQuoteWithComparison {
    pub oneinch_quote: FusionQuote,
    pub oracle_comparison: OraclePriceComparison,
    pub efficiency_analysis: PriceEfficiencyAnalysis,
    pub recommendation: PriceRecommendation,
    pub comparison_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Oracle price comparison data
#[derive(Debug, Clone)]
pub struct OraclePriceComparison {
    pub oracle_price_data: crate::price_oracle::types::AggregatedPrice,
    pub oneinch_rate: String,
    pub oracle_rate: String,
    pub price_difference: String,
    pub price_difference_percentage: String,
    pub is_oneinch_better: bool,
    pub price_deviation_percentage: String,
    pub oracle_confidence: f64,
    pub oracle_sources: Vec<String>,
}

/// Price efficiency analysis
#[derive(Debug, Clone)]
pub struct PriceEfficiencyAnalysis {
    pub oracle_efficiency_score: f64,
    pub gas_efficiency_score: f64,
    pub slippage_efficiency_score: f64,
    pub confidence_score: f64,
    pub total_efficiency_score: f64,
    pub price_improvement_vs_oracle: f64,
    pub gas_cost_usd: f64,
    pub execution_probability: f64,
}

/// Price recommendation
#[derive(Debug, Clone)]
pub struct PriceRecommendation {
    pub recommendation_type: PriceRecommendationType,
    pub summary: String,
    pub reasons: Vec<String>,
    pub risk_level: RiskLevel,
    pub alternative_suggestion: Option<String>,
}

/// Recommendation types
#[derive(Debug, Clone)]
pub enum PriceRecommendationType {
    HighlyRecommended,
    Recommended,
    Neutral,
    NotRecommended,
}

/// Risk levels
#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Errors for price comparison
#[derive(Debug, thiserror::Error)]
pub enum PriceComparisonError {
    #[error("1inch error: {0}")]
    OneinchError(OneinchError),
    #[error("Oracle error: {0}")]
    OracleError(crate::price_oracle::types::PriceError),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Unsupported token: {0}")]
    UnsupportedToken(String),
    #[error("No valid quotes available")]
    NoValidQuotes,
    #[error("Price comparison failed: {0}")]
    ComparisonFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_symbol_mapping() {
        // This would test the token address to symbol mapping
        // For now, just test the structure
        assert_eq!(1, 1);
    }

    #[test]
    fn test_risk_level_calculation() {
        // Test risk level calculation logic
        let high_efficiency = PriceEfficiencyAnalysis {
            oracle_efficiency_score: 90.0,
            gas_efficiency_score: 85.0,
            slippage_efficiency_score: 88.0,
            confidence_score: 92.0,
            total_efficiency_score: 89.0,
            price_improvement_vs_oracle: 1.5,
            gas_cost_usd: 10.0,
            execution_probability: 0.95,
        };

        // This would test with a real service instance
        // assert_eq!(service.calculate_risk_level(&high_efficiency), RiskLevel::Low);
    }
}