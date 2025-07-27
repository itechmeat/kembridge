// Bridge Event Handler Service (Task 4.1.9)
// Integrates Ethereum and NEAR event listeners with bridge logic

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use kembridge_blockchain::{
    ethereum::{BridgeEvent as EthereumBridgeEvent, EthereumEventListener, EventListenerConfig},
    near::{NearBridgeEvent, NearEventListener, NearEventListenerConfig},
};

use crate::{BridgeService, BridgeError, SwapStatus};

/// Event handler that processes incoming blockchain events
pub struct BridgeEventHandler {
    bridge_service: Arc<BridgeService>,
}

impl BridgeEventHandler {
    /// Create new bridge event handler
    pub fn new(bridge_service: Arc<BridgeService>) -> Self {
        Self {
            bridge_service,
        }
    }

    /// Start event processing with provided channels
    pub async fn start_processing(
        self,
        mut ethereum_receiver: mpsc::UnboundedReceiver<EthereumBridgeEvent>,
        mut near_receiver: mpsc::UnboundedReceiver<NearBridgeEvent>,
    ) -> Result<(), BridgeError> {
        info!("Starting bridge event handler");

        // Start event processing loops
        let bridge_service_eth = Arc::clone(&self.bridge_service);
        tokio::spawn(async move {
            while let Some(event) = ethereum_receiver.recv().await {
                if let Err(e) = Self::process_ethereum_event(&bridge_service_eth, event).await {
                    error!(error = %e, "Failed to process Ethereum event");
                }
            }
            info!("Ethereum event processing loop ended");
        });

        let bridge_service_near = Arc::clone(&self.bridge_service);
        tokio::spawn(async move {
            while let Some(event) = near_receiver.recv().await {
                if let Err(e) = Self::process_near_event(&bridge_service_near, event).await {
                    error!(error = %e, "Failed to process NEAR event");
                }
            }
            info!("NEAR event processing loop ended");
        });

        info!("Bridge event handler started successfully");
        Ok(())
    }

    /// Start event listeners and return a configured event handler
    pub async fn start_with_listeners(
        bridge_service: Arc<BridgeService>,
        ethereum_listener: Option<EthereumEventListener>,
        near_listener: Option<NearEventListener>,
    ) -> Result<(), BridgeError> {
        let (eth_sender, eth_receiver) = mpsc::unbounded_channel();
        let (near_sender, near_receiver) = mpsc::unbounded_channel();

        // Start Ethereum event listener if available
        if let Some(eth_listener) = ethereum_listener {
            tokio::spawn(async move {
                if let Err(e) = eth_listener.start_listening().await {
                    error!(error = %e, "Ethereum event listener failed");
                }
            });
        }

        // Start NEAR event listener if available
        if let Some(near_listener) = near_listener {
            tokio::spawn(async move {
                if let Err(e) = near_listener.start_listening().await {
                    error!(error = %e, "NEAR event listener failed");
                }
            });
        }

        // Create and start event handler
        let handler = Self::new(bridge_service);
        handler.start_processing(eth_receiver, near_receiver).await?;

        Ok(())
    }

    /// Process Ethereum bridge events
    async fn process_ethereum_event(
        bridge_service: &Arc<BridgeService>,
        event: EthereumBridgeEvent,
    ) -> Result<(), BridgeError> {
        match event {
            EthereumBridgeEvent::TokensLocked {
                user_address,
                amount,
                recipient_chain,
                quantum_hash,
                tx_hash,
                block_number,
            } => {
                info!(
                    user_address = %user_address,
                    amount = %amount,
                    recipient_chain = %recipient_chain,
                    tx_hash = %tx_hash,
                    block_number = block_number,
                    "Processing Ethereum TokensLocked event"
                );

                // Find matching swap operation by quantum hash or create new one
                match Self::find_swap_by_quantum_hash(bridge_service, &quantum_hash).await {
                    Ok(Some(swap_id)) => {
                        // Update existing swap with Ethereum transaction info
                        bridge_service.update_swap_status_with_details(
                            swap_id,
                            SwapStatus::EthLocked,
                            Some("ethereum_tokens_locked_confirmed"),
                            Some(&tx_hash.to_string()),
                            None,
                        ).await?;

                        info!(swap_id = %swap_id, "Updated existing swap with Ethereum lock");
                    }
                    Ok(None) => {
                        // Create new incoming swap operation
                        warn!(
                            quantum_hash = %quantum_hash,
                            "Received Ethereum lock event without existing swap - creating new incoming swap"
                        );
                        Self::create_incoming_swap_from_ethereum(
                            bridge_service,
                            user_address,
                            amount,
                            &recipient_chain,
                            &quantum_hash,
                            &tx_hash.to_string(),
                            block_number,
                        ).await?;
                    }
                    Err(e) => {
                        error!(error = %e, quantum_hash = %quantum_hash, "Failed to find swap by quantum hash");
                        return Err(e);
                    }
                }
            }
            EthereumBridgeEvent::TokensUnlocked {
                recipient,
                amount,
                source_chain,
                quantum_hash,
                tx_hash,
                block_number,
            } => {
                info!(
                    recipient = %recipient,
                    amount = %amount,
                    source_chain = %source_chain,
                    tx_hash = %tx_hash,
                    block_number = block_number,
                    "Processing Ethereum TokensUnlocked event"
                );

                // Find and complete the corresponding swap
                if let Some(swap_id) = Self::find_swap_by_quantum_hash(bridge_service, &quantum_hash).await? {
                    bridge_service.update_swap_status_with_details(
                        swap_id,
                        SwapStatus::Completed,
                        Some("ethereum_tokens_unlocked_confirmed"),
                        Some(&tx_hash.to_string()),
                        Some(0.1), // Low risk score for successful completion
                    ).await?;

                    info!(swap_id = %swap_id, "Completed swap with Ethereum unlock");
                } else {
                    warn!(quantum_hash = %quantum_hash, "Received unlock event without matching swap");
                }
            }
            EthereumBridgeEvent::BridgeDeposit { .. } | EthereumBridgeEvent::BridgeWithdrawal { .. } => {
                // Handle other Ethereum events as needed
                debug!("Processing other Ethereum bridge event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Process NEAR bridge events
    async fn process_near_event(
        bridge_service: &Arc<BridgeService>,
        event: NearBridgeEvent,
    ) -> Result<(), BridgeError> {
        match event {
            NearBridgeEvent::TokensMinted {
                recipient_account,
                amount,
                source_chain,
                quantum_hash,
                tx_hash,
                block_height,
            } => {
                info!(
                    recipient_account = %recipient_account,
                    amount = amount,
                    source_chain = %source_chain,
                    tx_hash = %tx_hash,
                    block_height = block_height,
                    "Processing NEAR TokensMinted event"
                );

                // Find and update the corresponding swap
                if let Some(swap_id) = Self::find_swap_by_quantum_hash(bridge_service, &quantum_hash).await? {
                    bridge_service.update_swap_status_with_details(
                        swap_id,
                        SwapStatus::NearMinted,
                        Some("near_tokens_minted_confirmed"),
                        Some(&tx_hash),
                        None,
                    ).await?;

                    info!(swap_id = %swap_id, "Updated swap with NEAR mint");
                } else {
                    warn!(quantum_hash = %quantum_hash, "Received NEAR mint event without matching swap");
                }
            }
            NearBridgeEvent::TokensLocked {
                user_account,
                amount,
                recipient_chain,
                quantum_hash,
                tx_hash,
                block_height,
            } => {
                info!(
                    user_account = %user_account,
                    amount = amount,
                    recipient_chain = %recipient_chain,
                    tx_hash = %tx_hash,
                    block_height = block_height,
                    "Processing NEAR TokensLocked event"
                );

                // Handle NEAR → Ethereum direction
                match Self::find_swap_by_quantum_hash(bridge_service, &quantum_hash).await {
                    Ok(Some(swap_id)) => {
                        bridge_service.update_swap_status_with_details(
                            swap_id,
                            SwapStatus::NearMinted, // Reusing status for NEAR locked
                            Some("near_tokens_locked_confirmed"),
                            Some(&tx_hash),
                            None,
                        ).await?;
                    }
                    Ok(None) => {
                        // Create incoming swap from NEAR
                        Self::create_incoming_swap_from_near(
                            bridge_service,
                            &user_account,
                            amount,
                            &recipient_chain,
                            &quantum_hash,
                            &tx_hash,
                            block_height,
                        ).await?;
                    }
                    Err(e) => return Err(e),
                }
            }
            NearBridgeEvent::TokensBurned { .. } |
            NearBridgeEvent::TokensUnlocked { .. } |
            NearBridgeEvent::BridgeDeposit { .. } |
            NearBridgeEvent::BridgeWithdrawal { .. } => {
                // Handle other NEAR events as needed
                debug!("Processing other NEAR bridge event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Find swap operation by quantum hash
    async fn find_swap_by_quantum_hash(
        bridge_service: &Arc<BridgeService>,
        quantum_hash: &str,
    ) -> Result<Option<Uuid>, BridgeError> {
        // NOTE: Quantum hash lookup requires quantum signature verification integration
        // For now, we'll return None to indicate no matching swap found
        // In a real implementation, this would query the transactions table
        // looking for a transaction with matching quantum_key_id or metadata

        debug!(quantum_hash = %quantum_hash, "Searching for swap by quantum hash");
        
        // This is a mock implementation - in reality we would:
        // 1. Query transactions table for matching quantum_key_id
        // 2. Or search in metadata JSON field for quantum_hash
        // 3. Return the swap_id if found

        Ok(None)
    }

    /// Create new incoming swap from Ethereum event
    async fn create_incoming_swap_from_ethereum(
        bridge_service: &Arc<BridgeService>,
        user_address: ethers::types::Address,
        amount: ethers::types::U256,
        recipient_chain: &str,
        quantum_hash: &str,
        tx_hash: &str,
        block_number: u64,
    ) -> Result<(), BridgeError> {
        info!(
            user_address = %user_address,
            amount = %amount,
            recipient_chain = %recipient_chain,
            "Creating incoming swap from Ethereum event"
        );

        // TODO: Implement creation of incoming swap operation
        // This would involve:
        // 1. Create a new user if address doesn't exist
        // 2. Create SwapOperation with detected parameters
        // 3. Set status to EthLocked since we detected the lock
        // 4. Store quantum_hash in metadata
        // 5. Continue with NEAR minting process

        warn!("Incoming swap creation not yet implemented - would create swap for quantum_hash: {}", quantum_hash);
        Ok(())
    }

    /// Create new incoming swap from NEAR event
    async fn create_incoming_swap_from_near(
        bridge_service: &Arc<BridgeService>,
        user_account: &str,
        amount: u128,
        recipient_chain: &str,
        quantum_hash: &str,
        tx_hash: &str,
        block_height: u64,
    ) -> Result<(), BridgeError> {
        info!(
            user_account = %user_account,
            amount = amount,
            recipient_chain = %recipient_chain,
            "Creating incoming swap from NEAR event"
        );

        // TODO: Implement creation of incoming swap operation for NEAR → ETH
        // Similar to Ethereum version but for NEAR side

        warn!("Incoming NEAR swap creation not yet implemented - would create swap for quantum_hash: {}", quantum_hash);
        Ok(())
    }
}

/// Configuration for the bridge event handler
#[derive(Debug, Clone)]
pub struct BridgeEventHandlerConfig {
    pub ethereum_config: Option<EventListenerConfig>,
    pub near_config: Option<NearEventListenerConfig>,
}

impl Default for BridgeEventHandlerConfig {
    fn default() -> Self {
        Self {
            ethereum_config: Some(EventListenerConfig::default()),
            near_config: Some(NearEventListenerConfig::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_event_handler_config_default() {
        let config = BridgeEventHandlerConfig::default();
        assert!(config.ethereum_config.is_some());
        assert!(config.near_config.is_some());
    }
}