//! ML-KEM-1024 implementation using FIPS 203 standard
//! 
//! This module provides a high-level wrapper around the ml-kem crate
//! for post-quantum key encapsulation with 256-bit security level.

use crate::error::{QuantumCryptoError, QuantumResult};
use ml_kem::{MlKem1024, KemCore};
use ml_kem::kem::{Encapsulate, Decapsulate};
use rand::thread_rng;

/// Main interface for ML-KEM-1024 cryptographic operations
pub struct MlKemCrypto;

impl MlKemCrypto {
    /// Generate a new ML-KEM-1024 key pair
    /// 
    /// Returns a tuple of (decapsulation_key, encapsulation_key).
    /// The decapsulation key is kept private, the encapsulation key can be shared.
    pub fn generate_keypair() -> QuantumResult<MlKemKeyPair> {
        let mut rng = thread_rng();
        let (dk, ek) = MlKem1024::generate(&mut rng);
        
        Ok(MlKemKeyPair {
            decapsulation_key: dk,
            encapsulation_key: ek,
        })
    }
    
    /// Verify that the ML-KEM implementation works correctly
    /// 
    /// This is a utility function for testing the ML-KEM implementation.
    /// It performs a full round-trip: generate keys, encapsulate, decapsulate, and verify.
    pub fn verify_round_trip() -> QuantumResult<()> {
        let mut rng = thread_rng();
        
        // Generate a key pair
        let (dk, ek) = MlKem1024::generate(&mut rng);
        
        // Encapsulate - creates shared secret and ciphertext
        let (ciphertext, sender_shared_key) = ek.encapsulate(&mut rng)
            .map_err(|_| QuantumCryptoError::EncapsulationFailed)?;
        
        // Decapsulate - recover shared secret from ciphertext
        let receiver_shared_key = dk.decapsulate(&ciphertext)
            .map_err(|_| QuantumCryptoError::DecapsulationFailed)?;
        
        // Verify that both sides have the same shared secret
        if sender_shared_key == receiver_shared_key {
            Ok(())
        } else {
            Err(QuantumCryptoError::VerificationFailed)
        }
    }
    
    /// Get ML-KEM-1024 algorithm parameters
    /// 
    /// Returns information about the ML-KEM-1024 algorithm for informational purposes.
    pub fn algorithm_info() -> AlgorithmInfo {
        AlgorithmInfo {
            name: "ML-KEM-1024".to_string(),
            standard: "FIPS 203".to_string(),
            security_level: 256,
            public_key_size: 1568,
            private_key_size: 3168,
            ciphertext_size: 1568,
            shared_secret_size: 32,
        }
    }
}

/// ML-KEM-1024 key pair
/// 
/// Contains both the decapsulation key (private) and encapsulation key (public).
/// The decapsulation key must be kept secret, while the encapsulation key can be shared.
pub struct MlKemKeyPair {
    /// Private key for decapsulation operations
    pub decapsulation_key: ml_kem::kem::DecapsulationKey<ml_kem::MlKem1024Params>,
    /// Public key for encapsulation operations  
    pub encapsulation_key: ml_kem::kem::EncapsulationKey<ml_kem::MlKem1024Params>,
}

impl MlKemKeyPair {
    /// Get the encapsulation key (public key)
    pub fn public_key(&self) -> &ml_kem::kem::EncapsulationKey<ml_kem::MlKem1024Params> {
        &self.encapsulation_key
    }
    
    /// Get the decapsulation key (private key)
    pub fn private_key(&self) -> &ml_kem::kem::DecapsulationKey<ml_kem::MlKem1024Params> {
        &self.decapsulation_key
    }
}

/// Information about the ML-KEM-1024 algorithm
#[derive(Debug, Clone)]
pub struct AlgorithmInfo {
    /// Algorithm name
    pub name: String,
    /// Standard specification
    pub standard: String,
    /// Security level in bits
    pub security_level: u32,
    /// Public key size in bytes
    pub public_key_size: usize,
    /// Private key size in bytes
    pub private_key_size: usize,
    /// Ciphertext size in bytes
    pub ciphertext_size: usize,
    /// Shared secret size in bytes
    pub shared_secret_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        // Test that we can generate a key pair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        
        // Basic sanity checks - we should have both keys
        let _public_key = keypair.public_key();
        let _private_key = keypair.private_key();
    }
    
    #[test]
    fn test_verify_round_trip() {
        // Test the full round-trip functionality
        MlKemCrypto::verify_round_trip().unwrap();
    }

    #[test]
    fn test_algorithm_info() {
        let info = MlKemCrypto::algorithm_info();
        assert_eq!(info.name, "ML-KEM-1024");
        assert_eq!(info.standard, "FIPS 203");
        assert_eq!(info.security_level, 256);
        assert_eq!(info.public_key_size, 1568);
        assert_eq!(info.private_key_size, 3168);
        assert_eq!(info.ciphertext_size, 1568);
        assert_eq!(info.shared_secret_size, 32);
    }
}