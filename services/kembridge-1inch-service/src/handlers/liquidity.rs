use axum::{extract::{State, Path}, Json};
use bigdecimal::BigDecimal;
use crate::{errors::Result, services::AppState, types::LiquidityInfo};
use kembridge_common::ServiceResponse;
use std::str::FromStr;

pub async fn get_liquidity_info(
    State(_state): State<AppState>,
    Path((from_token, to_token)): Path<(String, String)>,
) -> Result<Json<ServiceResponse<LiquidityInfo>>> {
    // Mock liquidity data
    let liquidity = LiquidityInfo {
        token_pair: crate::types::TokenPair {
            token_a: crate::types::TokenInfo {
                address: from_token,
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                decimals: 18,
                chain_id: 1,
                logo_uri: None,
                price_usd: Some(BigDecimal::from(2000)),
            },
            token_b: crate::types::TokenInfo {
                address: to_token,
                symbol: "USDT".to_string(),
                name: "Tether USD".to_string(),
                decimals: 6,
                chain_id: 1,
                logo_uri: None,
                price_usd: Some(BigDecimal::from(1)),
            },
        },
        total_liquidity_usd: BigDecimal::from(50000000), // $50M
        available_protocols: vec!["Uniswap V3".to_string(), "Curve".to_string()],
        current_spread: BigDecimal::from_str("0.003").unwrap(), // 0.3%
        volume_24h: Some(BigDecimal::from(10000000)), // $10M
        price_impact_levels: vec![
            crate::types::PriceImpactLevel {
                amount_usd: BigDecimal::from(1000),
                price_impact: BigDecimal::from_str("0.01").unwrap(),
            },
            crate::types::PriceImpactLevel {
                amount_usd: BigDecimal::from(10000),
                price_impact: BigDecimal::from_str("0.1").unwrap(),
            },
        ],
    };
    
    Ok(Json(ServiceResponse::success(liquidity)))
}