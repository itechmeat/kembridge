// NEAR wallet management functionality
// Phase 4.2: NEAR Protocol Adapter

use crate::near::{NearError, Result};
use near_crypto::{PublicKey, SecretKey};
// Simplified wallet without near-primitives dependency
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// NEAR wallet representation
#[derive(Debug, Clone)]
pub struct NearWallet {
    account_id: String,
    public_key: PublicKey,
    secret_key: Option<SecretKey>, // None for view-only wallets
}

/// Wallet creation options
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletOptions {
    pub account_id: String,
    pub public_key: Option<String>,
    pub secret_key: Option<String>,
}

impl NearWallet {
    /// Create a new wallet from account ID and keys
    pub fn new(
        account_id: String,
        public_key: PublicKey,
        secret_key: Option<SecretKey>,
    ) -> Result<Self> {
        // Basic validation
        if account_id.is_empty() {
            return Err(NearError::InvalidAccountId("Account ID cannot be empty".to_string()));
        }

        Ok(Self {
            account_id,
            public_key,
            secret_key,
        })
    }

    /// Create wallet from options
    pub fn from_options(options: WalletOptions) -> Result<Self> {
        let account_id = options.account_id;

        let public_key = if let Some(pk_str) = options.public_key {
            PublicKey::from_str(&pk_str)
                .map_err(|e| NearError::CryptoError(format!("Invalid public key: {}", e)))?
        } else {
            return Err(NearError::WalletError("Public key is required".to_string()));
        };

        let secret_key = if let Some(sk_str) = options.secret_key {
            Some(SecretKey::from_str(&sk_str)
                .map_err(|e| NearError::CryptoError(format!("Invalid secret key: {}", e)))?)
        } else {
            None
        };

        Ok(Self {
            account_id,
            public_key,
            secret_key,
        })
    }

    /// Generate a new random wallet (for testing purposes)
    #[cfg(test)]
    pub fn generate_random(account_id: String) -> Result<Self> {
        let secret_key = SecretKey::from_random(near_crypto::KeyType::ED25519);
        let public_key = secret_key.public_key();
        
        Self::new(account_id, public_key, Some(secret_key))
    }

    /// Get account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Get public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Check if wallet can sign transactions
    pub fn can_sign(&self) -> bool {
        self.secret_key.is_some()
    }

    /// Get secret key (if available)
    pub fn secret_key(&self) -> Option<&SecretKey> {
        self.secret_key.as_ref()
    }

    /// Sign data with the wallet's secret key
    pub fn sign_data(&self, data: &[u8]) -> Result<near_crypto::Signature> {
        match &self.secret_key {
            Some(sk) => Ok(sk.sign(data)),
            None => Err(NearError::WalletError("No secret key available for signing".to_string())),
        }
    }

    /// Verify signature against this wallet's public key
    pub fn verify_signature(&self, data: &[u8], signature: &near_crypto::Signature) -> bool {
        signature.verify(data, &self.public_key)
    }

    /// Convert to view-only wallet (removes secret key)
    pub fn to_view_only(&self) -> Self {
        Self {
            account_id: self.account_id.clone(),
            public_key: self.public_key.clone(),
            secret_key: None,
        }
    }
}

impl std::fmt::Display for NearWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NearWallet {{ account_id: {}, public_key: {}, can_sign: {} }}",
            self.account_id,
            self.public_key,
            self.can_sign()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = NearWallet::generate_random("test.testnet".to_string()).unwrap();
        assert_eq!(wallet.account_id(), "test.testnet");
        assert!(wallet.can_sign());
    }

    #[test]
    fn test_view_only_wallet() {
        let wallet = NearWallet::generate_random("test.testnet".to_string()).unwrap();
        let view_only = wallet.to_view_only();
        
        assert_eq!(view_only.account_id(), wallet.account_id());
        assert_eq!(view_only.public_key(), wallet.public_key());
        assert!(!view_only.can_sign());
    }

    #[test]
    fn test_signature_verification() {
        let wallet = NearWallet::generate_random("test.testnet".to_string()).unwrap();
        let data = b"test message";
        
        let signature = wallet.sign_data(data).unwrap();
        assert!(wallet.verify_signature(data, &signature));
    }
}