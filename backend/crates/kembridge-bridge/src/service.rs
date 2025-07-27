use crate::{
    BridgeError, SwapOperation, SwapInitResponse, SwapResult, SwapStatus,
    SwapEngine, StateMachine, ValidationService, TimeoutManager,
};
use kembridge_crypto::QuantumKeyManager;
use kembridge_blockchain::{ethereum::EthereumAdapter, near::NearAdapter};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use tracing::{info, warn};

pub struct BridgeService {
    ethereum_adapter: Arc<EthereumAdapter>,
    near_adapter: Arc<NearAdapter>,
    quantum_manager: Arc<QuantumKeyManager>,
    swap_engine: SwapEngine,
    state_machine: StateMachine,
    validation_service: ValidationService,
    timeout_manager: TimeoutManager,
    db_pool: PgPool,
}

impl BridgeService {
    pub async fn new(
        ethereum_adapter: Arc<EthereumAdapter>,
        near_adapter: Arc<NearAdapter>,
        quantum_manager: Arc<QuantumKeyManager>,
        db_pool: PgPool,
    ) -> Result<Self, BridgeError> {
        Ok(Self {
            ethereum_adapter: ethereum_adapter.clone(),
            near_adapter: near_adapter.clone(),
            quantum_manager: quantum_manager.clone(),
            swap_engine: SwapEngine::new(quantum_manager.clone()).await?,
            state_machine: StateMachine::new(),
            validation_service: ValidationService::new(),
            timeout_manager: TimeoutManager::new(),
            db_pool,
        })
    }

    pub async fn init_swap(
        &self,
        user_id: Uuid,
        from_chain: &str,
        to_chain: &str,
        amount: u128,
        recipient: &str,
    ) -> Result<SwapInitResponse, BridgeError> {
        tracing::info!("Initializing swap for user {}: {} {} -> {}", user_id, amount, from_chain, to_chain);

        // Validate swap parameters
        self.validation_service.validate_swap_params(
            from_chain, to_chain, amount, recipient
        ).await?;

        // Create swap operation
        let swap_id = Uuid::new_v4();
        let now = Utc::now();
        let swap_operation = SwapOperation {
            swap_id,
            user_id,
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            amount,
            recipient: recipient.to_string(),
            status: SwapStatus::Initialized,
            quantum_key_id: None,
            eth_tx_hash: None,
            near_tx_hash: None,
            created_at: now,
            updated_at: now,
            expires_at: now + chrono::Duration::minutes(30), // 30 minutes timeout
        };

        // Save to database
        self.save_swap_operation(&swap_operation).await?;

        // Start timeout monitoring in background
        let service_clone = Arc::new(self.clone());
        let service_clone_2 = service_clone.clone();
        tokio::spawn(async move {
            if let Err(e) = service_clone.timeout_manager.monitor_operation_timeout(swap_id, service_clone_2).await {
                tracing::error!("Timeout monitoring failed for swap {}: {}", swap_id, e);
            }
        });

        Ok(SwapInitResponse {
            swap_id,
            status: SwapStatus::Initialized,
            estimated_time: chrono::Duration::minutes(5),
        })
    }

    pub async fn execute_swap(&self, swap_id: Uuid) -> Result<SwapResult, BridgeError> {
        tracing::info!("Executing atomic swap {}", swap_id);

        let swap_operation = self.get_swap_operation(swap_id).await?;

        // Check if swap is in correct state
        if swap_operation.status != SwapStatus::Initialized {
            return Err(BridgeError::InvalidStateTransition {
                from: swap_operation.status,
                to: SwapStatus::EthLocking,
            });
        }

        // Execute atomic swap with proper state transitions
        match (swap_operation.from_chain.as_str(), swap_operation.to_chain.as_str()) {
            ("ethereum", "near") => {
                self.execute_atomic_eth_to_near_swap(&swap_operation).await
            }
            ("near", "ethereum") => {
                self.execute_atomic_near_to_eth_swap(&swap_operation).await
            }
            _ => Err(BridgeError::UnsupportedChain {
                chain: format!("{} -> {}", swap_operation.from_chain, swap_operation.to_chain),
            }),
        }
    }

    /// Execute atomic ETH → NEAR swap with proper state management
    async fn execute_atomic_eth_to_near_swap(&self, swap_operation: &SwapOperation) -> Result<SwapResult, BridgeError> {
        tracing::info!("Starting atomic ETH → NEAR swap {}", swap_operation.swap_id);

        // Phase 1: ETH Locking with state transition
        tracing::info!("Phase 1: Locking ETH tokens for swap {}", swap_operation.swap_id);
        self.update_swap_status_with_details(
            swap_operation.swap_id, 
            SwapStatus::EthLocking,
            Some("starting_eth_lock_phase"),
            None,
            None
        ).await?;
        
        // Generate quantum protection
        let (quantum_key_id, protected_data) = self.swap_engine.generate_quantum_protection(swap_operation).await?;
        
        // Lock ETH tokens
        let eth_lock_result = self.swap_engine.lock_eth_tokens(
            &self.ethereum_adapter,
            &swap_operation.swap_id.to_string(),
            swap_operation.amount,
            &protected_data,
        ).await?;

        // Update status to ETH locked
        self.update_swap_status_with_details(
            swap_operation.swap_id, 
            SwapStatus::EthLocked,
            Some("eth_tokens_locked_successfully"),
            Some(&eth_lock_result.transaction_hash),
            None
        ).await?;
        tracing::info!("Phase 1 completed: ETH tokens locked for swap {}", swap_operation.swap_id);

        // Phase 2: NEAR Minting with state transition
        tracing::info!("Phase 2: Minting NEAR tokens for swap {}", swap_operation.swap_id);
        self.update_swap_status_with_details(
            swap_operation.swap_id, 
            SwapStatus::NearMinting,
            Some("starting_near_mint_phase"),
            None,
            None
        ).await?;
        
        // Derive NEAR address (simplified for now)
        let near_address = format!("derived_{}", swap_operation.recipient);
        
        // Mint wrapped tokens on NEAR
        let near_mint_result = self.swap_engine.mint_near_tokens(
            &self.near_adapter,
            swap_operation.amount,
            &near_address,
            &protected_data,
        ).await?;

        // Update status to NEAR minted
        self.update_swap_status_with_details(
            swap_operation.swap_id, 
            SwapStatus::NearMinted,
            Some("near_tokens_minted_successfully"),
            Some(&near_mint_result.transaction_hash),
            None
        ).await?;
        tracing::info!("Phase 2 completed: NEAR tokens minted for swap {}", swap_operation.swap_id);

        // Phase 3: Atomic completion verification
        tracing::info!("Phase 3: Verifying atomic completion for swap {}", swap_operation.swap_id);
        self.swap_engine.verify_atomic_completion(&eth_lock_result, &near_mint_result).await?;

        // Final status update
        self.update_swap_status_with_details(
            swap_operation.swap_id, 
            SwapStatus::Completed,
            Some("atomic_swap_completed_successfully"),
            None,
            Some(0.1) // Low risk score for successful completion
        ).await?;
        tracing::info!("Atomic ETH → NEAR swap {} completed successfully", swap_operation.swap_id);

        Ok(SwapResult {
            swap_id: swap_operation.swap_id,
            eth_tx_hash: Some(eth_lock_result.transaction_hash),
            near_tx_hash: Some(near_mint_result.transaction_hash),
            status: SwapStatus::Completed,
            quantum_key_id: Some(quantum_key_id),
        })
    }

    /// Execute atomic NEAR → ETH swap with proper state management
    async fn execute_atomic_near_to_eth_swap(&self, swap_operation: &SwapOperation) -> Result<SwapResult, BridgeError> {
        tracing::info!("Starting atomic NEAR → ETH swap {}", swap_operation.swap_id);

        // Phase 1: NEAR Locking
        tracing::info!("Phase 1: Locking NEAR tokens for swap {}", swap_operation.swap_id);
        self.update_swap_status(swap_operation.swap_id, SwapStatus::NearMinting).await?; // Reusing state for NEAR lock
        
        // Generate quantum protection
        let (quantum_key_id, protected_data) = self.swap_engine.generate_quantum_protection(swap_operation).await?;
        
        // Lock NEAR tokens
        let near_lock_result = self.swap_engine.lock_near_tokens(
            &self.near_adapter,
            &swap_operation.swap_id.to_string(),
            swap_operation.amount,
            &protected_data,
        ).await?;

        self.update_swap_status(swap_operation.swap_id, SwapStatus::NearMinted).await?; // Reusing state for NEAR locked
        tracing::info!("Phase 1 completed: NEAR tokens locked for swap {}", swap_operation.swap_id);

        // Phase 2: ETH Unlocking
        tracing::info!("Phase 2: Unlocking ETH tokens for swap {}", swap_operation.swap_id);
        self.update_swap_status(swap_operation.swap_id, SwapStatus::EthLocking).await?; // Reusing state for ETH unlock
        
        // Unlock ETH tokens
        let eth_unlock_result = self.swap_engine.unlock_eth_tokens(
            &self.ethereum_adapter,
            &swap_operation.swap_id.to_string(),
            swap_operation.amount,
            &swap_operation.recipient,
            &protected_data,
        ).await?;

        self.update_swap_status(swap_operation.swap_id, SwapStatus::EthLocked).await?; // Reusing state for ETH unlocked
        tracing::info!("Phase 2 completed: ETH tokens unlocked for swap {}", swap_operation.swap_id);

        // Phase 3: Atomic completion verification
        tracing::info!("Phase 3: Verifying atomic completion for swap {}", swap_operation.swap_id);
        let eth_as_lock = crate::EthereumLockResult {
            transaction_hash: eth_unlock_result.transaction_hash.clone(),
            confirmed: eth_unlock_result.confirmed,
            quantum_hash: eth_unlock_result.quantum_hash.clone(),
        };
        self.swap_engine.verify_atomic_completion(&eth_as_lock, &near_lock_result).await?;

        // Final status update
        self.update_swap_status(swap_operation.swap_id, SwapStatus::Completed).await?;
        tracing::info!("Atomic NEAR → ETH swap {} completed successfully", swap_operation.swap_id);

        Ok(SwapResult {
            swap_id: swap_operation.swap_id,
            eth_tx_hash: Some(eth_unlock_result.transaction_hash),
            near_tx_hash: Some(near_lock_result.transaction_hash),
            status: SwapStatus::Completed,
            quantum_key_id: Some(quantum_key_id),
        })
    }

    pub async fn get_swap_operation(&self, swap_id: Uuid) -> Result<SwapOperation, BridgeError> {
        let row = sqlx::query!(
            "SELECT id, user_id, source_chain, destination_chain, amount_in, status, quantum_key_id, source_tx_hash, destination_tx_hash, created_at, updated_at, expires_at 
             FROM transactions WHERE id = $1",
            swap_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match row {
            Some(row) => {
                let status = match row.status.as_str() {
                    "initialized" => SwapStatus::Initialized,
                    "eth_locking" => SwapStatus::EthLocking,
                    "eth_locked" => SwapStatus::EthLocked,
                    "near_minting" => SwapStatus::NearMinting,
                    "near_minted" => SwapStatus::NearMinted,
                    "completed" => SwapStatus::Completed,
                    "failed" => SwapStatus::Failed,
                    "cancelled" => SwapStatus::Cancelled,
                    "timeout" => SwapStatus::Timeout,
                    "rolled_back" => SwapStatus::RolledBack,
                    _ => SwapStatus::Failed,
                };

                // Convert BigDecimal to u128 - simplified conversion for now
                let amount_f64 = row.amount_in.to_f64().unwrap_or(0.0);
                let amount = (amount_f64 * 1_000_000_000_000_000_000f64) as u128;

                Ok(SwapOperation {
                    swap_id: row.id,
                    user_id: row.user_id,
                    from_chain: row.source_chain,
                    to_chain: row.destination_chain,
                    amount,
                    recipient: "temp_recipient".to_string(), // Will be added to schema later
                    status,
                    quantum_key_id: row.quantum_key_id.map(|id| id.to_string()),
                    eth_tx_hash: row.source_tx_hash,
                    near_tx_hash: row.destination_tx_hash,
                    created_at: row.created_at.unwrap_or(Utc::now()),
                    updated_at: row.updated_at.unwrap_or(Utc::now()),
                    expires_at: row.expires_at.unwrap_or(Utc::now() + chrono::Duration::hours(24)),
                })
            }
            None => Err(BridgeError::SwapNotFound),
        }
    }

    pub async fn update_swap_status(&self, swap_id: Uuid, new_status: SwapStatus) -> Result<(), BridgeError> {
        self.update_swap_status_with_details(swap_id, new_status, None, None, None).await
    }

    pub async fn update_swap_status_with_details(
        &self, 
        swap_id: Uuid, 
        new_status: SwapStatus,
        reason: Option<&str>,
        tx_hash: Option<&str>,
        risk_score: Option<f64>
    ) -> Result<(), BridgeError> {
        let mut swap_operation = self.get_swap_operation(swap_id).await?;

        // Validate state transition
        self.state_machine.transition_state(&mut swap_operation, new_status.clone())?;

        // Convert risk score to proper decimal format
        let risk_score_decimal = risk_score.map(|score| {
            // Convert f64 to BigDecimal with 4 decimal places for risk_score domain
            BigDecimal::from_f64(score.min(1.0).max(0.0))
                .unwrap_or_else(|| BigDecimal::from(0))
        });

        // Create metadata for the status update
        let metadata = serde_json::json!({
            "previous_status": swap_operation.status.to_string(),
            "updated_by": "bridge_service",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "swap_id": swap_id.to_string()
        });

        // Update in database using the PostgreSQL function for proper audit trail
        let result = sqlx::query!(
            "SELECT update_transaction_status($1, $2, $3, $4, $5, $6) as result",
            swap_id,                                    // p_transaction_id
            new_status.to_string(),                     // p_new_status
            reason.unwrap_or("bridge_service_update"),  // p_reason
            tx_hash,                                    // p_tx_hash
            risk_score_decimal,                         // p_risk_score
            metadata                                    // p_metadata
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| BridgeError::OperationFailed(format!("Failed to update transaction status: {}", e)))?;

        if !result.result.unwrap_or(false) {
            return Err(BridgeError::OperationFailed(
                format!("Database function rejected status update for swap {}", swap_id)
            ));
        }

        tracing::info!(
            "Updated swap {} status: {} → {} (reason: {})", 
            swap_id, 
            swap_operation.status,
            new_status,
            reason.unwrap_or("bridge_service_update")
        );
        
        Ok(())
    }

    async fn save_swap_operation(&self, swap_operation: &SwapOperation) -> Result<(), BridgeError> {
        // Convert amount from wei to decimal format for database storage
        let amount_decimal = BigDecimal::from(swap_operation.amount) 
            / BigDecimal::from(1_000_000_000_000_000_000u64); // Convert from wei to ETH/NEAR
        
        // Calculate expected amount out (simplified 1:1 for hackathon)
        let expected_amount_out = amount_decimal.clone();
        
        // Determine token types based on bridge direction
        let (source_token, destination_token) = match (swap_operation.from_chain.as_str(), swap_operation.to_chain.as_str()) {
            ("ethereum", "near") => ("ETH", "wETH"),
            ("near", "ethereum") => ("NEAR", "wNEAR"),
            _ => ("ETH", "NEAR"), // Fallback
        };
        
        // Calculate expiration hours (30 minutes = 0.5 hours)
        let expires_in_hours = if let Some(expires_at) = Some(swap_operation.expires_at) {
            let duration = expires_at.signed_duration_since(swap_operation.created_at);
            (duration.num_minutes() as f64 / 60.0).max(0.5) as i32
        } else {
            1 // Default 1 hour
        };
        
        // Use PostgreSQL function to create transaction with proper validation and audit trail
        let result = sqlx::query!(
            "SELECT create_bridge_transaction($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) as transaction_id",
            swap_operation.user_id,           // p_user_id
            swap_operation.from_chain,        // p_source_chain  
            swap_operation.to_chain,          // p_destination_chain
            source_token,                     // p_source_token
            destination_token,                // p_destination_token
            amount_decimal,                   // p_amount_in
            expected_amount_out,              // p_expected_amount_out
            swap_operation.quantum_key_id.as_ref().and_then(|s| s.parse::<Uuid>().ok()),    // p_quantum_key_id
            expires_in_hours,                 // p_expires_in_hours
            Option::<String>::None,           // p_oneinch_quote_id (not used in basic bridge)
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| BridgeError::OperationFailed(format!("Database transaction creation failed: {}", e)))?;

        // The function returns the generated transaction_id, but we use our own swap_id
        // Update the transaction with our specific swap_id if needed
        sqlx::query!(
            "UPDATE transactions SET id = $1 WHERE id = $2",
            swap_operation.swap_id,
            result.transaction_id.unwrap_or(swap_operation.swap_id)
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| BridgeError::OperationFailed(format!("Failed to update transaction ID: {}", e)))?;

        tracing::info!(
            "Saved swap operation {} with amount {} {} → {} {}", 
            swap_operation.swap_id,
            amount_decimal,
            source_token,
            expected_amount_out,
            destination_token
        );
        
        Ok(())
    }
}

// Clone implementation for Arc usage
impl Clone for BridgeService {
    fn clone(&self) -> Self {
        Self {
            ethereum_adapter: self.ethereum_adapter.clone(),
            near_adapter: self.near_adapter.clone(),
            quantum_manager: self.quantum_manager.clone(),
            swap_engine: self.swap_engine.clone(),
            state_machine: self.state_machine.clone(),
            validation_service: self.validation_service.clone(),
            timeout_manager: self.timeout_manager.clone(),
            db_pool: self.db_pool.clone(),
        }
    }
}