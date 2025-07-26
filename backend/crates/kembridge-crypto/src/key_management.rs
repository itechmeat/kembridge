//! High-level key management for ML-KEM-1024
//! 
//! This module provides convenient wrappers around the ML-KEM primitives
//! with additional metadata, security features, and ease-of-use functions.

use crate::ml_kem::MlKemCrypto;
use crate::error::QuantumResult;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// High-level quantum key manager
/// 
/// Provides convenient methods for working with ML-KEM-1024 keys
/// including metadata management and secure memory handling.
pub struct QuantumKeyManager {
    // Will be expanded in Phase 3.2 for database integration
}

impl QuantumKeyManager {
    /// Create a new quantum key manager
    pub fn new() -> Self {
        Self {}
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