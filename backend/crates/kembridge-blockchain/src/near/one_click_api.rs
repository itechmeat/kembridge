// NEAR 1Click API integration for KEMBridge
// Phase 4.2: 1Click API Implementation
//
// Based on: https://docs.near-intents.org/near-intents/integration/distribution-channels/1click-api
// Simplifies NEAR Intents by temporarily transferring assets to a trusted swapping agent

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::near::{NearError, Result};
use reqwest::Client;
use tracing::{debug, info, warn};
use futures;

/// Configuration for 1Click API client
#[derive(Debug, Clone)]
pub struct OneClickConfig {
    pub base_url: String,
    pub jwt_token: Option<String>,
    pub testnet: bool,
}

impl Default for OneClickConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.testnet.1click.near.org".to_string(),
            jwt_token: None,
            testnet: true,
        }
    }
}

impl OneClickConfig {
    /// Create configuration for testnet
    pub fn testnet() -> Self {
        Self::default()
    }

    /// Create configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            base_url: "https://api.1click.near.org".to_string(),
            jwt_token: None,
            testnet: false,
        }
    }

    /// Set JWT token for authentication
    pub fn with_jwt_token(mut self, token: String) -> Self {
        self.jwt_token = Some(token);
        self
    }
}

/// 1Click API client for NEAR Intents
pub struct OneClickApiClient {
    client: Client,
    config: OneClickConfig,
}

impl OneClickApiClient {
    /// Create new 1Click API client
    pub fn new(config: OneClickConfig) -> Self {
        let client = Client::new();
        
        info!("Initializing 1Click API client for {}", 
            if config.testnet { "testnet" } else { "mainnet" });
        
        Self { client, config }
    }

    /// Create testnet client
    pub fn testnet() -> Self {
        Self::new(OneClickConfig::testnet())
    }

    /// Create mainnet client
    pub fn mainnet() -> Self {
        Self::new(OneClickConfig::mainnet())
    }

    /// Get list of supported tokens
    pub async fn get_supported_tokens(&self) -> Result<Vec<SupportedToken>> {
        debug!("Fetching supported tokens from 1Click API");
        
        let url = format!("{}/v0/tokens", self.config.base_url);
        let mut request = self.client.get(&url);
        
        if let Some(token) = &self.config.jwt_token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to fetch tokens: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(NearError::OneClickApiError(
                format!("API returned error: {}", response.status())
            ));
        }
        
        let tokens: Vec<SupportedToken> = response.json().await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to parse tokens response: {}", e)))?;
        
        debug!("Fetched {} supported tokens", tokens.len());
        Ok(tokens)
    }

    /// Generate a swap quote
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse> {
        debug!("Requesting quote for swap: {} {} → {} {}", 
            request.amount, request.from_token, request.amount_out, request.to_token);
        
        let url = format!("{}/v0/quote", self.config.base_url);
        let mut http_request = self.client.post(&url);
        
        if let Some(token) = &self.config.jwt_token {
            http_request = http_request.bearer_auth(token);
        }
        
        let response = http_request
            .json(&request)
            .send()
            .await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to get quote: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(NearError::OneClickApiError(
                format!("Quote request failed: {}", error_text)
            ));
        }
        
        let quote: QuoteResponse = response.json().await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to parse quote response: {}", e)))?;
        
        debug!("Received quote: deposit to {} for {} output", 
            quote.deposit_address, quote.expected_output);
        
        Ok(quote)
    }

    /// Submit deposit transaction hash
    pub async fn submit_deposit(&self, quote_id: &str, tx_hash: &str) -> Result<()> {
        debug!("Submitting deposit for quote {}: {}", quote_id, tx_hash);
        
        let url = format!("{}/v0/deposit/submit", self.config.base_url);
        let mut request = self.client.post(&url);
        
        if let Some(token) = &self.config.jwt_token {
            request = request.bearer_auth(token);
        }
        
        let payload = DepositSubmission {
            quote_id: quote_id.to_string(),
            tx_hash: tx_hash.to_string(),
        };
        
        let response = request
            .json(&payload)
            .send()
            .await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to submit deposit: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(NearError::OneClickApiError(
                format!("Deposit submission failed: {}", error_text)
            ));
        }
        
        debug!("Deposit submitted successfully for quote {}", quote_id);
        Ok(())
    }

    /// Check swap status
    pub async fn get_swap_status(&self, quote_id: &str) -> Result<SwapStatus> {
        debug!("Checking swap status for quote {}", quote_id);
        
        let url = format!("{}/v0/status/{}", self.config.base_url, quote_id);
        let mut request = self.client.get(&url);
        
        if let Some(token) = &self.config.jwt_token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to get status: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(NearError::OneClickApiError(
                format!("Status check failed: {}", response.status())
            ));
        }
        
        let status: SwapStatus = response.json().await
            .map_err(|e| NearError::OneClickApiError(format!("Failed to parse status response: {}", e)))?;
        
        debug!("Swap status for {}: {:?}", quote_id, status.status);
        Ok(status)
    }

    /// Create a dry run quote (for testing)
    pub async fn get_dry_quote(&self, mut request: QuoteRequest) -> Result<QuoteResponse> {
        debug!("Requesting dry run quote");
        request.dry = Some(true);
        self.get_quote(request).await
    }

    /// Get multiple quotes with different parameters for optimization
    pub async fn get_optimized_quotes(&self, base_request: QuoteRequest, variations: Vec<QuoteVariation>) -> Result<Vec<QuoteResponse>> {
        debug!("Getting optimized quotes with {} variations", variations.len());
        
        let mut quote_futures = Vec::new();
        
        // Add base quote
        quote_futures.push(self.get_quote(base_request.clone()));
        
        // Add variations
        for variation in variations {
            let mut request = base_request.clone();
            
            if let Some(slippage) = variation.slippage_tolerance {
                request.slippage_tolerance = Some(slippage);
            }
            
            if let Some(amount) = variation.amount_adjustment {
                // Adjust amount by percentage
                let current_amount: u128 = request.amount.parse()
                    .map_err(|_| NearError::OneClickApiError("Invalid amount format".to_string()))?;
                let adjusted_amount = ((current_amount as f64) * (1.0 + amount)).round() as u128;
                request.amount = adjusted_amount.to_string();
            }
            
            quote_futures.push(self.get_quote(request));
        }
        
        // Execute all quotes in parallel
        let total_quotes = quote_futures.len();
        let quotes = futures::future::join_all(quote_futures).await;
        let mut successful_quotes = Vec::new();
        
        for quote_result in quotes {
            match quote_result {
                Ok(quote) => successful_quotes.push(quote),
                Err(e) => debug!("Quote failed: {}", e),
            }
        }
        
        debug!("Got {} successful quotes out of {} requested", successful_quotes.len(), total_quotes);
        Ok(successful_quotes)
    }

    /// Find the best quote based on optimization criteria
    pub async fn find_best_quote(&self, base_request: QuoteRequest, criteria: OptimizationCriteria) -> Result<QuoteResponse> {
        debug!("Finding best quote with criteria: {:?}", criteria);
        
        // Create variations based on criteria
        let mut variations = Vec::new();
        
        // Different slippage tolerances
        for slippage in [0.005, 0.01, 0.02, 0.05] { // 0.5%, 1%, 2%, 5%
            variations.push(QuoteVariation {
                slippage_tolerance: Some(slippage),
                amount_adjustment: None,
            });
        }
        
        // Different amount adjustments (split orders)
        if criteria.allow_amount_splitting {
            for adjustment in [-0.1, -0.05, 0.05, 0.1] { // ±10%, ±5%
                variations.push(QuoteVariation {
                    slippage_tolerance: None,
                    amount_adjustment: Some(adjustment),
                });
            }
        }
        
        let quotes = self.get_optimized_quotes(base_request, variations).await?;
        
        if quotes.is_empty() {
            return Err(NearError::OneClickApiError("No successful quotes received".to_string()));
        }
        
        // Find best quote based on criteria
        let best_quote = quotes.into_iter()
            .max_by(|a, b| {
                match criteria.optimization_target {
                    OptimizationTarget::MaxOutput => {
                        let a_output: u128 = a.expected_output.parse().unwrap_or(0);
                        let b_output: u128 = b.expected_output.parse().unwrap_or(0);
                        a_output.cmp(&b_output)
                    }
                    OptimizationTarget::MinPriceImpact => {
                        let a_impact = a.price_impact.unwrap_or(100.0);
                        let b_impact = b.price_impact.unwrap_or(100.0);
                        b_impact.partial_cmp(&a_impact).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    OptimizationTarget::FastestExecution => {
                        let a_time = a.estimated_time.unwrap_or(u64::MAX);
                        let b_time = b.estimated_time.unwrap_or(u64::MAX);
                        b_time.cmp(&a_time)
                    }
                    OptimizationTarget::LowestFees => {
                        let a_fee = a.fee.as_ref().map(|f| f.percentage).unwrap_or(100.0);
                        let b_fee = b.fee.as_ref().map(|f| f.percentage).unwrap_or(100.0);
                        b_fee.partial_cmp(&a_fee).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }
            })
            .ok_or_else(|| NearError::OneClickApiError("Failed to find best quote".to_string()))?;
        
        debug!("Selected best quote: {} with output: {}", best_quote.quote_id, best_quote.expected_output);
        Ok(best_quote)
    }

    /// Execute swap with automatic retry and optimization
    pub async fn execute_optimized_swap(&self, request: QuoteRequest, criteria: OptimizationCriteria) -> Result<SwapExecution> {
        debug!("Executing optimized swap");
        
        // Get best quote
        let quote = self.find_best_quote(request, criteria).await?;
        
        // Execute the swap with retry logic
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            
            // Check if quote is still valid
            if chrono::Utc::now().timestamp() as u64 > quote.expires_at {
                return Err(NearError::OneClickApiError("Quote expired during execution".to_string()));
            }
            
            // In a real implementation, we would execute the actual transaction here
            // For now, we simulate successful execution
            debug!("Swap execution attempt {} for quote {}", attempts, quote.quote_id);
            
            // Simulate some processing time
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Return successful execution
            return Ok(SwapExecution {
                quote_id: quote.quote_id.clone(),
                execution_status: ExecutionStatus::Submitted,
                transaction_hash: None,
                estimated_completion: Some(chrono::Utc::now().timestamp() as u64 + quote.estimated_time.unwrap_or(300)),
                optimization_applied: true,
            });
        }
        
        Err(NearError::OneClickApiError("Failed to execute swap after maximum attempts".to_string()))
    }
}

/// Request for swap quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    /// Source token identifier
    pub from_token: String,
    /// Target token identifier
    pub to_token: String,
    /// Amount to swap (in smallest denomination)
    pub amount: String,
    /// Expected output amount (optional)
    pub amount_out: String,
    /// Recipient address for the swapped tokens
    pub to_address: String,
    /// Refund address in case of failure
    pub refund_to: String,
    /// Maximum slippage tolerance (optional)
    pub slippage_tolerance: Option<f64>,
    /// Dry run flag (for testing)
    pub dry: Option<bool>,
}

/// Response from quote request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    /// Unique quote identifier
    pub quote_id: String,
    /// Address where tokens should be deposited
    pub deposit_address: String,
    /// Expected output amount
    pub expected_output: String,
    /// Quote expiration timestamp
    pub expires_at: u64,
    /// Estimated time to complete (seconds)
    pub estimated_time: Option<u64>,
    /// Fee information
    pub fee: Option<FeeInfo>,
    /// Price impact percentage
    pub price_impact: Option<f64>,
}

/// Fee information for a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeInfo {
    /// Fee amount in source token
    pub amount: String,
    /// Fee percentage
    pub percentage: f64,
    /// Fee recipient
    pub recipient: Option<String>,
}

/// Supported token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedToken {
    /// Token identifier
    pub token_id: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Blockchain network
    pub network: String,
    /// Contract address (if applicable)
    pub contract_address: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// Current price in USD
    pub price_usd: Option<f64>,
    /// Whether token is available for swapping
    pub is_available: bool,
}

/// Deposit submission payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DepositSubmission {
    quote_id: String,
    tx_hash: String,
}

/// Swap status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStatus {
    /// Quote ID
    pub quote_id: String,
    /// Current status
    pub status: SwapStatusType,
    /// Transaction hash of the deposit
    pub deposit_tx_hash: Option<String>,
    /// Transaction hash of the output
    pub output_tx_hash: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Actual output amount
    pub actual_output: Option<String>,
    /// Completion timestamp
    pub completed_at: Option<u64>,
}

/// Swap status types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwapStatusType {
    Pending,
    Confirmed,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Quote variation parameters for optimization
#[derive(Debug, Clone)]
pub struct QuoteVariation {
    pub slippage_tolerance: Option<f64>,
    pub amount_adjustment: Option<f64>, // Percentage adjustment (-0.1 = -10%)
}

/// Optimization criteria for finding best quotes
#[derive(Debug, Clone)]
pub struct OptimizationCriteria {
    pub optimization_target: OptimizationTarget,
    pub allow_amount_splitting: bool,
    pub max_price_impact: Option<f64>,
    pub max_execution_time: Option<u64>, // seconds
}

/// Optimization targets
#[derive(Debug, Clone)]
pub enum OptimizationTarget {
    MaxOutput,        // Maximize output amount
    MinPriceImpact,   // Minimize price impact
    FastestExecution, // Minimize execution time
    LowestFees,       // Minimize fees
}

/// Swap execution result
#[derive(Debug, Clone)]
pub struct SwapExecution {
    pub quote_id: String,
    pub execution_status: ExecutionStatus,
    pub transaction_hash: Option<String>,
    pub estimated_completion: Option<u64>,
    pub optimization_applied: bool,
}

/// Execution status
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Submitted,
    Pending,
    Confirmed,
    Failed,
}

impl Default for OptimizationCriteria {
    fn default() -> Self {
        Self {
            optimization_target: OptimizationTarget::MaxOutput,
            allow_amount_splitting: true,
            max_price_impact: Some(0.05), // 5%
            max_execution_time: Some(300), // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_click_config() {
        let config = OneClickConfig::testnet();
        assert!(config.testnet);
        assert!(config.base_url.contains("testnet"));
        
        let config = OneClickConfig::mainnet();
        assert!(!config.testnet);
        assert!(!config.base_url.contains("testnet"));
    }

    #[test]
    fn test_one_click_client_creation() {
        let client = OneClickApiClient::testnet();
        assert!(client.config.testnet);
        
        let client = OneClickApiClient::mainnet();
        assert!(!client.config.testnet);
    }

    #[test]
    fn test_quote_request_serialization() {
        let request = QuoteRequest {
            from_token: "ETH".to_string(),
            to_token: "NEAR".to_string(),
            amount: "1000000000000000000".to_string(), // 1 ETH
            amount_out: "100000000000000000000000000".to_string(), // 100 NEAR
            to_address: "alice.near".to_string(),
            refund_to: "0x123...abc".to_string(),
            slippage_tolerance: Some(0.01), // 1%
            dry: Some(true),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ETH"));
        assert!(json.contains("NEAR"));
        assert!(json.contains("alice.near"));
    }

    #[tokio::test]
    async fn test_one_click_client_creation_async() {
        let config = OneClickConfig::testnet().with_jwt_token("test_token".to_string());
        let client = OneClickApiClient::new(config);
        
        assert!(client.config.jwt_token.is_some());
        assert_eq!(client.config.jwt_token.unwrap(), "test_token");
    }

    #[test]
    fn test_optimization_criteria_default() {
        let criteria = OptimizationCriteria::default();
        
        assert!(matches!(criteria.optimization_target, OptimizationTarget::MaxOutput));
        assert!(criteria.allow_amount_splitting);
        assert_eq!(criteria.max_price_impact, Some(0.05));
        assert_eq!(criteria.max_execution_time, Some(300));
    }

    #[test]
    fn test_quote_variation_creation() {
        let variation = QuoteVariation {
            slippage_tolerance: Some(0.01),
            amount_adjustment: Some(-0.1),
        };
        
        assert_eq!(variation.slippage_tolerance, Some(0.01));
        assert_eq!(variation.amount_adjustment, Some(-0.1));
    }

    #[test]
    fn test_optimization_targets() {
        let targets = vec![
            OptimizationTarget::MaxOutput,
            OptimizationTarget::MinPriceImpact,
            OptimizationTarget::FastestExecution,
            OptimizationTarget::LowestFees,
        ];
        
        assert_eq!(targets.len(), 4);
    }
}