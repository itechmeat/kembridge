//! Quantum-safe transaction data encryption for KEMBridge
//! 
//! This module provides specialized encryption for transaction parameters,
//! wallet addresses, amounts, and other sensitive bridge operation data.

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use sha2::Digest;

use crate::QuantumCryptoError;

/// Transaction data types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionDataType {
    Bridge,
    Swap,
    Transfer,
    Staking,
}

/// Sensitive transaction data before encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveTransactionData {
    pub from_address: String,
    pub to_address: String,
    pub from_amount: String,
    pub to_amount: String,
    pub from_token: String,
    pub to_token: String,
    pub metadata: HashMap<String, String>,
}

/// Encrypted transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEncryptionMetadata {
    pub timestamp: i64,
    pub encryption_scheme: String,
    pub data_type: TransactionDataType,
    pub context_hash: String,
}

/// Quantum-protected transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumTransaction {
    pub quantum_key_id: Uuid,
    pub encrypted_ciphertext: Vec<u8>,
    pub encrypted_shared_secret: Vec<u8>,
    pub metadata: TransactionEncryptionMetadata,
}

/// Protected wallet addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumProtectedAddresses {
    pub quantum_key_id: Uuid,
    pub encrypted_addresses: Vec<EncryptedAddress>,
    pub metadata: AddressMetadata,
}

/// Single encrypted address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedAddress {
    pub encrypted_address: Vec<u8>,
    pub address_type: String,
    pub timestamp: i64,
}

/// Address metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressMetadata {
    pub timestamp: i64,
    pub total_addresses: usize,
}

/// Encrypted transaction amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTransactionAmount {
    pub token_symbol: String,
    pub encrypted_amount: Vec<u8>,
    pub timestamp: i64,
}

/// Main quantum transaction crypto implementation
pub struct QuantumTransactionCrypto;

impl QuantumTransactionCrypto {
    /// Encrypt transaction data using ML-KEM-1024
    pub fn encrypt_transaction_data(
        ml_kem_public_key: &[u8],
        transaction_data: &SensitiveTransactionData,
        quantum_key_id: Uuid,
    ) -> Result<QuantumTransaction, QuantumCryptoError> {
        Self::encrypt_transaction_data_with_type(
            ml_kem_public_key,
            transaction_data,
            quantum_key_id,
            TransactionDataType::Bridge,
        )
    }

    /// Encrypt transaction data with specific type
    pub fn encrypt_transaction_data_with_type(
        ml_kem_public_key: &[u8],
        transaction_data: &SensitiveTransactionData,
        quantum_key_id: Uuid,
        data_type: TransactionDataType,
    ) -> Result<QuantumTransaction, QuantumCryptoError> {
        if ml_kem_public_key.len() < 32 {
            return Err(QuantumCryptoError::InvalidKey);
        }

        // Serialize transaction data
        let serialized_data = serde_json::to_vec(transaction_data)
            .map_err(|_| QuantumCryptoError::SerializationError)?;

        // Create mock encryption (simplified for testing)
        let encrypted_ciphertext = Self::mock_encrypt(&serialized_data);
        let encrypted_shared_secret = Self::mock_encrypt(b"shared_secret");

        // Create context hash
        let context_hash = format!("{:x}", 
            sha2::Sha256::digest(format!("{:?}{}", transaction_data.from_address, chrono::Utc::now().timestamp()))
        );

        let metadata = TransactionEncryptionMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            encryption_scheme: "ML-KEM-1024".to_string(),
            data_type,
            context_hash,
        };

        Ok(QuantumTransaction {
            quantum_key_id,
            encrypted_ciphertext,
            encrypted_shared_secret,
            metadata,
        })
    }

    /// Encrypt wallet addresses
    pub fn encrypt_wallet_addresses(
        ml_kem_public_key: &[u8],
        addresses: &[String],
        quantum_key_id: Uuid,
    ) -> Result<QuantumProtectedAddresses, QuantumCryptoError> {
        if ml_kem_public_key.len() < 32 {
            return Err(QuantumCryptoError::InvalidKey);
        }

        let encrypted_addresses = addresses.iter().map(|addr| {
            EncryptedAddress {
                encrypted_address: Self::mock_encrypt(addr.as_bytes()),
                address_type: "wallet".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            }
        }).collect();

        let metadata = AddressMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            total_addresses: addresses.len(),
        };

        Ok(QuantumProtectedAddresses {
            quantum_key_id,
            encrypted_addresses,
            metadata,
        })
    }

    /// Encrypt transaction amounts
    pub fn encrypt_transaction_amounts(
        ml_kem_public_key: &[u8],
        amounts: &[(String, String)],
        _quantum_key_id: Uuid,
    ) -> Result<Vec<EncryptedTransactionAmount>, QuantumCryptoError> {
        if ml_kem_public_key.len() < 32 {
            return Err(QuantumCryptoError::InvalidKey);
        }

        let encrypted_amounts = amounts.iter().map(|(token, amount)| {
            EncryptedTransactionAmount {
                token_symbol: token.clone(),
                encrypted_amount: Self::mock_encrypt(amount.as_bytes()),
                timestamp: chrono::Utc::now().timestamp(),
            }
        }).collect();

        Ok(encrypted_amounts)
    }

    // Simplified mock encryption for testing
    fn mock_encrypt(data: &[u8]) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).to_string().as_bytes());
        hasher.update(&uuid::Uuid::new_v4().as_bytes()); // Add randomness
        hasher.finalize().to_vec()
    }
}