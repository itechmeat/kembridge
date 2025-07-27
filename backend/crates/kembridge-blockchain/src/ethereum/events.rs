// Ethereum Event Listeners for Bridge Transactions (Task 4.1.9)
use ethers::{
    providers::{Provider, Http, Middleware, StreamExt},
    types::{Filter, Log, Address, H256, U256},
    abi::{Event, EventParam, ParamType, Token},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

use super::EthereumError;

/// Ethereum bridge event types we listen for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// ETH tokens locked in bridge contract
    TokensLocked {
        user_address: Address,
        amount: U256,
        recipient_chain: String,
        quantum_hash: String,
        tx_hash: H256,
        block_number: u64,
    },
    /// ETH tokens unlocked from bridge contract
    TokensUnlocked {
        recipient: Address,
        amount: U256,
        source_chain: String,
        quantum_hash: String,
        tx_hash: H256,
        block_number: u64,
    },
    /// Bridge deposit detected
    BridgeDeposit {
        user_address: Address,
        amount: U256,
        destination_chain: String,
        tx_hash: H256,
        block_number: u64,
    },
    /// Bridge withdrawal completed
    BridgeWithdrawal {
        recipient: Address,
        amount: U256,
        source_chain: String,
        tx_hash: H256,
        block_number: u64,
    },
}

/// Configuration for Ethereum event listener
#[derive(Debug, Clone)]
pub struct EventListenerConfig {
    /// Bridge contract address to monitor
    pub bridge_contract_address: Address,
    /// Starting block number (None = latest)
    pub start_block: Option<u64>,
    /// Confirmation blocks required
    pub confirmation_blocks: u64,
    /// Filter for specific events (None = all)
    pub event_filter: Option<Vec<String>>,
}

impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            bridge_contract_address: Address::zero(), // Will be set from config
            start_block: None,
            confirmation_blocks: 12, // Ethereum standard
            event_filter: None,
        }
    }
}

/// Ethereum event listener for bridge operations
pub struct EthereumEventListener {
    provider: Arc<Provider<Http>>,
    config: EventListenerConfig,
    event_sender: mpsc::UnboundedSender<BridgeEvent>,
}

impl EthereumEventListener {
    /// Create new event listener
    pub fn new(
        provider: Arc<Provider<Http>>,
        config: EventListenerConfig,
        event_sender: mpsc::UnboundedSender<BridgeEvent>,
    ) -> Self {
        Self {
            provider,
            config,
            event_sender,
        }
    }

    /// Start listening for bridge events
    pub async fn start_listening(&self) -> Result<(), EthereumError> {
        info!(
            contract_address = %self.config.bridge_contract_address,
            start_block = ?self.config.start_block,
            confirmations = self.config.confirmation_blocks,
            "Starting Ethereum event listener"
        );

        // Get current block number
        let current_block = self.provider.get_block_number().await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;

        let start_block = self.config.start_block.unwrap_or(current_block.as_u64());

        info!(current_block = current_block.as_u64(), start_block, "Event listener initialized");

        // Create event filters for bridge contract
        let tokens_locked_filter = self.create_tokens_locked_filter(start_block)?;
        let tokens_unlocked_filter = self.create_tokens_unlocked_filter(start_block)?;
        let bridge_deposit_filter = self.create_bridge_deposit_filter(start_block)?;
        let bridge_withdrawal_filter = self.create_bridge_withdrawal_filter(start_block)?;

        // Start monitoring each event type
        let provider_clone = Arc::clone(&self.provider);
        let sender_clone = self.event_sender.clone();
        let confirmations = self.config.confirmation_blocks;

        // Monitor TokensLocked events
        tokio::spawn(async move {
            if let Err(e) = monitor_event_stream(
                provider_clone,
                tokens_locked_filter,
                sender_clone,
                confirmations,
                "TokensLocked",
                parse_tokens_locked_event,
            ).await {
                error!(error = %e, "TokensLocked event monitoring failed");
            }
        });

        let provider_clone = Arc::clone(&self.provider);
        let sender_clone = self.event_sender.clone();

        // Monitor TokensUnlocked events
        tokio::spawn(async move {
            if let Err(e) = monitor_event_stream(
                provider_clone,
                tokens_unlocked_filter,
                sender_clone,
                confirmations,
                "TokensUnlocked",
                parse_tokens_unlocked_event,
            ).await {
                error!(error = %e, "TokensUnlocked event monitoring failed");
            }
        });

        let provider_clone = Arc::clone(&self.provider);
        let sender_clone = self.event_sender.clone();

        // Monitor BridgeDeposit events
        tokio::spawn(async move {
            if let Err(e) = monitor_event_stream(
                provider_clone,
                bridge_deposit_filter,
                sender_clone,
                confirmations,
                "BridgeDeposit",
                parse_bridge_deposit_event,
            ).await {
                error!(error = %e, "BridgeDeposit event monitoring failed");
            }
        });

        let provider_clone = Arc::clone(&self.provider);
        let sender_clone = self.event_sender.clone();

        // Monitor BridgeWithdrawal events
        tokio::spawn(async move {
            if let Err(e) = monitor_event_stream(
                provider_clone,
                bridge_withdrawal_filter,
                sender_clone,
                confirmations,
                "BridgeWithdrawal",
                parse_bridge_withdrawal_event,
            ).await {
                error!(error = %e, "BridgeWithdrawal event monitoring failed");
            }
        });

        info!("All Ethereum event listeners started successfully");
        Ok(())
    }

    /// Create filter for TokensLocked events
    fn create_tokens_locked_filter(&self, start_block: u64) -> Result<Filter, EthereumError> {
        // TokensLocked(address indexed user, uint256 amount, string recipientChain, string quantumHash)
        let event_signature = ethers::utils::keccak256("TokensLocked(address,uint256,string,string)");
        
        Ok(Filter::new()
            .address(self.config.bridge_contract_address)
            .topic0(H256::from(event_signature))
            .from_block(start_block))
    }

    /// Create filter for TokensUnlocked events
    fn create_tokens_unlocked_filter(&self, start_block: u64) -> Result<Filter, EthereumError> {
        // TokensUnlocked(address indexed recipient, uint256 amount, string sourceChain, string quantumHash)
        let event_signature = ethers::utils::keccak256("TokensUnlocked(address,uint256,string,string)");
        
        Ok(Filter::new()
            .address(self.config.bridge_contract_address)
            .topic0(H256::from(event_signature))
            .from_block(start_block))
    }

    /// Create filter for BridgeDeposit events (alternative event name)
    fn create_bridge_deposit_filter(&self, start_block: u64) -> Result<Filter, EthereumError> {
        // BridgeDeposit(address indexed user, uint256 amount, string destinationChain)
        let event_signature = ethers::utils::keccak256("BridgeDeposit(address,uint256,string)");
        
        Ok(Filter::new()
            .address(self.config.bridge_contract_address)
            .topic0(H256::from(event_signature))
            .from_block(start_block))
    }

    /// Create filter for BridgeWithdrawal events (alternative event name)
    fn create_bridge_withdrawal_filter(&self, start_block: u64) -> Result<Filter, EthereumError> {
        // BridgeWithdrawal(address indexed recipient, uint256 amount, string sourceChain)
        let event_signature = ethers::utils::keccak256("BridgeWithdrawal(address,uint256,string)");
        
        Ok(Filter::new()
            .address(self.config.bridge_contract_address)
            .topic0(H256::from(event_signature))
            .from_block(start_block))
    }
}

/// Monitor a specific event stream
async fn monitor_event_stream<F>(
    provider: Arc<Provider<Http>>,
    filter: Filter,
    sender: mpsc::UnboundedSender<BridgeEvent>,
    confirmations: u64,
    event_name: &str,
    parser: F,
) -> Result<(), EthereumError>
where
    F: Fn(&Log) -> Result<BridgeEvent, EthereumError> + Send + Sync + 'static,
{
    info!(event_name = event_name, "Starting event stream monitor");

    let mut stream = provider.watch(&filter).await
        .map_err(|e| EthereumError::NetworkError(e.to_string()))?
        .stream();

    while let Some(log) = stream.next().await {
        debug!(
            event_name = event_name,
            tx_hash = %log.transaction_hash.unwrap_or_default(),
            block_number = ?log.block_number,
            "Processing event log"
        );

        // Wait for confirmations
        if let Some(block_number) = log.block_number {
            let current_block = match provider.get_block_number().await {
                Ok(block) => block.as_u64(),
                Err(e) => {
                    warn!(error = %e, "Failed to get current block number");
                    continue;
                }
            };

            let block_confirmations = current_block.saturating_sub(block_number.as_u64());
            if block_confirmations < confirmations {
                debug!(
                    block_number = block_number.as_u64(),
                    current_block,
                    confirmations = block_confirmations,
                    required_confirmations = confirmations,
                    "Waiting for more confirmations"
                );
                continue;
            }
        }

        // Parse the event
        match parser(&log) {
            Ok(bridge_event) => {
                info!(
                    event_name = event_name,
                    tx_hash = %log.transaction_hash.unwrap_or_default(),
                    "Successfully parsed bridge event"
                );

                // Send to bridge service
                if let Err(e) = sender.send(bridge_event) {
                    error!(error = %e, event_name = event_name, "Failed to send bridge event");
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    event_name = event_name,
                    tx_hash = %log.transaction_hash.unwrap_or_default(),
                    "Failed to parse event log"
                );
            }
        }
    }

    warn!(event_name = event_name, "Event stream ended");
    Ok(())
}

/// Parse TokensLocked event from log
fn parse_tokens_locked_event(log: &Log) -> Result<BridgeEvent, EthereumError> {
    // TokensLocked(address indexed user, uint256 amount, string recipientChain, string quantumHash)
    
    if log.topics.len() < 2 {
        return Err(EthereumError::InvalidTransaction("Insufficient topics in TokensLocked event".to_string()));
    }

    // Extract indexed user address from topic[1]
    let user_address = Address::from(log.topics[1]);

    // Decode non-indexed parameters from data
    let event = Event {
        name: "TokensLocked".to_string(),
        inputs: vec![
            EventParam {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "recipientChain".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
            EventParam {
                name: "quantumHash".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
        ],
        anonymous: false,
    };

    let tokens = event.parse_log(ethers::abi::RawLog {
        topics: log.topics.clone(),
        data: log.data.to_vec(),
    }).map_err(|e| EthereumError::InvalidTransaction(e.to_string()))?;

    let amount = match &tokens.params[0].value {
        Token::Uint(val) => *val,
        _ => return Err(EthereumError::InvalidTransaction("Invalid amount in TokensLocked event".to_string())),
    };

    let recipient_chain = match &tokens.params[1].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid recipientChain in TokensLocked event".to_string())),
    };

    let quantum_hash = match &tokens.params[2].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid quantumHash in TokensLocked event".to_string())),
    };

    Ok(BridgeEvent::TokensLocked {
        user_address,
        amount,
        recipient_chain,
        quantum_hash,
        tx_hash: log.transaction_hash.unwrap_or_default(),
        block_number: log.block_number.map(|b| b.as_u64()).unwrap_or_default(),
    })
}

/// Parse TokensUnlocked event from log
fn parse_tokens_unlocked_event(log: &Log) -> Result<BridgeEvent, EthereumError> {
    // TokensUnlocked(address indexed recipient, uint256 amount, string sourceChain, string quantumHash)
    
    if log.topics.len() < 2 {
        return Err(EthereumError::InvalidTransaction("Insufficient topics in TokensUnlocked event".to_string()));
    }

    // Extract indexed recipient address from topic[1]
    let recipient = Address::from(log.topics[1]);

    // Decode non-indexed parameters from data
    let event = Event {
        name: "TokensUnlocked".to_string(),
        inputs: vec![
            EventParam {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "sourceChain".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
            EventParam {
                name: "quantumHash".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
        ],
        anonymous: false,
    };

    let tokens = event.parse_log(ethers::abi::RawLog {
        topics: log.topics.clone(),
        data: log.data.to_vec(),
    }).map_err(|e| EthereumError::InvalidTransaction(e.to_string()))?;

    let amount = match &tokens.params[0].value {
        Token::Uint(val) => *val,
        _ => return Err(EthereumError::InvalidTransaction("Invalid amount in TokensUnlocked event".to_string())),
    };

    let source_chain = match &tokens.params[1].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid sourceChain in TokensUnlocked event".to_string())),
    };

    let quantum_hash = match &tokens.params[2].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid quantumHash in TokensUnlocked event".to_string())),
    };

    Ok(BridgeEvent::TokensUnlocked {
        recipient,
        amount,
        source_chain,
        quantum_hash,
        tx_hash: log.transaction_hash.unwrap_or_default(),
        block_number: log.block_number.map(|b| b.as_u64()).unwrap_or_default(),
    })
}

/// Parse BridgeDeposit event from log
fn parse_bridge_deposit_event(log: &Log) -> Result<BridgeEvent, EthereumError> {
    // BridgeDeposit(address indexed user, uint256 amount, string destinationChain)
    
    if log.topics.len() < 2 {
        return Err(EthereumError::InvalidTransaction("Insufficient topics in BridgeDeposit event".to_string()));
    }

    // Extract indexed user address from topic[1]
    let user_address = Address::from(log.topics[1]);

    // Decode non-indexed parameters from data
    let event = Event {
        name: "BridgeDeposit".to_string(),
        inputs: vec![
            EventParam {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "destinationChain".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
        ],
        anonymous: false,
    };

    let tokens = event.parse_log(ethers::abi::RawLog {
        topics: log.topics.clone(),
        data: log.data.to_vec(),
    }).map_err(|e| EthereumError::InvalidTransaction(e.to_string()))?;

    let amount = match &tokens.params[0].value {
        Token::Uint(val) => *val,
        _ => return Err(EthereumError::InvalidTransaction("Invalid amount in BridgeDeposit event".to_string())),
    };

    let destination_chain = match &tokens.params[1].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid destinationChain in BridgeDeposit event".to_string())),
    };

    Ok(BridgeEvent::BridgeDeposit {
        user_address,
        amount,
        destination_chain,
        tx_hash: log.transaction_hash.unwrap_or_default(),
        block_number: log.block_number.map(|b| b.as_u64()).unwrap_or_default(),
    })
}

/// Parse BridgeWithdrawal event from log
fn parse_bridge_withdrawal_event(log: &Log) -> Result<BridgeEvent, EthereumError> {
    // BridgeWithdrawal(address indexed recipient, uint256 amount, string sourceChain)
    
    if log.topics.len() < 2 {
        return Err(EthereumError::InvalidTransaction("Insufficient topics in BridgeWithdrawal event".to_string()));
    }

    // Extract indexed recipient address from topic[1]
    let recipient = Address::from(log.topics[1]);

    // Decode non-indexed parameters from data
    let event = Event {
        name: "BridgeWithdrawal".to_string(),
        inputs: vec![
            EventParam {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "sourceChain".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
        ],
        anonymous: false,
    };

    let tokens = event.parse_log(ethers::abi::RawLog {
        topics: log.topics.clone(),
        data: log.data.to_vec(),
    }).map_err(|e| EthereumError::InvalidTransaction(e.to_string()))?;

    let amount = match &tokens.params[0].value {
        Token::Uint(val) => *val,
        _ => return Err(EthereumError::InvalidTransaction("Invalid amount in BridgeWithdrawal event".to_string())),
    };

    let source_chain = match &tokens.params[1].value {
        Token::String(val) => val.clone(),
        _ => return Err(EthereumError::InvalidTransaction("Invalid sourceChain in BridgeWithdrawal event".to_string())),
    };

    Ok(BridgeEvent::BridgeWithdrawal {
        recipient,
        amount,
        source_chain,
        tx_hash: log.transaction_hash.unwrap_or_default(),
        block_number: log.block_number.map(|b| b.as_u64()).unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{Bytes, H160};

    #[test]
    fn test_event_listener_config_default() {
        let config = EventListenerConfig::default();
        assert_eq!(config.bridge_contract_address, Address::zero());
        assert_eq!(config.confirmation_blocks, 12);
        assert!(config.start_block.is_none());
        assert!(config.event_filter.is_none());
    }

    #[test]
    fn test_bridge_event_serialization() {
        let event = BridgeEvent::TokensLocked {
            user_address: H160::zero(),
            amount: U256::from(1000),
            recipient_chain: "near".to_string(),
            quantum_hash: "test_hash".to_string(),
            tx_hash: H256::zero(),
            block_number: 12345,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let _deserialized: BridgeEvent = serde_json::from_str(&serialized).unwrap();
    }
}