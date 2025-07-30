//! High-level key management for ML-KEM-1024
//! 
//! This module provides convenient wrappers around the ML-KEM primitives
//! with additional metadata, security features, and ease-of-use functions.

use crate::ml_kem::{MlKemCrypto, MlKemKeyPair};
use crate::error::{QuantumResult, QuantumCryptoError};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use ethers::types::{Address, H256};
use ethers::signers::{LocalWallet, Signer};
use ethers::core::k256::ecdsa::SigningKey;
use std::str::FromStr;
use std::collections::HashMap;

/// High-level quantum key manager
/// 
/// Provides convenient methods for working with ML-KEM-1024 keys
/// including metadata management and secure memory handling.
pub struct QuantumKeyManager {
    // In-memory key storage for development (production should use secure storage)
    quantum_keys: HashMap<Uuid, MlKemKeyPair>,
    ethereum_keys: HashMap<Uuid, LocalWallet>,
}

impl QuantumKeyManager {
    /// Create a new quantum key manager
    pub fn new() -> Self {
        Self {
            quantum_keys: HashMap::new(),
            ethereum_keys: HashMap::new(),
        }
    }

    /// Verify that the ML-KEM implementation works correctly
    /// 
    /// Performs a round-trip test to ensure the implementation is working.
    /// This is useful for system verification and testing.
    pub fn verify_implementation(&self) -> QuantumResult<()> {
        MlKemCrypto::verify_round_trip()
    }

    /// Get algorithm information
    pub fn algorithm_info(&self) -> crate::ml_kem::AlgorithmInfo {
        MlKemCrypto::algorithm_info()
    }
    
    /// Generate and store a new ML-KEM-1024 key pair
    pub fn generate_quantum_keypair(&mut self, user_id: Uuid) -> QuantumResult<Uuid> {
        let keypair = MlKemCrypto::generate_keypair()?;
        let key_id = Uuid::new_v4();
        
        self.quantum_keys.insert(key_id, keypair);
        
        tracing::info!(
            key_id = %key_id,
            user_id = %user_id,
            "Generated new ML-KEM-1024 key pair"
        );
        
        Ok(key_id)
    }
    
    /// Generate and store a new Ethereum key pair
    pub fn generate_ethereum_keypair(&mut self, user_id: Uuid) -> QuantumResult<Uuid> {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let key_id = Uuid::new_v4();
        
        self.ethereum_keys.insert(key_id, wallet.clone());
        
        tracing::info!(
            key_id = %key_id,
            user_id = %user_id,
            address = %wallet.address(),
            "Generated new Ethereum key pair"
        );
        
        Ok(key_id)
    }
    
    /// Get Ethereum private key for signing
    pub fn get_ethereum_private_key(&self, key_id: Uuid) -> QuantumResult<Vec<u8>> {
        let wallet = self.ethereum_keys.get(&key_id)
            .ok_or_else(|| QuantumCryptoError::InvalidData(
                format!("Ethereum key not found: {}", key_id)
            ))?;
        
        // Convert signing key to bytes
        let signing_key = wallet.signer();
        let private_key_bytes = signing_key.to_bytes();
        
        Ok(private_key_bytes.to_vec())
    }
    
    /// Get Ethereum wallet for signing transactions
    pub fn get_ethereum_wallet(&self, key_id: Uuid) -> QuantumResult<LocalWallet> {
        self.ethereum_keys.get(&key_id)
            .cloned()
            .ok_or_else(|| QuantumCryptoError::InvalidData(
                format!("Ethereum wallet not found: {}", key_id)
            ))
    }
    
    /// Get ML-KEM public key for encapsulation
    pub fn get_quantum_public_key(&self, key_id: Uuid) -> QuantumResult<&[u8; 1568]> {
        let keypair = self.quantum_keys.get(&key_id)
            .ok_or_else(|| QuantumCryptoError::InvalidData(
                format!("Quantum key not found: {}", key_id)
            ))?;
        
        Ok(keypair.public_key())
    }
    
    /// Get ML-KEM private key for decapsulation  
    pub fn get_quantum_private_key(&self, key_id: Uuid) -> QuantumResult<&[u8; 3168]> {
        let keypair = self.quantum_keys.get(&key_id)
            .ok_or_else(|| QuantumCryptoError::InvalidData(
                format!("Quantum key not found: {}", key_id)
            ))?;
        
        Ok(keypair.private_key())
    }
    
    /// Create quantum hash from transaction data
    pub fn create_quantum_hash(&self, 
        user_address: Address, 
        amount: u64, 
        chain: &str, 
        timestamp: i64
    ) -> String {
        let data = format!("{}{}{}{}", user_address, amount, chain, timestamp);
        let hash = ethers::utils::keccak256(data.as_bytes());
        format!("0x{}", hex::encode(hash))
    }
    
    /// Generate admin key from environment (for demo purposes)
    pub fn get_admin_key(&mut self) -> QuantumResult<Uuid> {
        // Check if we already have an admin key
        for (key_id, wallet) in &self.ethereum_keys {
            if wallet.address().to_string().starts_with("0x000") {
                return Ok(*key_id);
            }
        }
        
        // Generate new admin key
        let admin_wallet = LocalWallet::from_str(
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        ).map_err(|e| QuantumCryptoError::InvalidData(e.to_string()))?;
        
        let key_id = Uuid::new_v4();
        self.ethereum_keys.insert(key_id, admin_wallet);
        
        tracing::info!(
            key_id = %key_id,
            "Generated admin key for bridge operations"
        );
        
        Ok(key_id)
    }
    
    /// Check if key exists
    pub fn has_quantum_key(&self, key_id: Uuid) -> bool {
        self.quantum_keys.contains_key(&key_id)
    }
    
    /// Check if Ethereum key exists
    pub fn has_ethereum_key(&self, key_id: Uuid) -> bool {
        self.ethereum_keys.contains_key(&key_id)
    }
    
    /// Get key statistics
    pub fn get_key_stats(&self) -> (usize, usize) {
        (self.quantum_keys.len(), self.ethereum_keys.len())
    }
}

impl Default for QuantumKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A simplified quantum key pair for Phase 3.1
/// 
/// In Phase 3.2, this will be expanded with full key serialization
/// and database integration capabilities.
#[derive(Debug)]
pub struct QuantumKeyPair {
    /// When this key pair was created
    pub created_at: DateTime<Utc>,
    /// Unique identifier for this key pair
    pub key_id: Uuid,
    /// Algorithm information
    pub algorithm: String,
}

impl QuantumKeyPair {
    /// Create a new quantum key pair metadata
    pub fn new() -> Self {
        Self {
            created_at: Utc::now(),
            key_id: Uuid::new_v4(),
            algorithm: "ML-KEM-1024".to_string(),
        }
    }

    /// Get the key pair's age
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Check if the key pair is older than the specified duration
    pub fn is_older_than(&self, duration: chrono::Duration) -> bool {
        self.age() > duration
    }
}

/// Result of an encapsulation operation (placeholder for Phase 3.1)
/// 
/// In Phase 3.2, this will contain actual ciphertext and shared secret data.
#[derive(Debug)]
pub struct EncapsulationResult {
    /// Placeholder for ciphertext
    pub operation_id: Uuid,
    /// When the operation was performed
    pub timestamp: DateTime<Utc>,
}

impl EncapsulationResult {
    /// Create a new encapsulation result
    pub fn new() -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_creation() {
        let manager = QuantumKeyManager::new();
        let info = manager.algorithm_info();
        assert_eq!(info.name, "ML-KEM-1024");
    }

    #[test]
    fn test_implementation_verification() {
        let manager = QuantumKeyManager::new();
        
        // This should pass if ML-KEM is working correctly
        manager.verify_implementation().unwrap();
    }
    
    #[test]
    fn test_quantum_key_generation() {
        let mut manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        
        let key_id = manager.generate_quantum_keypair(user_id).unwrap();
        assert!(manager.has_quantum_key(key_id));
        
        let public_key = manager.get_quantum_public_key(key_id).unwrap();
        assert_eq!(public_key.len(), 1568);
        
        let private_key = manager.get_quantum_private_key(key_id).unwrap();
        assert_eq!(private_key.len(), 3168);
    }
    
    #[test]
    fn test_ethereum_key_generation() {
        let mut manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        
        let key_id = manager.generate_ethereum_keypair(user_id).unwrap();
        assert!(manager.has_ethereum_key(key_id));
        
        let private_key = manager.get_ethereum_private_key(key_id).unwrap();
        assert_eq!(private_key.len(), 32);
        
        let wallet = manager.get_ethereum_wallet(key_id).unwrap();
        assert!(wallet.address() != Address::zero());
    }
    
    #[test]
    fn test_quantum_hash_creation() {
        let manager = QuantumKeyManager::new();
        let user_address = Address::from_str("0x742d35cc6634c0532925a3b8d1d1cc4d35a4b4d4").unwrap();
        
        let hash = manager.create_quantum_hash(user_address, 1000, "ethereum", 1234567890);
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // 0x + 64 hex chars
    }
    
    #[test]
    fn test_admin_key_generation() {
        let mut manager = QuantumKeyManager::new();
        
        let admin_key_id = manager.get_admin_key().unwrap();
        assert!(manager.has_ethereum_key(admin_key_id));
        
        let admin_wallet = manager.get_ethereum_wallet(admin_key_id).unwrap();
        let address_str = admin_wallet.address().to_string();
        // Admin key should be deterministic from the fixed private key
        assert!(!address_str.is_empty() && address_str.starts_with("0x"));
    }
    
    #[test]
    fn test_key_statistics() {
        let mut manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        
        assert_eq!(manager.get_key_stats(), (0, 0));
        
        let _quantum_key_id = manager.generate_quantum_keypair(user_id).unwrap();
        assert_eq!(manager.get_key_stats(), (1, 0));
        
        let _eth_key_id = manager.generate_ethereum_keypair(user_id).unwrap();
        assert_eq!(manager.get_key_stats(), (1, 1));
    }

    #[test]
    fn test_keypair_metadata() {
        let keypair = QuantumKeyPair::new();
        
        assert_eq!(keypair.algorithm, "ML-KEM-1024");
        assert!(keypair.age().num_seconds() >= 0);
        assert!(!keypair.is_older_than(chrono::Duration::minutes(1)));
    }

    #[test]
    fn test_encapsulation_result() {
        let result = EncapsulationResult::new();
        
        // Should have a valid operation ID and recent timestamp
        assert!(result.timestamp <= Utc::now());
    }
}