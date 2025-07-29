// src/oneinch/quote_engine.rs - Quote engine for 1inch Fusion+ integration

use super::{client::FusionClient, types::*};
use crate::constants::*;
use bigdecimal::BigDecimal;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Quote engine for managing 1inch quotes and price comparisons
pub struct QuoteEngine {
    client: Arc<FusionClient>,
}

impl QuoteEngine {
    /// Create new QuoteEngine instance
    pub fn new(client: Arc<FusionClient>) -> Self {
        Self { client }
    }

    /// Get quote with enhanced validation
    pub async fn get_quote(&self, params: &QuoteParams) -> Result<FusionQuote, OneinchError> {
        info!("Getting 1inch quote: {} -> {}", params.from_token, params.to_token);

        // Validate quote parameters
        self.validate_quote_params(params)?;

        // Get quote from 1inch API
        let quote = self.client.get_quote(params).await?;

        // Validate quote response
        self.validate_quote_response(&quote)?;

        debug!(
            "1inch quote received: {} {} -> {} {} (rate: {})",
            quote.from_amount,
            quote.from_token.symbol,
            quote.to_amount,
            quote.to_token.symbol,
            self.calculate_exchange_rate(&quote.from_amount, &quote.to_amount)
        );

        Ok(quote)
    }

    /// Get multiple quotes for comparison
    pub async fn get_multiple_quotes(&self, params: &QuoteParams, slippages: &[f64]) -> Result<Vec<FusionQuote>, OneinchError> {
        let mut quotes = Vec::new();

        for &slippage in slippages {
            let mut quote_params = params.clone();
            quote_params.slippage = Some(slippage);

            match self.get_quote(&quote_params).await {
                Ok(quote) => quotes.push(quote),
                Err(e) => {
                    warn!("Failed to get quote with slippage {}: {:?}", slippage, e);
                }
            }
        }

        if quotes.is_empty() {
            return Err(OneinchError::InsufficientLiquidity);
        }

        Ok(quotes)
    }

    /// Find best quote from multiple options
    pub fn find_best_quote<'a>(&self, quotes: &'a [FusionQuote], criteria: &QuoteSelectionCriteria) -> Option<&'a FusionQuote> {
        if quotes.is_empty() {
            return None;
        }

        let best_quote = match criteria {
            QuoteSelectionCriteria::MaxOutput => {
                quotes.iter().max_by(|a, b| a.to_amount.cmp(&b.to_amount))
            }
            QuoteSelectionCriteria::MinGas => {
                quotes.iter().min_by(|a, b| a.estimated_gas.cmp(&b.estimated_gas))
            }
            QuoteSelectionCriteria::BestEfficiency => {
                // Calculate efficiency as output amount per gas unit
                quotes.iter().max_by(|a, b| {
                    let efficiency_a = &a.to_amount / &a.estimated_gas;
                    let efficiency_b = &b.to_amount / &b.estimated_gas;
                    efficiency_a.cmp(&efficiency_b)
                })
            }
            QuoteSelectionCriteria::FastestExecution => {
                // For now, prefer quotes with less complex routing (fewer protocols)
                quotes.iter().min_by_key(|quote| quote.protocols.len())
            }
        };

        best_quote
    }

    /// Compare quote with price oracle data
    pub async fn compare_with_oracle(&self, quote: &FusionQuote, oracle_price: &BigDecimal) -> Result<QuoteComparison, OneinchError> {
        let quote_rate = self.calculate_exchange_rate(&quote.from_amount, &quote.to_amount);
        let price_difference = (&quote_rate - oracle_price) / oracle_price * BigDecimal::from(100);

        let comparison = QuoteComparison {
            oneinch_rate: quote_rate,
            oracle_rate: oracle_price.clone(),
            price_difference_percent: price_difference.clone(),
            is_favorable: price_difference >= BigDecimal::from(0),
            recommendation: self.get_recommendation(&price_difference),
        };

        info!(
            "Quote comparison: 1inch rate: {}, Oracle rate: {}, Difference: {}%",
            comparison.oneinch_rate, comparison.oracle_rate, comparison.price_difference_percent
        );

        Ok(comparison)
    }

    /// Calculate exchange rate from amounts
    fn calculate_exchange_rate(&self, from_amount: &BigDecimal, to_amount: &BigDecimal) -> BigDecimal {
        if from_amount == &BigDecimal::from(0) {
            return BigDecimal::from(0);
        }
        to_amount / from_amount
    }

    /// Get recommendation based on price difference
    fn get_recommendation(&self, price_difference: &BigDecimal) -> QuoteRecommendation {
        let diff = price_difference.clone();
        
        if diff >= BigDecimal::from(2) {
            QuoteRecommendation::HighlyRecommended
        } else if diff >= BigDecimal::from(0) {
            QuoteRecommendation::Recommended
        } else if diff >= BigDecimal::from(-1) {
            QuoteRecommendation::Acceptable
        } else if diff >= BigDecimal::from(-3) {
            QuoteRecommendation::Caution
        } else {
            QuoteRecommendation::NotRecommended
        }
    }

    /// Validate quote parameters
    fn validate_quote_params(&self, params: &QuoteParams) -> Result<(), OneinchError> {
        // Check amount is positive
        if params.amount <= BigDecimal::from(0) {
            return Err(OneinchError::InvalidParams(
                "Amount must be positive".to_string()
            ));
        }

        // Check addresses format
        if !self.is_valid_address(&params.from_token) {
            return Err(OneinchError::InvalidParams(
                "Invalid from_token address".to_string()
            ));
        }

        if !self.is_valid_address(&params.to_token) {
            return Err(OneinchError::InvalidParams(
                "Invalid to_token address".to_string()
            ));
        }

        if !self.is_valid_address(&params.from_address) {
            return Err(OneinchError::InvalidParams(
                "Invalid from_address".to_string()
            ));
        }

        // Check slippage if provided
        if let Some(slippage) = params.slippage {
            if slippage < ONEINCH_MIN_SLIPPAGE || slippage > ONEINCH_MAX_SLIPPAGE {
                return Err(OneinchError::InvalidParams(
                    format!("Slippage must be between {}% and {}%", ONEINCH_MIN_SLIPPAGE, ONEINCH_MAX_SLIPPAGE)
                ));
            }
        }

        Ok(())
    }

    /// Validate quote response
    fn validate_quote_response(&self, quote: &FusionQuote) -> Result<(), OneinchError> {
        // Check if quote is not expired
        if quote.expires_at < Utc::now() {
            return Err(OneinchError::OrderExpired);
        }

        // Check amounts are positive
        if quote.from_amount <= BigDecimal::from(0) || quote.to_amount <= BigDecimal::from(0) {
            return Err(OneinchError::InvalidParams(
                "Quote amounts must be positive".to_string()
            ));
        }

        // TODO (check): Interesting
        // NOTE: Empty protocols list doesn't mean insufficient liquidity
        // 1inch Swap API may return valid quotes without detailed protocol info
        // Only check for insufficient liquidity if we have zero to_amount
        if quote.to_amount == BigDecimal::from(0) {
            return Err(OneinchError::InsufficientLiquidity);
        }

        Ok(())
    }

    /// Basic Ethereum address validation
    fn is_valid_address(&self, address: &str) -> bool {
        address.starts_with("0x") && address.len() == 42 && address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Get quote expiration time
    pub fn get_quote_expiration(&self, quote: &FusionQuote) -> Duration {
        quote.expires_at - Utc::now()
    }

    /// Check if quote is still valid
    pub fn is_quote_valid(&self, quote: &FusionQuote) -> bool {
        quote.expires_at > Utc::now()
    }

    /// Calculate price impact
    pub fn calculate_price_impact(&self, input_amount: &BigDecimal, output_amount: &BigDecimal, market_price: &BigDecimal) -> BigDecimal {
        let expected_output = input_amount * market_price;
        if expected_output == BigDecimal::from(0) {
            return BigDecimal::from(0);
        }
        
        ((output_amount - &expected_output) / &expected_output) * BigDecimal::from(100)
    }

    /// Get quote summary for logging/display
    pub fn get_quote_summary(&self, quote: &FusionQuote) -> QuoteSummary {
        QuoteSummary {
            from_symbol: quote.from_token.symbol.clone(),
            to_symbol: quote.to_token.symbol.clone(),
            from_amount: quote.from_amount.clone(),
            to_amount: quote.to_amount.clone(),
            exchange_rate: self.calculate_exchange_rate(&quote.from_amount, &quote.to_amount),
            estimated_gas: quote.estimated_gas.clone(),
            protocols_count: quote.protocols.len(),
            expires_in_seconds: self.get_quote_expiration(quote).num_seconds(),
        }
    }
}

/// Quote selection criteria
#[derive(Debug, Clone)]
pub enum QuoteSelectionCriteria {
    MaxOutput,
    MinGas,
    BestEfficiency,
    FastestExecution,
}

/// Quote comparison result
#[derive(Debug, Clone)]
pub struct QuoteComparison {
    pub oneinch_rate: BigDecimal,
    pub oracle_rate: BigDecimal,
    pub price_difference_percent: BigDecimal,
    pub is_favorable: bool,
    pub recommendation: QuoteRecommendation,
}

/// Quote recommendation
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteRecommendation {
    HighlyRecommended,
    Recommended,
    Acceptable,
    Caution,
    NotRecommended,
}

/// Quote summary for display
#[derive(Debug, Clone)]
pub struct QuoteSummary {
    pub from_symbol: String,
    pub to_symbol: String,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub exchange_rate: BigDecimal,
    pub estimated_gas: BigDecimal,
    pub protocols_count: usize,
    pub expires_in_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    fn create_test_quote_engine() -> QuoteEngine {
        let client = Arc::new(FusionClient::new(
            "test_key".to_string(),
            ONEINCH_SEPOLIA_CHAIN_ID,
        ));
        QuoteEngine::new(client)
    }

    #[test]
    fn test_exchange_rate_calculation() {
        let engine = create_test_quote_engine();
        
        let from_amount = BigDecimal::from(1000);
        let to_amount = BigDecimal::from(2000);
        let rate = engine.calculate_exchange_rate(&from_amount, &to_amount);
        
        assert_eq!(rate, BigDecimal::from(2));
    }

    #[test]
    fn test_exchange_rate_zero_amount() {
        let engine = create_test_quote_engine();
        
        let from_amount = BigDecimal::from(0);
        let to_amount = BigDecimal::from(2000);
        let rate = engine.calculate_exchange_rate(&from_amount, &to_amount);
        
        assert_eq!(rate, BigDecimal::from(0));
    }

    #[test]
    fn test_recommendation_logic() {
        let engine = create_test_quote_engine();
        
        assert_eq!(engine.get_recommendation(&BigDecimal::from(5)), QuoteRecommendation::HighlyRecommended);
        assert_eq!(engine.get_recommendation(&BigDecimal::from(1)), QuoteRecommendation::Recommended);
        let negative_half = &(BigDecimal::from(-1) / BigDecimal::from(2));
        assert_eq!(engine.get_recommendation(negative_half), QuoteRecommendation::Acceptable);
        assert_eq!(engine.get_recommendation(&BigDecimal::from(-2)), QuoteRecommendation::Caution);
        assert_eq!(engine.get_recommendation(&BigDecimal::from(-5)), QuoteRecommendation::NotRecommended);
    }

    #[test]
    fn test_price_impact_calculation() {
        let engine = create_test_quote_engine();
        
        let input_amount = BigDecimal::from(1000);
        let output_amount = BigDecimal::from(1950); // 5% slippage
        let market_price = BigDecimal::from(2); // Expected: 2000
        
        let price_impact = engine.calculate_price_impact(&input_amount, &output_amount, &market_price);
        let expected_impact = BigDecimal::from(-5) / BigDecimal::from(2);
        assert_eq!(price_impact, expected_impact); // -2.5% price impact
    }

    #[test]
    fn test_address_validation() {
        let engine = create_test_quote_engine();
        
        assert!(engine.is_valid_address("0x1234567890123456789012345678901234567890"));
        assert!(!engine.is_valid_address("invalid_address"));
        assert!(!engine.is_valid_address("0x123")); // Too short
    }

    #[test]
    fn test_quote_params_validation() {
        let engine = create_test_quote_engine();
        
        let valid_params = QuoteParams {
            from_token: "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D".to_string(),
            to_token: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: BigDecimal::from(1000),
            from_address: "0x1234567890123456789012345678901234567890".to_string(),
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge".to_string()),
        };
        
        assert!(engine.validate_quote_params(&valid_params).is_ok());
        
        // Test invalid amount
        let mut invalid_params = valid_params.clone();
        invalid_params.amount = BigDecimal::from(0);
        assert!(engine.validate_quote_params(&invalid_params).is_err());
        
        // Test invalid slippage
        invalid_params.amount = BigDecimal::from(1000);
        invalid_params.slippage = Some(100.0); // 100% slippage
        assert!(engine.validate_quote_params(&invalid_params).is_err());
    }
}