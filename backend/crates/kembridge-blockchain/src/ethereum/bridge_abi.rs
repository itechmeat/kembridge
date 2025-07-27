// SimpleBridge Contract ABI for Ethereum Integration
use ethers::abi::{Abi, Event, EventParam, Function, Param, ParamType, StateMutability};
use ethers::types::Address;

/// Get the SimpleBridge contract ABI
pub fn get_bridge_abi() -> Abi {
    let abi_json = r#"[
        {
            "inputs": [],
            "stateMutability": "nonpayable",
            "type": "constructor"
        },
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "user",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "destinationChain",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                }
            ],
            "name": "BridgeDeposit",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "recipient",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "sourceChain",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                }
            ],
            "name": "BridgeWithdrawal",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "user",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "recipientChain",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "quantumHash",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                }
            ],
            "name": "TokensLocked",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "recipient",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "sourceChain",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "string",
                    "name": "quantumHash",
                    "type": "string"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "timestamp",
                    "type": "uint256"
                }
            ],
            "name": "TokensUnlocked",
            "type": "event"
        },
        {
            "inputs": [
                {
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                }
            ],
            "name": "emergencyWithdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getBalance",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getBridgeStats",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "balance",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "locked",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "unlocked",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "activeBalance",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "user",
                    "type": "address"
                }
            ],
            "name": "getUserBalance",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "bytes32",
                    "name": "hash",
                    "type": "bytes32"
                }
            ],
            "name": "isProcessed",
            "outputs": [
                {
                    "internalType": "bool",
                    "name": "",
                    "type": "bool"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "string",
                    "name": "recipientChain",
                    "type": "string"
                },
                {
                    "internalType": "string",
                    "name": "quantumHash",
                    "type": "string"
                }
            ],
            "name": "lockTokens",
            "outputs": [],
            "stateMutability": "payable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "MAX_LOCK_AMOUNT",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "MIN_LOCK_AMOUNT",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "owner",
            "outputs": [
                {
                    "internalType": "address",
                    "name": "",
                    "type": "address"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "bytes32",
                    "name": "",
                    "type": "bytes32"
                }
            ],
            "name": "processedHashes",
            "outputs": [
                {
                    "internalType": "bool",
                    "name": "",
                    "type": "bool"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalLocked",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalUnlocked",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "newOwner",
                    "type": "address"
                }
            ],
            "name": "transferOwnership",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address payable",
                    "name": "recipient",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "amount",
                    "type": "uint256"
                },
                {
                    "internalType": "string",
                    "name": "sourceChain",
                    "type": "string"
                },
                {
                    "internalType": "string",
                    "name": "quantumHash",
                    "type": "string"
                }
            ],
            "name": "unlockTokens",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "",
                    "type": "address"
                }
            ],
            "name": "userBalances",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "stateMutability": "payable",
            "type": "receive"
        },
        {
            "stateMutability": "payable",
            "type": "fallback"
        }
    ]"#;

    serde_json::from_str(abi_json).expect("Invalid ABI JSON")
}

/// Get the lockTokens function signature
pub fn lock_tokens_function() -> Function {
    Function {
        name: "lockTokens".to_string(),
        inputs: vec![
            Param {
                name: "recipientChain".to_string(),
                kind: ParamType::String,
                internal_type: Some("string".to_string()),
            },
            Param {
                name: "quantumHash".to_string(),
                kind: ParamType::String,
                internal_type: Some("string".to_string()),
            },
        ],
        outputs: vec![],
        constant: Some(false),
        state_mutability: StateMutability::Payable,
    }
}

/// Get the unlockTokens function signature
pub fn unlock_tokens_function() -> Function {
    Function {
        name: "unlockTokens".to_string(),
        inputs: vec![
            Param {
                name: "recipient".to_string(),
                kind: ParamType::Address,
                internal_type: Some("address".to_string()),
            },
            Param {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "sourceChain".to_string(),
                kind: ParamType::String,
                internal_type: Some("string".to_string()),
            },
            Param {
                name: "quantumHash".to_string(),
                kind: ParamType::String,
                internal_type: Some("string".to_string()),
            },
        ],
        outputs: vec![],
        constant: Some(false),
        state_mutability: StateMutability::NonPayable,
    }
}

/// Get the getBridgeStats function signature
pub fn get_bridge_stats_function() -> Function {
    Function {
        name: "getBridgeStats".to_string(),
        inputs: vec![],
        outputs: vec![
            Param {
                name: "balance".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "locked".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "unlocked".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
            Param {
                name: "activeBalance".to_string(),
                kind: ParamType::Uint(256),
                internal_type: Some("uint256".to_string()),
            },
        ],
        constant: Some(true),
        state_mutability: StateMutability::View,
    }
}

/// Bridge contract constants
pub struct BridgeConstants;

impl BridgeConstants {
    /// Minimum lock amount in wei (0.001 ETH)
    pub const MIN_LOCK_AMOUNT: u64 = 1_000_000_000_000_000; // 0.001 ETH
    
    /// Maximum lock amount in wei (10 ETH)
    pub const MAX_LOCK_AMOUNT: u64 = 10_000_000_000_000_000_000; // 10 ETH
    
    /// Function selectors for quick access
    pub const LOCK_TOKENS_SELECTOR: [u8; 4] = [0x6f, 0x34, 0x7a, 0x2e]; // keccak256("lockTokens(string,string)")
    pub const UNLOCK_TOKENS_SELECTOR: [u8; 4] = [0x7c, 0x5c, 0x5d, 0x8e]; // keccak256("unlockTokens(address,uint256,string,string)")
    pub const GET_BRIDGE_STATS_SELECTOR: [u8; 4] = [0x9f, 0x2f, 0x44, 0x0d]; // keccak256("getBridgeStats()")
}

/// Bridge contract deployment information
pub struct BridgeDeployment {
    /// Contract address on Sepolia testnet
    pub address: Address,
    /// Block number when contract was deployed
    pub deployed_block: u64,
    /// Transaction hash of deployment
    pub deployment_tx: Option<String>,
}

impl BridgeDeployment {
    /// Create a new bridge deployment info
    pub fn new(address: Address, deployed_block: u64, deployment_tx: Option<String>) -> Self {
        Self {
            address,
            deployed_block,
            deployment_tx,
        }
    }
    
    /// Get the contract address as a string
    pub fn address_string(&self) -> String {
        format!("{:?}", self.address)
    }
}