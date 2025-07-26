// Phase 4.1: Wallet types and transaction status
use ethers::{
    types::{Address, U256, Transaction, TransactionReceipt},
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub address: Address,
    pub eth_balance: U256,
    pub nonce: U256,
    pub token_balances: Vec<TokenBalance>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token_address: Address,
    pub balance: U256,
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    NotFound,
    Pending { transaction: Transaction },
    Confirmed { receipt: TransactionReceipt, confirmations: u64 },
}