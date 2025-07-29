// src/price_oracle/validator.rs - Price validation mechanisms
use bigdecimal::{BigDecimal as Decimal, FromPrimitive, ToPrimitive};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::price_oracle::types::{PriceData, PriceError, TradingPair, ValidationResult};

/// Price validator with configurable rules
pub struct PriceValidator {
    config: Arc<AppConfig>,
    validation_rules: ValidationRules,
    historical_prices: HashMap<String, Vec<PriceData>>, // For anomaly detection
}

/// Validation rules configuration
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub max_change_percent: f64,
    pub max_staleness_seconds: i64,
    pub min_confidence: f64,
    pub enable_anomaly_detection: bool,
    pub anomaly_threshold: f64,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_price: Decimal::from_f64(0.0001).unwrap(), // $0.0001 minimum
            max_price: Decimal::from(1000000),             // $1M maximum
            max_change_percent: 50.0,                      // 50% max change
            max_staleness_seconds: 300,                    // 5 minutes
            min_confidence: 0.1,                           // 10% minimum confidence
            enable_anomaly_detection: true,
            anomaly_threshold: 3.0, // 3 standard deviations
        }
    }
}

impl PriceValidator {
    /// Create new price validator
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            validation_rules: ValidationRules::default(),
            historical_prices: HashMap::new(),
        }
    }

    /// Create validator with custom rules
    pub fn with_rules(config: Arc<AppConfig>, rules: ValidationRules) -> Self {
        Self {
            config,
            validation_rules: rules,
            historical_prices: HashMap::new(),
        }
    }

    /// Validate price data
    pub fn validate_price(&self, price: &PriceData) -> Result<PriceData, PriceError> {
        info!("Validating price for {}: ${}", price.symbol, price.price);

        // Basic validation checks
        let validation_result = self.validate_basic_rules(price)?;
        if !validation_result.is_valid {
            return Err(PriceError::ValidationFailed(
                validation_result
                    .reason
                    .unwrap_or("Unknown validation error".to_string()),
            ));
        }

        // Advanced validation checks
        if let Err(e) = self.validate_advanced_rules(price) {
            warn!("Advanced validation failed for {}: {}", price.symbol, e);
            // Don't fail completely, just adjust confidence
            let mut adjusted_price = price.clone();
            adjusted_price.confidence *= 0.7; // Reduce confidence for suspicious data
            return Ok(adjusted_price);
        }

        // Anomaly detection
        if self.validation_rules.enable_anomaly_detection {
            if let Some(anomaly_score) = self.detect_anomaly(price) {
                if anomaly_score > self.validation_rules.anomaly_threshold {
                    warn!(
                        "Anomaly detected for {} with score {}",
                        price.symbol, anomaly_score
                    );
                    let mut adjusted_price = price.clone();
                    adjusted_price.confidence *= (1.0 - anomaly_score / 10.0).max(0.1);
                    return Ok(adjusted_price);
                }
            }
        }

        info!("Price validation successful for {}", price.symbol);
        Ok(price.clone())
    }

    /// Validate basic rules (price range, staleness, confidence)
    fn validate_basic_rules(&self, price: &PriceData) -> Result<ValidationResult, PriceError> {
        // Check price range
        if price.price < self.validation_rules.min_price {
            return Ok(ValidationResult {
                is_valid: false,
                reason: Some(format!("Price too low: ${}", price.price)),
                adjusted_confidence: 0.0,
            });
        }

        if price.price > self.validation_rules.max_price {
            return Ok(ValidationResult {
                is_valid: false,
                reason: Some(format!("Price too high: ${}", price.price)),
                adjusted_confidence: 0.0,
            });
        }

        // Check staleness
        let age_seconds = Utc::now()
            .signed_duration_since(price.timestamp)
            .num_seconds();
        if age_seconds > self.validation_rules.max_staleness_seconds {
            return Ok(ValidationResult {
                is_valid: false,
                reason: Some(format!("Price too stale: {} seconds old", age_seconds)),
                adjusted_confidence: 0.0,
            });
        }

        // Check confidence
        if price.confidence < self.validation_rules.min_confidence {
            return Ok(ValidationResult {
                is_valid: false,
                reason: Some(format!("Confidence too low: {}", price.confidence)),
                adjusted_confidence: 0.0,
            });
        }

        // Check for zero or negative prices
        if price.price <= Decimal::from(0) {
            return Ok(ValidationResult {
                is_valid: false,
                reason: Some("Price must be positive".to_string()),
                adjusted_confidence: 0.0,
            });
        }

        Ok(ValidationResult {
            is_valid: true,
            reason: None,
            adjusted_confidence: price.confidence,
        })
    }

    /// Validate advanced rules (market-specific checks)
    fn validate_advanced_rules(&self, price: &PriceData) -> Result<(), PriceError> {
        // Symbol-specific validation
        if let Some(pair) = TradingPair::from_symbol(&price.symbol) {
            match pair {
                TradingPair::EthUsd => {
                    if price.price < Decimal::from(100) || price.price > Decimal::from(10000) {
                        return Err(PriceError::ValidationFailed(
                            "ETH price outside reasonable range".to_string(),
                        ));
                    }
                }
                TradingPair::NearUsd => {
                    if price.price < Decimal::from_f64(0.1).unwrap()
                        || price.price > Decimal::from(100)
                    {
                        return Err(PriceError::ValidationFailed(
                            "NEAR price outside reasonable range".to_string(),
                        ));
                    }
                }
                TradingPair::BtcUsd => {
                    if price.price < Decimal::from(10000) || price.price > Decimal::from(100000) {
                        return Err(PriceError::ValidationFailed(
                            "BTC price outside reasonable range".to_string(),
                        ));
                    }
                }
                TradingPair::UsdtUsd | TradingPair::UsdcUsd => {
                    if price.price < Decimal::from_f64(0.95).unwrap()
                        || price.price > Decimal::from_f64(1.05).unwrap()
                    {
                        return Err(PriceError::ValidationFailed(
                            "Stablecoin price outside reasonable range".to_string(),
                        ));
                    }
                }
            }
        }

        // Check for extreme 24h changes
        if let Some(change_24h) = price.change_24h {
            if change_24h.abs() > self.validation_rules.max_change_percent {
                return Err(PriceError::ValidationFailed(format!(
                    "24h change too extreme: {}%",
                    change_24h
                )));
            }
        }

        // Source-specific validation
        match price.source.as_str() {
            "1inch" => {
                if price.confidence < 0.8 {
                    return Err(PriceError::ValidationFailed(
                        "1inch confidence too low".to_string(),
                    ));
                }
            }
            "coingecko" => {
                if price.confidence < 0.7 {
                    return Err(PriceError::ValidationFailed(
                        "CoinGecko confidence too low".to_string(),
                    ));
                }
            }
            "binance" => {
                if price.confidence < 0.8 {
                    return Err(PriceError::ValidationFailed(
                        "Binance confidence too low".to_string(),
                    ));
                }
            }
            _ => {
                return Err(PriceError::ValidationFailed(format!(
                    "Unknown price source: {}",
                    price.source
                )));
            }
        }

        Ok(())
    }

    /// Detect price anomalies using statistical analysis
    fn detect_anomaly(&self, price: &PriceData) -> Option<f64> {
        if let Some(historical) = self.historical_prices.get(&price.symbol) {
            if historical.len() < 5 {
                return None; // Need more data for anomaly detection
            }

            // Calculate rolling statistics
            let recent_prices: Vec<Decimal> = historical
                .iter()
                .rev()
                .take(10) // Last 10 prices
                .map(|p| p.price.clone())
                .collect();

            let mean =
                recent_prices.iter().sum::<Decimal>() / Decimal::from(recent_prices.len() as i32);
            let variance = recent_prices
                .iter()
                .map(|p| {
                    let diff = p - &mean;
                    let diff_f64 = diff.to_f64().unwrap_or(0.0);
                    diff_f64 * diff_f64
                })
                .sum::<f64>()
                / recent_prices.len() as f64;

            let std_dev = variance.sqrt();
            let current_diff = (&price.price - &mean).to_f64().unwrap_or(0.0);
            let z_score = if std_dev > 0.0 {
                current_diff / std_dev
            } else {
                0.0
            };

            Some(z_score.abs())
        } else {
            None
        }
    }

    /// Add price to historical data for anomaly detection
    pub fn add_to_history(&mut self, price: &PriceData) {
        let history = self
            .historical_prices
            .entry(price.symbol.clone())
            .or_insert_with(Vec::new);

        history.push(price.clone());

        // Keep only last 50 prices to prevent memory bloat
        if history.len() > 50 {
            history.remove(0);
        }

        // Clean up old prices (older than 24 hours)
        let cutoff_time = Utc::now() - Duration::hours(24);
        history.retain(|p| p.timestamp > cutoff_time);
    }

    /// Validate multiple prices
    pub fn validate_multiple_prices(
        &self,
        prices: &[PriceData],
    ) -> Vec<Result<PriceData, PriceError>> {
        prices
            .iter()
            .map(|price| self.validate_price(price))
            .collect()
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> ValidationStats {
        let mut stats = ValidationStats::default();

        for (symbol, history) in &self.historical_prices {
            let symbol_stats = SymbolValidationStats {
                symbol: symbol.clone(),
                total_prices: history.len(),
                valid_prices: history.iter().filter(|p| p.confidence > 0.5).count(),
                avg_confidence: history.iter().map(|p| p.confidence).sum::<f64>()
                    / history.len() as f64,
                latest_price: history.last().map(|p| p.price.clone()),
                price_range: if !history.is_empty() {
                    let min = history.iter().map(|p| p.price.clone()).min().unwrap();
                    let max = history.iter().map(|p| p.price.clone()).max().unwrap();
                    Some((min, max))
                } else {
                    None
                },
            };

            stats.symbol_stats.insert(symbol.clone(), symbol_stats);
        }

        stats
    }

    /// Clear historical data
    pub fn clear_history(&mut self) {
        self.historical_prices.clear();
    }

    /// Update validation rules
    pub fn update_rules(&mut self, rules: ValidationRules) {
        self.validation_rules = rules;
        info!("Updated price validation rules");
    }
}

/// Validation statistics
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub symbol_stats: HashMap<String, SymbolValidationStats>,
    pub total_validations: usize,
    pub successful_validations: usize,
    pub failed_validations: usize,
}

/// Symbol-specific validation statistics
#[derive(Debug, Clone)]
pub struct SymbolValidationStats {
    pub symbol: String,
    pub total_prices: usize,
    pub valid_prices: usize,
    pub avg_confidence: f64,
    pub latest_price: Option<Decimal>,
    pub price_range: Option<(Decimal, Decimal)>,
}

/// Circuit breaker for price validation
pub struct PriceValidationCircuitBreaker {
    failure_count: usize,
    failure_threshold: usize,
    recovery_timeout: Duration,
    last_failure_time: Option<DateTime<Utc>>,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing fast
    HalfOpen, // Testing recovery
}

impl PriceValidationCircuitBreaker {
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
            state: CircuitBreakerState::Closed,
        }
    }

    pub fn call<F, R>(&mut self, f: F) -> Result<R, PriceError>
    where
        F: FnOnce() -> Result<R, PriceError>,
    {
        match self.state {
            CircuitBreakerState::Closed => match f() {
                Ok(result) => {
                    self.failure_count = 0;
                    Ok(result)
                }
                Err(e) => {
                    self.failure_count += 1;
                    self.last_failure_time = Some(Utc::now());

                    if self.failure_count >= self.failure_threshold {
                        self.state = CircuitBreakerState::Open;
                    }

                    Err(e)
                }
            },
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if Utc::now().signed_duration_since(last_failure) > self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.call(f)
                    } else {
                        Err(PriceError::ValidationFailed(
                            "Circuit breaker is open".to_string(),
                        ))
                    }
                } else {
                    Err(PriceError::ValidationFailed(
                        "Circuit breaker is open".to_string(),
                    ))
                }
            }
            CircuitBreakerState::HalfOpen => match f() {
                Ok(result) => {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    Ok(result)
                }
                Err(e) => {
                    self.state = CircuitBreakerState::Open;
                    self.last_failure_time = Some(Utc::now());
                    Err(e)
                }
            },
        }
    }
}

impl Default for PriceValidationCircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::minutes(1))
    }
}
