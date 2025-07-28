// src/handlers/fusion_plus.rs - 1inch Fusion+ Cross-Chain API Handlers

use axum::{
    extract::{State, Query, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use utoipa::{ToSchema, IntoParams};

use crate::{
    extractors::auth::AuthUser,
    oneinch::{
        FusionPlusQuoteRequest, FusionPlusQuoteResponse, 
        FusionPlusBuildOrderResponse, FusionPlusSubmitOrderRequest,
        FusionPlusActiveOrdersRequest, FusionPlusActiveOrdersResponse,
        ActiveOrder, OneinchError
    },
    state::AppState,
};

// Request/Response types for API

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct CrossChainQuoteRequest {
    pub src_chain: u64,
    pub dst_chain: u64,
    pub src_token_address: String,
    pub dst_token_address: String,
    pub amount: String,
    pub wallet_address: String,
    pub enable_estimate: Option<bool>,
    pub fee: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CrossChainQuoteResponse {
    pub quote_id: String,
    pub src_token_amount: String,
    pub dst_token_amount: String,
    pub recommended_preset: String,
    pub auction_duration_fast: u64,
    pub auction_duration_medium: u64,
    pub auction_duration_slow: u64,
    pub src_safety_deposit: String,
    pub dst_safety_deposit: String,
    pub time_locks: TimeLocksInfo,
    pub prices_usd: TokenPairPrices,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeLocksInfo {
    pub src_withdrawal: u64,
    pub dst_withdrawal: u64,
    pub src_cancellation: u64,
    pub dst_cancellation: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenPairPrices {
    pub src_token: String,
    pub dst_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BuildOrderRequest {
    pub quote_id: String,
    pub preset: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BuildOrderResponse {
    pub order_hash: String,
    pub order: OrderInfo,
    pub secret_hashes_count: usize,
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderInfo {
    pub salt: String,
    pub maker: String,
    pub receiver: String,
    pub maker_asset: String,
    pub taker_asset: String,
    pub making_amount: String,
    pub taking_amount: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitOrderRequest {
    pub order_hash: String,
    pub signature: String,
    pub src_chain_id: u64,
    pub quote_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ActiveOrdersQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub src_chain: Option<u64>,
    pub dst_chain: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ActiveOrdersResponse {
    pub total_items: u64,
    pub current_page: u64,
    pub total_pages: u64,
    pub orders: Vec<OrderSummary>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderSummary {
    pub order_hash: String,
    pub quote_id: String,
    pub src_chain_id: u64,
    pub dst_chain_id: u64,
    pub maker: String,
    pub making_amount: String,
    pub taking_amount: String,
    pub remaining_amount: String,
    pub auction_start_date: u64,
    pub auction_end_date: u64,
    pub status: String,
}

/// Get cross-chain quote using Fusion+
#[utoipa::path(
    get,
    path = "/api/v1/fusion-plus/quote",
    params(CrossChainQuoteRequest),
    responses(
        (status = 200, description = "Cross-chain quote retrieved successfully", body = CrossChainQuoteResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn get_cross_chain_quote(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<CrossChainQuoteRequest>,
) -> Result<Json<CrossChainQuoteResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("🔄 Getting cross-chain quote for user: {}", user.wallet_address);

    // Validate parameters
    if params.src_chain == params.dst_chain {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Source and destination chains must be different for cross-chain swaps"
            }))
        ));
    }

    if params.amount.is_empty() || params.amount == "0" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Amount must be greater than 0"
            }))
        ));
    }

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    // Create Fusion+ request
    let fusion_request = FusionPlusQuoteRequest {
        src_chain: params.src_chain,
        dst_chain: params.dst_chain,
        src_token_address: params.src_token_address,
        dst_token_address: params.dst_token_address,
        amount: params.amount,
        wallet_address: params.wallet_address,
        enable_estimate: params.enable_estimate.unwrap_or(true),
        fee: params.fee,
        is_permit2: None,
        permit: None,
    };

    match state.oneinch_service.get_cross_chain_quote(fusion_request).await {
        Ok(quote) => {
            let response = CrossChainQuoteResponse {
                quote_id: quote.quote_id.as_str().unwrap_or("").to_string(),
                src_token_amount: quote.src_token_amount,
                dst_token_amount: quote.dst_token_amount,
                recommended_preset: quote.recommended_preset,
                auction_duration_fast: quote.presets.fast.auction_duration,
                auction_duration_medium: quote.presets.medium.auction_duration,
                auction_duration_slow: quote.presets.slow.auction_duration,
                src_safety_deposit: quote.src_safety_deposit,
                dst_safety_deposit: quote.dst_safety_deposit,
                time_locks: TimeLocksInfo {
                    src_withdrawal: quote.time_locks.src_withdrawal,
                    dst_withdrawal: quote.time_locks.dst_withdrawal,
                    src_cancellation: quote.time_locks.src_cancellation,
                    dst_cancellation: quote.time_locks.dst_cancellation,
                },
                prices_usd: TokenPairPrices {
                    src_token: quote.prices.usd.src_token,
                    dst_token: quote.prices.usd.dst_token,
                },
            };

            info!("✅ Cross-chain quote retrieved: {} -> {}", 
                response.src_token_amount, response.dst_token_amount);

            Ok(Json(response))
        },
        Err(e) => {
            error!("❌ Failed to get cross-chain quote: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get cross-chain quote: {}", e)
                }))
            ))
        }
    }
}

/// Build cross-chain order from quote
#[utoipa::path(
    post,
    path = "/api/v1/fusion-plus/build-order",
    request_body = BuildOrderRequest,
    responses(
        (status = 200, description = "Order built successfully", body = BuildOrderResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn build_cross_chain_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<BuildOrderRequest>,
) -> Result<Json<BuildOrderResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("🔨 Building cross-chain order for user: {}", user.wallet_address);

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    // Note: In a real implementation, we would need to:
    // 1. Retrieve the original quote by quote_id
    // 2. Validate that the quote is still valid
    // 3. Build the order using the quote data
    
    // For now, return an error indicating this needs implementation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Order building requires quote retrieval - not yet implemented",
            "note": "This endpoint needs access to stored quotes or quote reconstruction"
        }))
    ))
}

/// Submit cross-chain order for execution
#[utoipa::path(
    post,
    path = "/api/v1/fusion-plus/submit-order",
    request_body = SubmitOrderRequest,
    responses(
        (status = 200, description = "Order submitted successfully"),
        (status = 400, description = "Invalid request parameters"),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn submit_cross_chain_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<SubmitOrderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("📤 Submitting cross-chain order for user: {}", user.wallet_address);

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    // Note: In a real implementation, we would need to:
    // 1. Validate the signature
    // 2. Reconstruct the full order from the order_hash
    // 3. Submit to the Fusion+ relayer
    
    // For now, return an error indicating this needs implementation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Order submission requires order reconstruction and signature validation",
            "note": "This endpoint needs full order signing and validation implementation"
        }))
    ))
}

/// Get active cross-chain orders
#[utoipa::path(
    get,
    path = "/api/v1/fusion-plus/orders/active",
    params(ActiveOrdersQuery),
    responses(
        (status = 200, description = "Active orders retrieved successfully", body = ActiveOrdersResponse),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn get_active_cross_chain_orders(
    State(state): State<AppState>,
    user: AuthUser,
    Query(params): Query<ActiveOrdersQuery>,
) -> Result<Json<ActiveOrdersResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("📋 Getting active cross-chain orders for user: {}", user.wallet_address);

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    let fusion_request = FusionPlusActiveOrdersRequest {
        page: params.page,
        limit: params.limit,
        src_chain: params.src_chain,
        dst_chain: params.dst_chain,
    };

    match state.oneinch_service.get_active_cross_chain_orders(fusion_request).await {
        Ok(orders) => {
            let response = ActiveOrdersResponse {
                total_items: orders.meta.total_items,
                current_page: orders.meta.current_page,
                total_pages: orders.meta.total_pages,
                orders: orders.items.into_iter().map(|order| OrderSummary {
                    order_hash: order.order_hash,
                    quote_id: order.quote_id,
                    src_chain_id: order.src_chain_id,
                    dst_chain_id: order.dst_chain_id,
                    maker: order.order.maker,
                    making_amount: order.order.making_amount,
                    taking_amount: order.order.taking_amount,
                    remaining_amount: order.remaining_maker_amount,
                    auction_start_date: order.auction_start_date,
                    auction_end_date: order.auction_end_date,
                    status: if order.fills.is_empty() { "active".to_string() } else { "partially_filled".to_string() },
                }).collect(),
            };

            info!("✅ Retrieved {} active cross-chain orders", response.orders.len());
            Ok(Json(response))
        },
        Err(e) => {
            error!("❌ Failed to get active orders: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get active orders: {}", e)
                }))
            ))
        }
    }
}

/// Get cross-chain order by hash
#[utoipa::path(
    get,
    path = "/api/v1/fusion-plus/orders/{order_hash}",
    params(
        ("order_hash" = String, Path, description = "Order hash to retrieve")
    ),
    responses(
        (status = 200, description = "Order retrieved successfully", body = OrderSummary),
        (status = 404, description = "Order not found"),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn get_cross_chain_order_by_hash(
    State(state): State<AppState>,
    user: AuthUser,
    Path(order_hash): Path<String>,
) -> Result<Json<OrderSummary>, (StatusCode, Json<serde_json::Value>)> {
    info!("🔍 Getting cross-chain order {} for user: {}", order_hash, user.wallet_address);

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    match state.oneinch_service.get_cross_chain_order_by_hash(&order_hash).await {
        Ok(order) => {
            let response = OrderSummary {
                order_hash: order.order_hash,
                quote_id: order.quote_id,
                src_chain_id: order.src_chain_id,
                dst_chain_id: order.dst_chain_id,
                maker: order.order.maker,
                making_amount: order.order.making_amount,
                taking_amount: order.order.taking_amount,
                remaining_amount: order.remaining_maker_amount,
                auction_start_date: order.auction_start_date,
                auction_end_date: order.auction_end_date,
                status: if order.fills.is_empty() { "active".to_string() } else { "partially_filled".to_string() },
            };

            info!("✅ Retrieved cross-chain order: {}", order_hash);
            Ok(Json(response))
        },
        Err(e) => {
            error!("❌ Failed to get order {}: {}", order_hash, e);
            
            // Check if it's a not found error
            if let OneinchError::ApiError { code: 404, .. } = e {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Order not found"
                    }))
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to get order: {}", e)
                    }))
                ))
            }
        }
    }
}

/// Get escrow factory address for chain
#[utoipa::path(
    get,
    path = "/api/v1/fusion-plus/escrow-factory/{chain_id}",
    params(
        ("chain_id" = u64, Path, description = "Chain ID to get escrow factory for")
    ),
    responses(
        (status = 200, description = "Escrow factory address retrieved", body = String),
        (status = 503, description = "Fusion+ service unavailable")
    ),
    tag = "Fusion+"
)]
pub async fn get_escrow_factory(
    State(state): State<AppState>,
    user: AuthUser,
    Path(chain_id): Path<u64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("🏭 Getting escrow factory for chain {} (user: {})", chain_id, user.wallet_address);

    // Check if Fusion+ is available
    if !state.oneinch_service.has_fusion_plus() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Fusion+ cross-chain functionality is not available"
            }))
        ));
    }

    match state.oneinch_service.get_escrow_factory(chain_id).await {
        Ok(factory_address) => {
            info!("✅ Retrieved escrow factory for chain {}: {}", chain_id, factory_address);
            Ok(Json(serde_json::json!({
                "chain_id": chain_id,
                "factory_address": factory_address
            })))
        },
        Err(e) => {
            error!("❌ Failed to get escrow factory for chain {}: {}", chain_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get escrow factory: {}", e)
                }))
            ))
        }
    }
}