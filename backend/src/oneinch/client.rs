// src/oneinch/client.rs - HTTP client for 1inch Fusion+ API

use super::types::*;
use crate::constants::*;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// HTTP client for 1inch Fusion+ API
pub struct FusionClient {
    client: Client,
    api_key: String,
    base_url: String,
    chain_id: u64,
    max_retries: u32,
    retry_delay: Duration,
}

impl FusionClient {
    /// Create new FusionClient instance
    pub fn new(api_key: String, chain_id: u64) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .expect("Invalid API key format"),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(ONEINCH_FUSION_TIMEOUT_SEC))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: ONEINCH_FUSION_API_BASE.to_string(),
            chain_id,
            max_retries: ONEINCH_FUSION_MAX_RETRIES,
            retry_delay: Duration::from_millis(ONEINCH_FUSION_RETRY_DELAY_MS),
        }
    }

    /// Get quote for token swap
    pub async fn get_quote(&self, params: &QuoteParams) -> Result<FusionQuote, OneinchError> {
        let url = format!("{}/quote", self.base_url);
        
        let mut request_body = serde_json::json!({
            "fromTokenAddress": params.from_token,
            "toTokenAddress": params.to_token,
            "amount": params.amount.to_string(),
            "walletAddress": params.from_address,
            "source": params.source.as_ref().unwrap_or(&ONEINCH_DEFAULT_SOURCE.to_string())
        });

        if let Some(slippage) = params.slippage {
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

        let response = self.execute_with_retry(|| {
            self.client.post(&url).json(&request_body).send()
        }).await?;

        self.handle_response(response).await
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

    /// Get supported tokens for current chain
    pub async fn get_tokens(&self) -> Result<Vec<Token>, OneinchError> {
        let url = format!("{}/tokens/{}", self.base_url, self.chain_id);

        let response = self.execute_with_retry(|| {
            self.client.get(&url).send()
        }).await?;

        let tokens_map: std::collections::HashMap<String, Token> = self.handle_response(response).await?;
        Ok(tokens_map.into_values().collect())
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

    /// Handle API response and parse JSON
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, OneinchError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        
        if status.is_success() {
            let text = response.text().await?;
            serde_json::from_str(&text).map_err(|e| {
                OneinchError::SerializationError(e)
            })
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            match status.as_u16() {
                401 => Err(OneinchError::AuthenticationFailed),
                429 => Err(OneinchError::RateLimitExceeded),
                400..=499 => {
                    // Try to parse error details
                    if let Ok(error_data) = serde_json::from_str::<Value>(&error_text) {
                        let message = error_data["error"]["description"]
                            .as_str()
                            .or_else(|| error_data["message"].as_str())
                            .unwrap_or(&error_text);
                        
                        Err(OneinchError::ApiError {
                            code: status.as_u16(),
                            message: message.to_string(),
                        })
                    } else {
                        Err(OneinchError::ApiError {
                            code: status.as_u16(),
                            message: error_text,
                        })
                    }
                }
                _ => Err(OneinchError::ApiError {
                    code: status.as_u16(),
                    message: error_text,
                }),
            }
        }
    }

    /// Check if API is healthy
    pub async fn health_check(&self) -> Result<bool, OneinchError> {
        let url = format!("{}/healthcheck", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get current chain ID
    pub fn get_chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Update chain ID (for multi-chain support)
    pub fn set_chain_id(&mut self, chain_id: u64) {
        self.chain_id = chain_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn test_fusion_client_creation() {
        let client = FusionClient::new("test_api_key".to_string(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.base_url, ONEINCH_FUSION_API_BASE);
        assert_eq!(client.max_retries, ONEINCH_FUSION_MAX_RETRIES);
    }

    #[test]
    fn test_chain_id_update() {
        let mut client = FusionClient::new("test_api_key".to_string(), ONEINCH_ETHEREUM_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
        
        client.set_chain_id(ONEINCH_BSC_CHAIN_ID);
        assert_eq!(client.get_chain_id(), ONEINCH_BSC_CHAIN_ID);
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