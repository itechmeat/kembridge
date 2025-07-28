// src/services/mod.rs - Service layer modules
#![allow(dead_code)]

use crate::config::AppConfig;
use anyhow::Result;
use redis::aio::ConnectionManager;

/// Authentication service - Web3 wallet authentication with JWT
pub use kembridge_auth::AuthService;

/// User management service - implemented in Phase 2.3
pub mod user;
pub use user::UserService;

/// Quantum cryptography service - implemented in Phase 3.2
pub mod quantum;
pub use quantum::{QuantumService, QuantumServiceError};

/// Risk analysis client - implemented in Phase 5.2
pub mod risk_client;
pub use risk_client::RiskClient;

/// Risk integration service - implemented in Phase 5.2
pub mod risk_integration;
pub use risk_integration::{RiskIntegrationService, OperationDecision};

/// Manual review service - implemented in Phase 5.2.4
pub mod manual_review;
pub mod manual_review_service;
pub use manual_review::ManualReviewService;

/// Bridge integration service - implemented in Phase 6.2.1
pub mod bridge_integration;
pub use bridge_integration::BridgeIntegrationService;

/// Rate limiting service - implemented in Phase 7 (H7)
pub mod rate_limit;
pub use rate_limit::{RateLimitService, RateLimitResult, RateLimitStats, ViolatorInfo, AlertCondition, AlertSeverity};

#[cfg(test)]
mod user_service_tests;

/// Bridge service with risk integration - implemented in Phase 5.2.7
pub struct BridgeService {
    inner: std::sync::Arc<kembridge_bridge::BridgeService>,
    risk_integration_service: Option<std::sync::Arc<RiskIntegrationService>>,
}

impl BridgeService {
    pub async fn new(
        db: sqlx::PgPool,
        quantum_service: std::sync::Arc<QuantumService>,
        config: &AppConfig,
    ) -> Result<Self> {
        // For now, create a minimal bridge service that doesn't require full adapter initialization
        // This allows testing of the dynamic pricing endpoint without full blockchain integration
        tracing::info!("Creating BridgeService in test mode for dynamic pricing testing");
        
        // Create mock adapters for testing
        let ethereum_adapter = match kembridge_blockchain::ethereum::EthereumAdapter::new(config.ethereum_config()).await {
            Ok(adapter) => {
                tracing::info!("Successfully initialized Ethereum adapter");
                std::sync::Arc::new(adapter)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize Ethereum adapter: {}. Creating mock adapter for testing.", e);
                // For testing, we'll create a minimal mock - in production this would be handled differently
                return Err(anyhow::anyhow!("Bridge service requires valid Ethereum configuration for production use"));
            }
        };
        
        let near_adapter = match kembridge_blockchain::near::NearAdapter::new(config.near_config()).await {
            Ok(adapter) => {
                tracing::info!("Successfully initialized NEAR adapter");
                std::sync::Arc::new(adapter)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize NEAR adapter: {}. Creating mock adapter for testing.", e);
                // For testing, we'll create a minimal mock - in production this would be handled differently
                return Err(anyhow::anyhow!("Bridge service requires valid NEAR configuration for production use"));
            }
        };

        let quantum_manager = quantum_service.get_quantum_manager();

        // Initialize real bridge service from kembridge-bridge crate
        let inner = std::sync::Arc::new(
            kembridge_bridge::BridgeService::new(
                ethereum_adapter,
                near_adapter,
                quantum_manager,
                db.clone(),
            ).await
                .map_err(|e| anyhow::anyhow!("Failed to create bridge service: {}", e))?
        );

        Ok(Self {
            inner,
            risk_integration_service: None,
        })
    }


    /// Set risk integration service for automatic profile updates (Phase 5.2.7)
    pub fn with_risk_integration(mut self, risk_integration_service: std::sync::Arc<RiskIntegrationService>) -> Self {
        self.risk_integration_service = Some(risk_integration_service);
        self
    }

    /// Execute swap with automatic risk profile updates
    pub async fn execute_swap(&self, swap_id: uuid::Uuid) -> Result<kembridge_bridge::SwapResult, kembridge_bridge::BridgeError> {
        // Get swap operation first
        let swap_operation = self.inner.get_swap_operation(swap_id).await?;
        
        // Execute the actual swap using the inner bridge service
        let result = self.inner.execute_swap(swap_id).await?;

        // Update risk profile after successful transaction completion (Phase 5.2.7)
        if result.status == kembridge_bridge::SwapStatus::Completed {
            if let Some(ref risk_service) = self.risk_integration_service {
                let transaction_data = serde_json::json!({
                    "event_type": "transaction_completed",
                    "transaction_type": "bridge_swap",
                    "swap_id": swap_operation.swap_id,
                    "user_id": swap_operation.user_id,
                    "from_chain": swap_operation.from_chain,
                    "to_chain": swap_operation.to_chain,
                    "amount": swap_operation.amount,
                    "recipient": swap_operation.recipient,
                    "status": "completed",
                    "eth_tx_hash": result.eth_tx_hash,
                    "near_tx_hash": result.near_tx_hash,
                    "quantum_key_id": result.quantum_key_id,
                    "timestamp": chrono::Utc::now(),
                    "source": "bridge_service"
                });

                // Update risk profile (non-blocking)
                if let Err(e) = risk_service.update_user_profile_after_transaction(
                    swap_operation.user_id,
                    transaction_data
                ).await {
                    tracing::warn!(
                        user_id = %swap_operation.user_id,
                        swap_id = %swap_operation.swap_id,
                        error = %e,
                        "Failed to update risk profile after transaction completion"
                    );
                    // Don't fail the transaction for risk profile update errors
                }
            }
        }

        Ok(result)
    }

    /// Get swap operation (delegate to inner service)
    pub async fn get_swap_operation(&self, swap_id: uuid::Uuid) -> Result<kembridge_bridge::SwapOperation, kembridge_bridge::BridgeError> {
        self.inner.get_swap_operation(swap_id).await
    }

    /// Get inner bridge service for advanced integrations
    pub fn inner(&self) -> std::sync::Arc<kembridge_bridge::BridgeService> {
        self.inner.clone()
    }

    /// Initialize swap (delegate to inner service)
    pub async fn init_swap(
        &self,
        user_id: uuid::Uuid,
        from_chain: &str,
        to_chain: &str,
        amount: u128,
        recipient: &str,
    ) -> Result<kembridge_bridge::SwapInitResponse, kembridge_bridge::BridgeError> {
        self.inner.init_swap(user_id, from_chain, to_chain, amount, recipient).await
    }
}


/// AI risk engine client - will be implemented in Phase 5.1
/// TODO: Phase 5.1 - Replace with real FastAPI ML risk analysis client
pub struct AiClient;

impl AiClient {
    pub fn new(_ai_engine_url: &str) -> Result<Self> {
        Ok(Self)
    }
}