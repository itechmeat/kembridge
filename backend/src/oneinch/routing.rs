// src/oneinch/routing.rs - Intelligent routing for 1inch Fusion+ integration

use super::{client::FusionClient, types::*};
use crate::constants::*;
use bigdecimal::BigDecimal;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Intelligent routing engine for optimal swap paths
pub struct RoutingEngine {
    client: Arc<FusionClient>,
}

impl RoutingEngine {
    /// Create new RoutingEngine instance
    pub fn new(client: Arc<FusionClient>) -> Self {
        Self { client }
    }

    /// Find optimal routing strategy for swap
    pub async fn find_optimal_route(&self, params: &RouteSearchParams) -> Result<OptimalRoute, OneinchError> {
        info!("Finding optimal route: {} -> {} ({})", params.from_token, params.to_token, params.amount);

        // Get multiple quote variants
        let route_options = self.generate_route_options(params).await?;
        
        if route_options.is_empty() {
            return Err(OneinchError::InsufficientLiquidity);
        }

        // Analyze and score each route
        let scored_routes = self.score_routes(&route_options, &params.optimization_criteria).await?;

        // Select best route based on criteria
        let best_route = self.select_best_route(scored_routes, &params.optimization_criteria)?;

        info!("Selected optimal route with score: {:.2}", best_route.total_score);
        Ok(best_route)
    }

    /// Generate multiple route options for comparison
    async fn generate_route_options(&self, params: &RouteSearchParams) -> Result<Vec<RouteOption>, OneinchError> {
        let mut routes = Vec::new();

        // Generate routes with different slippage tolerances
        let slippages = self.get_adaptive_slippages(&params.risk_tolerance);
        
        for slippage in slippages {
            match self.get_route_for_slippage(params, slippage).await {
                Ok(route) => routes.push(route),
                Err(e) => {
                    warn!("Failed to get route for slippage {}: {:?}", slippage, e);
                }
            }
        }

        // Generate routes with different gas settings
        if params.include_gas_optimization {
            let gas_routes = self.generate_gas_optimized_routes(params).await?;
            routes.extend(gas_routes);
        }

        // Generate routes for different execution speeds
        let speed_routes = self.generate_speed_optimized_routes(params).await?;
        routes.extend(speed_routes);

        debug!("Generated {} route options", routes.len());
        Ok(routes)
    }

    /// Get route for specific slippage tolerance
    async fn get_route_for_slippage(&self, params: &RouteSearchParams, slippage: f64) -> Result<RouteOption, OneinchError> {
        let quote_params = QuoteParams {
            from_token: params.from_token.clone(),
            to_token: params.to_token.clone(),
            amount: params.amount.clone(),
            from_address: params.from_address.clone(),
            slippage: Some(slippage),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some(ONEINCH_ROUTING_SOURCE.to_string()),
        };

        let quote = self.client.get_quote(&quote_params).await?;
        let completion_time = self.estimate_completion_time(&quote).await;
        
        Ok(RouteOption {
            quote,
            slippage_tolerance: slippage,
            route_type: RouteType::Standard,
            gas_optimization: GasOptimization::Standard,
            execution_speed: ExecutionSpeed::Standard,
            estimated_completion_time: completion_time,
        })
    }

    /// Generate gas-optimized routes
    async fn generate_gas_optimized_routes(&self, params: &RouteSearchParams) -> Result<Vec<RouteOption>, OneinchError> {
        let mut routes = Vec::new();

        // Fast execution (higher gas)
        if let Ok(route) = self.get_gas_optimized_route(params, GasOptimization::Fast).await {
            routes.push(route);
        }

        // Economy execution (lower gas)
        if let Ok(route) = self.get_gas_optimized_route(params, GasOptimization::Economy).await {
            routes.push(route);
        }

        Ok(routes)
    }

    /// Get gas-optimized route
    async fn get_gas_optimized_route(&self, params: &RouteSearchParams, gas_opt: GasOptimization) -> Result<RouteOption, OneinchError> {
        let slippage = match gas_opt {
            GasOptimization::Fast => ONEINCH_DEFAULT_SLIPPAGE + 0.2, // Higher slippage for faster execution
            GasOptimization::Economy => ONEINCH_DEFAULT_SLIPPAGE - 0.1, // Lower slippage for better price
            GasOptimization::Standard => ONEINCH_DEFAULT_SLIPPAGE,
        };

        let mut route = self.get_route_for_slippage(params, slippage).await?;
        route.gas_optimization = gas_opt;
        route.route_type = RouteType::GasOptimized;

        Ok(route)
    }

    /// Generate speed-optimized routes
    async fn generate_speed_optimized_routes(&self, params: &RouteSearchParams) -> Result<Vec<RouteOption>, OneinchError> {
        let mut routes = Vec::new();

        // Instant execution (simple routing)
        if let Ok(route) = self.get_speed_optimized_route(params, ExecutionSpeed::Instant).await {
            routes.push(route);
        }

        // Fast execution (moderate complexity)
        if let Ok(route) = self.get_speed_optimized_route(params, ExecutionSpeed::Fast).await {
            routes.push(route);
        }

        Ok(routes)
    }

    /// Get speed-optimized route
    async fn get_speed_optimized_route(&self, params: &RouteSearchParams, speed: ExecutionSpeed) -> Result<RouteOption, OneinchError> {
        let slippage = match speed {
            ExecutionSpeed::Instant => ONEINCH_DEFAULT_SLIPPAGE + 0.5, // Higher slippage for instant execution
            ExecutionSpeed::Fast => ONEINCH_DEFAULT_SLIPPAGE + 0.2,
            ExecutionSpeed::Standard => ONEINCH_DEFAULT_SLIPPAGE,
            ExecutionSpeed::Economy => ONEINCH_DEFAULT_SLIPPAGE - 0.1,
        };

        let mut route = self.get_route_for_slippage(params, slippage).await?;
        route.execution_speed = speed;
        route.route_type = RouteType::SpeedOptimized;

        Ok(route)
    }

    /// Score all routes based on optimization criteria
    async fn score_routes(&self, routes: &[RouteOption], criteria: &OptimizationCriteria) -> Result<Vec<ScoredRoute>, OneinchError> {
        let mut scored_routes = Vec::new();

        for route in routes {
            let score = self.calculate_route_score(route, criteria).await?;
            scored_routes.push(ScoredRoute {
                route: route.clone(),
                total_score: score.total,
                output_score: score.output,
                gas_score: score.gas,
                speed_score: score.speed,
                risk_score: score.risk,
            });
        }

        // Sort by total score (descending)
        scored_routes.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_routes)
    }

    /// Calculate comprehensive score for a route
    async fn calculate_route_score(&self, route: &RouteOption, criteria: &OptimizationCriteria) -> Result<RouteScore, OneinchError> {
        // Output amount score (0-100)
        let output_score = self.calculate_output_score(&route.quote);

        // Gas efficiency score (0-100)
        let gas_score = self.calculate_gas_score(&route.quote, &route.gas_optimization);

        // Speed score (0-100)
        let speed_score = self.calculate_speed_score(route);

        // Risk score (0-100, higher is safer)
        let risk_score = self.calculate_risk_score(route);

        // Weighted total score
        let total = (output_score * criteria.output_weight) +
                   (gas_score * criteria.gas_weight) +
                   (speed_score * criteria.speed_weight) +
                   (risk_score * criteria.risk_weight);

        Ok(RouteScore {
            total,
            output: output_score,
            gas: gas_score,
            speed: speed_score,
            risk: risk_score,
        })
    }

    /// Calculate output amount score
    fn calculate_output_score(&self, quote: &FusionQuote) -> f64 {
        // Normalize output amount to 0-100 scale
        // This would ideally compare against market rates
        let efficiency = &quote.to_amount / &quote.from_amount;
        let efficiency_float = efficiency.to_string().parse::<f64>().unwrap_or(0.0);
        
        // Simple scoring - in practice would compare against benchmarks
        (efficiency_float * 50.0).min(100.0).max(0.0)
    }

    /// Calculate gas efficiency score
    fn calculate_gas_score(&self, quote: &FusionQuote, gas_opt: &GasOptimization) -> f64 {
        let gas_float = quote.estimated_gas.to_string().parse::<f64>().unwrap_or(0.0);
        
        // Base score inversely proportional to gas usage
        let base_score = 100.0 - (gas_float / 1000000.0).min(100.0);

        // Adjust based on gas optimization strategy
        match gas_opt {
            GasOptimization::Economy => base_score * 1.2, // Bonus for economy
            GasOptimization::Fast => base_score * 0.9,    // Penalty for fast
            GasOptimization::Standard => base_score,
        }
    }

    /// Calculate execution speed score
    fn calculate_speed_score(&self, route: &RouteOption) -> f64 {
        let complexity_penalty = route.quote.protocols.len() as f64 * 5.0;
        
        let base_score = match route.execution_speed {
            ExecutionSpeed::Instant => 100.0,
            ExecutionSpeed::Fast => 80.0,
            ExecutionSpeed::Standard => 60.0,
            ExecutionSpeed::Economy => 40.0,
        };

        (base_score - complexity_penalty).max(0.0).min(100.0)
    }

    /// Calculate risk score
    fn calculate_risk_score(&self, route: &RouteOption) -> f64 {
        let mut risk_score = 100.0; // Start with maximum safety

        // Penalize high slippage
        if route.slippage_tolerance > 2.0 {
            risk_score -= (route.slippage_tolerance - 2.0) * 10.0;
        }

        // Penalize complex routing
        if route.quote.protocols.len() > 3 {
            risk_score -= (route.quote.protocols.len() - 3) as f64 * 5.0;
        }

        // Bonus for standard optimization
        match route.gas_optimization {
            GasOptimization::Standard => risk_score += 5.0,
            _ => {}
        }

        risk_score.max(0.0).min(100.0)
    }

    /// Select best route from scored options
    fn select_best_route(&self, mut scored_routes: Vec<ScoredRoute>, criteria: &OptimizationCriteria) -> Result<OptimalRoute, OneinchError> {
        if scored_routes.is_empty() {
            return Err(OneinchError::InsufficientLiquidity);
        }

        // Sort by total score if not already sorted
        scored_routes.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap_or(std::cmp::Ordering::Equal));

        let best = scored_routes.into_iter().next().unwrap();
        
        // Generate reasoning before moving fields
        let reasoning = self.generate_selection_reasoning(&best, criteria);
        let confidence = self.calculate_confidence(best.total_score);
        
        Ok(OptimalRoute {
            selected_route: best.route,
            total_score: best.total_score,
            reasoning,
            alternative_routes: Vec::new(), // Could include top alternatives
            confidence,
        })
    }

    /// Generate reasoning for route selection
    fn generate_selection_reasoning(&self, route: &ScoredRoute, criteria: &OptimizationCriteria) -> String {
        let mut reasons = Vec::new();

        if route.output_score > 80.0 && criteria.output_weight > 0.3 {
            reasons.push("Excellent output amount".to_string());
        }

        if route.gas_score > 80.0 && criteria.gas_weight > 0.3 {
            reasons.push("Gas efficient".to_string());
        }

        if route.speed_score > 80.0 && criteria.speed_weight > 0.3 {
            reasons.push("Fast execution".to_string());
        }

        if route.risk_score > 80.0 && criteria.risk_weight > 0.3 {
            reasons.push("Low risk profile".to_string());
        }

        match route.route.route_type {
            RouteType::GasOptimized => reasons.push("Gas optimized routing".to_string()),
            RouteType::SpeedOptimized => reasons.push("Speed optimized routing".to_string()),
            _ => {}
        }

        if reasons.is_empty() {
            "Balanced approach across all criteria".to_string()
        } else {
            reasons.join(", ")
        }
    }

    /// Calculate confidence in route selection
    fn calculate_confidence(&self, score: f64) -> f64 {
        // Confidence based on how good the score is
        if score > 90.0 {
            0.95
        } else if score > 80.0 {
            0.85
        } else if score > 70.0 {
            0.75
        } else if score > 60.0 {
            0.65
        } else {
            0.50
        }
    }

    /// Get adaptive slippages based on risk tolerance
    fn get_adaptive_slippages(&self, risk_tolerance: &RiskTolerance) -> Vec<f64> {
        match risk_tolerance {
            RiskTolerance::Conservative => vec![
                ONEINCH_MIN_SLIPPAGE,
                ONEINCH_DEFAULT_SLIPPAGE - 0.1,
                ONEINCH_DEFAULT_SLIPPAGE,
            ],
            RiskTolerance::Moderate => vec![
                ONEINCH_DEFAULT_SLIPPAGE - 0.2,
                ONEINCH_DEFAULT_SLIPPAGE,
                ONEINCH_DEFAULT_SLIPPAGE + 0.3,
                ONEINCH_DEFAULT_SLIPPAGE + 0.5,
            ],
            RiskTolerance::Aggressive => vec![
                ONEINCH_DEFAULT_SLIPPAGE,
                ONEINCH_DEFAULT_SLIPPAGE + 0.5,
                ONEINCH_DEFAULT_SLIPPAGE + 1.0,
                ONEINCH_DEFAULT_SLIPPAGE + 1.5,
            ],
        }
    }

    /// Estimate completion time for route
    async fn estimate_completion_time(&self, quote: &FusionQuote) -> u64 {
        // Base time estimation in seconds
        let base_time = 30u64; // 30 seconds base
        
        // Add time based on routing complexity
        let complexity_time = quote.protocols.len() as u64 * 10;
        
        // Add time based on network conditions (simplified)
        let network_time = 20; // Would be based on actual network conditions
        
        base_time + complexity_time + network_time
    }
}

/// Parameters for route search
#[derive(Debug, Clone)]
pub struct RouteSearchParams {
    pub from_token: String,
    pub to_token: String,
    pub amount: BigDecimal,
    pub from_address: String,
    pub optimization_criteria: OptimizationCriteria,
    pub risk_tolerance: RiskTolerance,
    pub include_gas_optimization: bool,
    pub max_slippage: Option<f64>,
}

/// Optimization criteria for route selection
#[derive(Debug, Clone)]
pub struct OptimizationCriteria {
    pub output_weight: f64,   // 0.0 - 1.0
    pub gas_weight: f64,      // 0.0 - 1.0
    pub speed_weight: f64,    // 0.0 - 1.0
    pub risk_weight: f64,     // 0.0 - 1.0
}

impl Default for OptimizationCriteria {
    fn default() -> Self {
        Self {
            output_weight: 0.4,  // Prioritize output amount
            gas_weight: 0.25,    // Moderate gas consideration
            speed_weight: 0.25,  // Moderate speed consideration
            risk_weight: 0.1,    // Light risk consideration
        }
    }
}

/// Risk tolerance levels
#[derive(Debug, Clone)]
pub enum RiskTolerance {
    Conservative,
    Moderate,
    Aggressive,
}

/// Route option with specific parameters
#[derive(Debug, Clone)]
pub struct RouteOption {
    pub quote: FusionQuote,
    pub slippage_tolerance: f64,
    pub route_type: RouteType,
    pub gas_optimization: GasOptimization,
    pub execution_speed: ExecutionSpeed,
    pub estimated_completion_time: u64,
}

/// Types of routes
#[derive(Debug, Clone)]
pub enum RouteType {
    Standard,
    GasOptimized,
    SpeedOptimized,
    RiskMinimized,
}

/// Gas optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum GasOptimization {
    Economy,   // Minimize gas cost
    Standard,  // Balanced approach
    Fast,      // Prioritize speed over cost
}

/// Execution speed preferences
#[derive(Debug, Clone)]
pub enum ExecutionSpeed {
    Economy,   // Slowest, cheapest
    Standard,  // Balanced
    Fast,      // Faster execution
    Instant,   // Fastest possible
}

/// Scored route with detailed metrics
#[derive(Debug, Clone)]
pub struct ScoredRoute {
    pub route: RouteOption,
    pub total_score: f64,
    pub output_score: f64,
    pub gas_score: f64,
    pub speed_score: f64,
    pub risk_score: f64,
}

/// Route scoring breakdown
#[derive(Debug, Clone)]
pub struct RouteScore {
    pub total: f64,
    pub output: f64,
    pub gas: f64,
    pub speed: f64,
    pub risk: f64,
}

/// Final optimal route selection
#[derive(Debug, Clone)]
pub struct OptimalRoute {
    pub selected_route: RouteOption,
    pub total_score: f64,
    pub reasoning: String,
    pub alternative_routes: Vec<ScoredRoute>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    fn create_test_routing_engine() -> RoutingEngine {
        let client = Arc::new(FusionClient::new(
            "test_key".to_string(),
            ONEINCH_SEPOLIA_CHAIN_ID,
        ));
        RoutingEngine::new(client)
    }

    #[test]
    fn test_optimization_criteria_default() {
        let criteria = OptimizationCriteria::default();
        assert_eq!(criteria.output_weight, 0.4);
        assert_eq!(criteria.gas_weight, 0.25);
        assert_eq!(criteria.speed_weight, 0.25);
        assert_eq!(criteria.risk_weight, 0.1);
        
        // Weights should sum to 1.0
        let total = criteria.output_weight + criteria.gas_weight + criteria.speed_weight + criteria.risk_weight;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adaptive_slippages() {
        let engine = create_test_routing_engine();
        
        let conservative = engine.get_adaptive_slippages(&RiskTolerance::Conservative);
        let moderate = engine.get_adaptive_slippages(&RiskTolerance::Moderate);
        let aggressive = engine.get_adaptive_slippages(&RiskTolerance::Aggressive);
        
        assert!(conservative.len() >= 2);
        assert!(moderate.len() >= 3);
        assert!(aggressive.len() >= 3);
        
        // Conservative should have lower max slippage
        assert!(conservative.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() 
                < aggressive.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = create_test_routing_engine();
        
        assert!(engine.calculate_confidence(95.0) > 0.9);
        assert!(engine.calculate_confidence(85.0) > 0.8);
        assert!(engine.calculate_confidence(75.0) > 0.7);
        assert!(engine.calculate_confidence(65.0) > 0.6);
        assert!(engine.calculate_confidence(55.0) >= 0.5);
    }

    #[test]
    fn test_gas_optimization_types() {
        assert_eq!(GasOptimization::Standard, GasOptimization::Standard);
        assert_ne!(GasOptimization::Economy, GasOptimization::Fast);
    }
}