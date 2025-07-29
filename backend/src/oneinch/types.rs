// src/oneinch/types.rs - Type definitions for 1inch Fusion+ integration

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for 1inch integration
#[derive(Error, Debug)]
pub enum OneinchError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API error: {code} - {message}")]
    ApiError { code: u16, message: String },
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(u64),
    
    #[error("Order expired")]
    OrderExpired,
    
    #[error("Slippage too high: {actual}% > {max}%")]
    SlippageTooHigh { actual: f64, max: f64 },
    
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Parameters for getting a quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteParams {
    pub from_token: String,
    pub to_token: String,
    pub amount: BigDecimal,
    pub from_address: String,
    pub slippage: Option<f64>,
    pub disable_estimate: Option<bool>,
    pub allow_partial_fill: Option<bool>,
    pub source: Option<String>,
}

/// Parameters for executing a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub from_token: String,
    pub to_token: String,
    pub amount: BigDecimal,
    pub from_address: String,
    pub slippage: f64,
    pub deadline: Option<u64>,
    pub referrer: Option<String>,
    pub fee: Option<f64>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
}

/// Protocol information used in the swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub name: String,
    pub part: f64,
    pub from_token_address: String,
    pub to_token_address: String,
}

/// Quote response from 1inch Swap API (simplified format for price data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    #[serde(rename = "fromToken", skip_serializing_if = "Option::is_none")]
    pub from_token: Option<Token>,
    #[serde(rename = "toToken", skip_serializing_if = "Option::is_none")]
    pub to_token: Option<Token>,
    #[serde(rename = "fromTokenAmount", skip_serializing_if = "Option::is_none")]
    pub from_amount: Option<String>,
    #[serde(rename = "toTokenAmount", skip_serializing_if = "Option::is_none")]
    pub to_amount: Option<String>,
    #[serde(rename = "dstAmount")]
    pub dst_amount: String, // This is what we actually get
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<Protocol>>,
    #[serde(rename = "estimatedGas", skip_serializing_if = "Option::is_none")]
    pub estimated_gas: Option<String>,
}

/// Quote response from 1inch Fusion+
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionQuote {
    pub from_token: Token,
    pub to_token: Token,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub protocols: Vec<Protocol>,
    pub estimated_gas: BigDecimal,
    pub gas_price: BigDecimal,
    pub quote_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Order creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub from_token_address: String,
    pub to_token_address: String,
    pub amount: String,
    pub from_address: String,
    pub slippage: f64,
    pub deadline: u64,
    pub quote_id: String,
}

/// Order creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order_hash: String,
    pub order: FusionOrder,
    pub quote_id: String,
    pub signature: Option<String>,
}

/// Fusion order details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionOrder {
    pub maker: String,
    pub maker_asset: String,
    pub taker_asset: String,
    pub making_amount: String,
    pub taking_amount: String,
    pub salt: String,
    pub receiver: String,
    pub deadline: u64,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled(BigDecimal), // Amount filled
    Filled,
    Expired,
    Cancelled,
    Failed,
}

impl OrderStatus {
    pub fn from_string(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "pending" => OrderStatus::Pending,
            "filled" => OrderStatus::Filled,
            "expired" => OrderStatus::Expired,
            "cancelled" => OrderStatus::Cancelled,
            "failed" => OrderStatus::Failed,
            _ => OrderStatus::Failed,
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(self, OrderStatus::Filled | OrderStatus::Expired | OrderStatus::Cancelled | OrderStatus::Failed)
    }

    pub fn to_string(&self) -> String {
        match self {
            OrderStatus::Pending => "pending".to_string(),
            OrderStatus::PartiallyFilled(_) => "partially_filled".to_string(),
            OrderStatus::Filled => "filled".to_string(),
            OrderStatus::Expired => "expired".to_string(),
            OrderStatus::Cancelled => "cancelled".to_string(),
            OrderStatus::Failed => "failed".to_string(),
        }
    }
}

/// Order status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusResponse {
    pub order_hash: String,
    pub status: OrderStatus,
    pub fills: Vec<Fill>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fill information for an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub tx_hash: String,
    pub amount: BigDecimal,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

/// Swap execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub order_hash: String,
    pub tx_hash: Option<String>,
    pub status: OrderStatus,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub actual_gas_used: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
}

/// Slippage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageConfig {
    pub max_slippage: f64,
    pub min_return_amount: BigDecimal,
    pub price_impact_threshold: f64,
    pub deadline: u64,
}

/// Network configuration for 1inch
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub chain_id: u64,
    pub name: String,
    pub native_token: String,
    pub block_time: u64, // seconds
    pub confirmation_blocks: u64,
}

impl NetworkConfig {
    pub fn ethereum() -> Self {
        Self {
            chain_id: 1,
            name: "Ethereum".to_string(),
            native_token: "ETH".to_string(),
            block_time: 12,
            confirmation_blocks: 12,
        }
    }

    pub fn bsc() -> Self {
        Self {
            chain_id: 56,
            name: "BSC".to_string(),
            native_token: "BNB".to_string(),
            block_time: 3,
            confirmation_blocks: 15,
        }
    }

    pub fn polygon() -> Self {
        Self {
            chain_id: 137,
            name: "Polygon".to_string(),
            native_token: "MATIC".to_string(),
            block_time: 2,
            confirmation_blocks: 20,
        }
    }

    pub fn sepolia() -> Self {
        Self {
            chain_id: 11155111,
            name: "Sepolia".to_string(),
            native_token: "ETH".to_string(),
            block_time: 12,
            confirmation_blocks: 3, // Faster for testnet
        }
    }
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_remaining: u64,
    pub reset_time: DateTime<Utc>,
    pub daily_limit: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_order_status_conversion() {
        assert_eq!(OrderStatus::from_string("pending"), OrderStatus::Pending);
        assert_eq!(OrderStatus::from_string("FILLED"), OrderStatus::Filled);
        assert_eq!(OrderStatus::from_string("expired"), OrderStatus::Expired);
        assert_eq!(OrderStatus::from_string("cancelled"), OrderStatus::Cancelled);
        assert_eq!(OrderStatus::from_string("unknown"), OrderStatus::Failed);
    }

    #[test]
    fn test_order_status_finality() {
        assert!(!OrderStatus::Pending.is_final());
        assert!(OrderStatus::Filled.is_final());
        assert!(OrderStatus::Expired.is_final());
        assert!(OrderStatus::Cancelled.is_final());
        assert!(OrderStatus::Failed.is_final());
    }

    #[test]
    fn test_network_configs() {
        let eth = NetworkConfig::ethereum();
        assert_eq!(eth.chain_id, ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(eth.name, "Ethereum");

        let bsc = NetworkConfig::bsc();
        assert_eq!(bsc.chain_id, ONEINCH_BSC_CHAIN_ID);
        assert_eq!(bsc.name, "BSC");

        let polygon = NetworkConfig::polygon();
        assert_eq!(polygon.chain_id, ONEINCH_POLYGON_CHAIN_ID);
        assert_eq!(polygon.name, "Polygon");

        let sepolia = NetworkConfig::sepolia();
        assert_eq!(sepolia.chain_id, ONEINCH_SEPOLIA_CHAIN_ID);
        assert_eq!(sepolia.name, "Sepolia");
    }

    #[test]
    fn test_quote_params_serialization() {
        let params = QuoteParams {
            from_token: "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D".to_string(),
            to_token: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: BigDecimal::from(1000),
            from_address: "0x1234567890123456789012345678901234567890".to_string(),
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge".to_string()),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: QuoteParams = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(params.from_token, deserialized.from_token);
        assert_eq!(params.to_token, deserialized.to_token);
        assert_eq!(params.slippage, deserialized.slippage);
    }
}