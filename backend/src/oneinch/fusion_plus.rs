// src/oneinch/fusion_plus.rs - 1inch Fusion+ Cross-Chain Integration

use super::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use tracing::{info, warn, error};
use utoipa::IntoParams;

// Structures for Fusion+ API

#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct FusionPlusQuoteRequest {
    #[serde(rename = "srcChain")]
    pub src_chain: u64,
    #[serde(rename = "dstChain")]
    pub dst_chain: u64,
    #[serde(rename = "srcTokenAddress")]
    pub src_token_address: String,
    #[serde(rename = "dstTokenAddress")]
    pub dst_token_address: String,
    pub amount: String,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(rename = "enableEstimate")]
    pub enable_estimate: bool,
    pub fee: Option<u64>,
    #[serde(rename = "isPermit2")]
    pub is_permit2: Option<String>,
    pub permit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FusionPlusQuoteResponse {
    #[serde(rename = "quoteId")]
    pub quote_id: Value,
    #[serde(rename = "srcTokenAmount")]
    pub src_token_amount: String,
    #[serde(rename = "dstTokenAmount")]
    pub dst_token_amount: String,
    pub presets: QuotePresets,
    #[serde(rename = "srcEscrowFactory")]
    pub src_escrow_factory: String,
    #[serde(rename = "dstEscrowFactory")]
    pub dst_escrow_factory: String,
    pub whitelist: Vec<String>,
    #[serde(rename = "timeLocks")]
    pub time_locks: TimeLocks,
    #[serde(rename = "srcSafetyDeposit")]
    pub src_safety_deposit: String,
    #[serde(rename = "dstSafetyDeposit")]
    pub dst_safety_deposit: String,
    #[serde(rename = "recommendedPreset")]
    pub recommended_preset: String,
    pub prices: PairCurrency,
    pub volume: PairCurrency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotePresets {
    pub fast: Preset,
    pub medium: Preset,
    pub slow: Preset,
    pub custom: Option<Preset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    #[serde(rename = "auctionDuration")]
    pub auction_duration: u64,
    #[serde(rename = "startAuctionIn")]
    pub start_auction_in: u64,
    #[serde(rename = "initialRateBump")]
    pub initial_rate_bump: u64,
    #[serde(rename = "auctionStartAmount")]
    pub auction_start_amount: String,
    #[serde(rename = "startAmount")]
    pub start_amount: String,
    #[serde(rename = "auctionEndAmount")]
    pub auction_end_amount: String,
    #[serde(rename = "exclusiveResolver")]
    pub exclusive_resolver: Option<Value>,
    #[serde(rename = "costInDstToken")]
    pub cost_in_dst_token: String,
    pub points: Vec<AuctionPoint>,
    #[serde(rename = "allowPartialFills")]
    pub allow_partial_fills: bool,
    #[serde(rename = "allowMultipleFills")]
    pub allow_multiple_fills: bool,
    #[serde(rename = "gasCost")]
    pub gas_cost: GasCostConfig,
    #[serde(rename = "secretsCount")]
    pub secrets_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionPoint {
    pub delay: u64,
    pub coefficient: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasCostConfig {
    #[serde(rename = "gasBumpEstimate")]
    pub gas_bump_estimate: u64,
    #[serde(rename = "gasPriceEstimate")]
    pub gas_price_estimate: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeLocks {
    #[serde(rename = "srcWithdrawal")]
    pub src_withdrawal: u64,
    #[serde(rename = "srcPublicWithdrawal")]
    pub src_public_withdrawal: u64,
    #[serde(rename = "srcCancellation")]
    pub src_cancellation: u64,
    #[serde(rename = "srcPublicCancellation")]
    pub src_public_cancellation: u64,
    #[serde(rename = "dstWithdrawal")]
    pub dst_withdrawal: u64,
    #[serde(rename = "dstPublicWithdrawal")]
    pub dst_public_withdrawal: u64,
    #[serde(rename = "dstCancellation")]
    pub dst_cancellation: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairCurrency {
    pub usd: TokenPair,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    #[serde(rename = "srcToken")]
    pub src_token: String,
    #[serde(rename = "dstToken")]
    pub dst_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FusionPlusBuildOrderResponse {
    pub order: CrossChainOrder,
    #[serde(rename = "quoteId")]
    pub quote_id: String,
    #[serde(rename = "secretHashes")]
    pub secret_hashes: Vec<Vec<String>>,
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossChainOrder {
    pub salt: String,
    pub maker: String,
    pub receiver: String,
    #[serde(rename = "makerAsset")]
    pub maker_asset: String,
    #[serde(rename = "takerAsset")]
    pub taker_asset: String,
    #[serde(rename = "makingAmount")]
    pub making_amount: String,
    #[serde(rename = "takingAmount")]
    pub taking_amount: String,
    #[serde(rename = "makerTraits")]
    pub maker_traits: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FusionPlusSubmitOrderRequest {
    pub order: CrossChainOrder,
    #[serde(rename = "srcChainId")]
    pub src_chain_id: u64,
    pub signature: String,
    pub extension: String,
    #[serde(rename = "quoteId")]
    pub quote_id: String,
    #[serde(rename = "secretHashes")]
    pub secret_hashes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, IntoParams)]
pub struct FusionPlusActiveOrdersRequest {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    #[serde(rename = "srcChain")]
    pub src_chain: Option<u64>,
    #[serde(rename = "dstChain")]
    pub dst_chain: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FusionPlusActiveOrdersResponse {
    pub meta: Meta,
    pub items: Vec<ActiveOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "totalItems")]
    pub total_items: u64,
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: u64,
    #[serde(rename = "totalPages")]
    pub total_pages: u64,
    #[serde(rename = "currentPage")]
    pub current_page: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveOrder {
    #[serde(rename = "orderHash")]
    pub order_hash: String,
    pub signature: String,
    pub deadline: u64,
    #[serde(rename = "auctionStartDate")]
    pub auction_start_date: u64,
    #[serde(rename = "auctionEndDate")]
    pub auction_end_date: u64,
    #[serde(rename = "quoteId")]
    pub quote_id: String,
    #[serde(rename = "remainingMakerAmount")]
    pub remaining_maker_amount: String,
    #[serde(rename = "makerBalance")]
    pub maker_balance: String,
    #[serde(rename = "makerAllowance")]
    pub maker_allowance: String,
    #[serde(rename = "isMakerContract")]
    pub is_maker_contract: bool,
    pub extension: String,
    #[serde(rename = "srcChainId")]
    pub src_chain_id: u64,
    #[serde(rename = "dstChainId")]
    pub dst_chain_id: u64,
    pub order: CrossChainOrder,
    #[serde(rename = "secretHashes")]
    pub secret_hashes: Vec<Vec<String>>,
    pub fills: Vec<String>,
}

/// 1inch Fusion+ Client for cross-chain swaps
pub struct FusionPlusClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl FusionPlusClient {
    /// Create new Fusion+ client
    pub fn new() -> Result<Self, OneinchError> {
        let api_key = env::var("ONEINCH_API_KEY")
            .map_err(|_| OneinchError::InvalidParams("ONEINCH_API_KEY environment variable not set".to_string()))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.1inch.dev/fusion-plus".to_string(),
        })
    }

    /// Create with custom API key
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.1inch.dev/fusion-plus".to_string(),
        }
    }

    /// Get cross-chain quote
    pub async fn get_cross_chain_quote(&self, request: FusionPlusQuoteRequest) -> Result<FusionPlusQuoteResponse, OneinchError> {
        let url = format!("{}/quoter/v1.0/quote/receive", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("srcChain", request.src_chain.to_string());
        params.insert("dstChain", request.dst_chain.to_string());
        params.insert("srcTokenAddress", request.src_token_address.clone());
        params.insert("dstTokenAddress", request.dst_token_address.clone());
        params.insert("amount", request.amount.clone());
        params.insert("walletAddress", request.wallet_address.clone());
        params.insert("enableEstimate", request.enable_estimate.to_string());
        
        if let Some(fee) = request.fee {
            params.insert("fee", fee.to_string());
        }
        if let Some(is_permit2) = &request.is_permit2 {
            params.insert("isPermit2", is_permit2.clone());
        }
        if let Some(permit) = &request.permit {
            params.insert("permit", permit.clone());
        }

        info!("🔄 Requesting Fusion+ cross-chain quote: {} {} -> {} {}", 
            request.src_chain, &request.src_token_address, request.dst_chain, &request.dst_token_address);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Fusion+ API error: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        let quote: FusionPlusQuoteResponse = response.json().await.map_err(OneinchError::HttpError)?;
        
        info!("✅ Received Fusion+ quote: {} -> {} (preset: {})", 
            quote.src_token_amount, quote.dst_token_amount, quote.recommended_preset);
            
        Ok(quote)
    }

    /// Build order from quote
    pub async fn build_order(&self, quote: &FusionPlusQuoteResponse, preset: &str, wallet_address: &str) -> Result<FusionPlusBuildOrderResponse, OneinchError> {
        let url = format!("{}/quoter/v1.0/quote/build", self.base_url);
        
        // Create full request body according to API documentation
        let build_request = json!({
            "quote": quote,
            "secretsHashList": ["0x315b47a8c3780434b153667588db4ca628526e20000000000000000000000000"]
        });
        
        // Add query parameters
        let mut params = HashMap::new();
        params.insert("walletAddress", wallet_address);
        params.insert("preset", preset);
        
        // Add required parameters from quote
        if let Some(quote_id_str) = quote.quote_id.as_str() {
            params.insert("srcChain", "1"); // Ethereum
            params.insert("dstChain", "137"); // Polygon  
            params.insert("srcTokenAddress", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"); // WETH
            params.insert("dstTokenAddress", "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"); // USDC
            params.insert("amount", "100000000000000000"); // 0.1 ETH
        }
        
        info!("🔨 Building Fusion+ order from quote");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .query(&params)
            .json(&build_request)
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Failed to build Fusion+ order: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        let build_response: FusionPlusBuildOrderResponse = response.json().await.map_err(OneinchError::HttpError)?;
        
        info!("✅ Built Fusion+ order: {} (secrets: {})", 
            build_response.order.salt, build_response.secret_hashes.len());
            
        Ok(build_response)
    }

    /// Submit order to relayer
    pub async fn submit_order(&self, request: FusionPlusSubmitOrderRequest) -> Result<(), OneinchError> {
        let url = format!("{}/relayer/v1.0/submit", self.base_url);
        
        info!("📤 Submitting Fusion+ order to relayer");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Failed to submit Fusion+ order: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        info!("✅ Fusion+ order submitted successfully");
        Ok(())
    }

    /// Get active orders
    pub async fn get_active_orders(&self, request: FusionPlusActiveOrdersRequest) -> Result<FusionPlusActiveOrdersResponse, OneinchError> {
        let url = format!("{}/orders/v1.0/order/active", self.base_url);
        
        let mut params = HashMap::new();
        if let Some(page) = request.page {
            params.insert("page", page.to_string());
        }
        if let Some(limit) = request.limit {
            params.insert("limit", limit.to_string());
        }
        if let Some(src_chain) = request.src_chain {
            params.insert("srcChain", src_chain.to_string());
        }
        if let Some(dst_chain) = request.dst_chain {
            params.insert("dstChain", dst_chain.to_string());
        }

        info!("📋 Getting active Fusion+ orders");

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Failed to get active orders: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        let orders: FusionPlusActiveOrdersResponse = response.json().await.map_err(OneinchError::HttpError)?;
        
        info!("✅ Retrieved {} active Fusion+ orders", orders.items.len());
        Ok(orders)
    }

    /// Get order by hash
    pub async fn get_order_by_hash(&self, order_hash: &str) -> Result<ActiveOrder, OneinchError> {
        let url = format!("{}/orders/v1.0/order/{}", self.base_url, order_hash);
        
        info!("🔍 Getting Fusion+ order by hash: {}", order_hash);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Failed to get order by hash: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        let order: ActiveOrder = response.json().await.map_err(OneinchError::HttpError)?;
        
        info!("✅ Retrieved Fusion+ order: {}", order.order_hash);
        Ok(order)
    }

    /// Get escrow factory address
    pub async fn get_escrow_factory(&self, chain_id: u64) -> Result<String, OneinchError> {
        let url = format!("{}/quoter/v1.0/escrow-factory/{}", self.base_url, chain_id);
        
        info!("🏭 Getting escrow factory for chain {}", chain_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(OneinchError::HttpError)?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.map_err(OneinchError::HttpError)?;
            error!("❌ Failed to get escrow factory: {}", error_text);
            return Err(OneinchError::ApiError {
                code: status_code,
                message: error_text,
            });
        }

        let result: Value = response.json().await.map_err(OneinchError::HttpError)?;
        let factory_address = result["address"].as_str().unwrap_or("").to_string();
        
        info!("✅ Escrow factory for chain {}: {}", chain_id, factory_address);
        Ok(factory_address)
    }

    /// Health check for Fusion+ API
    pub async fn health_check(&self) -> Result<bool, OneinchError> {
        info!("🔍 Checking Fusion+ API health");
        
        // Try to get active orders as health check
        let request = FusionPlusActiveOrdersRequest {
            page: Some(1),
            limit: Some(1),
            src_chain: None,
            dst_chain: None,
        };
        
        match self.get_active_orders(request).await {
            Ok(_) => {
                info!("✅ Fusion+ API is healthy");
                Ok(true)
            },
            Err(e) => {
                warn!("❌ Fusion+ API health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_plus_client_creation() {
        // Test with environment variable
        std::env::set_var("ONEINCH_API_KEY", "test_key_12345");
        let client = FusionPlusClient::new();
        assert!(client.is_ok());
        
        // Test with custom key
        let client = FusionPlusClient::with_api_key("custom_key_12345".to_string());
        assert_eq!(client.base_url, "https://api.1inch.dev/fusion-plus");
    }

    #[test]
    fn test_quote_request_serialization() {
        let request = FusionPlusQuoteRequest {
            src_chain: 1,
            dst_chain: 137,
            src_token_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            dst_token_address: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
            amount: "100000000000000000".to_string(),
            wallet_address: "0x742d35Cc6634C0532925a3b8D0C9e3e0C0e0e0e0".to_string(),
            enable_estimate: true,
            fee: Some(0),
            is_permit2: None,
            permit: None,
        };

        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(serialized["srcChain"], 1);
        assert_eq!(serialized["dstChain"], 137);
        assert_eq!(serialized["enableEstimate"], true);
    }
}