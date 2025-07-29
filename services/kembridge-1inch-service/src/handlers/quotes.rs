use axum::{
    extract::{State, Query},
    Json,
};
use crate::errors::{OneinchServiceError, Result};
use crate::services::AppState;
use crate::types::{QuoteRequest, QuoteResponse, EnhancedQuoteRequest, EnhancedQuoteResponse};
use bigdecimal::BigDecimal;
use kembridge_common::ServiceResponse;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, error};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct QuoteQuery {
    pub chain_id: u64,
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub slippage: Option<String>,
    pub user_address: Option<String>,
}

pub async fn get_quote(
    State(state): State<AppState>,
    Query(query): Query<QuoteQuery>,
) -> Result<Json<ServiceResponse<QuoteResponse>>> {
    info!("Quote request: {} -> {} on chain {}", 
          query.from_token, query.to_token, query.chain_id);

    // Parse amount
    let amount = query.amount.parse()
        .map_err(|_| OneinchServiceError::validation_error("amount", "Invalid amount format"))?;

    // Parse slippage if provided
    let slippage = if let Some(slippage_str) = query.slippage {
        Some(slippage_str.parse()
            .map_err(|_| OneinchServiceError::validation_error("slippage", "Invalid slippage format"))?)
    } else {
        None
    };

    let request = QuoteRequest {
        chain_id: query.chain_id,
        from_token: query.from_token,
        to_token: query.to_token,
        amount,
        slippage,
        user_address: query.user_address,
    };

    // Validate request
    request.validate()
        .map_err(|e| OneinchServiceError::validation_error("request", &e.to_string()))?;

    // Get quote
    match state.quote_manager.get_quote(&request).await {
        Ok(quote) => {
            info!("Quote generated successfully: {} {} -> {} {}", 
                  quote.from_amount, quote.from_token.symbol,
                  quote.to_amount, quote.to_token.symbol);
            Ok(Json(ServiceResponse::success(quote)))
        }
        Err(e) => {
            error!("Failed to get quote: {}", e);
            Err(e)
        }
    }
}

pub async fn get_enhanced_quote(
    State(state): State<AppState>,
    Json(request): Json<EnhancedQuoteRequest>,
) -> Result<Json<ServiceResponse<EnhancedQuoteResponse>>> {
    info!("Enhanced quote request: {} -> {} on chain {}", 
          request.base.from_token, request.base.to_token, request.base.chain_id);

    // Validate request
    request.base.validate()
        .map_err(|e| OneinchServiceError::validation_error("request", &e.to_string()))?;

    // Get enhanced quote
    match state.quote_manager.get_enhanced_quote(&request).await {
        Ok(quote) => {
            info!("Enhanced quote generated successfully with {} alternative routes", 
                  quote.alternative_routes.len());
            Ok(Json(ServiceResponse::success(quote)))
        }
        Err(e) => {
            error!("Failed to get enhanced quote: {}", e);
            Err(e)
        }
    }
}

pub async fn get_intelligent_routing(
    State(state): State<AppState>,
    Json(request): Json<IntelligentRoutingRequest>,
) -> Result<Json<ServiceResponse<IntelligentRoutingResponse>>> {
    info!("Intelligent routing request for optimization: {:?}", request.optimization_weights);

    // Validate request
    request.base.validate()
        .map_err(|e| OneinchServiceError::validation_error("request", &e.to_string()))?;

    // Get base quote first
    let base_quote = state.quote_manager.get_quote(&request.base).await?;

    // Apply intelligent routing optimization
    let optimized_routes = apply_intelligent_routing(&base_quote, &request.optimization_weights).await?;
    let recommended_route_id = optimized_routes.first().map(|r| r.route_id.clone());

    let response = IntelligentRoutingResponse {
        base_quote,
        optimized_routes,
        optimization_summary: OptimizationSummary {
            gas_savings_percentage: BigDecimal::from(5), // Example
            time_improvement_percentage: BigDecimal::from(10), // Example
            price_improvement_percentage: BigDecimal::from(2), // Example
            recommended_route_id,
        },
    };

    Ok(Json(ServiceResponse::success(response)))
}

async fn apply_intelligent_routing(
    base_quote: &QuoteResponse,
    optimization_weights: &OptimizationWeights,
) -> Result<Vec<crate::types::RouteOption>> {
    // In production, this would:
    // 1. Analyze multiple DEX protocols
    // 2. Consider gas costs vs output amounts
    // 3. Factor in time preferences
    // 4. Apply machine learning for optimal routing

    let mut routes = Vec::new();

    // Example optimized route
    routes.push(crate::types::RouteOption {
        route_id: "optimized_route_1".to_string(),
        protocols: base_quote.protocols.clone(),
        to_amount: &base_quote.to_amount * BigDecimal::from_str("1.02").unwrap(), // 2% better
        gas_estimate: (base_quote.gas_estimate as f64 * 0.95) as u64, // 5% less gas
        price_impact: &base_quote.price_impact * BigDecimal::from_str("0.9").unwrap(), // 10% less impact
        confidence_score: BigDecimal::from(85), // High confidence
    });

    Ok(routes)
}

// Additional types for intelligent routing
#[derive(Debug, serde::Deserialize, Validate)]
pub struct IntelligentRoutingRequest {
    #[serde(flatten)]
    #[validate(nested)]
    pub base: QuoteRequest,
    
    pub optimization_weights: OptimizationWeights,
    pub max_routes: Option<u8>,
    pub include_experimental: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OptimizationWeights {
    pub gas_efficiency: f64,      // 0.0 - 1.0
    pub price_optimization: f64,  // 0.0 - 1.0  
    pub time_preference: f64,     // 0.0 - 1.0
    pub reliability: f64,         // 0.0 - 1.0
}

#[derive(Debug, serde::Serialize)]
pub struct IntelligentRoutingResponse {
    pub base_quote: QuoteResponse,
    pub optimized_routes: Vec<crate::types::RouteOption>,
    pub optimization_summary: OptimizationSummary,
}

#[derive(Debug, serde::Serialize)]
pub struct OptimizationSummary {
    pub gas_savings_percentage: bigdecimal::BigDecimal,
    pub time_improvement_percentage: bigdecimal::BigDecimal,
    pub price_improvement_percentage: bigdecimal::BigDecimal,
    pub recommended_route_id: Option<String>,
}