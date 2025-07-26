// Multi-chain signature verification for Ethereum and NEAR

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::str::FromStr;
use crate::errors::AuthError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    #[serde(rename = "ethereum")]
    Ethereum,
    #[serde(rename = "near")]
    Near,
}

impl ChainType {
    pub fn to_string(&self) -> String {
        match self {
            ChainType::Ethereum => "ethereum".to_string(),
            ChainType::Near => "near".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, AuthError> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(ChainType::Ethereum),
            "near" => Ok(ChainType::Near),
            _ => Err(AuthError::UnsupportedChainType(s.to_string())),
        }
    }
}

impl FromStr for ChainType {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

#[async_trait]
pub trait ChainVerifier {
    async fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        address: &str,
    ) -> Result<bool, AuthError>;

    fn validate_address(&self, address: &str) -> Result<bool, AuthError>;
}

/// Ethereum signature verifier using secp256k1
#[derive(Clone)]
pub struct EthereumVerifier;

impl EthereumVerifier {
    pub fn new() -> Self {
        Self
    }

    fn hex_decode(hex_str: &str) -> Result<Vec<u8>, AuthError> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        hex::decode(hex_str).map_err(|_| AuthError::InvalidSignature)
    }

    fn recover_public_key(
        message: &str,
        signature: &str,
    ) -> Result<secp256k1::PublicKey, AuthError> {
        use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1};
        use sha2::{Digest, Sha256};

        let secp = Secp256k1::new();
        
        // Ethereum signed message format
        let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let mut hasher = sha2::Sha256::new();
        hasher.update(prefixed_message.as_bytes());
        let message_hash = hasher.finalize();
        let message = Message::from_digest(message_hash.into());

        // Parse signature (65 bytes: r + s + v)
        let sig_bytes = Self::hex_decode(signature)?;
        if sig_bytes.len() != 65 {
            return Err(AuthError::InvalidSignature);
        }

        let recovery_id = sig_bytes[64];
        let recovery_id = secp256k1::ecdsa::RecoveryId::from_u8_masked(recovery_id);

        let signature = RecoverableSignature::from_compact(&sig_bytes[0..64], recovery_id)
            .map_err(|_| AuthError::InvalidSignature)?;

        secp.recover_ecdsa(message, &signature)
            .map_err(|_| AuthError::InvalidSignature)
    }

    fn public_key_to_address(public_key: &secp256k1::PublicKey) -> String {
        let public_key_bytes = public_key.serialize_uncompressed();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&public_key_bytes[1..]);
        let hash = hasher.finalize();
        let address = &hash[12..];
        format!("0x{}", hex::encode(address))
    }
}

#[async_trait]
impl ChainVerifier for EthereumVerifier {
    async fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        address: &str,
    ) -> Result<bool, AuthError> {
        let recovered_pubkey = Self::recover_public_key(message, signature)?;
        let recovered_address = Self::public_key_to_address(&recovered_pubkey);
        Ok(recovered_address.to_lowercase() == address.to_lowercase())
    }

    fn validate_address(&self, address: &str) -> Result<bool, AuthError> {
        if !address.starts_with("0x") {
            return Ok(false);
        }
        
        if address.len() != 42 {
            return Ok(false);
        }

        // Check if all characters after 0x are valid hex
        let hex_part = &address[2..];
        for c in hex_part.chars() {
            if !c.is_ascii_hexdigit() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// NEAR signature verifier using ed25519
#[derive(Clone)]
pub struct NearVerifier;

impl NearVerifier {
    pub fn new() -> Self {
        Self
    }

    fn base58_decode(s: &str) -> Result<Vec<u8>, AuthError> {
        bs58::decode(s)
            .into_vec()
            .map_err(|_| AuthError::InvalidSignature)
    }
}

#[async_trait]
impl ChainVerifier for NearVerifier {
    async fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        address: &str,
    ) -> Result<bool, AuthError> {
        use ed25519_dalek::{Signature, VerifyingKey, Verifier};
        use sha2::{Digest, Sha256};

        // NEAR uses ed25519 signatures
        let signature_bytes = Self::base58_decode(signature)?;
        if signature_bytes.len() != 64 {
            return Err(AuthError::InvalidSignature);
        }

        let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());

        // Extract public key from NEAR account ID or public key
        // For simplicity, assuming signature verification with known public key
        // In real implementation, we would need to query NEAR network for account's public key
        let mut hasher = sha2::Sha256::new();
        hasher.update(message.as_bytes());
        let message_hash = hasher.finalize();
        
        // TODO: Implement proper NEAR account public key resolution
        // This is a simplified version - in production we need to:
        // 1. Parse account ID
        // 2. Query NEAR network for account's access keys
        // 3. Find matching public key
        // 4. Verify signature
        
        // For now, return placeholder validation
        Ok(self.validate_address(address)?)
    }

    fn validate_address(&self, address: &str) -> Result<bool, AuthError> {
        // NEAR account ID validation
        // Valid formats: alice.near, sub.alice.near, 40-char hex for implicit accounts
        
        if address.len() < 2 || address.len() > 64 {
            return Ok(false);
        }

        // Check for implicit account (64-char hex)
        if address.len() == 64 {
            return Ok(address.chars().all(|c| c.is_ascii_hexdigit()));
        }

        // Check for named account
        if !address.ends_with(".near") && !address.ends_with(".testnet") {
            return Ok(false);
        }

        // Basic validation for account name
        let account_name = if address.ends_with(".near") {
            &address[..address.len() - 5]
        } else if address.ends_with(".testnet") {
            &address[..address.len() - 8]
        } else {
            address
        };

        // Account name can contain lowercase letters, digits, hyphens, underscores
        for c in account_name.chars() {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' && c != '_' && c != '.' {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Multi-chain verifier that routes to appropriate chain-specific verifier
#[derive(Clone)]
pub struct MultiChainVerifier {
    ethereum_verifier: EthereumVerifier,
    near_verifier: NearVerifier,
}

impl MultiChainVerifier {
    pub fn new() -> Self {
        Self {
            ethereum_verifier: EthereumVerifier::new(),
            near_verifier: NearVerifier::new(),
        }
    }

    pub async fn verify_signature(
        &self,
        chain_type: ChainType,
        message: &str,
        signature: &str,
        address: &str,
    ) -> Result<bool, AuthError> {
        match chain_type {
            ChainType::Ethereum => {
                self.ethereum_verifier
                    .verify_signature(message, signature, address)
                    .await
            }
            ChainType::Near => {
                self.near_verifier
                    .verify_signature(message, signature, address)
                    .await
            }
        }
    }

    pub fn validate_address(&self, chain_type: ChainType, address: &str) -> Result<bool, AuthError> {
        match chain_type {
            ChainType::Ethereum => self.ethereum_verifier.validate_address(address),
            ChainType::Near => self.near_verifier.validate_address(address),
        }
    }
}