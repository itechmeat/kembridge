//! ML-KEM-1024 implementation using FIPS 203 standard
//! 
//! This module provides a high-level wrapper around the ml-kem crate
//! for post-quantum key encapsulation with 256-bit security level.

use crate::error::{QuantumCryptoError, QuantumResult};
use ml_kem::{MlKem1024, KemCore, EncodedSizeUser};
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
        
        // Convert to byte arrays for storage
        let dk_bytes = dk.as_bytes();
        let ek_bytes = ek.as_bytes();
        
        // Copy bytes to fixed-size arrays
        let mut dk_array = [0u8; 3168];
        let mut ek_array = [0u8; 1568];
        dk_array.copy_from_slice(&dk_bytes);
        ek_array.copy_from_slice(&ek_bytes);
        
        Ok(MlKemKeyPair {
            decapsulation_key: dk_array,
            encapsulation_key: ek_array,
        })
    }

    /// Perform ML-KEM encapsulation with public key bytes
    pub fn encapsulate_with_bytes(
        public_key_bytes: &[u8]
    ) -> QuantumResult<(Vec<u8>, [u8; 32])> {
        if public_key_bytes.len() != 1568 {
            return Err(QuantumCryptoError::InvalidKeySize { 
                expected: 1568, 
                actual: public_key_bytes.len() 
            });
        }

        let mut key_array = [0u8; 1568];
        key_array.copy_from_slice(public_key_bytes);

        // Import the encapsulation key from bytes
        let ek: ml_kem::kem::EncapsulationKey<ml_kem::MlKem1024Params> = 
            ml_kem::kem::EncapsulationKey::from_bytes(&key_array.into());

        let mut rng = thread_rng();
        let (ciphertext, shared_secret) = ek.encapsulate(&mut rng)
            .map_err(|_| QuantumCryptoError::EncapsulationFailed)?;
        
        Ok((ciphertext.to_vec(), shared_secret.into()))
    }

    /// Perform ML-KEM decapsulation with private key bytes
    pub fn decapsulate_with_bytes(
        private_key_bytes: &[u8],
        ciphertext_bytes: &[u8]
    ) -> QuantumResult<[u8; 32]> {
        if private_key_bytes.len() != 3168 {
            return Err(QuantumCryptoError::InvalidKeySize { 
                expected: 3168, 
                actual: private_key_bytes.len() 
            });
        }

        if ciphertext_bytes.len() != 1568 {
            return Err(QuantumCryptoError::InvalidData(
                format!("Invalid ciphertext size: {} bytes", ciphertext_bytes.len())
            ));
        }

        let mut private_key_array = [0u8; 3168];
        private_key_array.copy_from_slice(private_key_bytes);

        let mut ciphertext_array = [0u8; 1568];
        ciphertext_array.copy_from_slice(ciphertext_bytes);

        // Import the decapsulation key and ciphertext from bytes
        let dk: ml_kem::kem::DecapsulationKey<ml_kem::MlKem1024Params> = 
            ml_kem::kem::DecapsulationKey::from_bytes(&private_key_array.into());
        let ct = ciphertext_array.into();

        let shared_secret = dk.decapsulate(&ct)
            .map_err(|_| QuantumCryptoError::DecapsulationFailed)?;
        
        Ok(shared_secret.into())
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
    /// Private key for decapsulation operations (3168 bytes)
    pub decapsulation_key: [u8; 3168],
    /// Public key for encapsulation operations (1568 bytes)
    pub encapsulation_key: [u8; 1568],
}

impl MlKemKeyPair {
    /// Get the encapsulation key (public key) as bytes
    pub fn public_key(&self) -> &[u8; 1568] {
        &self.encapsulation_key
    }
    
    /// Get the decapsulation key (private key) as bytes
    pub fn private_key(&self) -> &[u8; 3168] {
        &self.decapsulation_key
    }

    /// Get public key as slice
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.encapsulation_key
    }

    /// Get private key as slice
    pub fn private_key_bytes(&self) -> &[u8] {
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

    #[test]
    fn test_key_sizes() {
        // Generate keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        
        // Verify key sizes
        assert_eq!(keypair.public_key_bytes().len(), 1568);
        assert_eq!(keypair.private_key_bytes().len(), 3168);
    }

    #[test]
    fn test_encapsulation_decapsulation_with_bytes() {
        // Generate keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        
        // Test encapsulation with bytes
        let (ciphertext_bytes, shared_secret1) = MlKemCrypto::encapsulate_with_bytes(
            keypair.public_key_bytes()
        ).unwrap();
        
        // Test decapsulation with bytes
        let shared_secret2 = MlKemCrypto::decapsulate_with_bytes(
            keypair.private_key_bytes(),
            &ciphertext_bytes
        ).unwrap();
        
        // Verify shared secrets match
        assert_eq!(shared_secret1, shared_secret2);
        assert_eq!(shared_secret1.len(), 32);
        assert_eq!(ciphertext_bytes.len(), 1568);
    }
}