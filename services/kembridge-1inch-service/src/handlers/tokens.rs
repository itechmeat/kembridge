use axum::{extract::{State, Path}, Json};
use crate::{errors::Result, services::AppState, types::TokenInfo};
use kembridge_common::ServiceResponse;

pub async fn get_supported_tokens(
    State(_state): State<AppState>,
) -> Result<Json<ServiceResponse<Vec<TokenInfo>>>> {
    // Mock data for now
    let tokens = vec![
        TokenInfo {
            address: "0xA0b86a33E6BA30b22cDCAA4f1ee79f4b14b6a67e".to_string(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
            chain_id: 1,
            logo_uri: Some("https://tokens.1inch.io/0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.png".to_string()),
            price_usd: Some(bigdecimal::BigDecimal::from(2000)),
        },
        TokenInfo {
            address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            symbol: "USDT".to_string(),
            name: "Tether USD".to_string(),
            decimals: 6,
            chain_id: 1,
            logo_uri: Some("https://tokens.1inch.io/0xdac17f958d2ee523a2206206994597c13d831ec7.png".to_string()),
            price_usd: Some(bigdecimal::BigDecimal::from(1)),
        },
    ];
    
    Ok(Json(ServiceResponse::success(tokens)))
}

pub async fn get_tokens_by_chain(
    State(_state): State<AppState>,
    Path(chain_id): Path<u64>,
) -> Result<Json<ServiceResponse<Vec<TokenInfo>>>> {
    let mut tokens = vec![];
    
    match chain_id {
        1 => { // Ethereum
            tokens.push(TokenInfo {
                address: "0xA0b86a33E6BA30b22cDCAA4f1ee79f4b14b6a67e".to_string(),
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                decimals: 18,
                chain_id: 1,
                logo_uri: Some("https://tokens.1inch.io/0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.png".to_string()),
                price_usd: Some(bigdecimal::BigDecimal::from(2000)),
            });
        }
        56 => { // BSC
            tokens.push(TokenInfo {
                address: "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c".to_string(),
                symbol: "BNB".to_string(),
                name: "BNB".to_string(),
                decimals: 18,
                chain_id: 56,
                logo_uri: None,
                price_usd: Some(bigdecimal::BigDecimal::from(300)),
            });
        }
        _ => {} // Empty for unsupported chains
    }
    
    Ok(Json(ServiceResponse::success(tokens)))
}