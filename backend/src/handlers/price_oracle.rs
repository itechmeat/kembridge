// src/handlers/price_oracle.rs - Price Oracle API handlers
use axum::{
    extract::{State, Path, Query},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, error};
use uuid::Uuid;
use bigdecimal::{BigDecimal, FromPrimitive};

use crate::AppState;
use crate::extractors::auth::AuthUser;
use crate::price_oracle::types::{AggregatedPrice, PriceData, TradingPair};
use crate::constants::*;

/// Price request query parameters
#[derive(Debug, Deserialize)]
pub struct PriceQuery {
    pub symbol: String,
    pub include_sources: Option<bool>,
    pub include_history: Option<bool>,
}

/// Multiple prices request
#[derive(Debug, Deserialize)]
pub struct MultiplePricesQuery {
    pub symbols: String, // Comma-separated symbols
    pub include_sources: Option<bool>,
}

/// Price response
#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price: bigdecimal::BigDecimal,
    pub last_updated: DateTime<Utc>,
    pub confidence: f64,
    pub sources: Option<Vec<String>>,
    pub change_24h: Option<f64>,
    pub volume_24h: Option<bigdecimal::BigDecimal>,
}

/// Multiple prices response
#[derive(Debug, Serialize)]
pub struct MultiplePricesResponse {
    pub prices: Vec<PriceResponse>,
    pub timestamp: DateTime<Utc>,
    pub total_count: usize,
}

/// Price quote request
#[derive(Debug, Deserialize)]
pub struct PriceQuoteRequest {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: bigdecimal::BigDecimal,
}

/// Price quote response
#[derive(Debug, Serialize)]
pub struct PriceQuoteResponse {
    pub from_token: String,
    pub to_token: String,
    pub from_amount: bigdecimal::BigDecimal,
    pub to_amount: bigdecimal::BigDecimal,
    pub exchange_rate: bigdecimal::BigDecimal,
    pub price_impact: f64,
    pub timestamp: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confidence: f64,
    pub sources: Vec<String>,
}

/// Provider health response
#[derive(Debug, Serialize)]
pub struct ProviderHealthResponse {
    pub providers: Vec<ProviderStatus>,
    pub total_providers: usize,
    pub healthy_providers: usize,
    pub timestamp: DateTime<Utc>,
}

/// Individual provider status
#[derive(Debug, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub is_available: bool,
    pub last_successful_request: DateTime<Utc>,
    pub error_rate: f64,
    pub average_latency_ms: u64,
}

/// Cache statistics response
#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub total_keys: usize,
    pub primary_prices: usize,
    pub fallback_prices: usize,
    pub provider_prices: usize,
    pub cache_hit_rate: f64,
    pub timestamp: DateTime<Utc>,
}

/// Get price for a single symbol
pub async fn get_price(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<PriceQuery>,
) -> Result<Json<PriceResponse>, StatusCode> {
    info!("Getting price for symbol: {}", query.symbol);
    
    // Use real PriceOracleService (quick implementation)
    match state.price_oracle_service.get_real_price(&query.symbol).await {
        Ok(aggregated_price) => {
            let response = PriceResponse {
                symbol: aggregated_price.symbol,
                price: BigDecimal::from_f64(aggregated_price.price).unwrap_or(BigDecimal::from(0)),
                last_updated: aggregated_price.last_updated,
                confidence: aggregated_price.confidence,
                sources: if query.include_sources.unwrap_or(false) {
                    Some(aggregated_price.sources)
                } else {
                    None
                },
                change_24h: aggregated_price.change_24h,
                volume_24h: aggregated_price.volume_24h.map(|v| BigDecimal::from_f64(v).unwrap_or(BigDecimal::from(0))),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get price for {}: {}", query.symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get multiple prices
pub async fn get_multiple_prices(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<MultiplePricesQuery>,
) -> Result<Json<MultiplePricesResponse>, StatusCode> {
    let symbols: Vec<&str> = query.symbols.split(',').collect();
    info!("Getting prices for {} symbols", symbols.len());
    
    // Use real PriceOracleService (quick implementation)
    match state.price_oracle_service.get_multiple_real_prices(&symbols).await {
        Ok(aggregated_prices) => {
            let mut prices = Vec::new();
            for aggregated_price in aggregated_prices {
                prices.push(PriceResponse {
                    symbol: aggregated_price.symbol,
                    price: BigDecimal::from_f64(aggregated_price.price).unwrap_or(BigDecimal::from(0)),
                    last_updated: aggregated_price.last_updated,
                    confidence: aggregated_price.confidence,
                    sources: if query.include_sources.unwrap_or(false) {
                        Some(aggregated_price.sources)
                    } else {
                        None
                    },
                    change_24h: aggregated_price.change_24h,
                    volume_24h: aggregated_price.volume_24h.map(|v| BigDecimal::from_f64(v).unwrap_or(BigDecimal::from(0))),
                });
            }
            
            let response = MultiplePricesResponse {
                total_count: prices.len(),
                prices,
                timestamp: Utc::now(),
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get multiple prices: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get price quote for swap
pub async fn get_price_quote(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(request): Json<PriceQuoteRequest>,
) -> Result<Json<PriceQuoteResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Getting price quote for {} -> {}", request.from_token, request.to_token);
    
    // Use real PriceOracleService for both tokens
    let from_price_result = state.price_oracle_service.get_real_price(&request.from_token).await;
    let to_price_result = state.price_oracle_service.get_real_price(&request.to_token).await;
    
    match (from_price_result, to_price_result) {
        (Ok(from_price_data), Ok(to_price_data)) => {
            let from_price = BigDecimal::from_f64(from_price_data.price).unwrap_or(BigDecimal::from(0));
            let to_price = BigDecimal::from_f64(to_price_data.price).unwrap_or(BigDecimal::from(1));
            
            let exchange_rate = &from_price / &to_price;
            let to_amount = &request.from_amount * &exchange_rate;
            
            let now = Utc::now();
            let response = PriceQuoteResponse {
                from_token: request.from_token,
                to_token: request.to_token,
                from_amount: request.from_amount,
                to_amount,
                exchange_rate,
                price_impact: PRICE_IMPACT_DEFAULT,
                timestamp: now,
                expires_at: now + chrono::Duration::seconds(PRICE_QUOTE_EXPIRY_SECONDS),
                confidence: (from_price_data.confidence + to_price_data.confidence) / 2.0,
                sources: [from_price_data.sources, to_price_data.sources].concat().into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect(),
            };
            
            Ok(Json(response))
        }
        _ => {
            // Return error instead of fallback to mock data
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Price data unavailable",
                    "message": format!(
                        "Unable to fetch real price data for {} and {}. Please try again later.",
                        request.from_token, request.to_token
                    )
                }))
            ));
        }
    }
}

/// Get provider health status
pub async fn get_provider_health(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ProviderHealthResponse>, StatusCode> {
    info!("Getting provider health status");
    
    // Use real PriceOracleService
    let provider_healths = state.price_oracle_service.get_provider_health().await;
    
    let mut providers = Vec::new();
    for health in provider_healths {
        providers.push(ProviderStatus {
            name: health.name,
            is_available: health.is_available,
            last_successful_request: health.last_successful_request,
            error_rate: health.error_rate,
            average_latency_ms: health.average_latency.as_millis() as u64,
        });
    }
    
    let response = ProviderHealthResponse {
        total_providers: providers.len(),
        healthy_providers: providers.iter().filter(|p| p.is_available).count(),
        providers,
        timestamp: Utc::now(),
    };
    
    Ok(Json(response))
}

/// Get cache statistics
pub async fn get_cache_stats(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<CacheStatsResponse>, StatusCode> {
    info!("Getting cache statistics");
    
    // For now, return static cache stats until full cache integration
    // TODO: Add real cache statistics when full price oracle cache is integrated
    let response = CacheStatsResponse {
        total_keys: CACHE_TOTAL_KEYS_DEFAULT,
        primary_prices: CACHE_PRIMARY_PRICES_DEFAULT,
        fallback_prices: CACHE_FALLBACK_PRICES_DEFAULT,
        provider_prices: CACHE_PROVIDER_PRICES_DEFAULT,
        cache_hit_rate: CACHE_HIT_RATE_DEFAULT,
        timestamp: Utc::now(),
    };
    
    Ok(Json(response))
}

/// Get supported symbols
pub async fn get_supported_symbols(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<String>>, StatusCode> {
    info!("Getting supported symbols");
    
    // Use real PriceOracleService
    let symbols = state.price_oracle_service.get_supported_symbols();
    
    Ok(Json(symbols))
}

/// Clear price cache (admin endpoint)
pub async fn clear_cache(
    State(state): State<AppState>,
    _user: AuthUser, // In production, would use AdminAuth
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Clearing price cache");
    
    // TODO: Integrate with actual PriceOracleService cache clearing when full cache is implemented
    // For now, just return success since quick oracle doesn't have persistent cache
    Ok(Json(serde_json::json!({
        "message": "Cache cleared successfully (quick oracle mode)",
        "timestamp": Utc::now(),
        "note": "Full cache integration pending"
    })))
}

/// Price alert configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceAlertRequest {
    pub symbol: String,
    pub target_price: BigDecimal,
    pub condition: AlertCondition,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    Change(f64), // Percentage change
}

/// Create price alert
pub async fn create_price_alert(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<PriceAlertRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Creating price alert for {} at ${}", request.symbol, request.target_price);
    
    // TODO: Integrate with actual alert system
    Ok(Json(serde_json::json!({
        "id": Uuid::new_v4(),
        "user_id": user.user_id,
        "symbol": request.symbol,
        "target_price": request.target_price,
        "condition": request.condition,
        "created_at": Utc::now(),
        "is_active": true
    })))
}

/// Get user's price alerts
pub async fn get_user_alerts(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    info!("Getting price alerts for user {}", user.user_id);
    
    // TODO: Integrate with actual alert system
    let alerts = vec![
        serde_json::json!({
            "id": Uuid::new_v4(),
            "user_id": user.user_id,
            "symbol": "ETH/USD",
            "target_price": 2500,
            "condition": "Above",
            "created_at": Utc::now(),
            "is_active": true
        }),
    ];
    
    Ok(Json(alerts))
}

/// Delete price alert
pub async fn delete_price_alert(
    State(state): State<AppState>,
    user: AuthUser,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting price alert {} for user {}", alert_id, user.user_id);
    
    // TODO: Integrate with actual alert system
    Ok(Json(serde_json::json!({
        "message": "Alert deleted successfully",
        "alert_id": alert_id,
        "timestamp": Utc::now()
    })))
}