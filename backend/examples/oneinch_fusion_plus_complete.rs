use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

// Structures for Fusion+ API

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
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
pub struct QuoteResponse {
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
pub struct BuildOrderResponse {
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
pub struct SubmitOrderRequest {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveOrdersRequest {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    #[serde(rename = "srcChain")]
    pub src_chain: Option<u64>,
    #[serde(rename = "dstChain")]
    pub dst_chain: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveOrdersResponse {
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

pub struct FusionPlusClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl FusionPlusClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("ONEINCH_API_KEY")
            .map_err(|_| "ONEINCH_API_KEY environment variable not set")?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.1inch.dev/fusion-plus".to_string(),
        })
    }

    // Get quote for cross-chain swap
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/quoter/v1.0/quote/receive", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("srcChain", request.src_chain.to_string());
        params.insert("dstChain", request.dst_chain.to_string());
        params.insert("srcTokenAddress", request.src_token_address);
        params.insert("dstTokenAddress", request.dst_token_address);
        params.insert("amount", request.amount);
        params.insert("walletAddress", request.wallet_address);
        params.insert("enableEstimate", request.enable_estimate.to_string());
        
        if let Some(fee) = request.fee {
            params.insert("fee", fee.to_string());
        }
        if let Some(is_permit2) = request.is_permit2 {
            params.insert("isPermit2", is_permit2);
        }
        if let Some(permit) = request.permit {
            params.insert("permit", permit);
        }

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let quote: QuoteResponse = response.json().await?;
        Ok(quote)
    }

    // Get quote with custom settings
    pub async fn get_quote_with_preset(&self, request: Value) -> Result<QuoteResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/quoter/v1.0/quote/preset", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let quote: QuoteResponse = response.json().await?;
        Ok(quote)
    }

    // Build order based on quote
    pub async fn build_order(&self, quote: &QuoteResponse, preset: &str, wallet_address: &str) -> Result<BuildOrderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/quoter/v1.0/quote/build", self.base_url);
        
        // Create full request object according to documentation
        let build_request = json!({
            "quote": quote,
            "secretsHashList": ["0x315b47a8c3780434b153667588db4ca628526e20000000000000000000000000"]
        });
        
        // Add query parameters according to documentation
        let mut params = HashMap::new();
        params.insert("walletAddress", wallet_address);
        params.insert("preset", preset);
        
        // Add required parameters from quote
        if let Some(quote_id_str) = quote.quote_id.as_str() {
            // Extract parameters from original quote request
            params.insert("srcChain", "1"); // Ethereum
            params.insert("dstChain", "137"); // Polygon  
            params.insert("srcTokenAddress", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"); // WETH
            params.insert("dstTokenAddress", "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"); // USDC
            params.insert("amount", "100000000000000000"); // 0.1 ETH
        }
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .query(&params)
            .json(&build_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        // Log response for debugging
        let response_text = response.text().await?;
        println!("Build order response: {}", response_text);
        
        let build_response: BuildOrderResponse = serde_json::from_str(&response_text)?;
        Ok(build_response)
    }

    // Submit order to relayer
    pub async fn submit_order(&self, request: SubmitOrderRequest) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/relayer/v1.0/submit", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        Ok(())
    }

    // Get active orders
    pub async fn get_active_orders(&self, request: ActiveOrdersRequest) -> Result<ActiveOrdersResponse, Box<dyn std::error::Error>> {
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let orders: ActiveOrdersResponse = response.json().await?;
        Ok(orders)
    }

    // Get order by hash
    pub async fn get_order_by_hash(&self, order_hash: &str) -> Result<ActiveOrder, Box<dyn std::error::Error>> {
        let url = format!("{}/orders/v1.0/order/{}", self.base_url, order_hash);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let order: ActiveOrder = response.json().await?;
        Ok(order)
    }

    // Get escrow factory address
    pub async fn get_escrow_factory(&self, chain_id: u64) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/quoter/v1.0/escrow-factory/{}", self.base_url, chain_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let result: Value = response.json().await?;
        Ok(result["address"].as_str().unwrap_or("").to_string())
    }
}

// Example of using the complete Fusion+ API
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    println!("🚀 1inch Fusion+ Complete Integration Demo");
    
    let client = FusionPlusClient::new()?;
    
    // Constants for testing
    const ETHEREUM_CHAIN_ID: u64 = 1;
    const POLYGON_CHAIN_ID: u64 = 137;
    const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const USDC_POLYGON_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
    const TEST_WALLET: &str = "0x742d35Cc6634C0532925a3b8D0C9e3e0C0e0e0e0";
    const AMOUNT: &str = "100000000000000000"; // 0.1 ETH
    
    println!("\n📊 Step 1: Getting quote for cross-chain swap");
    println!("From: ETH on Ethereum -> USDC on Polygon");
    
    let quote_request = QuoteRequest {
        src_chain: ETHEREUM_CHAIN_ID,
        dst_chain: POLYGON_CHAIN_ID,
        src_token_address: WETH_ADDRESS.to_string(),
        dst_token_address: USDC_POLYGON_ADDRESS.to_string(),
        amount: AMOUNT.to_string(),
        wallet_address: TEST_WALLET.to_string(),
        enable_estimate: true,
        fee: Some(0),
        is_permit2: None,
        permit: None,
    };
    
    match client.get_quote(quote_request).await {
        Ok(quote) => {
            println!("✅ Quote received successfully!");
            println!("   Source amount: {} WETH", quote.src_token_amount);
            println!("   Destination amount: {} USDC", quote.dst_token_amount);
            println!("   Recommended preset: {}", quote.recommended_preset);
            println!("   Quote ID: {:?}", quote.quote_id);
            
            // Show available presets
            println!("\n📋 Available presets:");
            println!("   Fast: {} seconds auction", quote.presets.fast.auction_duration);
            println!("   Medium: {} seconds auction", quote.presets.medium.auction_duration);
            println!("   Slow: {} seconds auction", quote.presets.slow.auction_duration);
            
            // Show security information
            println!("\n🔒 Security deposits:");
            println!("   Source chain: {} ETH", quote.src_safety_deposit);
            println!("   Destination chain: {} MATIC", quote.dst_safety_deposit);
            
            // Show time locks
            println!("\n⏰ Time locks:");
            println!("   Source withdrawal: {} seconds", quote.time_locks.src_withdrawal);
            println!("   Destination withdrawal: {} seconds", quote.time_locks.dst_withdrawal);
            
            // If there is a quote_id, try to build an order
            if let Some(quote_id_str) = quote.quote_id.as_str() {
                println!("\n🔨 Step 2: Building order from quote");
                
                match client.build_order(&quote, &quote.recommended_preset, TEST_WALLET).await {
                    Ok(build_response) => {
                        println!("✅ Order built successfully!");
                        println!("   Order salt: {}", build_response.order.salt);
                        println!("   Maker: {}", build_response.order.maker);
                        println!("   Making amount: {}", build_response.order.making_amount);
                        println!("   Taking amount: {}", build_response.order.taking_amount);
                        println!("   Secret hashes count: {}", build_response.secret_hashes.len());
                        
                        // Usually, an order needs to be signed with a private key before submission
                        println!("\n⚠️  Note: Order needs to be signed with private key before submission");
                        println!("   This demo skips the signing step for security reasons");
                    }
                    Err(e) => {
                        println!("❌ Failed to build order: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get quote: {}", e);
        }
    }
    
    println!("\n📋 Step 3: Getting active orders");
    
    let active_orders_request = ActiveOrdersRequest {
        page: Some(1),
        limit: Some(10),
        src_chain: Some(ETHEREUM_CHAIN_ID),
        dst_chain: Some(POLYGON_CHAIN_ID),
    };
    
    match client.get_active_orders(active_orders_request).await {
        Ok(orders) => {
            println!("✅ Active orders retrieved successfully!");
            println!("   Total items: {}", orders.meta.total_items);
            println!("   Current page: {}", orders.meta.current_page);
            println!("   Total pages: {}", orders.meta.total_pages);
            println!("   Orders on this page: {}", orders.items.len());
            
            for (i, order) in orders.items.iter().enumerate() {
                println!("\n   Order #{}: {}", i + 1, order.order_hash);
                println!("     Maker: {}", order.order.maker);
                println!("     Making amount: {}", order.order.making_amount);
                println!("     Remaining: {}", order.remaining_maker_amount);
                println!("     Auction start: {}", order.auction_start_date);
                println!("     Auction end: {}", order.auction_end_date);
            }
        }
        Err(e) => {
            println!("❌ Failed to get active orders: {}", e);
        }
    }
    
    println!("\n🏭 Step 4: Getting escrow factory addresses");
    
    match client.get_escrow_factory(ETHEREUM_CHAIN_ID).await {
        Ok(factory_address) => {
            println!("✅ Ethereum escrow factory: {}", factory_address);
        }
        Err(e) => {
            println!("❌ Failed to get Ethereum escrow factory: {}", e);
        }
    }
    
    match client.get_escrow_factory(POLYGON_CHAIN_ID).await {
        Ok(factory_address) => {
            println!("✅ Polygon escrow factory: {}", factory_address);
        }
        Err(e) => {
            println!("❌ Failed to get Polygon escrow factory: {}", e);
        }
    }
    
    println!("\n🎯 Fusion+ Integration Summary:");
    println!("✅ Quote API - Get cross-chain swap quotes");
    println!("✅ Build API - Build orders from quotes");
    println!("✅ Orders API - Query active orders");
    println!("✅ Relayer API - Submit orders (signing required)");
    println!("✅ Escrow API - Get factory addresses");
    
    println!("\n📚 Next steps for production:");
    println!("1. Implement order signing with private keys");
    println!("2. Add order monitoring and status tracking");
    println!("3. Implement secret submission for order completion");
    println!("4. Add error handling and retry logic");
    println!("5. Implement order cancellation and recovery");
    
    Ok(())
}