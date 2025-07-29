// src/handlers/oneinch.rs - HTTP handlers for 1inch Fusion+ integration

use axum::{
    extract::{State, Query, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::{info, warn};

use crate::{
    extractors::auth::AuthUser,
    oneinch::{
        types::*,
        quote_engine::{QuoteSelectionCriteria, QuoteRecommendation},
        routing::{RouteSearchParams, OptimizationCriteria, RiskTolerance},
    },
    state::AppState,
    constants::*,
};

/// Quote request parameters
#[derive(Debug, Deserialize, ToSchema)]
pub struct QuoteRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub slippage: Option<f64>,
    pub include_gas: Option<bool>,
    pub enable_estimate: Option<bool>,
}

/// Enhanced quote request with multiple criteria
#[derive(Debug, Deserialize, ToSchema)]
pub struct EnhancedQuoteRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub slippages: Option<Vec<f64>>, // Multiple slippages for comparison
    pub selection_criteria: Option<String>, // "max_output", "min_gas", "best_efficiency", "fastest"
    pub compare_with_oracle: Option<bool>,
}

/// Swap execution request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SwapExecutionRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub slippage: f64,
    pub deadline_minutes: Option<u64>,
    pub referrer: Option<String>,
}

/// Signed swap execution request (after client signs the order)
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignedSwapExecutionRequest {
    pub order_hash: String,
    pub signature: String,
}

/// Intelligent routing request
#[derive(Debug, Deserialize, ToSchema)]
pub struct IntelligentRoutingRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub optimization_strategy: Option<String>, // "output", "gas", "speed", "balanced"
    pub risk_tolerance: Option<String>,        // "conservative", "moderate", "aggressive"
    pub max_slippage: Option<f64>,
    pub include_gas_optimization: Option<bool>,
    pub custom_weights: Option<OptimizationWeights>,
}

/// Custom optimization weights
#[derive(Debug, Deserialize, ToSchema)]
pub struct OptimizationWeights {
    pub output_weight: f64,
    pub gas_weight: f64,
    pub speed_weight: f64,
    pub risk_weight: f64,
}

/// Quote response for API
#[derive(Debug, Serialize, ToSchema)]
pub struct QuoteResponse {
    pub quote_id: String,
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: String,
    pub to_amount: String,
    pub exchange_rate: String,
    pub estimated_gas: String,
    pub protocols: Vec<ProtocolInfo>,
    pub expires_in_seconds: i64,
    pub confidence_score: Option<f64>,
    pub price_impact: Option<String>,
    pub recommendation: Option<String>,
}

/// Enhanced quote response with comparison data
#[derive(Debug, Serialize, ToSchema)]
pub struct EnhancedQuoteResponse {
    pub quotes: Vec<QuoteWithRating>,
    pub best_quote: QuoteResponse,
    pub oracle_comparison: Option<OracleComparisonData>,
    pub recommendation: String,
}

/// Quote with rating information
#[derive(Debug, Serialize, ToSchema)]
pub struct QuoteWithRating {
    pub quote: QuoteResponse,
    pub efficiency_score: f64,
    pub ranking: u8,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// Oracle comparison data
#[derive(Debug, Serialize, ToSchema)]
pub struct OracleComparisonData {
    pub oracle_rate: String,
    pub oneinch_rate: String,
    pub price_difference_percent: String,
    pub is_favorable: bool,
    pub recommendation: String,
}

/// Swap result response
#[derive(Debug, Serialize, ToSchema)]
pub struct SwapResponse {
    pub order_hash: String,
    pub status: String,
    pub from_amount: String,
    pub to_amount: String,
    pub estimated_completion_time: Option<String>,
    pub transaction_hash: Option<String>,
}

/// Order status response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrderStatusResponse {
    pub order_hash: String,
    pub status: String,
    pub fills: Vec<FillInfo>,
    pub created_at: String,
    pub updated_at: String,
    pub completion_percentage: f64,
}

/// Token information for API responses
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

/// Protocol information for API responses
#[derive(Debug, Serialize, ToSchema)]
pub struct ProtocolInfo {
    pub name: String,
    pub percentage: f64,
}

/// Fill information
#[derive(Debug, Serialize, ToSchema)]
pub struct FillInfo {
    pub transaction_hash: String,
    pub amount: String,
    pub price: String,
    pub timestamp: String,
}

/// Supported tokens response
#[derive(Debug, Serialize, ToSchema)]
pub struct SupportedTokensResponse {
    pub tokens: Vec<TokenInfo>,
    pub chain_id: u64,
    pub total_count: usize,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct OneinchHealthResponse {
    pub status: String,
    pub api_available: bool,
    pub supported_chains: Vec<u64>,
    pub response_time_ms: Option<u64>,
}

/// Intelligent routing response
#[derive(Debug, Serialize, ToSchema)]
pub struct IntelligentRoutingResponse {
    pub optimal_route: RouteInfo,
    pub alternative_routes: Vec<RouteInfo>,
    pub selection_reasoning: String,
    pub confidence_score: f64,
    pub estimated_savings: Option<SavingsInfo>,
}

/// Route information
#[derive(Debug, Serialize, ToSchema)]
pub struct RouteInfo {
    pub quote: QuoteResponse,
    pub route_type: String,
    pub optimization_type: String,
    pub execution_speed: String,
    pub estimated_completion_time: u64,
    pub scores: RouteScores,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// Route scoring breakdown
#[derive(Debug, Serialize, ToSchema)]
pub struct RouteScores {
    pub total_score: f64,
    pub output_score: f64,
    pub gas_score: f64,
    pub speed_score: f64,
    pub risk_score: f64,
}

/// Savings information
#[derive(Debug, Serialize, ToSchema)]
pub struct SavingsInfo {
    pub gas_savings_percent: f64,
    pub output_improvement_percent: f64,
    pub time_savings_seconds: i64,
}

/// Get quote from 1inch Fusion+
#[utoipa::path(
    post,
    path = "/api/v1/swap/quote",
    request_body = QuoteRequest,
    responses(
        (status = 200, description = "Quote retrieved successfully", body = QuoteResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required"),
        (status = 429, description = "Rate limit exceeded"),
        (status = 503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn get_quote(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    // Convert request to internal format
    let quote_params = QuoteParams {
        from_token: request.from_token,
        to_token: request.to_token,
        amount: request.amount.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        from_address: user.wallet_address,
        slippage: request.slippage,
        disable_estimate: request.enable_estimate.map(|x| !x),
        allow_partial_fill: Some(true),
        source: Some("kembridge".to_string()),
    };

    let quote = oneinch_service.get_quote(&quote_params).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    // Convert to API response format
    let response = QuoteResponse {
        quote_id: quote.quote_id,
        from_token: TokenInfo {
            address: quote.from_token.address,
            symbol: quote.from_token.symbol,
            name: quote.from_token.name,
            decimals: quote.from_token.decimals,
        },
        to_token: TokenInfo {
            address: quote.to_token.address,
            symbol: quote.to_token.symbol,
            name: quote.to_token.name,
            decimals: quote.to_token.decimals,
        },
        from_amount: quote.from_amount.to_string(),
        to_amount: quote.to_amount.to_string(),
        exchange_rate: (&quote.to_amount / &quote.from_amount).to_string(),
        estimated_gas: quote.estimated_gas.to_string(),
        protocols: quote.protocols.iter().map(|p| ProtocolInfo {
            name: p.name.clone(),
            percentage: p.part,
        }).collect(),
        expires_in_seconds: (quote.expires_at - quote.created_at).num_seconds(),
        confidence_score: None, // TODO: Implement confidence scoring
        price_impact: None,     // TODO: Implement price impact calculation
        recommendation: None,   // TODO: Implement recommendation logic
    };

    Ok(Json(response))
}

/// Get enhanced quote with multiple criteria and oracle comparison
#[utoipa::path(
    post,
    path = "/api/v1/swap/quote/enhanced",
    request_body = EnhancedQuoteRequest,
    responses(
        (status = 200, description = "Enhanced quote retrieved successfully", body = EnhancedQuoteResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn get_enhanced_quote(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<EnhancedQuoteRequest>,
) -> Result<Json<EnhancedQuoteResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    // Use default slippages if not provided
    let slippages = request.slippages.unwrap_or_else(|| vec![
        ONEINCH_MIN_SLIPPAGE,
        ONEINCH_DEFAULT_SLIPPAGE,
        ONEINCH_DEFAULT_SLIPPAGE + 0.5,
        ONEINCH_DEFAULT_SLIPPAGE + 1.0,
    ]);

    // Get multiple quotes
    let base_params = QuoteParams {
        from_token: request.from_token,
        to_token: request.to_token,
        amount: request.amount.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        from_address: user.wallet_address,
        slippage: None, // Will be set for each quote
        disable_estimate: Some(false),
        allow_partial_fill: Some(true),
        source: Some("kembridge".to_string()),
    };

    let quotes = oneinch_service.quote_engine.get_multiple_quotes(&base_params, &slippages).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    if quotes.is_empty() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    // Determine selection criteria
    let criteria = match request.selection_criteria.as_deref() {
        Some("max_output") => QuoteSelectionCriteria::MaxOutput,
        Some("min_gas") => QuoteSelectionCriteria::MinGas,
        Some("best_efficiency") => QuoteSelectionCriteria::BestEfficiency,
        Some("fastest") => QuoteSelectionCriteria::FastestExecution,
        _ => QuoteSelectionCriteria::BestEfficiency,
    };

    // Find best quote
    let best_quote = oneinch_service.quote_engine.find_best_quote(&quotes, &criteria)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Implement oracle comparison if requested
    let oracle_comparison = if request.compare_with_oracle.unwrap_or(false) {
        // This would require integration with price oracle service
        None
    } else {
        None
    };

    // Convert quotes to response format with ratings
    let quotes_with_rating: Vec<QuoteWithRating> = quotes.iter().enumerate().map(|(i, quote)| {
        let efficiency = &quote.to_amount / &quote.estimated_gas;
        QuoteWithRating {
            quote: convert_quote_to_response(quote),
            efficiency_score: efficiency.to_string().parse().unwrap_or(0.0),
            ranking: (i + 1) as u8,
            pros: get_quote_pros(quote, &best_quote),
            cons: get_quote_cons(quote, &best_quote),
        }
    }).collect();

    let response = EnhancedQuoteResponse {
        quotes: quotes_with_rating,
        best_quote: convert_quote_to_response(best_quote),
        oracle_comparison,
        recommendation: format!("Best quote selected based on {} criteria", 
            match criteria {
                QuoteSelectionCriteria::MaxOutput => "maximum output",
                QuoteSelectionCriteria::MinGas => "minimum gas cost",
                QuoteSelectionCriteria::BestEfficiency => "best efficiency",
                QuoteSelectionCriteria::FastestExecution => "fastest execution",
            }
        ),
    };

    Ok(Json(response))
}

/// Execute a signed swap order (after client-side signing)
#[utoipa::path(
    post,
    path = "/api/v1/swap/execute-signed",
    request_body = SignedSwapExecutionRequest,
    responses(
        (status = 200, description = "Signed swap executed successfully", body = SwapResponse),
        (status = 400, description = "Invalid request parameters or signature"),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn execute_signed_swap(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<SignedSwapExecutionRequest>,
) -> Result<Json<SwapResponse>, StatusCode> {
    info!("Executing signed swap for user: {}", user.user_id);

    // Execute the signed order
    let swap_result = state.oneinch_service.adapter
        .execute_signed_order(&request.order_hash, &request.signature)
        .await
        .map_err(|e| {
            warn!("Failed to execute signed swap: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

    let response = SwapResponse {
        order_hash: swap_result.order_hash,
        status: swap_result.status.to_string(),
        from_amount: swap_result.from_amount.to_string(),
        to_amount: swap_result.to_amount.to_string(),
        estimated_completion_time: None,
        transaction_hash: swap_result.tx_hash,
    };

    Ok(Json(response))
}

/// Execute swap order (creates order for client-side signing)
#[utoipa::path(
    post,
    path = "/api/v1/swap/execute",
    request_body = SwapExecutionRequest,
    responses(
        (status = 200, description = "Order created for signing", body = SwapResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn execute_swap(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<SwapExecutionRequest>,
) -> Result<Json<SwapResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    let swap_params = SwapParams {
        from_token: request.from_token,
        to_token: request.to_token,
        amount: request.amount.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        from_address: user.wallet_address,
        slippage: request.slippage,
        deadline: request.deadline_minutes.map(|m| m * 60), // Convert minutes to seconds
        referrer: request.referrer,
        fee: None,
    };

    let result = oneinch_service.execute_swap(&swap_params).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let response = SwapResponse {
        order_hash: result.order_hash,
        status: format!("{:?}", result.status),
        from_amount: result.from_amount.to_string(),
        to_amount: result.to_amount.to_string(),
        estimated_completion_time: None, // TODO: Calculate based on network conditions
        transaction_hash: result.tx_hash,
    };

    Ok(Json(response))
}

/// Get order status
#[utoipa::path(
    get,
    path = "/api/v1/swap/order/{order_hash}",
    params(
        ("order_hash" = String, Path, description = "Order hash to check status")
    ),
    responses(
        (status = 200, description = "Order status retrieved successfully", body = OrderStatusResponse),
        (status = 404, description = "Order not found"),
        (status = 401, description = "Authentication required")
    ),
    tag = "1inch Swap"
)]
pub async fn get_order_status(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(order_hash): Path<String>,
) -> Result<Json<OrderStatusResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    let order_details = oneinch_service.adapter.get_order_details(&order_hash).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let fills: Vec<FillInfo> = order_details.fills.iter().map(|fill| FillInfo {
        transaction_hash: fill.tx_hash.clone(),
        amount: fill.amount.to_string(),
        price: fill.price.to_string(),
        timestamp: fill.timestamp.to_rfc3339(),
    }).collect();

    let total_filled = order_details.fills.iter()
        .map(|f| &f.amount)
        .fold(bigdecimal::BigDecimal::from(0), |acc, x| acc + x);
    
    let completion_percentage = if order_details.fills.is_empty() {
        0.0
    } else {
        // This would need the original order amount for accurate calculation
        // For now, we'll use a simple heuristic based on status
        match order_details.status {
            OrderStatus::Filled => 100.0,
            OrderStatus::PartiallyFilled(_) => 50.0, // Approximate
            _ => 0.0,
        }
    };

    let response = OrderStatusResponse {
        order_hash,
        status: format!("{:?}", order_details.status),
        fills,
        created_at: order_details.created_at.to_rfc3339(),
        updated_at: order_details.updated_at.to_rfc3339(),
        completion_percentage,
    };

    Ok(Json(response))
}

/// Get supported tokens
#[utoipa::path(
    get,
    path = "/api/v1/swap/tokens",
    responses(
        (status = 200, description = "Supported tokens retrieved successfully", body = SupportedTokensResponse),
        (status = 503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn get_supported_tokens(
    State(state): State<AppState>,
) -> Result<Json<SupportedTokensResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    // Try to get tokens from 1inch API
    match oneinch_service.adapter.get_supported_tokens().await {
        Ok(tokens) => {
            let token_infos: Vec<TokenInfo> = tokens.iter().map(|token| TokenInfo {
                address: token.address.clone(),
                symbol: token.symbol.clone(),
                name: token.name.clone(),
                decimals: token.decimals,
            }).collect();

            let response = SupportedTokensResponse {
                chain_id: oneinch_service.adapter.get_chain_id(),
                total_count: token_infos.len(),
                tokens: token_infos,
            };

            Ok(Json(response))
        },
        Err(_) => {
            // TODO (check): Interesting
            // Fallback to mock tokens for unsupported networks (e.g. Sepolia testnet)
            warn!("1inch API unavailable, using mock token data for demo purposes");
            
            let mock_tokens = vec![
                TokenInfo {
                    address: ETHEREUM_NATIVE_TOKEN.to_string(),
                    symbol: "ETH".to_string(),
                    name: "Ethereum".to_string(),
                    decimals: 18,
                },
                TokenInfo {
                    address: "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14".to_string(),
                    symbol: "WETH".to_string(),
                    name: "Wrapped Ether".to_string(),
                    decimals: 18,
                },
                TokenInfo {
                    address: "0x779877A7B0D9E8603169DdbD7836e478b4624789".to_string(),
                    symbol: "LINK".to_string(),
                    name: "Chainlink Token".to_string(),
                    decimals: 18,
                },
                TokenInfo {
                    address: "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984".to_string(),
                    symbol: "UNI".to_string(),
                    name: "Uniswap".to_string(),
                    decimals: 18,
                },
                TokenInfo {
                    address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
                    symbol: "DAI".to_string(),
                    name: "Dai Stablecoin".to_string(),
                    decimals: 18,
                },
                TokenInfo {
                    address: "0xA0b86a33E6441041946Ffc3e6ED01E73c23e632c".to_string(),
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                },
            ];

            let response = SupportedTokensResponse {
                chain_id: oneinch_service.adapter.get_chain_id(),
                total_count: mock_tokens.len(),
                tokens: mock_tokens,
            };

            Ok(Json(response))
        }
    }
}

/// Get intelligent routing recommendations
#[utoipa::path(
    post,
    path = "/api/v1/swap/routing/intelligent",
    request_body = IntelligentRoutingRequest,
    responses(
        (status = 200, description = "Intelligent routing completed successfully", body = IntelligentRoutingResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required"),
        (status =503, description = "1inch API unavailable")
    ),
    tag = "1inch Swap"
)]
pub async fn get_intelligent_routing(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<IntelligentRoutingRequest>,
) -> Result<Json<IntelligentRoutingResponse>, StatusCode> {
    let oneinch_service = &state.oneinch_service;
    
    // Parse optimization criteria
    let optimization_criteria = if let Some(custom_weights) = request.custom_weights {
        OptimizationCriteria {
            output_weight: custom_weights.output_weight,
            gas_weight: custom_weights.gas_weight,
            speed_weight: custom_weights.speed_weight,
            risk_weight: custom_weights.risk_weight,
        }
    } else {
        match request.optimization_strategy.as_deref() {
            Some("output") => OptimizationCriteria {
                output_weight: 0.7,
                gas_weight: 0.1,
                speed_weight: 0.1,
                risk_weight: 0.1,
            },
            Some("gas") => OptimizationCriteria {
                output_weight: 0.2,
                gas_weight: 0.6,
                speed_weight: 0.1,
                risk_weight: 0.1,
            },
            Some("speed") => OptimizationCriteria {
                output_weight: 0.2,
                gas_weight: 0.1,
                speed_weight: 0.6,
                risk_weight: 0.1,
            },
            _ => OptimizationCriteria::default(), // "balanced"
        }
    };

    // Parse risk tolerance
    let risk_tolerance = match request.risk_tolerance.as_deref() {
        Some("conservative") => RiskTolerance::Conservative,
        Some("aggressive") => RiskTolerance::Aggressive,
        _ => RiskTolerance::Moderate,
    };

    // Create route search parameters
    let route_params = RouteSearchParams {
        from_token: request.from_token,
        to_token: request.to_token,
        amount: request.amount.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        from_address: user.wallet_address,
        optimization_criteria,
        risk_tolerance,
        include_gas_optimization: request.include_gas_optimization.unwrap_or(true),
        max_slippage: request.max_slippage,
    };

    // Find optimal route
    let optimal_route = oneinch_service.find_optimal_route(&route_params).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    // Convert to response format
    let optimal_route_info = convert_route_to_info(&optimal_route.selected_route);
    let alternative_route_infos = Vec::new(); // optimal_route.alternative_routes would be converted here

    let response = IntelligentRoutingResponse {
        optimal_route: optimal_route_info,
        alternative_routes: alternative_route_infos,
        selection_reasoning: optimal_route.reasoning,
        confidence_score: optimal_route.confidence,
        estimated_savings: None, // TODO: Calculate savings compared to naive approach
    };

    Ok(Json(response))
}

/// Health check for 1inch integration
#[utoipa::path(
    get,
    path = "/api/v1/swap/health",
    responses(
        (status = 200, description = "1inch health status", body = OneinchHealthResponse)
    ),
    tag = "1inch Swap"
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> Json<OneinchHealthResponse> {
    let oneinch_service = &state.oneinch_service;
    
    let start_time = std::time::Instant::now();
    let api_available = oneinch_service.adapter.health_check().await.unwrap_or(false);
    let response_time = start_time.elapsed().as_millis() as u64;

    let response = OneinchHealthResponse {
        status: if api_available { "healthy".to_string() } else { "degraded".to_string() },
        api_available,
        supported_chains: crate::oneinch::OneinchService::get_supported_chains(),
        response_time_ms: Some(response_time),
    };

    Json(response)
}

// Helper functions

fn convert_route_to_info(route: &crate::oneinch::routing::RouteOption) -> RouteInfo {
    RouteInfo {
        quote: convert_quote_to_response(&route.quote),
        route_type: format!("{:?}", route.route_type),
        optimization_type: format!("{:?}", route.gas_optimization),
        execution_speed: format!("{:?}", route.execution_speed),
        estimated_completion_time: route.estimated_completion_time,
        scores: RouteScores {
            total_score: 0.0,   // Would be calculated from ScoredRoute
            output_score: 0.0,  // Would be calculated from ScoredRoute
            gas_score: 0.0,     // Would be calculated from ScoredRoute
            speed_score: 0.0,   // Would be calculated from ScoredRoute
            risk_score: 0.0,    // Would be calculated from ScoredRoute
        },
        pros: Vec::new(),       // Would be calculated based on route characteristics
        cons: Vec::new(),       // Would be calculated based on route characteristics
    }
}

fn convert_quote_to_response(quote: &FusionQuote) -> QuoteResponse {
    QuoteResponse {
        quote_id: quote.quote_id.clone(),
        from_token: TokenInfo {
            address: quote.from_token.address.clone(),
            symbol: quote.from_token.symbol.clone(),
            name: quote.from_token.name.clone(),
            decimals: quote.from_token.decimals,
        },
        to_token: TokenInfo {
            address: quote.to_token.address.clone(),
            symbol: quote.to_token.symbol.clone(),
            name: quote.to_token.name.clone(),
            decimals: quote.to_token.decimals,
        },
        from_amount: quote.from_amount.to_string(),
        to_amount: quote.to_amount.to_string(),
        exchange_rate: (&quote.to_amount / &quote.from_amount).to_string(),
        estimated_gas: quote.estimated_gas.to_string(),
        protocols: quote.protocols.iter().map(|p| ProtocolInfo {
            name: p.name.clone(),
            percentage: p.part,
        }).collect(),
        expires_in_seconds: (quote.expires_at - quote.created_at).num_seconds(),
        confidence_score: None,
        price_impact: None,
        recommendation: None,
    }
}

fn get_quote_pros(quote: &FusionQuote, best_quote: &FusionQuote) -> Vec<String> {
    let mut pros = Vec::new();
    
    if quote.to_amount >= best_quote.to_amount {
        pros.push("Highest output amount".to_string());
    }
    
    if quote.estimated_gas <= best_quote.estimated_gas {
        pros.push("Low gas cost".to_string());
    }
    
    if quote.protocols.len() <= 2 {
        pros.push("Simple routing".to_string());
    }
    
    pros
}

fn get_quote_cons(quote: &FusionQuote, best_quote: &FusionQuote) -> Vec<String> {
    let mut cons = Vec::new();
    
    if quote.to_amount < best_quote.to_amount {
        let diff = ((&best_quote.to_amount - &quote.to_amount) / &best_quote.to_amount * bigdecimal::BigDecimal::from(100)).to_string();
        cons.push(format!("{}% less output", diff));
    }
    
    if quote.estimated_gas > best_quote.estimated_gas {
        cons.push("Higher gas cost".to_string());
    }
    
    if quote.protocols.len() > 3 {
        cons.push("Complex routing".to_string());
    }
    
    cons
}

/// Comprehensive 1inch integration health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct OneinchIntegrationHealthResponse {
    pub status: String,
    pub chain_id: u64,
    pub chain_supported: bool,
    pub api_connectivity: serde_json::Value,
    pub api_key: serde_json::Value,
    pub tokens: serde_json::Value,
    pub timestamp: String,
    pub recommendations: Vec<String>,
}

/// Liquidity information response
#[derive(Debug, Serialize, ToSchema)]
pub struct LiquidityInfoResponse {
    pub from_token: String,
    pub to_token: String,
    pub available: bool,
    pub liquidity_score: Option<f64>,
    pub protocols: Option<Vec<ProtocolInfo>>,
    pub estimated_gas: Option<String>,
    pub error: Option<String>,
}

/// API key validation response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyValidationResponse {
    pub valid: bool,
    pub message: String,
    pub chain_id: u64,
    pub timestamp: String,
}

/// Comprehensive 1inch integration health check
/// 
/// Performs a complete health check of the 1inch integration including:
/// - API connectivity test
/// - API key validation
/// - Token listing verification
/// - Chain support verification
#[utoipa::path(
    get,
    path = "/api/v1/swap/health/comprehensive",
    responses(
        (status = 200, description = "Comprehensive health check completed", body = OneinchIntegrationHealthResponse),
        (status = 503, description = "Service unavailable")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "1inch Integration"
)]
pub async fn comprehensive_health_check(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<OneinchIntegrationHealthResponse>, StatusCode> {
    info!("🔍 Starting comprehensive 1inch health check");

    let oneinch_service = &state.oneinch_service;
    
    match oneinch_service.comprehensive_health_check().await {
        Ok(health_data) => {
            let mut recommendations = Vec::new();
            
            // Generate recommendations based on health check results
            if let Some(api_connectivity) = health_data.get("api_connectivity") {
                if api_connectivity.get("accessible") == Some(&serde_json::Value::Bool(false)) {
                    recommendations.push("Check network connectivity to 1inch API".to_string());
                }
            }
            
            if let Some(api_key) = health_data.get("api_key") {
                if api_key.get("authenticated") == Some(&serde_json::Value::Bool(false)) {
                    recommendations.push("Verify 1inch API key is valid and has proper permissions".to_string());
                }
            }
            
            if let Some(tokens) = health_data.get("tokens") {
                if let Some(count) = tokens.get("count") {
                    if count.as_u64().unwrap_or(0) < 10 {
                        recommendations.push("Low token count - check chain ID and API access".to_string());
                    }
                }
            }
            
            if !health_data.get("chain_supported").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false) {
                recommendations.push("Current chain ID is not officially supported by 1inch".to_string());
            }

            let response = OneinchIntegrationHealthResponse {
                status: "completed".to_string(),
                chain_id: health_data.get("chain_id").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(1))).as_u64().unwrap_or(1),
                chain_supported: health_data.get("chain_supported").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false),
                api_connectivity: health_data.get("api_connectivity").cloned().unwrap_or(serde_json::json!({"status": "unknown"})),
                api_key: health_data.get("api_key").cloned().unwrap_or(serde_json::json!({"status": "unknown"})),
                tokens: health_data.get("tokens").cloned().unwrap_or(serde_json::json!({"status": "unknown"})),
                timestamp: health_data.get("timestamp").unwrap_or(&serde_json::Value::String("unknown".to_string())).as_str().unwrap_or("unknown").to_string(),
                recommendations,
            };

            info!("✅ Comprehensive 1inch health check completed successfully");
            Ok(Json(response))
        },
        Err(e) => {
            warn!("❌ Comprehensive 1inch health check failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Validate 1inch API key
/// 
/// Tests the provided API key by making a real request to 1inch API
#[utoipa::path(
    get,
    path = "/api/v1/swap/validate-api-key",
    responses(
        (status = 200, description = "API key validation completed", body = ApiKeyValidationResponse),
        (status = 503, description = "Service unavailable")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "1inch Integration"
)]
pub async fn validate_api_key(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ApiKeyValidationResponse>, StatusCode> {
    info!("🔑 Validating 1inch API key");

    let oneinch_service = &state.oneinch_service;
    
    match oneinch_service.validate_api_key().await {
        Ok(valid) => {
            let message = if valid {
                "API key is valid and authenticated".to_string()
            } else {
                "API key is invalid or lacks proper permissions".to_string()
            };

            let response = ApiKeyValidationResponse {
                valid,
                message,
                chain_id: oneinch_service.adapter.client.get_chain_id(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            info!("✅ API key validation completed: {}", valid);
            Ok(Json(response))
        },
        Err(e) => {
            warn!("❌ API key validation failed: {}", e);
            
            let response = ApiKeyValidationResponse {
                valid: false,
                message: format!("Validation failed: {}", e),
                chain_id: oneinch_service.adapter.client.get_chain_id(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            Ok(Json(response))
        }
    }
}

/// Get liquidity information for token pair
/// 
/// Checks real-time liquidity availability and quality for a specific token pair
#[utoipa::path(
    get,
    path = "/api/v1/swap/liquidity/{from_token}/{to_token}",
    params(
        ("from_token" = String, Path, description = "Source token address"),
        ("to_token" = String, Path, description = "Destination token address")
    ),
    responses(
        (status = 200, description = "Liquidity information retrieved", body = LiquidityInfoResponse),
        (status = 503, description = "Service unavailable")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "1inch Integration"
)]
pub async fn get_liquidity_info(
    State(state): State<AppState>,
    Path((from_token, to_token)): Path<(String, String)>,
    _user: AuthUser,
) -> Result<Json<LiquidityInfoResponse>, StatusCode> {
    info!("🔍 Getting liquidity info for {}/{}", from_token, to_token);

    let oneinch_service = &state.oneinch_service;
    
    match oneinch_service.get_liquidity_info(&from_token, &to_token).await {
        Ok(liquidity_data) => {
            let available = liquidity_data.get("available").unwrap_or(&serde_json::Value::Bool(false)).as_bool().unwrap_or(false);
            
            let response = LiquidityInfoResponse {
                from_token: from_token.clone(),
                to_token: to_token.clone(),
                available,
                liquidity_score: liquidity_data.get("liquidity_score").and_then(|v| v.as_f64()),
                protocols: if available {
                    liquidity_data.get("protocols").and_then(|protocols| {
                        if let Some(protocols_array) = protocols.as_array() {
                            Some(protocols_array.iter().map(|p| ProtocolInfo {
                                name: p.get("name").unwrap_or(&serde_json::Value::String("unknown".to_string())).as_str().unwrap_or("unknown").to_string(),
                                percentage: p.get("part").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))).as_f64().unwrap_or(0.0),
                            }).collect())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                },
                estimated_gas: liquidity_data.get("estimated_gas").and_then(|v| v.as_str()).map(|s| s.to_string()),
                error: liquidity_data.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };

            info!("✅ Liquidity info retrieved: available={}, score={:?}", available, response.liquidity_score);
            Ok(Json(response))
        },
        Err(e) => {
            warn!("❌ Failed to get liquidity info: {}", e);
            
            let response = LiquidityInfoResponse {
                from_token,
                to_token,
                available: false,
                liquidity_score: None,
                protocols: None,
                estimated_gas: None,
                error: Some(e.to_string()),
            };

            Ok(Json(response))
        }
    }
}