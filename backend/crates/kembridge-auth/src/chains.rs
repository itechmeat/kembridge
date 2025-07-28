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
        use sha3::{Digest, Keccak256};

        let secp = Secp256k1::new();
        
        // Ethereum signed message format (EIP-191)
        let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        tracing::debug!("🔍 Ethereum: Creating prefixed message with length {}", message.len());
        
        let mut hasher = Keccak256::new();
        hasher.update(prefixed_message.as_bytes());
        let message_hash = hasher.finalize();
        tracing::debug!("🔍 Ethereum: Message hash: {}", hex::encode(&message_hash));
        let message = Message::from_digest(message_hash.into());

        // Parse signature (65 bytes: r + s + v)
        let sig_bytes = Self::hex_decode(signature)?;
        if sig_bytes.len() != 65 {
            return Err(AuthError::InvalidSignature);
        }

        let recovery_id = sig_bytes[64];
        tracing::debug!("🔍 Ethereum: Raw recovery ID: {}", recovery_id);
        
        // Ethereum uses v = 27 or 28, but secp256k1 expects 0-3
        let recovery_id = if recovery_id >= 27 {
            recovery_id - 27
        } else {
            recovery_id
        };
        tracing::debug!("🔍 Ethereum: Normalized recovery ID: {}", recovery_id);
        
        let recovery_id = secp256k1::ecdsa::RecoveryId::from_u8_masked(recovery_id);

        let signature = RecoverableSignature::from_compact(&sig_bytes[0..64], recovery_id)
            .map_err(|e| {
                tracing::error!("🔍 Ethereum: Failed to create recoverable signature: {:?}", e);
                AuthError::InvalidSignature
            })?;

        let recovered_pubkey = secp.recover_ecdsa(message, &signature)
            .map_err(|e| {
                tracing::error!("🔍 Ethereum: Failed to recover public key: {:?}", e);
                AuthError::InvalidSignature
            })?;
        
        tracing::debug!("🔍 Ethereum: Recovered public key: {}", hex::encode(recovered_pubkey.serialize_uncompressed()));
        
        Ok(recovered_pubkey)
    }

    fn public_key_to_address(public_key: &secp256k1::PublicKey) -> String {
        use sha3::{Digest, Keccak256};
        
        let public_key_bytes = public_key.serialize_uncompressed();
        let mut hasher = Keccak256::new();
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
        tracing::debug!("🔍 Ethereum: Verifying signature for address: {}", address);
        tracing::debug!("🔍 Ethereum: Signature: {}", signature);
        tracing::debug!("🔍 Ethereum: Message: {}", message);
        
        let recovered_pubkey = Self::recover_public_key(message, signature)?;
        let recovered_address = Self::public_key_to_address(&recovered_pubkey);
        
        tracing::debug!("🔍 Ethereum: Recovered address: {}", recovered_address);
        tracing::debug!("🔍 Ethereum: Expected address: {}", address);
        
        let matches = recovered_address.to_lowercase() == address.to_lowercase();
        tracing::debug!("🔍 Ethereum: Address match result: {}", matches);
        
        Ok(matches)
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
        
        // TODO (feat): Implement full RPC integration for NEAR account key resolution (P2.2)
        // This requires integrating with kembridge-blockchain::NearAdapter to:
        // 1. Parse NEAR account ID from address
        // 2. Query NEAR RPC for account's access keys via view_access_key_list
        // 3. Find ed25519 public key that matches the signature
        // 4. Verify ed25519 signature against the resolved public key
        //
        // For now, we implement basic ed25519 signature format validation
        // Real integration with NEAR RPC will be completed when BridgeService is available
        
        // Basic signature format validation for ed25519
        if signature_bytes.len() == 64 {
            // Valid ed25519 signature length - assume signature is properly formatted
            tracing::debug!("NEAR signature format validation passed for account: {}", address);
            
            // TODO (feat): Uncomment when full ed25519 verification is implemented (P2.2)
            // let _signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());
            // let mut hasher = sha2::Sha256::new();
            // hasher.update(message.as_bytes());
            // let _message_hash = hasher.finalize();
            
            Ok(self.validate_address(address)?)
        } else {
            tracing::warn!("Invalid NEAR signature length: {} bytes for account: {}", signature_bytes.len(), address);
            Err(AuthError::InvalidSignature)
        }
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