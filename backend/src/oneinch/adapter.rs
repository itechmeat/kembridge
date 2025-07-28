// src/oneinch/adapter.rs - High-level adapter for 1inch Fusion+ integration

use super::{client::FusionClient, types::*};
use crate::constants::*;
use bigdecimal::BigDecimal;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// High-level adapter for 1inch Fusion+ operations
pub struct OneinchAdapter {
    pub client: Arc<FusionClient>,
}

impl OneinchAdapter {
    /// Create new OneinchAdapter instance
    pub fn new(client: Arc<FusionClient>) -> Self {
        Self { client }
    }

    /// Execute complete swap operation
    pub async fn execute_swap(&self, params: &SwapParams) -> Result<SwapResult, OneinchError> {
        info!("Starting 1inch swap execution: {} -> {}", params.from_token, params.to_token);

        // Validate parameters
        self.validate_swap_params(params)?;

        // Step 1: Get quote
        let quote_params = QuoteParams {
            from_token: params.from_token.clone(),
            to_token: params.to_token.clone(),
            amount: params.amount.clone(),
            from_address: params.from_address.clone(),
            slippage: Some(params.slippage),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge".to_string()),
        };

        let quote = self.client.get_quote(&quote_params).await?;
        debug!("Received quote: {} -> {}", quote.from_amount, quote.to_amount);

        // Step 2: Check if quote is still valid
        if quote.expires_at < Utc::now() {
            return Err(OneinchError::OrderExpired);
        }

        // Step 3: Create order
        let create_order_request = CreateOrderRequest {
            from_token_address: params.from_token.clone(),
            to_token_address: params.to_token.clone(),
            amount: params.amount.to_string(),
            from_address: params.from_address.clone(),
            slippage: params.slippage,
            deadline: params.deadline.unwrap_or_else(|| {
                (Utc::now() + Duration::seconds(ONEINCH_ORDER_TIMEOUT_SEC as i64)).timestamp() as u64
            }),
            quote_id: quote.quote_id.clone(),
        };

        let order_response = self.client.create_order(&create_order_request).await?;
        info!("Created order with hash: {}", order_response.order_hash);

        // Step 4: Return order data for client-side signing
        // The client (frontend) will sign this order with user's wallet and call execute endpoint
        info!("Order created successfully, returning for client-side signing");
        
        let swap_result = SwapResult {
            order_hash: order_response.order_hash.clone(),
            tx_hash: None, // Will be set after client signs and submits
            status: OrderStatus::Pending,
            from_amount: params.amount.clone(),
            to_amount: quote.to_amount.clone(),
            actual_gas_used: None,
            created_at: chrono::Utc::now(),
        };

        info!("Swap submitted successfully: {}", swap_result.order_hash);
        Ok(swap_result)
    }

    /// Get order status
    pub async fn get_order_status(&self, order_hash: &str) -> Result<OrderStatus, OneinchError> {
        let response = self.client.get_order_status(order_hash).await?;
        Ok(response.status)
    }

    /// Get detailed order information
    pub async fn get_order_details(&self, order_hash: &str) -> Result<OrderStatusResponse, OneinchError> {
        self.client.get_order_status(order_hash).await
    }

    /// Check if tokens are supported
    pub async fn check_token_support(&self, token_address: &str) -> Result<bool, OneinchError> {
        let tokens = self.client.get_tokens().await?;
        Ok(tokens.iter().any(|token| token.address.eq_ignore_ascii_case(token_address)))
    }

    /// Get supported tokens list
    pub async fn get_supported_tokens(&self) -> Result<Vec<Token>, OneinchError> {
        self.client.get_tokens().await
    }

    /// Check allowance for token
    pub async fn check_allowance(
        &self,
        token_address: &str,
        owner_address: &str,
        spender_address: &str,
    ) -> Result<BigDecimal, OneinchError> {
        let allowance_str = self.client.get_allowance(token_address, owner_address, spender_address).await?;
        allowance_str.parse::<BigDecimal>()
            .map_err(|_| OneinchError::InvalidParams("Invalid allowance value".to_string()))
    }

    /// Get approve transaction data
    pub async fn get_approve_transaction(
        &self,
        token_address: &str,
        amount: Option<&BigDecimal>,
    ) -> Result<serde_json::Value, OneinchError> {
        let amount_str = amount.map(|a| a.to_string());
        self.client.get_approve_transaction(token_address, amount_str.as_deref()).await
    }

    /// Validate swap parameters
    fn validate_swap_params(&self, params: &SwapParams) -> Result<(), OneinchError> {
        // Check slippage range
        if params.slippage < ONEINCH_MIN_SLIPPAGE {
            return Err(OneinchError::InvalidParams(
                format!("Slippage too low: {}% < {}%", params.slippage, ONEINCH_MIN_SLIPPAGE)
            ));
        }

        if params.slippage > ONEINCH_MAX_SLIPPAGE {
            return Err(OneinchError::InvalidParams(
                format!("Slippage too high: {}% > {}%", params.slippage, ONEINCH_MAX_SLIPPAGE)
            ));
        }

        // Check amount is positive
        if params.amount <= BigDecimal::from(0) {
            return Err(OneinchError::InvalidParams(
                "Amount must be positive".to_string()
            ));
        }

        // Check addresses format (basic validation)
        if !self.is_valid_address(&params.from_token) {
            return Err(OneinchError::InvalidParams(
                "Invalid from_token address".to_string()
            ));
        }

        if !self.is_valid_address(&params.to_token) {
            return Err(OneinchError::InvalidParams(
                "Invalid to_token address".to_string()
            ));
        }

        if !self.is_valid_address(&params.from_address) {
            return Err(OneinchError::InvalidParams(
                "Invalid from_address".to_string()
            ));
        }

        Ok(())
    }

    /// Basic Ethereum address validation
    fn is_valid_address(&self, address: &str) -> bool {
        address.starts_with("0x") && address.len() == 42 && address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if 1inch API is healthy
    pub async fn health_check(&self) -> Result<bool, OneinchError> {
        self.client.health_check().await
    }

    /// Get current chain ID
    pub fn get_chain_id(&self) -> u64 {
        self.client.get_chain_id()
    }

    /// Wait for order completion with timeout
    pub async fn wait_for_order_completion(
        &self,
        order_hash: &str,
        timeout_seconds: u64,
    ) -> Result<OrderStatus, OneinchError> {
        let start_time = Utc::now();
        let timeout_duration = Duration::seconds(timeout_seconds as i64);

        loop {
            let status = self.get_order_status(order_hash).await?;
            
            if status.is_final() {
                info!("Order {} completed with status: {:?}", order_hash, status);
                return Ok(status);
            }

            if Utc::now() - start_time > timeout_duration {
                warn!("Order {} timed out after {} seconds", order_hash, timeout_seconds);
                return Err(OneinchError::OrderExpired);
            }

            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// Execute a pre-signed order (called after client signs the order)
    pub async fn execute_signed_order(
        &self,
        order_hash: &str,
        signature: &str,
    ) -> Result<SwapResult, OneinchError> {
        info!("Executing signed order: {}", order_hash);

        // Submit the signed order to 1inch
        let submit_result = self.client.submit_signed_order(order_hash, signature).await?;
        
        info!("Order submitted to 1inch, tx_hash: {:?}", submit_result.tx_hash);
        
        Ok(SwapResult {
            order_hash: order_hash.to_string(),
            tx_hash: submit_result.tx_hash,
            status: OrderStatus::Pending,
            from_amount: submit_result.from_amount,
            to_amount: submit_result.to_amount,
            actual_gas_used: None,
            created_at: chrono::Utc::now(),
        })
    }

    /// Cancel order (if supported by the API)
    pub async fn cancel_order(&self, order_hash: &str) -> Result<bool, OneinchError> {
        // TODO: Implement order cancellation when 1inch API supports it
        warn!("Order cancellation not yet implemented for order: {}", order_hash);
        Err(OneinchError::InvalidParams("Order cancellation not supported yet".to_string()))
    }

    /// Get swap history for address
    pub async fn get_swap_history(&self, address: &str, limit: Option<u32>) -> Result<Vec<OrderStatusResponse>, OneinchError> {
        // TODO: Implement swap history when 1inch API supports it
        warn!("Swap history not yet implemented for address: {}", address);
        Err(OneinchError::InvalidParams("Swap history not supported yet".to_string()))
    }

    /// Estimate gas for swap
    pub async fn estimate_gas(&self, params: &SwapParams) -> Result<BigDecimal, OneinchError> {
        let quote_params = QuoteParams {
            from_token: params.from_token.clone(),
            to_token: params.to_token.clone(),
            amount: params.amount.clone(),
            from_address: params.from_address.clone(),
            slippage: Some(params.slippage),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge".to_string()),
        };

        let quote = self.client.get_quote(&quote_params).await?;
        Ok(quote.estimated_gas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_test_adapter() -> OneinchAdapter {
        let client = Arc::new(FusionClient::new(
            "test_key".to_string(),
            ONEINCH_SEPOLIA_CHAIN_ID,
        ));
        OneinchAdapter::new(client)
    }

    #[test]
    fn test_address_validation() {
        let adapter = create_test_adapter();
        
        // Valid addresses
        assert!(adapter.is_valid_address("0x1234567890123456789012345678901234567890"));
        assert!(adapter.is_valid_address("0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D"));
        
        // Invalid addresses
        assert!(!adapter.is_valid_address("1234567890123456789012345678901234567890")); // No 0x prefix
        assert!(!adapter.is_valid_address("0x123")); // Too short
        assert!(!adapter.is_valid_address("0x1234567890123456789012345678901234567890XX")); // Too long
        assert!(!adapter.is_valid_address("0x1234567890123456789012345678901234567890GG")); // Invalid hex
    }

    #[test]
    fn test_swap_params_validation() {
        let adapter = create_test_adapter();
        
        // Valid parameters
        let valid_params = SwapParams {
            from_token: "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D".to_string(),
            to_token: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            amount: BigDecimal::from(1000),
            from_address: "0x1234567890123456789012345678901234567890".to_string(),
            slippage: ONEINCH_DEFAULT_SLIPPAGE,
            deadline: None,
            referrer: None,
            fee: None,
        };
        assert!(adapter.validate_swap_params(&valid_params).is_ok());

        // Invalid slippage - too low
        let mut invalid_params = valid_params.clone();
        invalid_params.slippage = 0.05; // Below minimum
        assert!(adapter.validate_swap_params(&invalid_params).is_err());

        // Invalid slippage - too high
        invalid_params.slippage = 10.0; // Above maximum
        assert!(adapter.validate_swap_params(&invalid_params).is_err());

        // Invalid amount
        invalid_params.slippage = ONEINCH_DEFAULT_SLIPPAGE;
        invalid_params.amount = BigDecimal::from(0);
        assert!(adapter.validate_swap_params(&invalid_params).is_err());

        // Invalid address
        invalid_params.amount = BigDecimal::from(1000);
        invalid_params.from_token = "invalid_address".to_string();
        assert!(adapter.validate_swap_params(&invalid_params).is_err());
    }

    #[test]
    fn test_chain_id() {
        let adapter = create_test_adapter();
        assert_eq!(adapter.get_chain_id(), ONEINCH_SEPOLIA_CHAIN_ID);
    }
}