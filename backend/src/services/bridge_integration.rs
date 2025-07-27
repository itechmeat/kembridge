// src/services/bridge_integration.rs - Bridge Integration Service with 1inch

use crate::oneinch::{OneinchBridgeIntegration, OneinchService};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn, error};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Service wrapper for OneinchBridgeIntegration
pub struct BridgeIntegrationService {
    inner: Option<OneinchBridgeIntegration>,
    oneinch_service: Arc<OneinchService>,
}

impl BridgeIntegrationService {
    /// Create new bridge integration service
    pub fn new(
        oneinch_service: Arc<OneinchService>,
        bridge_service: Option<Arc<super::BridgeService>>,
    ) -> Self {
        info!("Creating BridgeIntegrationService");
        
        let inner = if let Some(bridge_service) = bridge_service {
            // Create real integration with available bridge service
            info!("Bridge service available, creating full integration");
            
            // Get the inner BridgeService from our wrapper
            // Access the inner kembridge_bridge::BridgeService
            Some(OneinchBridgeIntegration::new(
                oneinch_service.clone(),
                bridge_service.inner(),
            ))
        } else {
            warn!("Bridge service not available - 1inch quotes will work but bridge execution will fail");
            None
        };

        Self { 
            inner,
            oneinch_service,
        }
    }

    /// Check if bridge integration is available (always true for 1inch quotes)
    pub fn is_available(&self) -> bool {
        // 1inch quotes should always be available even if bridge is not
        true
    }

    /// Get quote for bridge swap using real 1inch API
    pub async fn get_bridge_quote(
        &self,
        from_chain: &str,
        to_chain: &str,
        from_token: &str,
        to_token: &str,
        amount: &str,
    ) -> Result<String, String> {
        info!("Getting bridge quote: {} {} on {} -> {} on {}", 
              amount, from_token, from_chain, to_token, to_chain);

        // Parse amount to BigDecimal
        let amount_decimal = BigDecimal::from_str(amount)
            .map_err(|e| format!("Invalid amount format: {}", e))?;

        // Check if chains are supported
        if !self.is_chain_supported(from_chain) {
            return Err(format!("Source chain '{}' is not supported", from_chain));
        }
        
        if !self.is_chain_supported(to_chain) && to_chain != "near" {
            return Err(format!("Destination chain '{}' is not supported", to_chain));
        }

        // For cross-chain swaps, we need to get quotes for both sides
        let quote_result = if from_chain == to_chain {
            // Same chain swap - direct 1inch quote
            self.get_single_chain_quote(from_chain, from_token, to_token, &amount_decimal).await
        } else {
            // Cross-chain swap - get optimization quotes
            self.get_cross_chain_quote(from_chain, to_chain, from_token, to_token, &amount_decimal).await
        };

        match quote_result {
            Ok(quote_id) => {
                info!("Successfully generated quote: {}", quote_id);
                Ok(quote_id)
            }
            Err(e) => {
                error!("Failed to get quote: {}", e);
                Err(e)
            }
        }
    }

    /// Get quote for single chain swap using 1inch
    async fn get_single_chain_quote(
        &self,
        chain: &str,
        from_token: &str,
        to_token: &str,
        amount: &BigDecimal,
    ) -> Result<String, String> {
        use crate::oneinch::QuoteParams;
        
        // Create quote parameters for 1inch
        let quote_params = QuoteParams {
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            amount: amount.clone(),
            from_address: "0x0000000000000000000000000000000000000000".to_string(), // Placeholder
            slippage: Some(1.0), // 1% slippage
            disable_estimate: Some(false),
            allow_partial_fill: Some(false),
            source: Some("kembridge".to_string()),
        };

        // Get quote from 1inch
        let quote = self.oneinch_service.get_quote(&quote_params)
            .await
            .map_err(|e| format!("1inch API error: {}", e))?;

        // Return the quote ID
        Ok(quote.quote_id)
    }

    /// Get quote for cross-chain swap (with 1inch optimization)
    async fn get_cross_chain_quote(
        &self,
        from_chain: &str,
        to_chain: &str,
        from_token: &str,
        to_token: &str,
        amount: &BigDecimal,
    ) -> Result<String, String> {
        // For cross-chain swaps, we use the full bridge integration if available
        if let Some(ref integration) = self.inner {
            // Use real bridge integration with 1inch optimization
            let user_id = Uuid::new_v4(); // Temporary user ID for quote
            
            let result = integration.execute_optimized_bridge_swap(
                user_id,
                from_chain,
                to_chain,
                from_token,
                to_token,
                amount.clone(),
                "0x0000000000000000000000000000000000000000", // Placeholder recipient
                Some("balanced".to_string()),
            ).await;

            match result {
                Ok(swap_result) => Ok(swap_result.bridge_swap_id.to_string()),
                Err(e) => Err(format!("Bridge integration error: {}", e)),
            }
        } else {
            // Bridge service not available, but we can still provide 1inch quotes
            // Get quote for the source chain part
            let source_quote = self.get_single_chain_quote(from_chain, from_token, "ETH", amount).await?;
            
            // Generate a combined quote ID
            let combined_quote_id = format!("cross_chain_{}", Uuid::new_v4());
            
            info!("Generated cross-chain quote {} (source: {})", combined_quote_id, source_quote);
            Ok(combined_quote_id)
        }
    }

    /// Check if chain is supported by 1inch
    fn is_chain_supported(&self, chain: &str) -> bool {
        match chain {
            "ethereum" | "eth" => true,
            "bsc" | "binance" => true,
            "polygon" | "matic" => true,
            "avalanche" | "avax" => true,
            "arbitrum" => true,
            "optimism" => true,
            _ => false,
        }
    }

    /// Get bridge swap status
    pub async fn get_bridge_swap_status(&self, swap_id: &str) -> Result<BridgeSwapStatus, String> {
        info!("Getting bridge swap status for: {}", swap_id);

        // Try to parse as UUID for real bridge swaps
        if let Ok(uuid) = Uuid::parse_str(swap_id) {
            if let Some(ref integration) = self.inner {
                match integration.get_bridge_swap_status(uuid).await {
                    Ok(swap_operation) => {
                        return Ok(BridgeSwapStatus {
                            swap_id: swap_id.to_string(),
                            status: format!("{:?}", swap_operation.status),
                            from_chain: swap_operation.from_chain,
                            to_chain: swap_operation.to_chain,
                            amount: swap_operation.amount.to_string(),
                            recipient: swap_operation.recipient,
                            eth_tx_hash: None, // Would be filled by real bridge service
                            near_tx_hash: None,
                            quantum_key_id: None,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            updated_at: chrono::Utc::now().to_rfc3339(),
                            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
                        });
                    }
                    Err(e) => {
                        error!("Failed to get bridge swap status: {}", e);
                        return Err(format!("Bridge error: {}", e));
                    }
                }
            }
        }

        // For quote-only or cross-chain quotes, return a pending status
        Ok(BridgeSwapStatus {
            swap_id: swap_id.to_string(),
            status: "quote_generated".to_string(),
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "0".to_string(),
            recipient: "unknown".to_string(),
            eth_tx_hash: None,
            near_tx_hash: None,
            quantum_key_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
        })
    }
}

/// Bridge swap status response
#[derive(Debug, Clone)]
pub struct BridgeSwapStatus {
    pub swap_id: String,
    pub status: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: String,
    pub recipient: String,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub quantum_key_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
}