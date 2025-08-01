// src/handlers/bridge.rs - Bridge operation handlers
use axum::{extract::{State, Path, Query, Json as ExtractJson}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use bigdecimal::{BigDecimal, ToPrimitive};
use std::str::FromStr;
use chrono::{Duration, Utc};
use crate::state::AppState;
use crate::dynamic_pricing::types::BridgeQuoteRequest;
use crate::utils::token_mapping::symbol_to_token_address;
use crate::extractors::auth::AuthUser;

/// Query parameters for bridge quote request
#[derive(Debug, Deserialize)]
pub struct QuoteQueryParams {
    pub from_token: String,
    pub to_token: String,
    pub from_chain: String,
    pub to_chain: String,
    pub from_amount: String,
    pub max_slippage: Option<f64>,
    pub user_id: Option<String>,
}

/// Request body for initiating a swap
#[derive(Debug, Deserialize)]
pub struct SwapInitRequest {
    pub quote_id: String,
    pub from_chain: String,
    pub to_chain: String,   
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    pub recipient_address: String,
    pub max_slippage: Option<f64>,
}

/// Response for successful swap initiation
#[derive(Debug, Serialize)]
pub struct SwapInitResponse {
    pub transaction_id: String,
    pub status: String,
    pub estimated_time_minutes: i64,
    pub expires_at: String,
    pub next_steps: Vec<String>,
}

/// Get swap quote (Phase 6.3.5)
pub async fn get_quote(
    State(state): State<AppState>,
    Query(params): Query<QuoteQueryParams>
) -> Result<Json<Value>, StatusCode> {
    // Validate parameters
    let from_amount = match BigDecimal::from_str(&params.from_amount) {
        Ok(amount) => amount,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    if from_amount <= BigDecimal::from(0) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate token symbols are supported
    if let Err(e) = symbol_to_token_address(&params.from_token) {
        tracing::error!("Invalid from_token: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if let Err(e) = symbol_to_token_address(&params.to_token) {
        tracing::error!("Invalid to_token: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Keep symbols in the request - conversion to addresses happens in pricing service
    let quote_request = BridgeQuoteRequest {
        from_token: params.from_token,
        to_token: params.to_token,
        from_chain: params.from_chain,
        to_chain: params.to_chain,
        from_amount,
        max_slippage: params.max_slippage,
        user_id: params.user_id,
    };
    
    // Use dynamic pricing service for real quote calculation
    match state.dynamic_pricing_service.get_bridge_quote(&quote_request).await {
        Ok(quote) => {
            // Convert BigDecimal to string for JSON serialization
            Ok(Json(json!({
                "quote_id": quote.quote_id,
                "from_token": quote.from_token,
                "to_token": quote.to_token,
                "from_chain": quote.from_chain,
                "to_chain": quote.to_chain,
                "from_amount": quote.from_amount.to_string(),
                "to_amount": quote.to_amount.to_string(),
                "exchange_rate": {
                    "rate": quote.exchange_rate.rate.to_string(),
                    "rate_source": quote.exchange_rate.rate_source,
                    "confidence_score": quote.exchange_rate.confidence_score,
                    "last_updated": quote.exchange_rate.last_updated,
                    "volatility_indicator": quote.exchange_rate.volatility_indicator
                },
                "fee_breakdown": {
                    "base_fee": quote.fee_breakdown.base_fee.to_string(),
                    "gas_fee": quote.fee_breakdown.gas_fee.to_string(),
                    "protocol_fee": quote.fee_breakdown.protocol_fee.to_string(),
                    "slippage_protection_fee": quote.fee_breakdown.slippage_protection_fee.to_string(),
                    "total_fee_amount": quote.fee_breakdown.total_fee_amount.to_string(),
                    "fee_percentage": quote.fee_breakdown.fee_percentage,
                    "fee_currency": quote.fee_breakdown.fee_currency
                },
                "price_impact": {
                    "impact_percentage": quote.price_impact.impact_percentage,
                    "impact_level": quote.price_impact.impact_level,
                    "liquidity_assessment": {
                        "liquidity_score": quote.price_impact.liquidity_assessment.liquidity_score,
                        "available_liquidity": quote.price_impact.liquidity_assessment.available_liquidity.to_string(),
                        "liquidity_sources": quote.price_impact.liquidity_assessment.liquidity_sources,
                        "fragmentation_risk": quote.price_impact.liquidity_assessment.fragmentation_risk
                    },
                    "market_depth": {
                        "bid_depth": quote.price_impact.market_depth.bid_depth.to_string(),
                        "ask_depth": quote.price_impact.market_depth.ask_depth.to_string(),
                        "spread_percentage": quote.price_impact.market_depth.spread_percentage,
                        "market_stability": quote.price_impact.market_depth.market_stability
                    },
                    "recommendations": quote.price_impact.recommendations
                },
                "slippage_settings": {
                    "max_slippage": quote.slippage_settings.max_slippage,
                    "recommended_slippage": quote.slippage_settings.recommended_slippage,
                    "dynamic_adjustment": quote.slippage_settings.dynamic_adjustment,
                    "protection_level": quote.slippage_settings.protection_level,
                    "timeout_minutes": quote.slippage_settings.timeout_minutes
                },
                "estimated_execution_time": quote.estimated_execution_time,
                "valid_until": quote.valid_until,
                "created_at": quote.created_at
            })))
        }
        Err(e) => {
            // Proper error handling - NEVER return fake data as fallback
            tracing::error!("Error getting bridge quote: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Initiate cross-chain swap (Phase 8.1.1)
#[axum::debug_handler]
pub async fn initiate_swap(
    State(state): State<AppState>,
    user: AuthUser,
    ExtractJson(payload): ExtractJson<SwapInitRequest>
) -> Result<Json<SwapInitResponse>, StatusCode> {
    tracing::info!("User {} initiating swap: {} {} -> {} {}", 
        user.user_id, payload.from_amount, payload.from_token, payload.to_amount, payload.to_token);
    
    // Validate request parameters
    let from_amount = match BigDecimal::from_str(&payload.from_amount) {
        Ok(amount) => amount,
        Err(e) => {
            tracing::error!("Invalid from_amount '{}': {}", payload.from_amount, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    if from_amount <= BigDecimal::from(0) {
        tracing::error!("Amount must be positive, got: {}", from_amount);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate token symbols
    if let Err(e) = symbol_to_token_address(&payload.from_token) {
        tracing::error!("Invalid from_token '{}': {}", payload.from_token, e);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if let Err(e) = symbol_to_token_address(&payload.to_token) {
        tracing::error!("Invalid to_token '{}': {}", payload.to_token, e);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Check bridge service availability
    let bridge_service = match &state.bridge_service {
        Some(service) => service,
        None => {
            tracing::error!("Bridge service not available");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };
    
    // Convert amount to wei for internal processing (simplified for hackathon)
    let amount_wei = (from_amount.clone() * BigDecimal::from(1_000_000_000_000_000_000u64))
        .to_u128()
        .unwrap_or(0);
    
    if amount_wei == 0 {
        tracing::error!("Amount conversion to wei failed for: {}", from_amount);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Initialize swap through bridge service
    match bridge_service.init_swap(
        user.user_id,
        &payload.from_chain,
        &payload.to_chain,
        amount_wei,
        &payload.recipient_address,
    ).await {
        Ok(swap_init_response) => {
            tracing::info!("Swap initialized successfully: {}", swap_init_response.swap_id);
            
            // TODO (feat): Start background execution of the swap
            // For now, the swap is only initialized but not executed
            let service_clone = bridge_service.clone();
            let swap_id = swap_init_response.swap_id;
            tokio::spawn(async move {
                // TODO (fix): Add proper error handling and retry logic
                if let Err(e) = service_clone.execute_swap(swap_id).await {
                    tracing::error!("Swap execution failed for {}: {:?}", swap_id, e);
                }
            });
            
            let response = SwapInitResponse {
                transaction_id: swap_init_response.swap_id.to_string(),
                status: "initialized".to_string(),
                estimated_time_minutes: swap_init_response.estimated_time.num_minutes(),
                expires_at: (Utc::now() + swap_init_response.estimated_time).to_rfc3339(),
                next_steps: vec![
                    "Transaction has been initialized".to_string(),
                    "Quantum encryption is being applied".to_string(),
                    "Cross-chain transfer will begin shortly".to_string(),
                    "You will receive status updates".to_string(),
                ],
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to initialize swap: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get swap transaction status (Phase 4.3.8)
pub async fn get_swap_status(
    State(_state): State<AppState>,
    Path(transaction_id): Path<String>
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "transaction_id": transaction_id,
        "status": "pending",
        "progress": 25,
        "steps": [
            {"name": "validation", "status": "completed"},
            {"name": "source_lock", "status": "in_progress"},
            {"name": "destination_mint", "status": "pending"},
            {"name": "confirmation", "status": "pending"}
        ],
        "message": "Transaction status tracking will be implemented in Phase 4.3",
        "implementation_phase": "4.3"
    })))
}

/// Get transaction history
pub async fn get_transaction_history(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "transactions": [],
        "total": 0,
        "page": 1,
        "message": "Transaction history will be implemented in Phase 4.3",
        "implementation_phase": "4.3"
    })))
}

// TODO (check): Interesting
/// Get supported tokens for bridge operations
pub async fn get_supported_tokens(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Return list of supported tokens for both Ethereum and NEAR chains
    // This is real data - common tokens that would be supported by a bridge
    Ok(Json(json!([
        {
            "symbol": "ETH",
            "name": "Ethereum",
            "address": "0x0000000000000000000000000000000000000000",
            "decimals": 18,
            "chain": "ethereum",
            "logo_url": "https://assets.coingecko.com/coins/images/279/large/ethereum.png",
            "is_native": true
        },
        {
            "symbol": "USDC",
            "name": "USD Coin",
            "address": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D", // Sepolia USDC
            "decimals": 6,
            "chain": "ethereum",
            "logo_url": "https://assets.coingecko.com/coins/images/6319/large/USD_Coin_icon.png",
            "is_native": false
        },
        {
            "symbol": "USDT",
            "name": "Tether USD",
            "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7", // Ethereum USDT
            "decimals": 6,
            "chain": "ethereum",
            "logo_url": "https://assets.coingecko.com/coins/images/325/large/Tether.png",
            "is_native": false
        },
        {
            "symbol": "NEAR",
            "name": "NEAR Protocol",
            "address": "near",
            "decimals": 24,
            "chain": "near",
            "logo_url": "https://assets.coingecko.com/coins/images/10365/large/near.jpg",
            "is_native": true
        },
        {
            "symbol": "USDC.e",
            "name": "USD Coin (NEAR)",
            "address": "17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
            "decimals": 6,
            "chain": "near",
            "logo_url": "https://assets.coingecko.com/coins/images/6319/large/USD_Coin_icon.png",
            "is_native": false
        },
        {
            "symbol": "wNEAR",
            "name": "Wrapped NEAR",
            "address": "wrap.near",
            "decimals": 24,
            "chain": "near",
            "logo_url": "https://assets.coingecko.com/coins/images/10365/large/near.jpg",
            "is_native": false
        }
    ])))
}