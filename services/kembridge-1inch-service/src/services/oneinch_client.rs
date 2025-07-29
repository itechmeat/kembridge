use crate::errors::{OneinchServiceError, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct OneinchClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    max_retries: u32,
}

impl OneinchClient {
    pub async fn new(
        base_url: String,
        api_key: Option<String>,
        timeout_ms: u64,
        max_retries: u32,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .user_agent("KEMBridge/1.0")
            .build()
            .map_err(|e| OneinchServiceError::internal(format!("Failed to create HTTP client: {}", e)))?;

        let client = Self {
            client,
            base_url,
            api_key,
            timeout: Duration::from_millis(timeout_ms),
            max_retries,
        };

        // Verify connection
        client.health_check().await?;

        Ok(client)
    }

    pub async fn get_quote(&self, request: &OneinchQuoteRequest) -> Result<OneinchQuoteResponse> {
        let url = format!("{}/v6.0/{}/quote", self.base_url, request.chain_id);
        
        let mut query_params = vec![
            ("src", request.src.clone()),
            ("dst", request.dst.clone()),
            ("amount", request.amount.clone()),
        ];

        if let Some(slippage) = &request.slippage {
            query_params.push(("slippage", slippage.clone()));
        }

        if let Some(from) = &request.from {
            query_params.push(("from", from.clone()));
        }

        self.execute_request::<OneinchQuoteResponse>(
            "GET", 
            &url, 
            Some(&query_params), 
            None::<&()>
        ).await
    }

    pub async fn get_swap_data(&self, request: &OneinchSwapRequest) -> Result<OneinchSwapResponse> {
        let url = format!("{}/v6.0/{}/swap", self.base_url, request.chain_id);
        
        let mut query_params = vec![
            ("src", request.src.clone()),
            ("dst", request.dst.clone()),
            ("amount", request.amount.clone()),
            ("from", request.from.clone()),
        ];

        if let Some(slippage) = &request.slippage {
            query_params.push(("slippage", slippage.clone()));
        }

        if let Some(disable_estimate) = request.disable_estimate {
            query_params.push(("disableEstimate", disable_estimate.to_string()));
        }

        self.execute_request::<OneinchSwapResponse>(
            "GET", 
            &url, 
            Some(&query_params), 
            None::<&()>
        ).await
    }

    pub async fn get_tokens(&self, chain_id: u64) -> Result<OneinchTokensResponse> {
        let url = format!("{}/v6.0/{}/tokens", self.base_url, chain_id);
        
        self.execute_request::<OneinchTokensResponse>(
            "GET", 
            &url, 
            None, 
            None::<&()>
        ).await
    }

    pub async fn get_protocols(&self, chain_id: u64) -> Result<OneinchProtocolsResponse> {
        let url = format!("{}/v6.0/{}/liquidity-sources", self.base_url, chain_id);
        
        self.execute_request::<OneinchProtocolsResponse>(
            "GET", 
            &url, 
            None, 
            None::<&()>
        ).await
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/v6.0/1/healthcheck", self.base_url);
        
        match self.execute_request::<serde_json::Value>("GET", &url, None, None::<&()>).await {
            Ok(_) => {
                info!("1inch API health check passed");
                Ok(())
            }
            Err(e) => {
                warn!("1inch API health check failed: {}", e);
                Err(e)
            }
        }
    }

    async fn execute_request<T>(
        &self,
        method: &str,
        url: &str,
        query_params: Option<&[(String, String)]>,
        body: Option<&impl Serialize>,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut retries = 0;
        let mut last_error = None;

        while retries <= self.max_retries {
            match self.make_request(method, url, query_params, body).await {
                Ok(response) => return self.handle_response(response).await,
                Err(e) => {
                    last_error = Some(e);
                    if retries < self.max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(retries)));
                        warn!("1inch API request failed, retrying in {:?}. Attempt {}/{}", 
                              delay, retries + 1, self.max_retries + 1);
                        tokio::time::sleep(delay).await;
                    }
                    retries += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| OneinchServiceError::internal("Request failed after all retries")))
    }

    async fn make_request(
        &self,
        method: &str,
        url: &str,
        query_params: Option<&[(String, String)]>,
        body: Option<&impl Serialize>,
    ) -> Result<Response> {
        let mut request_builder = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            _ => return Err(OneinchServiceError::internal(format!("Unsupported HTTP method: {}", method))),
        };

        // Add query parameters
        if let Some(params) = query_params {
            for (key, value) in params {
                request_builder = request_builder.query(&[(key, value)]);
            }
        }

        // Add authorization header
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add body if present
        if let Some(body_data) = body {
            request_builder = request_builder.json(body_data);
        }

        request_builder
            .send()
            .await
            .map_err(|e| {
                error!("Network error calling 1inch API: {}", e);
                if e.is_timeout() {
                    OneinchServiceError::NetworkTimeout {
                        operation: format!("{} {}", method, url),
                    }
                } else {
                    OneinchServiceError::oneinch_api(format!("Network error: {}", e))
                }
            })
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let url = response.url().clone();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| {
                    error!("Failed to deserialize 1inch response from {}: {}", url, e);
                    OneinchServiceError::oneinch_api(format!("Invalid response format: {}", e))
                })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            error!("1inch API error {} from {}: {}", status, url, error_text);

            let error = match status.as_u16() {
                400 => OneinchServiceError::invalid_quote(error_text),
                401 => OneinchServiceError::oneinch_api("Invalid API key".to_string()),
                403 => OneinchServiceError::oneinch_api("Access forbidden".to_string()),
                404 => OneinchServiceError::oneinch_api("Endpoint not found".to_string()),
                429 => OneinchServiceError::RateLimitExceeded { retry_after: 60 },
                500 => OneinchServiceError::oneinch_api("1inch internal server error".to_string()),
                502 | 503 | 504 => OneinchServiceError::oneinch_api("1inch service unavailable".to_string()),
                _ => OneinchServiceError::oneinch_api(format!("HTTP {}: {}", status, error_text)),
            };

            Err(error)
        }
    }
}

// 1inch API request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchQuoteRequest {
    pub chain_id: u64,
    pub src: String,
    pub dst: String,
    pub amount: String,
    pub slippage: Option<String>,
    pub from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchQuoteResponse {
    #[serde(rename = "dstAmount")]
    pub dst_amount: String,
    
    #[serde(rename = "srcToken")]
    pub src_token: OneinchToken,
    
    #[serde(rename = "dstToken")]
    pub dst_token: OneinchToken,
    
    pub protocols: Vec<Vec<OneinchProtocol>>,
    
    #[serde(rename = "estimatedGas")]
    pub estimated_gas: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchSwapRequest {
    pub chain_id: u64,
    pub src: String,
    pub dst: String,
    pub amount: String,
    pub from: String,
    pub slippage: Option<String>,
    pub disable_estimate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchSwapResponse {
    #[serde(flatten)]
    pub quote: OneinchQuoteResponse,
    
    pub tx: OneinchTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchTransaction {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas: Option<String>,
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchToken {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchProtocol {
    pub name: String,
    pub part: f64,
    #[serde(rename = "fromTokenAddress")]
    pub from_token_address: String,
    #[serde(rename = "toTokenAddress")]
    pub to_token_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchTokensResponse {
    pub tokens: std::collections::HashMap<String, OneinchToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchProtocolsResponse {
    pub protocols: Vec<OneinchLiquidityProtocol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneinchLiquidityProtocol {
    pub id: String,
    pub title: String,
    pub img: Option<String>,
}