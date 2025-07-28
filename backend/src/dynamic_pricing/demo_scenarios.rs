// src/dynamic_pricing/demo_scenarios.rs - Demo scenarios for dynamic pricing showcase

use std::sync::Arc;
use bigdecimal::BigDecimal;
use anyhow::Result;
use tracing::{debug, info};
use chrono::{Timelike, Datelike};

use crate::constants::*;
use super::{DynamicPricingService, BridgeQuoteRequest, BridgeQuote, DynamicPricingError};

/// Market scenario types for demonstration purposes
#[derive(Debug, Clone)]
pub enum MarketScenario {
    Stable,      // Normal market conditions with low volatility
    Volatile,    // High volatility market conditions
    Extreme,     // Extreme market stress conditions
}

/// Demo scenario manager for pricing demonstrations
pub struct DemoPricingScenarios {
    pricing_service: Arc<DynamicPricingService>,
}

impl DemoPricingScenarios {
    /// Create new demo scenarios manager
    pub fn new(pricing_service: Arc<DynamicPricingService>) -> Self {
        Self {
            pricing_service,
        }
    }

    /// Generate demo quote with specific market scenario
    pub async fn generate_demo_quote(
        &self,
        request: &BridgeQuoteRequest,
        scenario: MarketScenario,
    ) -> Result<DemoQuoteComparison, DynamicPricingError> {
        info!("Generating demo quote for scenario: {:?}", scenario);

        // Get base quote
        let base_quote = self.pricing_service.get_bridge_quote(request).await?;
        
        // Apply scenario-specific adjustments
        let scenario_quote = self.apply_scenario_adjustments(&base_quote, &scenario).await?;
        
        // Generate comparison data
        let comparison = DemoQuoteComparison {
            scenario: scenario.clone(),
            base_quote,
            scenario_quote,
            scenario_impact: self.calculate_scenario_impact(&scenario).await?,
            demo_insights: self.generate_demo_insights(&scenario, request).await?,
        };

        Ok(comparison)
    }

    /// Generate all scenarios for comparison
    pub async fn generate_all_scenarios(
        &self,
        request: &BridgeQuoteRequest,
    ) -> Result<Vec<DemoQuoteComparison>, DynamicPricingError> {
        info!("Generating all pricing scenarios for comparison");

        let scenarios = vec![
            MarketScenario::Stable,
            MarketScenario::Volatile,
            MarketScenario::Extreme,
        ];

        let mut comparisons = Vec::new();
        for scenario in scenarios {
            let comparison = self.generate_demo_quote(request, scenario).await?;
            comparisons.push(comparison);
        }

        Ok(comparisons)
    }

    /// Generate time-based pricing demonstration
    pub async fn generate_time_based_demo(
        &self,
        request: &BridgeQuoteRequest,
    ) -> Result<TimeBasedDemoResult, DynamicPricingError> {
        info!("Generating time-based pricing demonstration");

        let current_hour = chrono::Utc::now().hour();
        
        // Generate quotes for different times of day
        let peak_hours_quote = self.simulate_time_quote(request, 16).await?; // 4 PM UTC
        let off_peak_quote = self.simulate_time_quote(request, 3).await?;    // 3 AM UTC
        let business_hours_quote = self.simulate_time_quote(request, 10).await?; // 10 AM UTC
        
        let time_demo = TimeBasedDemoResult {
            current_hour,
            peak_hours_quote: peak_hours_quote.clone(),
            off_peak_quote: off_peak_quote.clone(),
            business_hours_quote,
            time_savings_analysis: self.calculate_time_savings_analysis(
                &peak_hours_quote, &off_peak_quote
            ).await?,
        };

        Ok(time_demo)
    }

    /// Generate size-based pricing demonstration
    pub async fn generate_size_based_demo(
        &self,
        base_request: &BridgeQuoteRequest,
    ) -> Result<SizeBasedDemoResult, DynamicPricingError> {
        info!("Generating size-based pricing demonstration");

        // Create requests for different trade sizes
        let small_request = self.create_sized_request(base_request, "100").await?;    // $250 (0.1 ETH)
        let medium_request = self.create_sized_request(base_request, "4").await?;     // $10k (4 ETH)
        let large_request = self.create_sized_request(base_request, "40").await?;     // $100k (40 ETH)

        let small_quote = self.pricing_service.get_bridge_quote(&small_request).await?;
        let medium_quote = self.pricing_service.get_bridge_quote(&medium_request).await?;
        let large_quote = self.pricing_service.get_bridge_quote(&large_request).await?;

        let size_demo = SizeBasedDemoResult {
            small_trade: TradeSize {
                amount: small_request.from_amount.clone(),
                quote: small_quote,
                usd_value: 250.0,
                fee_percentage: 0.0, // Will be calculated
            },
            medium_trade: TradeSize {
                amount: medium_request.from_amount.clone(),
                quote: medium_quote,
                usd_value: 10000.0,
                fee_percentage: 0.0,
            },
            large_trade: TradeSize {
                amount: large_request.from_amount.clone(),
                quote: large_quote,
                usd_value: 100000.0,
                fee_percentage: 0.0,
            },
            volume_discount_analysis: self.calculate_volume_discount_analysis().await?,
        };

        Ok(size_demo)
    }

    /// Apply scenario-specific adjustments to quote
    async fn apply_scenario_adjustments(
        &self,
        base_quote: &BridgeQuote,
        scenario: &MarketScenario,
    ) -> Result<BridgeQuote, DynamicPricingError> {
        let multiplier = match scenario {
            MarketScenario::Stable => BigDecimal::try_from(MARKET_SCENARIO_STABLE_MULTIPLIER).unwrap(),
            MarketScenario::Volatile => BigDecimal::try_from(MARKET_SCENARIO_VOLATILE_MULTIPLIER).unwrap(),
            MarketScenario::Extreme => BigDecimal::try_from(MARKET_SCENARIO_EXTREME_MULTIPLIER).unwrap(),
        };

        let mut scenario_quote = base_quote.clone();
        
        // Adjust fees based on scenario
        scenario_quote.fee_breakdown.total_fee_amount = &base_quote.fee_breakdown.total_fee_amount * &multiplier;
        scenario_quote.fee_breakdown.slippage_protection_fee = &base_quote.fee_breakdown.slippage_protection_fee * &multiplier;
        
        // Adjust final amount
        scenario_quote.to_amount = &base_quote.from_amount * &base_quote.exchange_rate.rate - &scenario_quote.fee_breakdown.total_fee_amount;
        
        // Update fee percentage
        if base_quote.from_amount > BigDecimal::from(0) {
            let new_fee_percentage = (&scenario_quote.fee_breakdown.total_fee_amount / &base_quote.from_amount) * BigDecimal::from(100);
            scenario_quote.fee_breakdown.fee_percentage = new_fee_percentage.to_string().parse::<f64>().unwrap_or(0.0);
        }

        debug!("Applied {:?} scenario: multiplier={:.2}, original_fee={}, new_fee={}", 
            scenario, multiplier, base_quote.fee_breakdown.total_fee_amount, scenario_quote.fee_breakdown.total_fee_amount);

        Ok(scenario_quote)
    }

    /// Calculate scenario impact metrics
    async fn calculate_scenario_impact(
        &self,
        scenario: &MarketScenario,
    ) -> Result<ScenarioImpact, DynamicPricingError> {
        let impact = match scenario {
            MarketScenario::Stable => ScenarioImpact {
                volatility_level: "Low".to_string(),
                risk_level: "Minimal".to_string(),
                recommended_action: "Standard trading recommended".to_string(),
                price_impact_factor: 1.0,
                execution_confidence: 95.0,
            },
            MarketScenario::Volatile => ScenarioImpact {
                volatility_level: "High".to_string(),
                risk_level: "Elevated".to_string(),
                recommended_action: "Consider smaller trades or wait for stability".to_string(),
                price_impact_factor: 1.3,
                execution_confidence: 75.0,
            },
            MarketScenario::Extreme => ScenarioImpact {
                volatility_level: "Extreme".to_string(),
                risk_level: "High".to_string(),
                recommended_action: "Avoid large trades, consider delaying non-urgent transactions".to_string(),
                price_impact_factor: 1.8,
                execution_confidence: 60.0,
            },
        };

        Ok(impact)
    }

    /// Generate demo insights for scenario
    async fn generate_demo_insights(
        &self,
        scenario: &MarketScenario,
        request: &BridgeQuoteRequest,
    ) -> Result<Vec<String>, DynamicPricingError> {
        let mut insights = Vec::new();

        match scenario {
            MarketScenario::Stable => {
                insights.push("✅ Optimal trading conditions with minimal slippage".to_string());
                insights.push("📈 Exchange rates are stable and predictable".to_string());
                insights.push("⚡ Fast execution with high confidence".to_string());
                insights.push("💰 Standard fees apply - no volatility premium".to_string());
            },
            MarketScenario::Volatile => {
                insights.push("⚠️ Increased market volatility detected".to_string());
                insights.push("📊 Higher slippage protection fees applied".to_string());
                insights.push("🎯 Consider splitting large trades for better execution".to_string());
                insights.push("⏱️ Price valid for shorter duration due to volatility".to_string());
            },
            MarketScenario::Extreme => {
                insights.push("🚨 Extreme market conditions - high risk environment".to_string());
                insights.push("💎 Maximum slippage protection automatically enabled".to_string());
                insights.push("🛡️ Additional security measures and confirmations required".to_string());
                insights.push("⏰ Consider delaying non-urgent transactions".to_string());
            },
        }

        // Add trade-size specific insights
        let amount_usd = self.estimate_usd_value(&request.from_amount, &request.from_token).await?;
        if amount_usd >= DEMO_LARGE_TRADE_THRESHOLD {
            insights.push("🏦 Large trade detected - VIP routing and dedicated support".to_string());
        } else if amount_usd >= DEMO_MEDIUM_TRADE_THRESHOLD {
            insights.push("📋 Medium trade - enhanced monitoring and priority processing".to_string());
        }

        Ok(insights)
    }

    /// Simulate time-based quote
    async fn simulate_time_quote(
        &self,
        request: &BridgeQuoteRequest,
        hour: u32,
    ) -> Result<BridgeQuote, DynamicPricingError> {
        // This is a simplified simulation - in real implementation,
        // we would temporarily modify the algorithm's time calculations
        let base_quote = self.pricing_service.get_bridge_quote(request).await?;
        
        // Apply time-based adjustments
        let time_multiplier = if (DEMO_PEAK_HOURS_START..=DEMO_PEAK_HOURS_END).contains(&hour) {
            BigDecimal::try_from(DEMO_PEAK_PREMIUM).unwrap()
        } else if (0..=6).contains(&hour) {
            BigDecimal::try_from(DEMO_OFF_PEAK_DISCOUNT).unwrap()
        } else {
            BigDecimal::from(1)
        };

        let mut time_quote = base_quote.clone();
        time_quote.fee_breakdown.total_fee_amount = &base_quote.fee_breakdown.total_fee_amount * &time_multiplier;
        time_quote.to_amount = &base_quote.from_amount * &base_quote.exchange_rate.rate - &time_quote.fee_breakdown.total_fee_amount;

        Ok(time_quote)
    }

    /// Create sized request for demonstration
    async fn create_sized_request(
        &self,
        base_request: &BridgeQuoteRequest,
        amount: &str,
    ) -> Result<BridgeQuoteRequest, DynamicPricingError> {
        let mut sized_request = base_request.clone();
        sized_request.from_amount = BigDecimal::try_from(amount.parse::<f64>().unwrap_or(1.0)).unwrap();
        Ok(sized_request)
    }

    /// Calculate time savings analysis
    async fn calculate_time_savings_analysis(
        &self,
        peak_quote: &BridgeQuote,
        off_peak_quote: &BridgeQuote,
    ) -> Result<TimeSavingsAnalysis, DynamicPricingError> {
        let peak_fee = &peak_quote.fee_breakdown.total_fee_amount;
        let off_peak_fee = &off_peak_quote.fee_breakdown.total_fee_amount;
        
        let savings_amount = peak_fee - off_peak_fee;
        let savings_percentage = if peak_fee > &BigDecimal::from(0) {
            (&savings_amount / peak_fee) * BigDecimal::from(100)
        } else {
            BigDecimal::from(0)
        };

        Ok(TimeSavingsAnalysis {
            peak_hour_fee: peak_fee.clone(),
            off_peak_fee: off_peak_fee.clone(),
            savings_amount,
            savings_percentage: savings_percentage.to_string().parse::<f64>().unwrap_or(0.0),
            optimal_trading_hours: "3 AM - 6 AM UTC (lowest fees)".to_string(),
            peak_trading_hours: "2 PM - 6 PM UTC (highest fees)".to_string(),
        })
    }

    /// Calculate volume discount analysis
    async fn calculate_volume_discount_analysis(&self) -> Result<VolumeDiscountAnalysis, DynamicPricingError> {
        Ok(VolumeDiscountAnalysis {
            small_trade_fee: 0.25,  // 0.25% for trades under $1k
            medium_trade_fee: 0.20, // 0.20% for trades $1k-$10k
            large_trade_fee: 0.15,  // 0.15% for trades $10k-$100k
            vip_trade_fee: 0.10,    // 0.10% for trades over $100k
            volume_tiers: vec![
                "Micro: < $1,000 - Standard rates".to_string(),
                "Small: $1,000 - $10,000 - 5% discount".to_string(),
                "Medium: $10,000 - $100,000 - 15% discount".to_string(),
                "Large: $100,000+ - 50% discount + VIP support".to_string(),
            ],
        })
    }

    /// Estimate USD value using centralized PriceService - NO FAKE PRICES!
    async fn estimate_usd_value(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<f64, DynamicPricingError> {
        // Use the centralized PriceService which has Oracle + 1inch integration
        // This approach follows DRY principles and uses existing real price infrastructure
        
        // Create a temporary PriceService with same oracle as pricing service
        let price_service = super::price_service::PriceService::new_oracle_only(
            self.pricing_service.price_oracle.clone()
        );
        
        // Use the centralized USD value estimation
        price_service.get_usd_value(amount, token).await
    }
}

/// Demo quote comparison result
#[derive(Debug, Clone)]
pub struct DemoQuoteComparison {
    pub scenario: MarketScenario,
    pub base_quote: BridgeQuote,
    pub scenario_quote: BridgeQuote,
    pub scenario_impact: ScenarioImpact,
    pub demo_insights: Vec<String>,
}

/// Scenario impact metrics
#[derive(Debug, Clone)]
pub struct ScenarioImpact {
    pub volatility_level: String,
    pub risk_level: String,
    pub recommended_action: String,
    pub price_impact_factor: f64,
    pub execution_confidence: f64,
}

/// Time-based demo result
#[derive(Debug, Clone)]
pub struct TimeBasedDemoResult {
    pub current_hour: u32,
    pub peak_hours_quote: BridgeQuote,
    pub off_peak_quote: BridgeQuote,
    pub business_hours_quote: BridgeQuote,
    pub time_savings_analysis: TimeSavingsAnalysis,
}

/// Time savings analysis
#[derive(Debug, Clone)]
pub struct TimeSavingsAnalysis {
    pub peak_hour_fee: BigDecimal,
    pub off_peak_fee: BigDecimal,
    pub savings_amount: BigDecimal,
    pub savings_percentage: f64,
    pub optimal_trading_hours: String,
    pub peak_trading_hours: String,
}

/// Size-based demo result
#[derive(Debug, Clone)]
pub struct SizeBasedDemoResult {
    pub small_trade: TradeSize,
    pub medium_trade: TradeSize,
    pub large_trade: TradeSize,
    pub volume_discount_analysis: VolumeDiscountAnalysis,
}

/// Trade size information
#[derive(Debug, Clone)]
pub struct TradeSize {
    pub amount: BigDecimal,
    pub quote: BridgeQuote,
    pub usd_value: f64,
    pub fee_percentage: f64,
}

/// Volume discount analysis
#[derive(Debug, Clone)]
pub struct VolumeDiscountAnalysis {
    pub small_trade_fee: f64,
    pub medium_trade_fee: f64,
    pub large_trade_fee: f64,
    pub vip_trade_fee: f64,
    pub volume_tiers: Vec<String>,
}