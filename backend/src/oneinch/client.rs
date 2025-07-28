// src/oneinch/client.rs - HTTP client for 1inch Fusion+ API

use super::{types::*, validation::*};
use crate::constants::*;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// HTTP client for 1inch Fusion+ API
pub struct FusionClient {
    client: Client,
    api_key: String,
    base_url: String,
    chain_id: u64,
    max_retries: u32,
    retry_delay: Duration,
    validator: OneinchApiKeyValidator,
}

impl FusionClient {
    /// Create new FusionClient instance with proper API validation
    pub fn new(api_key: String, chain_id: u64) -> Self {
        let validator = OneinchApiKeyValidator::new();
        
        // Validate API key format using our validator
        match validator.validate_key_format(&api_key) {
            Ok(false) => {
                warn!("⚠️  API key appears to be a test/placeholder key: '{}'", 
                    if api_key.len() > 10 { format!("{}...", &api_key[..10]) } else { api_key.clone() });
                warn!("⚠️  1inch API requests may fail with test key!");
            },
            Err(e) => {
                warn!("⚠️  API key validation failed: {}", e);
                warn!("⚠️  1inch API requests may fail!");
            },
            Ok(true) => {
                if !validator.looks_realistic(&api_key) {
                    warn!("⚠️  API key format is valid but looks unrealistic");
                }
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .expect("Invalid API key format"),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("User-Agent", HeaderValue::from_static("KEMBridge/1.0"));

        let client = Client::builder()
            .timeout(Duration::from_secs(ONEINCH_FUSION_TIMEOUT_SEC))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        // Use proper 1inch Fusion API URL with chain ID
        let base_url = format!("{}/v1.0/{}", ONEINCH_FUSION_API_BASE, chain_id);

        Self {
            client,
            api_key,
            base_url,
            chain_id,
            max_retries: ONEINCH_FUSION_MAX_RETRIES,
            retry_delay: Duration::from_millis(ONEINCH_FUSION_RETRY_DELAY_MS),
            validator,
        }
    }

    /// Get quote for token swap using proper 1inch Fusion API
    pub async fn get_quote(&self, params: &QuoteParams) -> Result<FusionQuote, OneinchError> {
        // Use proper 1inch Fusion quote endpoint
        let url = format!("{}/quote", self.base_url);
        
        // Validate required parameters
        if params.from_token.is_empty() || params.to_token.is_empty() {
            return Err(OneinchError::InvalidParams("Token addresses cannot be empty".to_string()));
        }
        
        if params.amount <= bigdecimal::BigDecimal::from(0) {
            return Err(OneinchError::InvalidParams("Amount must be greater than 0".to_string()));
        }

        // Build request according to 1inch Fusion API spec
        let mut request_body = serde_json::json!({
            "fromTokenAddress": params.from_token,
            "toTokenAddress": params.to_token,
            "amount": params.amount.to_string(),
            "walletAddress": params.from_address,
            "source": params.source.as_ref().unwrap_or(&ONEINCH_DEFAULT_SOURCE.to_string())
        });

        // Add optional parameters if provided
        if let Some(slippage) = params.slippage {
            if slippage < 0.0 || slippage > 50.0 {
                return Err(OneinchError::InvalidParams("Slippage must be between 0 and 50%".to_string()));
            }
            request_body["slippage"] = serde_json::Value::Number(
                serde_json::Number::from_f64(slippage)
                    .ok_or_else(|| OneinchError::InvalidParams("Invalid slippage value".to_string()))?
            );
        }

        if let Some(disable_estimate) = params.disable_estimate {
            request_body["disableEstimate"] = serde_json::Value::Bool(disable_estimate);
        }

        if let Some(allow_partial_fill) = params.allow_partial_fill {
            request_body["allowPartialFill"] = serde_json::Value::Bool(allow_partial_fill);
        }

        info!("🔄 Requesting 1inch Fusion quote: {} {} -> {}", 
            params.amount, params.from_token, params.to_token);

        let response = self.execute_with_retry(|| {
            self.client.post(&url).json(&request_body).send()
        }).await?;

        let quote: FusionQuote = self.handle_response(response).await?;
        
        info!("✅ Received 1inch Fusion quote: {} -> {} (gas: {})", 
            quote.from_amount, quote.to_amount, quote.estimated_gas);
            
        Ok(quote)
    }

    /// Create swap order
    pub async fn create_order(&self, request: &CreateOrderRequest) -> Result<CreateOrderResponse, OneinchError> {
        let url = format!("{}/order", self.base_url);
        
        let request_body = serde_json::json!({
            "fromTokenAddress": request.from_token_address,
            "toTokenAddress": request.to_token_address,
            "amount": request.amount,
            "fromAddress": request.from_address,
            "slippage": request.slippage,
            "deadline": request.deadline,
            "quoteId": request.quote_id
        });

        let response = self.execute_with_retry(|| {
            self.client.post(&url).json(&request_body).send()
        }).await?;

        self.handle_response(response).await
    }

    /// Submit order for execution
    pub async fn submit_order(&self, order: &FusionOrder, quote_id: &str, signature: &str) -> Result<SwapResult, OneinchError> {
        let url = format!("{}/order/submit", self.base_url);
        
        let request_body = serde_json::json!({
            "order": order,
            "quoteId": quote_id,
            "signature": signature
        });

        let response = self.execute_with_retry(|| {
            self.client.post(&url).json(&request_body).send()
        }).await?;

        self.handle_response(response).await
    }

    /// Submit a signed order to 1inch for execution
    pub async fn submit_signed_order(&self, order_hash: &str, signature: &str) -> Result<SwapResult, OneinchError> {
        let url = format!("{}/order/submit", self.base_url);
        
        let payload = serde_json::json!({
            "orderHash": order_hash,
            "signature": signature
        });

        let response = self.execute_with_retry(|| {
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&payload)
                .send()
        }).await?;

        self.handle_response(response).await
    }

    /// Get order status
    pub async fn get_order_status(&self, order_hash: &str) -> Result<OrderStatusResponse, OneinchError> {
        let url = format!("{}/order/{}", self.base_url, order_hash);

        let response = self.execute_with_retry(|| {
            self.client.get(&url).send()
        }).await?;

        self.handle_response(response).await
    }

    /// Get supported tokens for current chain from 1inch
    pub async fn get_tokens(&self) -> Result<Vec<Token>, OneinchError> {
        let url = format!("{}/tokens", self.base_url);

        info!("🔄 Fetching supported tokens from 1inch for chain {}", self.chain_id);

        let response = self.execute_with_retry(|| {
            self.client.get(&url).send()
        }).await?;

        // 1inch returns tokens as a map with address as key
        let tokens_response: Value = self.handle_response(response).await?;
        
        let mut tokens = Vec::new();
        
        if let Some(tokens_map) = tokens_response.as_object() {
            for (address, token_data) in tokens_map {
                if let Ok(token) = serde_json::from_value::<Token>(token_data.clone()) {
                    tokens.push(token);
                } else {
                    warn!("⚠️  Failed to parse token data for address: {}", address);
                }
            }
        }

        info!("✅ Fetched {} supported tokens from 1inch", tokens.len());
        Ok(tokens)
    }

    /// Get allowance for token
    pub async fn get_allowance(&self, token_address: &str, owner_address: &str, spender_address: &str) -> Result<String, OneinchError> {
        let url = format!("{}/approve/allowance", self.base_url);
        
        let params = [
            ("tokenAddress", token_address),
            ("walletAddress", owner_address),
            ("spenderAddress", spender_address),
        ];

        let response = self.execute_with_retry(|| {
            self.client.get(&url).query(&params).send()
        }).await?;

        let result: Value = self.handle_response(response).await?;
        Ok(result["allowance"].as_str().unwrap_or("0").to_string())
    }

    /// Get approve transaction data
    pub async fn get_approve_transaction(&self, token_address: &str, amount: Option<&str>) -> Result<Value, OneinchError> {
        let url = format!("{}/approve/transaction", self.base_url);
        
        let mut params = vec![("tokenAddress", token_address)];
        if let Some(amount) = amount {
            params.push(("amount", amount));
        }

        let response = self.execute_with_retry(|| {
            self.client.get(&url).query(&params).send()
        }).await?;

        self.handle_response(response).await
    }

    /// Execute HTTP request with retry logic
    async fn execute_with_retry<F, Fut>(&self, request_fn: F) -> Result<reqwest::Response, OneinchError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match request_fn().await {
                Ok(response) => {
                    if response.status().is_success() || response.status().as_u16() < 500 {
                        return Ok(response);
                    }
                    // Server error, should retry
                    last_error = Some(OneinchError::ApiError {
                        code: response.status().as_u16(),
                        message: format!("Server error on attempt {}", attempt + 1),
                    });
                }
                Err(e) => {
                    last_error = Some(OneinchError::HttpError(e));
                }
            }

            if attempt < self.max_retries {
                sleep(self.retry_delay * (attempt + 1) as u32).await;
            }
        }

        Err(last_error.unwrap())
    }

    /// Handle API response and parse JSON with detailed error reporting
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, OneinchError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let url = response.url().clone();
        
        if status.is_success() {
            let text = response.text().await?;
            
            // Log successful response for debugging
            info!("✅ 1inch API success: {} - {} bytes", url, text.len());
            
            serde_json::from_str(&text).map_err(|e| {
                warn!("❌ Failed to parse 1inch response: {} - Error: {}", text, e);
                OneinchError::SerializationError(e)
            })
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Log all API errors for debugging
            warn!("❌ 1inch API error: {} - Status: {} - Response: {}", url, status, error_text);
            
            match status.as_u16() {
                401 => {
                    warn!("🔑 1inch API authentication failed - check your API key!");
                    Err(OneinchError::AuthenticationFailed)
                },
                403 => {
                    warn!("🚫 1inch API access forbidden - check API key permissions!");
                    Err(OneinchError::ApiError {
                        code: 403,
                        message: "Access forbidden - check API key permissions".to_string(),
                    })
                },
                429 => {
                    warn!("⏰ 1inch API rate limit exceeded - backing off!");
                    Err(OneinchError::RateLimitExceeded)
                },
                400..=499 => {
                    // Try to parse detailed error information
                    if let Ok(error_data) = serde_json::from_str::<Value>(&error_text) {
                        let message = error_data["error"]["description"]
                            .as_str()
                            .or_else(|| error_data["message"].as_str())
                            .or_else(|| error_data["error"].as_str())
                            .unwrap_or(&error_text);
                        
                        warn!("🔍 1inch API client error: {}", message);
                        
                        Err(OneinchError::ApiError {
                            code: status.as_u16(),
                            message: format!("1inch API error: {}", message),
                        })
                    } else {
                        Err(OneinchError::ApiError {
                            code: status.as_u16(),
                            message: format!("1inch API error: {}", error_text),
                        })
                    }
                }
                500..=599 => {
                    warn!("🔥 1inch API server error - will retry if possible");
                    Err(OneinchError::ApiError {
                        code: status.as_u16(),
                        message: format!("1inch server error: {}", error_text),
                    })
                }
                _ => Err(OneinchError::ApiError {
                    code: status.as_u16(),
                    message: format!("1inch API unexpected error: {}", error_text),
                }),
            }
        }
    }

    /// Check if 1inch Fusion API is healthy and accessible
    pub async fn health_check(&self) -> Result<bool, OneinchError> {
        // Try to get supported tokens as a health check
        // This is more reliable than a dedicated healthcheck endpoint
        let url = format!("{}/tokens", self.base_url);
        
        info!("🔍 Checking 1inch API health: {}", url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    info!("✅ 1inch API is healthy");
                } else {
                    warn!("❌ 1inch API health check failed: {}", response.status());
                }
                Ok(is_healthy)
            },
            Err(e) => {
                warn!("❌ 1inch API health check failed with error: {}", e);
                Ok(false)
            }
        }
    }

    /// Get current chain ID
    pub fn get_chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Update chain ID (for multi-chain support)
    pub fn set_chain_id(&mut self, chain_id: u64) {
        self.chain_id = chain_id;
        // Update base URL with new chain ID
        self.base_url = format!("{}/v1.0/{}", ONEINCH_FUSION_API_BASE, chain_id);
    }

    /// Get real-time liquidity information for a token pair
    pub async fn get_liquidity_info(&self, from_token: &str, to_token: &str) -> Result<Value, OneinchError> {
        // Use a small quote to get liquidity information
        let small_amount = bigdecimal::BigDecimal::from(1000000); // Small amount for liquidity check
        
        let params = QuoteParams {
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            amount: small_amount,
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
            slippage: Some(0.1), // Very low slippage for liquidity check
            disable_estimate: Some(true), // Don't need gas estimation
            allow_partial_fill: Some(false),
            source: Some("kembridge-liquidity-check".to_string()),
        };

        match self.get_quote(&params).await {
            Ok(quote) => {
                // Extract liquidity information from quote
                Ok(serde_json::json!({
                    "available": true,
                    "protocols": quote.protocols,
                    "estimated_gas": quote.estimated_gas,
                    "from_amount": quote.from_amount,
                    "to_amount": quote.to_amount,
                    "liquidity_score": self.calculate_liquidity_score(&quote)
                }))
            },
            Err(e) => {
                warn!("❌ Failed to get liquidity info for {}/{}: {}", from_token, to_token, e);
                Ok(serde_json::json!({
                    "available": false,
                    "error": e.to_string()
                }))
            }
        }
    }

    /// Calculate liquidity score based on quote data
    fn calculate_liquidity_score(&self, quote: &FusionQuote) -> f64 {
        // Simple liquidity scoring based on:
        // 1. Number of protocols (more = better liquidity)
        // 2. Gas efficiency (lower gas = better liquidity)
        // 3. Price efficiency (better rates = better liquidity)
        
        let protocol_score = (quote.protocols.len() as f64).min(10.0) / 10.0; // 0-1 scale
        
        let gas_amount = quote.estimated_gas.to_string().parse::<f64>().unwrap_or(1000000.0);
        let gas_score = (1.0 - (gas_amount / 1000000.0).min(1.0)).max(0.0); // Lower gas = higher score
        
        // Weighted average
        (protocol_score * 0.6) + (gas_score * 0.4)
    }

    /// Validate API key by making a test request
    pub async fn validate_api_key(&self) -> Result<bool, OneinchError> {
        info!("🔑 Validating 1inch API key...");
        
        // First check format
        match self.validator.validate_key_format(&self.api_key) {
            Ok(false) => {
                warn!("❌ API key format is invalid or appears to be a test key");
                return Ok(false);
            },
            Err(e) => {
                warn!("❌ API key format validation failed: {}", e);
                return Ok(false);
            },
            Ok(true) => {
                // Format is valid, check remotely
            }
        }
        
        match self.get_tokens().await {
            Ok(_) => {
                info!("✅ 1inch API key is valid");
                Ok(true)
            },
            Err(OneinchError::AuthenticationFailed) => {
                warn!("❌ 1inch API key is invalid or expired");
                Ok(false)
            },
            Err(e) => {
                warn!("⚠️  Could not validate 1inch API key due to error: {}", e);
                Err(e)
            }
        }
    }

    /// Get detailed API key validation information
    pub async fn get_detailed_validation(&self) -> Result<ApiKeyValidationResult, OneinchError> {
        info!("🔍 Getting detailed API key validation...");
        
        match self.validator.validate_key_comprehensive(&self.api_key).await {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("❌ Detailed validation failed: {}", e);
                Err(OneinchError::OperationFailed(format!("Validation error: {}", e)))
            }
        }
    }
}

impl SelfValidating for FusionClient {
    type ValidationError = OneinchError;

    fn validate_configuration(&self) -> Result<(), Self::ValidationError> {
        // Check API key
        if let Err(e) = self.validator.validate_key_format(&self.api_key) {
            return Err(OneinchError::InvalidParams(format!("Invalid API key: {}", e)));
        }

        // Check chain ID
        let supported_chains = [
            ONEINCH_ETHEREUM_CHAIN_ID,
            ONEINCH_BSC_CHAIN_ID,
            ONEINCH_POLYGON_CHAIN_ID,
            ONEINCH_AVALANCHE_CHAIN_ID,
            ONEINCH_ARBITRUM_CHAIN_ID,
            ONEINCH_OPTIMISM_CHAIN_ID,
            ONEINCH_SEPOLIA_CHAIN_ID,
            ONEINCH_BSC_TESTNET_CHAIN_ID,
            ONEINCH_POLYGON_MUMBAI_CHAIN_ID,
        ];

        if !supported_chains.contains(&self.chain_id) {
            return Err(OneinchError::InvalidParams(format!("Unsupported chain ID: {}", self.chain_id)));
        }

        // Check basic settings
        if self.max_retries == 0 {
            return Err(OneinchError::InvalidParams("Max retries cannot be zero".to_string()));
        }

        if self.retry_delay.is_zero() {
            return Err(OneinchError::InvalidParams("Retry delay cannot be zero".to_string()));
        }

        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn test_fusion_client_creation() {
        let client = FusionClient::new("valid_api_key_12345".to_string(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.base_url, format!("{}/v1.0/{}", ONEINCH_FUSION_API_BASE, ONEINCH_ETHEREUM_CHAIN_ID));
        assert_eq!(client.max_retries, ONEINCH_FUSION_MAX_RETRIES);
    }

    #[test]
    fn test_chain_id_update() {
        let mut client = FusionClient::new("valid_api_key_12345".to_string(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
        
        client.set_chain_id(ONEINCH_BSC_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_BSC_CHAIN_ID);
        assert_eq!(client.base_url, format!("{}/v1.0/{}", ONEINCH_FUSION_API_BASE, ONEINCH_BSC_CHAIN_ID));
    }

    #[tokio::test]
    async fn test_quote_params_serialization() {
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

        // Test that we can serialize the parameters correctly
        let serialized = serde_json::to_value(&params).unwrap();
        assert!(serialized["from_token"].is_string());
        assert!(serialized["amount"].is_string());
        assert!(serialized["slippage"].is_number());
    }
}