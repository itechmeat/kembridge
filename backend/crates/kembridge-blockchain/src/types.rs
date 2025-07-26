use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Near,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHash {
    pub hash: String,
    pub chain: ChainType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub gas_price: Option<String>,
}