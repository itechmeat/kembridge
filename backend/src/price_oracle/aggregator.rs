// src/price_oracle/aggregator.rs - Price aggregation logic
use bigdecimal::{BigDecimal as Decimal, FromPrimitive, ToPrimitive};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::price_oracle::types::{AggregatedPrice, AggregationMethod, PriceData};

/// Price aggregator that combines data from multiple sources
pub struct PriceAggregator {
    method: AggregationMethod,
    min_sources: usize,
    max_deviation: f64, // Maximum allowed deviation from median (percentage)
}

impl PriceAggregator {
    /// Create new price aggregator
    pub fn new() -> Self {
        Self {
            method: AggregationMethod::WeightedAverage,
            min_sources: 1,
            max_deviation: 10.0, // 10% maximum deviation
        }
    }

    /// Create aggregator with custom configuration
    pub fn with_config(method: AggregationMethod, min_sources: usize, max_deviation: f64) -> Self {
        Self {
            method,
            min_sources,
            max_deviation,
        }
    }

    /// Aggregate prices from multiple sources
    pub fn aggregate_prices(&self, symbol: &str, prices: &[PriceData]) -> anyhow::Result<AggregatedPrice> {
        if prices.is_empty() {
            panic!("Cannot aggregate empty price list");
        }

        if prices.len() < self.min_sources {
            warn!(
                "Insufficient price sources for {}: {} < {}",
                symbol,
                prices.len(),
                self.min_sources
            );
        }

        // Filter out potentially invalid prices
        let valid_prices = match self.filter_valid_prices(prices) {
            Ok(prices) if !prices.is_empty() => prices,
            _ => {
                warn!("No valid prices found for {}, using original data", symbol);
                return self.aggregate_with_method(symbol, prices);
            }
        };

        info!(
            "Aggregating {} valid prices for {} using method {:?}",
            valid_prices.len(),
            symbol,
            self.method
        );

        self.aggregate_with_method(symbol, &valid_prices)
    }

    /// Filter out invalid prices based on deviation from median
    fn filter_valid_prices(&self, prices: &[PriceData]) -> anyhow::Result<Vec<PriceData>> {
        if prices.len() < 3 {
            return Ok(prices.to_vec()); // Not enough data for statistical filtering
        }

        // Calculate median price
        let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price.clone()).collect();
        sorted_prices.sort();

        let median = if sorted_prices.len() % 2 == 0 {
            let mid = sorted_prices.len() / 2;
            (&sorted_prices[mid - 1] + &sorted_prices[mid]) / Decimal::from(2)
        } else {
            sorted_prices[sorted_prices.len() / 2].clone()
        };

        // Filter prices within acceptable deviation
        let max_deviation_decimal = Decimal::from_f64(self.max_deviation / 100.0)
            .ok_or_else(|| anyhow::anyhow!("Invalid max deviation value"))?;

        Ok(prices
            .iter()
            .filter(|price| {
                let deviation = (&price.price - &median).abs() / &median;
                deviation <= max_deviation_decimal
            })
            .cloned()
            .collect())
    }

    /// Aggregate prices using the specified method
    fn aggregate_with_method(&self, symbol: &str, prices: &[PriceData]) -> anyhow::Result<AggregatedPrice> {
        match self.method {
            AggregationMethod::WeightedAverage => self.weighted_average(symbol, prices),
            AggregationMethod::MedianPrice => self.median_price(symbol, prices),
            AggregationMethod::HighestConfidence => self.highest_confidence(symbol, prices),
            AggregationMethod::MostRecentPrice => self.most_recent_price(symbol, prices),
        }
    }

    /// Calculate weighted average based on confidence scores
    fn weighted_average(
        &self,
        symbol: &str,
        prices: &[PriceData],
    ) -> anyhow::Result<AggregatedPrice> {
        let mut weighted_sum = Decimal::from(0);
        let mut total_weight = 0.0;
        let mut sources = Vec::new();
        let mut volume_24h_sum = Decimal::from(0);
        let mut change_24h_sum = 0.0;
        let mut change_24h_count = 0;

        for price in prices {
            let weight = price.confidence;
            let weight_decimal = Decimal::from_f64(weight)
                .ok_or_else(|| anyhow::anyhow!("Invalid weight value: {}", weight))?;
            weighted_sum += &price.price * weight_decimal;
            total_weight += weight;
            sources.push(price.source.clone());

            if let Some(ref volume) = price.volume_24h {
                volume_24h_sum += volume;
            }

            if let Some(change) = price.change_24h {
                change_24h_sum += change;
                change_24h_count += 1;
            }
        }

        let final_price = if total_weight > 0.0 {
            weighted_sum
                / Decimal::from_f64(total_weight)
                    .ok_or_else(|| anyhow::anyhow!("Invalid total weight for price aggregation"))?
        } else {
            return Err(anyhow::anyhow!(
                "No valid prices to aggregate for symbol: {}",
                symbol
            ));
        };

        let average_confidence = total_weight / prices.len() as f64;
        let price_variance = self.calculate_price_variance(prices);

        Ok(AggregatedPrice {
            symbol: symbol.to_string(),
            price: final_price,
            sources,
            confidence: average_confidence,
            last_updated: Utc::now(),
            price_variance,
            volume_24h: if volume_24h_sum > Decimal::from(0) {
                Some(volume_24h_sum)
            } else {
                None
            },
            change_24h: if change_24h_count > 0 {
                Some(change_24h_sum / change_24h_count as f64)
            } else {
                None
            },
        })
    }

    /// Calculate median price
    fn median_price(&self, symbol: &str, prices: &[PriceData]) -> anyhow::Result<AggregatedPrice> {
        let mut sorted_prices: Vec<&PriceData> = prices.iter().collect();
        sorted_prices.sort_by(|a, b| a.price.cmp(&b.price));

        let median_price = if sorted_prices.len() % 2 == 0 {
            let mid = sorted_prices.len() / 2;
            (&sorted_prices[mid - 1].price + &sorted_prices[mid].price) / Decimal::from(2)
        } else {
            sorted_prices[sorted_prices.len() / 2].price.clone()
        };

        let sources: Vec<String> = prices.iter().map(|p| p.source.clone()).collect();
        let average_confidence =
            prices.iter().map(|p| p.confidence).sum::<f64>() / prices.len() as f64;
        let price_variance = self.calculate_price_variance(prices);

        Ok(AggregatedPrice {
            symbol: symbol.to_string(),
            price: median_price,
            sources,
            confidence: average_confidence,
            last_updated: Utc::now(),
            price_variance,
            volume_24h: None,
            change_24h: None,
        })
    }

    /// Select price with highest confidence
    fn highest_confidence(
        &self,
        symbol: &str,
        prices: &[PriceData],
    ) -> anyhow::Result<AggregatedPrice> {
        let best_price = prices
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .ok_or_else(|| anyhow::anyhow!("Cannot compare confidence values"))
                    .unwrap()
            })
            .ok_or_else(|| anyhow::anyhow!("No prices available for best price selection"))?;

        let sources: Vec<String> = prices.iter().map(|p| p.source.clone()).collect();
        let price_variance = self.calculate_price_variance(prices);

        Ok(AggregatedPrice {
            symbol: symbol.to_string(),
            price: best_price.price.clone(),
            sources,
            confidence: best_price.confidence,
            last_updated: Utc::now(),
            price_variance,
            volume_24h: best_price.volume_24h.clone(),
            change_24h: best_price.change_24h,
        })
    }

    /// Select most recent price
    fn most_recent_price(&self, symbol: &str, prices: &[PriceData]) -> anyhow::Result<AggregatedPrice> {
        let most_recent = prices
            .iter()
            .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
            .unwrap();

        let sources: Vec<String> = prices.iter().map(|p| p.source.clone()).collect();
        let average_confidence =
            prices.iter().map(|p| p.confidence).sum::<f64>() / prices.len() as f64;
        let price_variance = self.calculate_price_variance(prices);

        Ok(AggregatedPrice {
            symbol: symbol.to_string(),
            price: most_recent.price.clone(),
            sources,
            confidence: average_confidence,
            last_updated: Utc::now(),
            price_variance,
            volume_24h: most_recent.volume_24h.clone(),
            change_24h: most_recent.change_24h,
        })
    }

    /// Calculate price variance as standard deviation
    fn calculate_price_variance(&self, prices: &[PriceData]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let mean = prices.iter().map(|p| p.price.clone()).sum::<Decimal>()
            / Decimal::from(prices.len() as i32);
        let variance = prices
            .iter()
            .map(|p| {
                let diff = &p.price - &mean;
                let diff_f64 = diff.to_f64().unwrap_or(0.0);
                diff_f64 * diff_f64
            })
            .sum::<f64>()
            / prices.len() as f64;

        variance.sqrt()
    }

    /// Get aggregation statistics
    pub fn get_statistics(&self, prices: &[PriceData]) -> PriceStatistics {
        if prices.is_empty() {
            return PriceStatistics::default();
        }

        let min_price = prices.iter().map(|p| p.price.clone()).min().unwrap();
        let max_price = prices.iter().map(|p| p.price.clone()).max().unwrap();
        let avg_price = prices.iter().map(|p| p.price.clone()).sum::<Decimal>()
            / Decimal::from(prices.len() as i32);
        let variance = self.calculate_price_variance(prices);

        PriceStatistics {
            count: prices.len(),
            min_price,
            max_price,
            avg_price,
            variance,
            sources: prices.iter().map(|p| p.source.clone()).collect(),
        }
    }
}

impl Default for PriceAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Price aggregation statistics
#[derive(Debug, Clone)]
pub struct PriceStatistics {
    pub count: usize,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub avg_price: Decimal,
    pub variance: f64,
    pub sources: Vec<String>,
}

impl Default for PriceStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            min_price: Decimal::from(0),
            max_price: Decimal::from(0),
            avg_price: Decimal::from(0),
            variance: 0.0,
            sources: Vec::new(),
        }
    }
}

/// Price aggregation configuration
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    pub method: AggregationMethod,
    pub min_sources: usize,
    pub max_deviation: f64,
    pub confidence_weights: HashMap<String, f64>, // Provider-specific weights
}

impl Default for AggregationConfig {
    fn default() -> Self {
        let mut confidence_weights = HashMap::new();
        confidence_weights.insert("1inch".to_string(), 1.0);
        confidence_weights.insert("coingecko".to_string(), 0.9);
        confidence_weights.insert("binance".to_string(), 0.95);

        Self {
            method: AggregationMethod::WeightedAverage,
            min_sources: 1,
            max_deviation: 10.0,
            confidence_weights,
        }
    }
}

/// Advanced price aggregator with configurable weights
pub struct AdvancedPriceAggregator {
    config: AggregationConfig,
}

impl AdvancedPriceAggregator {
    /// Create new advanced aggregator
    pub fn new(config: AggregationConfig) -> Self {
        Self { config }
    }

    /// Aggregate prices with provider-specific weights
    pub fn aggregate_with_weights(&self, symbol: &str, prices: &[PriceData]) -> anyhow::Result<AggregatedPrice> {
        let weighted_prices: Vec<PriceData> = prices
            .iter()
            .map(|price| {
                let weight = self
                    .config
                    .confidence_weights
                    .get(&price.source)
                    .unwrap_or(&1.0);

                let mut weighted_price = price.clone();
                weighted_price.confidence *= weight;
                weighted_price
            })
            .collect();

        let aggregator = PriceAggregator::with_config(
            self.config.method,
            self.config.min_sources,
            self.config.max_deviation,
        );

        aggregator.aggregate_prices(symbol, &weighted_prices)
    }
}
