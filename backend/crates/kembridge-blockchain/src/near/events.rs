// NEAR Event Listeners for Bridge Transactions (Task 4.1.9)
use near_jsonrpc_client::{methods, JsonRpcClient};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use super::{NearError, Result};

/// NEAR bridge event types we listen for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NearBridgeEvent {
    /// NEAR tokens locked in bridge contract
    TokensLocked {
        user_account: String,
        amount: u128,
        recipient_chain: String,
        quantum_hash: String,
        tx_hash: String,
        block_height: u64,
    },
    /// NEAR tokens unlocked from bridge contract
    TokensUnlocked {
        recipient_account: String,
        amount: u128,
        source_chain: String,
        quantum_hash: String,
        tx_hash: String,
        block_height: u64,
    },
    /// Wrapped tokens minted on NEAR
    TokensMinted {
        recipient_account: String,
        amount: u128,
        source_chain: String,
        quantum_hash: String,
        tx_hash: String,
        block_height: u64,
    },
    /// Wrapped tokens burned on NEAR
    TokensBurned {
        user_account: String,
        amount: u128,
        target_chain: String,
        quantum_hash: String,
        tx_hash: String,
        block_height: u64,
    },
    /// Bridge deposit detected
    BridgeDeposit {
        user_account: String,
        amount: u128,
        destination_chain: String,
        tx_hash: String,
        block_height: u64,
    },
    /// Bridge withdrawal completed
    BridgeWithdrawal {
        recipient_account: String,
        amount: u128,
        source_chain: String,
        tx_hash: String,
        block_height: u64,
    },
}

/// Configuration for NEAR event listener
#[derive(Debug, Clone)]
pub struct NearEventListenerConfig {
    /// Bridge contract account to monitor
    pub bridge_contract_account: String,
    /// Starting block height (None = latest)
    pub start_block: Option<u64>,
    /// Confirmation blocks required
    pub confirmation_blocks: u64,
    /// Filter for specific events (None = all)
    pub event_filter: Option<Vec<String>>,
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
}

impl Default for NearEventListenerConfig {
    fn default() -> Self {
        Self {
            bridge_contract_account: String::new(), // Will be set from config
            start_block: None,
            confirmation_blocks: 3, // NEAR finalizes quickly
            event_filter: None,
            poll_interval_secs: 5, // Poll every 5 seconds
        }
    }
}

/// NEAR event listener for bridge operations
pub struct NearEventListener {
    rpc_client: JsonRpcClient,
    config: NearEventListenerConfig,
    event_sender: mpsc::UnboundedSender<NearBridgeEvent>,
}

impl NearEventListener {
    /// Create new NEAR event listener
    pub fn new(
        rpc_client: JsonRpcClient,
        config: NearEventListenerConfig,
        event_sender: mpsc::UnboundedSender<NearBridgeEvent>,
    ) -> Self {
        Self {
            rpc_client,
            config,
            event_sender,
        }
    }

    /// Start listening for bridge events
    pub async fn start_listening(&self) -> Result<()> {
        info!(
            contract_account = %self.config.bridge_contract_account,
            start_block = ?self.config.start_block,
            confirmations = self.config.confirmation_blocks,
            poll_interval = self.config.poll_interval_secs,
            "Starting NEAR event listener"
        );

        // Get current block height
        let status_request = methods::status::RpcStatusRequest;
        let status = self.rpc_client.call(status_request).await
            .map_err(|e| NearError::RpcStatusError(e.to_string()))?;

        let current_block = status.sync_info.latest_block_height;
        let start_block = self.config.start_block.unwrap_or(current_block);

        info!(current_block, start_block, "NEAR event listener initialized");

        let mut last_checked_block = start_block;

        // Start polling loop
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.poll_interval_secs)
        );

        loop {
            interval.tick().await;

            match self.check_for_new_events(last_checked_block).await {
                Ok(latest_block) => {
                    if latest_block > last_checked_block {
                        last_checked_block = latest_block;
                        debug!(latest_block, "Updated last checked block");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Error checking for new events");
                    // Continue polling even if there's an error
                }
            }
        }
    }

    /// Check for new events since last block
    async fn check_for_new_events(&self, from_block: u64) -> Result<u64> {
        // Get current status
        let status_request = methods::status::RpcStatusRequest;
        let status = self.rpc_client.call(status_request).await
            .map_err(|e| NearError::RpcStatusError(e.to_string()))?;

        let current_block = status.sync_info.latest_block_height;
        let confirmed_block = current_block.saturating_sub(self.config.confirmation_blocks);

        if confirmed_block <= from_block {
            // No new confirmed blocks
            return Ok(from_block);
        }

        debug!(
            from_block,
            confirmed_block,
            current_block,
            "Checking blocks for events"
        );

        // TODO (feat): Implement actual event checking (P2.2)
        // For now, this is a simplified implementation
        // In a full implementation, we would:
        // 1. Query transaction outcomes for the bridge contract
        // 2. Parse logs and receipts for bridge events
        // 3. Parse contract method calls that emit events
        
        // Mock event detection for demonstration
        if rand::random::<f64>() < 0.1 { // 10% chance of mock event
            self.emit_mock_event(confirmed_block).await;
        }

        Ok(confirmed_block)
    }

    /// Emit a mock event for testing (TODO (feat): Replace with real event parsing) (P2.2)
    async fn emit_mock_event(&self, block_height: u64) {
        let mock_event = NearBridgeEvent::TokensMinted {
            recipient_account: "user.testnet".to_string(),
            amount: 1000000000000000000000000, // 1 NEAR in yoctoNEAR
            source_chain: "ethereum".to_string(),
            quantum_hash: format!("mock_quantum_hash_{}", block_height),
            tx_hash: format!("mock_tx_hash_{}", block_height),
            block_height,
        };

        info!(
            event_type = "TokensMinted",
            block_height,
            "Detected mock NEAR bridge event"
        );

        if let Err(e) = self.event_sender.send(mock_event) {
            error!(error = %e, "Failed to send NEAR bridge event");
        }
    }

    /// Parse transaction outcomes for bridge events (TODO (feat): Implement) (P2.2)
    async fn parse_transaction_outcomes(
        &self,
        _block_height: u64,
    ) -> Result<Vec<NearBridgeEvent>> {
        // TODO (feat): Implement actual transaction outcome parsing (P2.2)
        // This would involve:
        // 1. Querying block details
        // 2. Iterating through chunks and transactions
        // 3. Parsing transaction outcomes and receipts
        // 4. Looking for logs related to bridge contract
        // 5. Parsing event data from logs

        Ok(Vec::new())
    }

    /// Parse contract logs for bridge events
    fn parse_contract_log(&self, log: &str) -> Option<NearBridgeEvent> {
        // TODO (feat): Implement actual log parsing (P2.2)
        // NEAR contracts emit logs in various formats
        // Bridge contract would emit structured logs like:
        // EVENT_JSON:{"event": "tokens_locked", "data": {...}}
        
        if !log.starts_with("EVENT_JSON:") {
            return None;
        }

        let json_str = log.strip_prefix("EVENT_JSON:")?;
        let log_data: serde_json::Value = serde_json::from_str(json_str).ok()?;

        let event_type = log_data.get("event")?.as_str()?;
        let data = log_data.get("data")?;

        match event_type {
            "tokens_locked" => self.parse_tokens_locked_event(data),
            "tokens_unlocked" => self.parse_tokens_unlocked_event(data),
            "tokens_minted" => self.parse_tokens_minted_event(data),
            "tokens_burned" => self.parse_tokens_burned_event(data),
            "bridge_deposit" => self.parse_bridge_deposit_event(data),
            "bridge_withdrawal" => self.parse_bridge_withdrawal_event(data),
            _ => None,
        }
    }

    /// Parse tokens_locked event data
    fn parse_tokens_locked_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::TokensLocked {
            user_account: data.get("user_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            recipient_chain: data.get("recipient_chain")?.as_str()?.to_string(),
            quantum_hash: data.get("quantum_hash")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }

    /// Parse tokens_unlocked event data
    fn parse_tokens_unlocked_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::TokensUnlocked {
            recipient_account: data.get("recipient_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            source_chain: data.get("source_chain")?.as_str()?.to_string(),
            quantum_hash: data.get("quantum_hash")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }

    /// Parse tokens_minted event data
    fn parse_tokens_minted_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::TokensMinted {
            recipient_account: data.get("recipient_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            source_chain: data.get("source_chain")?.as_str()?.to_string(),
            quantum_hash: data.get("quantum_hash")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }

    /// Parse tokens_burned event data
    fn parse_tokens_burned_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::TokensBurned {
            user_account: data.get("user_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            target_chain: data.get("target_chain")?.as_str()?.to_string(),
            quantum_hash: data.get("quantum_hash")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }

    /// Parse bridge_deposit event data
    fn parse_bridge_deposit_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::BridgeDeposit {
            user_account: data.get("user_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            destination_chain: data.get("destination_chain")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }

    /// Parse bridge_withdrawal event data
    fn parse_bridge_withdrawal_event(&self, data: &serde_json::Value) -> Option<NearBridgeEvent> {
        Some(NearBridgeEvent::BridgeWithdrawal {
            recipient_account: data.get("recipient_account")?.as_str()?.to_string(),
            amount: data.get("amount")?.as_str()?.parse().ok()?,
            source_chain: data.get("source_chain")?.as_str()?.to_string(),
            tx_hash: data.get("tx_hash")?.as_str()?.to_string(),
            block_height: data.get("block_height")?.as_u64()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_event_listener_config_default() {
        let config = NearEventListenerConfig::default();
        assert_eq!(config.bridge_contract_account, "");
        assert_eq!(config.confirmation_blocks, 3);
        assert!(config.start_block.is_none());
        assert!(config.event_filter.is_none());
        assert_eq!(config.poll_interval_secs, 5);
    }

    #[test]
    fn test_near_bridge_event_serialization() {
        let event = NearBridgeEvent::TokensMinted {
            recipient_account: "user.testnet".to_string(),
            amount: 1000000000000000000000000,
            source_chain: "ethereum".to_string(),
            quantum_hash: "test_hash".to_string(),
            tx_hash: "test_tx_hash".to_string(),
            block_height: 12345,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let _deserialized: NearBridgeEvent = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_contract_log_parsing() {
        let listener = NearEventListener {
            rpc_client: JsonRpcClient::connect("https://rpc.testnet.near.org"),
            config: NearEventListenerConfig::default(),
            event_sender: mpsc::unbounded_channel().0,
        };

        let log = r#"EVENT_JSON:{"event":"tokens_minted","data":{"recipient_account":"user.testnet","amount":"1000000000000000000000000","source_chain":"ethereum","quantum_hash":"test_hash","tx_hash":"test_tx","block_height":12345}}"#;

        let event = listener.parse_contract_log(log);
        assert!(event.is_some());

        if let Some(NearBridgeEvent::TokensMinted { recipient_account, amount, .. }) = event {
            assert_eq!(recipient_account, "user.testnet");
            assert_eq!(amount, 1000000000000000000000000);
        } else {
            panic!("Expected TokensMinted event");
        }
    }
}