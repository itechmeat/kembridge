use near_sdk::{near, env, AccountId, PanicOnDefault, Promise, NearToken};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::{IterableMap, IterableSet};
use near_sdk::json_types::U128;

pub type Balance = u128;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct BridgeConfig {
    pub min_bridge_amount: Balance,  // 0.1 NEAR = 100000000000000000000000
    pub max_bridge_amount: Balance,  // 10 NEAR = 10000000000000000000000000
    pub bridge_fee_bp: u16,          // 50 basis points = 0.5%
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct BridgeStats {
    pub total_locked: Balance,
    pub total_unlocked: Balance,
    pub total_minted: Balance,
    pub total_burned: Balance,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LockEvent {
    pub user: AccountId,
    pub amount: U128,
    pub eth_recipient: String,
    pub quantum_hash: String,
    pub fee: U128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnlockEvent {
    pub near_recipient: AccountId,
    pub amount: U128,
    pub eth_tx_hash: String,
    pub quantum_hash: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MintEvent {
    pub recipient: AccountId,
    pub amount: U128,
    pub eth_tx_hash: String,
    pub quantum_hash: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BurnEvent {
    pub user: AccountId,
    pub amount: U128,
    pub eth_recipient: String,
    pub quantum_hash: String,
    pub fee: U128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EmergencyWithdrawEvent {
    pub owner: AccountId,
    pub amount: U128,
    pub timestamp: u64,
    pub reason: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct BridgeContract {
    owner: AccountId,
    paused: bool,
    fee_wallet: AccountId,
    config: BridgeConfig,
    bridge_stats: BridgeStats,
    user_balances: IterableMap<AccountId, Balance>,
    processed_eth_txs: IterableSet<String>,
}

#[near]
impl BridgeContract {
    #[init]
    pub fn new(owner: AccountId, fee_wallet: AccountId) -> Self {
        Self {
            owner,
            paused: false,
            fee_wallet,
            config: BridgeConfig {
                min_bridge_amount: 100_000_000_000_000_000_000_000,  // 0.1 NEAR
                max_bridge_amount: 10_000_000_000_000_000_000_000_000, // 10 NEAR
                bridge_fee_bp: 50,  // 0.5%
            },
            bridge_stats: BridgeStats {
                total_locked: 0,
                total_unlocked: 0,
                total_minted: 0,
                total_burned: 0,
            },
            user_balances: IterableMap::new(b"u"),
            processed_eth_txs: IterableSet::new(b"p"),
        }
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn transfer_ownership(&mut self, new_owner: AccountId) {
        self.assert_owner();
        self.owner = new_owner;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.assert_owner();
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_config(&self) -> BridgeConfig {
        BridgeConfig {
            min_bridge_amount: self.config.min_bridge_amount,
            max_bridge_amount: self.config.max_bridge_amount,
            bridge_fee_bp: self.config.bridge_fee_bp,
        }
    }

    pub fn update_config(&mut self, min_bridge_amount: Balance, max_bridge_amount: Balance, bridge_fee_bp: u16) {
        self.assert_owner();
        
        // Validate parameters
        assert!(min_bridge_amount > 0, "Minimum bridge amount must be positive");
        assert!(max_bridge_amount > min_bridge_amount, "Maximum must be greater than minimum");
        assert!(bridge_fee_bp <= 10000, "Fee cannot exceed 100% (10000 basis points)");
        
        self.config.min_bridge_amount = min_bridge_amount;
        self.config.max_bridge_amount = max_bridge_amount;
        self.config.bridge_fee_bp = bridge_fee_bp;
    }

    pub fn get_bridge_stats(&self) -> BridgeStats {
        BridgeStats {
            total_locked: self.bridge_stats.total_locked,
            total_unlocked: self.bridge_stats.total_unlocked,
            total_minted: self.bridge_stats.total_minted,
            total_burned: self.bridge_stats.total_burned,
        }
    }

    pub fn get_locked_balance(&self, account: AccountId) -> Balance {
        self.user_balances.get(&account).copied().unwrap_or(0)
    }

    pub fn is_eth_tx_processed(&self, eth_tx_hash: String) -> bool {
        self.processed_eth_txs.contains(&eth_tx_hash)
    }

    #[payable]
    pub fn lock_tokens(&mut self, eth_recipient: String, quantum_hash: String) -> Promise {
        assert!(!self.paused, "Contract is paused");
        
        let amount = env::attached_deposit().as_yoctonear();
        self.validate_amount(amount);
        
        let fee = self.calculate_fee(amount);
        let lock_amount = amount - fee;
        
        // Update user balance
        let user = env::signer_account_id();
        let current_balance = self.user_balances.get(&user).copied().unwrap_or(0);
        self.user_balances.insert(user.clone(), current_balance + lock_amount);
        
        // Update bridge stats
        self.bridge_stats.total_locked += lock_amount;
        
        // Emit lock event
        let event = LockEvent {
            user: user.clone(),
            amount: U128(amount),
            eth_recipient: eth_recipient.clone(),
            quantum_hash: quantum_hash.clone(),
            fee: U128(fee),
            timestamp: env::block_timestamp(),
        };
        
        env::log_str(&format!("EVENT_JSON:{}", 
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
        
        // Transfer fee to fee_wallet
        Promise::new(self.fee_wallet.clone()).transfer(NearToken::from_yoctonear(fee))
    }

    fn validate_amount(&self, amount: Balance) {
        assert!(amount >= self.config.min_bridge_amount, 
            "Amount below minimum: {} NEAR", 
            self.config.min_bridge_amount as f64 / 1e24
        );
        assert!(amount <= self.config.max_bridge_amount, 
            "Amount above maximum: {} NEAR", 
            self.config.max_bridge_amount as f64 / 1e24
        );
        assert!(amount > 0, "Amount must be positive");
    }

    fn calculate_fee(&self, amount: Balance) -> Balance {
        (amount * self.config.bridge_fee_bp as u128) / 10000
    }

    pub fn unlock_tokens(
        &mut self, 
        amount: U128, 
        near_recipient: AccountId, 
        eth_tx_hash: String,
        quantum_hash: String
    ) -> Promise {
        self.assert_owner();
        assert!(!self.paused, "Contract is paused");
        
        let amount_balance = amount.0;
        self.validate_amount(amount_balance);
        
        // Check replay protection
        assert!(!self.is_eth_tx_processed(eth_tx_hash.clone()), 
            "Ethereum transaction already processed: {}", eth_tx_hash);
        
        // Mark transaction as processed
        self.processed_eth_txs.insert(eth_tx_hash.clone());
        
        // Check if contract has enough locked balance
        let current_locked = self.user_balances.get(&near_recipient).copied().unwrap_or(0);
        assert!(current_locked >= amount_balance, 
            "Insufficient locked balance. Available: {}, requested: {}", 
            current_locked, amount_balance);
        
        // Update user balance (decrease locked amount)
        self.user_balances.insert(near_recipient.clone(), current_locked - amount_balance);
        
        // Update bridge stats
        self.bridge_stats.total_unlocked += amount_balance;
        
        // Emit unlock event
        let event = UnlockEvent {
            near_recipient: near_recipient.clone(),
            amount,
            eth_tx_hash: eth_tx_hash.clone(),
            quantum_hash: quantum_hash.clone(),
            timestamp: env::block_timestamp(),
        };
        
        env::log_str(&format!("EVENT_JSON:{}", 
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
        
        // Transfer unlocked tokens to recipient
        Promise::new(near_recipient).transfer(NearToken::from_yoctonear(amount_balance))
    }



    pub fn mark_eth_tx_processed(&mut self, eth_tx_hash: String) {
        self.assert_owner();
        self.processed_eth_txs.insert(eth_tx_hash);
    }

    pub fn mint_tokens(
        &mut self,
        recipient: AccountId,
        amount: U128,
        eth_tx_hash: String,
        quantum_hash: String
    ) -> Promise {
        self.assert_owner();
        assert!(!self.paused, "Contract is paused");
        
        let amount_balance = amount.0;
        self.validate_amount(amount_balance);
        
        // Check replay protection
        assert!(!self.is_eth_tx_processed(eth_tx_hash.clone()), 
            "Ethereum transaction already processed: {}", eth_tx_hash);
        
        // Mark transaction as processed
        self.processed_eth_txs.insert(eth_tx_hash.clone());
        
        // Update user balance (add minted tokens)
        let current_balance = self.user_balances.get(&recipient).copied().unwrap_or(0);
        self.user_balances.insert(recipient.clone(), current_balance + amount_balance);
        
        // Update bridge stats
        self.bridge_stats.total_minted += amount_balance;
        
        // Emit mint event
        let event = MintEvent {
            recipient: recipient.clone(),
            amount,
            eth_tx_hash: eth_tx_hash.clone(),
            quantum_hash: quantum_hash.clone(),
            timestamp: env::block_timestamp(),
        };
        
        env::log_str(&format!("EVENT_JSON:{}", 
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
        
        // Transfer minted tokens to recipient
        Promise::new(recipient).transfer(NearToken::from_yoctonear(amount_balance))
    }

    #[payable]
    pub fn burn_tokens(&mut self, eth_recipient: String, quantum_hash: String) -> Promise {
        assert!(!self.paused, "Contract is paused");
        
        let amount = env::attached_deposit().as_yoctonear();
        self.validate_amount(amount);
        
        let fee = self.calculate_fee(amount);
        let burn_amount = amount - fee;
        
        // Update user balance (decrease burned tokens)
        let user = env::signer_account_id();
        let current_balance = self.user_balances.get(&user).copied().unwrap_or(0);
        assert!(current_balance >= burn_amount, 
            "Insufficient balance for burn. Available: {}, requested: {}", 
            current_balance, burn_amount);
        
        self.user_balances.insert(user.clone(), current_balance - burn_amount);
        
        // Update bridge stats
        self.bridge_stats.total_burned += burn_amount;
        
        // Emit burn event
        let event = BurnEvent {
            user: user.clone(),
            amount: U128(amount),
            eth_recipient: eth_recipient.clone(),
            quantum_hash: quantum_hash.clone(),
            fee: U128(fee),
            timestamp: env::block_timestamp(),
        };
        
        env::log_str(&format!("EVENT_JSON:{}", 
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
        
        // Transfer fee to fee_wallet
        Promise::new(self.fee_wallet.clone()).transfer(NearToken::from_yoctonear(fee))
    }

    pub fn emergency_withdraw(&mut self, amount: U128) -> Promise {
        self.assert_owner();
        
        let amount_balance = amount.0;
        let contract_balance = env::account_balance().as_yoctonear();
        
        assert!(amount_balance > 0, "Amount must be positive");
        assert!(amount_balance <= contract_balance, 
            "Insufficient contract balance. Available: {}, requested: {}", 
            contract_balance, amount_balance);
        
        // Emit emergency withdraw event
        let event = EmergencyWithdrawEvent {
            owner: env::signer_account_id(),
            amount,
            timestamp: env::block_timestamp(),
            reason: "Emergency withdrawal by owner".to_string(),
        };
        
        env::log_str(&format!("EVENT_JSON:{}", 
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
        
        // Transfer amount to owner
        Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(amount_balance))
    }

    pub fn get_contract_balance(&self) -> U128 {
        U128(env::account_balance().as_yoctonear())
    }

    pub fn get_total_bridge_volume(&self) -> U128 {
        U128(self.bridge_stats.total_locked + self.bridge_stats.total_minted)
    }

    pub fn set_fee_wallet(&mut self, new_fee_wallet: AccountId) {
        self.assert_owner();
        self.fee_wallet = new_fee_wallet;
    }

    pub fn get_fee_wallet(&self) -> AccountId {
        self.fee_wallet.clone()
    }

    // Helper method for owner checks
    fn assert_owner(&self) {
        assert_eq!(
            env::signer_account_id(),
            self.owner,
            "Only owner can call this method"
        );
    }

    // Temporary method to reset state when migrating contract structure
    #[init(ignore_state)]
    pub fn migrate(owner: AccountId, fee_wallet: Option<AccountId>) -> Self {
        Self {
            owner: owner.clone(),
            paused: false,
            fee_wallet: fee_wallet.unwrap_or(owner),
            config: BridgeConfig {
                min_bridge_amount: 100_000_000_000_000_000_000_000,  // 0.1 NEAR
                max_bridge_amount: 10_000_000_000_000_000_000_000_000, // 10 NEAR
                bridge_fee_bp: 50,  // 0.5%
            },
            bridge_stats: BridgeStats {
                total_locked: 0,
                total_unlocked: 0,
                total_minted: 0,
                total_burned: 0,
            },
            user_balances: IterableMap::new(b"u"),
            processed_eth_txs: IterableSet::new(b"p"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    #[test]
    fn test_new_contract() {
        let owner: AccountId = "alice.testnet".parse().unwrap();
        let contract = BridgeContract::new(owner.clone(), accounts(1));
        assert_eq!(contract.get_owner(), owner);
        assert_eq!(contract.is_paused(), false);
    }

    #[test]
    fn test_transfer_ownership() {
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());

        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        contract.transfer_ownership(accounts(1));
        assert_eq!(contract.get_owner(), accounts(1));
    }

    #[test]
    fn test_set_paused() {
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());

        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Initially not paused
        assert_eq!(contract.is_paused(), false);
        
        // Pause the contract
        contract.set_paused(true);
        assert_eq!(contract.is_paused(), true);
        
        // Unpause the contract
        contract.set_paused(false);
        assert_eq!(contract.is_paused(), false);
    }

    #[test]
    #[should_panic(expected = "Only owner can call this method")]
    fn test_transfer_ownership_not_owner() {
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(1)); // Not the owner
        testing_env!(context.build());

        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        contract.transfer_ownership(accounts(2)); // Should panic
    }

    #[test]
    fn test_get_config() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        let config = contract.get_config();
        
        assert_eq!(config.min_bridge_amount, 100_000_000_000_000_000_000_000); // 0.1 NEAR
        assert_eq!(config.max_bridge_amount, 10_000_000_000_000_000_000_000_000); // 10 NEAR
        assert_eq!(config.bridge_fee_bp, 50); // 0.5%
    }

    #[test]
    fn test_update_config() {
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());

        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Update config
        contract.update_config(
            200_000_000_000_000_000_000_000, // 0.2 NEAR
            20_000_000_000_000_000_000_000_000, // 20 NEAR
            75 // 0.75%
        );
        
        let config = contract.get_config();
        assert_eq!(config.min_bridge_amount, 200_000_000_000_000_000_000_000);
        assert_eq!(config.max_bridge_amount, 20_000_000_000_000_000_000_000_000);
        assert_eq!(config.bridge_fee_bp, 75);
    }

    #[test]
    #[should_panic(expected = "Maximum must be greater than minimum")]
    fn test_update_config_invalid_range() {
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());

        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Try to set max less than min - should panic
        contract.update_config(
            1_000_000_000_000_000_000_000_000, // 1 NEAR
            500_000_000_000_000_000_000_000,   // 0.5 NEAR
            50
        );
    }

    #[test]
    fn test_get_bridge_stats() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        let stats = contract.get_bridge_stats();
        
        assert_eq!(stats.total_locked, 0);
        assert_eq!(stats.total_unlocked, 0);
        assert_eq!(stats.total_minted, 0);
        assert_eq!(stats.total_burned, 0);
    }

    #[test]
    fn test_get_locked_balance() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Non-existent account should return 0
        assert_eq!(contract.get_locked_balance(accounts(1)), 0);
    }

    #[test]
    fn test_is_eth_tx_processed() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Non-existent transaction should return false
        assert_eq!(contract.is_eth_tx_processed("0x123".to_string()), false);
    }

    #[test]
    fn test_calculate_fee() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Test fee calculation: 50 bp = 0.5%
        let amount = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        let fee = contract.calculate_fee(amount);
        assert_eq!(fee, 5_000_000_000_000_000_000_000); // 0.005 NEAR
    }

    #[test]
    fn test_validate_amount() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Valid amount should not panic
        contract.validate_amount(1_000_000_000_000_000_000_000_000); // 1 NEAR
    }

    #[test]
    #[should_panic(expected = "Amount below minimum")]
    fn test_validate_amount_too_small() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Amount below minimum should panic
        contract.validate_amount(50_000_000_000_000_000_000_000); // 0.05 NEAR
    }

    #[test]
    #[should_panic(expected = "Amount above maximum")]
    fn test_validate_amount_too_large() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Amount above maximum should panic
        contract.validate_amount(20_000_000_000_000_000_000_000_000); // 20 NEAR
    }

    #[test]
    fn test_replay_protection() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Non-processed transaction should return false
        assert_eq!(contract.is_eth_tx_processed("0x123".to_string()), false);
    }

    #[test]
    fn test_mark_eth_tx_processed() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        // Initially not processed
        assert_eq!(contract.is_eth_tx_processed("0x456".to_string()), false);
        
        // Mark as processed
        contract.mark_eth_tx_processed("0x456".to_string());
        
        // Should now be processed
        assert_eq!(contract.is_eth_tx_processed("0x456".to_string()), true);
    }

    #[test]
    fn test_mint_tokens() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        let recipient = accounts(1);
        let amount = U128(1_000_000_000_000_000_000_000_000); // 1 NEAR
        let eth_tx_hash = "0xmint123".to_string();
        
        // Initially no balance
        assert_eq!(contract.get_locked_balance(recipient.clone()), 0);
        
        // Mint tokens
        contract.mint_tokens(recipient.clone(), amount, eth_tx_hash.clone(), "qhash_mint".to_string());
        
        // Check balance updated
        assert_eq!(contract.get_locked_balance(recipient), amount.0);
        
        // Check stats updated
        let stats = contract.get_bridge_stats();
        assert_eq!(stats.total_minted, amount.0);
        
        // Check transaction marked as processed
        assert_eq!(contract.is_eth_tx_processed(eth_tx_hash), true);
    }

    #[test]
    #[should_panic(expected = "Ethereum transaction already processed")]
    fn test_mint_tokens_replay_protection() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        let recipient = accounts(1);
        let amount = U128(1_000_000_000_000_000_000_000_000);
        let eth_tx_hash = "0xmint456".to_string();
        
        // Mint once
        contract.mint_tokens(recipient.clone(), amount, eth_tx_hash.clone(), "qhash1".to_string());
        
        // Try to mint again with same eth_tx_hash - should panic
        contract.mint_tokens(recipient, amount, eth_tx_hash, "qhash2".to_string());
    }

    #[test]
    fn test_get_contract_balance() {
        let contract = BridgeContract::new(accounts(0), accounts(1));
        
        // Should return some balance (test environment usually has balance)
        let balance = contract.get_contract_balance();
        // Balance is always non-negative (u128), just check it exists
        let _ = balance;
    }

    #[test]
    fn test_get_total_bridge_volume() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        // Initially zero
        assert_eq!(contract.get_total_bridge_volume().0, 0);
        
        // Add some locked and minted amounts
        contract.bridge_stats.total_locked = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        contract.bridge_stats.total_minted = 2_000_000_000_000_000_000_000_000; // 2 NEAR
        
        // Should return sum
        assert_eq!(contract.get_total_bridge_volume().0, 3_000_000_000_000_000_000_000_000); // 3 NEAR
    }

    #[test]
    #[should_panic(expected = "Amount must be positive")]
    fn test_emergency_withdraw_zero_amount() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        // Should panic with zero amount
        contract.emergency_withdraw(U128(0));
    }

    #[test]
    #[should_panic(expected = "Only owner can call this method")]
    fn test_emergency_withdraw_not_owner() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(1)); // Different account
        testing_env!(context.build());
        
        // Should panic when called by non-owner
        contract.emergency_withdraw(U128(1000));
    }

    #[test]
    fn test_fee_wallet_management() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(0));
        testing_env!(context.build());
        
        // Initially set to accounts(1)
        assert_eq!(contract.get_fee_wallet(), accounts(1));
        
        // Change fee wallet
        contract.set_fee_wallet(accounts(2));
        assert_eq!(contract.get_fee_wallet(), accounts(2));
    }

    #[test]
    #[should_panic(expected = "Only owner can call this method")]
    fn test_set_fee_wallet_not_owner() {
        let mut contract = BridgeContract::new(accounts(0), accounts(1));
        let mut context = VMContextBuilder::new();
        context.signer_account_id(accounts(1)); // Different account
        testing_env!(context.build());
        
        // Should panic when called by non-owner
        contract.set_fee_wallet(accounts(2));
    }
}