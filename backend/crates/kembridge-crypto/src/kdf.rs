// src/kdf.rs - Key Derivation Functions for hybrid cryptography
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::QuantumCryptoError;

/// HKDF-SHA256 for extending ML-KEM shared secret to AES keys
pub struct KeyDerivation;

impl KeyDerivation {
    /// Extend ML-KEM shared secret to AES-256 key
    pub fn derive_encryption_key(
        shared_secret: &[u8; 32],
        context: &[u8]
    ) -> Result<[u8; 32], QuantumCryptoError> {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut aes_key = [0u8; 32];
        
        hk.expand(context, &mut aes_key)
            .map_err(|e| QuantumCryptoError::KeyDerivationFailed(e.to_string()))?;
            
        Ok(aes_key)
    }

    /// Extend shared secret to multiple keys (AES + HMAC)
    pub fn derive_multiple_keys(
        shared_secret: &[u8; 32],
        context: &[u8]
    ) -> Result<DerivedKeys, QuantumCryptoError> {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut output = [0u8; 64]; // 32 bytes for AES + 32 bytes for HMAC
        
        hk.expand(context, &mut output)
            .map_err(|e| QuantumCryptoError::KeyDerivationFailed(e.to_string()))?;
        
        let mut aes_key = [0u8; 32];
        let mut hmac_key = [0u8; 32];
        
        aes_key.copy_from_slice(&output[0..32]);
        hmac_key.copy_from_slice(&output[32..64]);
        
        // Clear intermediate buffer
        output.zeroize();
        
        Ok(DerivedKeys {
            encryption_key: aes_key,
            authentication_key: hmac_key,
        })
    }

    /// Generate context string for HKDF
    pub fn create_context(purpose: &str, version: u8) -> Vec<u8> {
        format!("KEMBridge-v{}-{}", version, purpose).into_bytes()
    }
}

/// Derived keys for hybrid scheme
#[derive(Debug)]
pub struct DerivedKeys {
    pub encryption_key: [u8; 32],
    pub authentication_key: [u8; 32],
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.encryption_key.zeroize();
        self.authentication_key.zeroize();
    }
}

/// Predefined contexts for various operations
pub mod contexts {
    use super::KeyDerivation;

    /// Context for bridge transaction encryption
    pub fn bridge_transaction() -> Vec<u8> {
        KeyDerivation::create_context("bridge-tx", 1)
    }

    /// Context for key exchange between chains
    pub fn key_exchange() -> Vec<u8> {
        KeyDerivation::create_context("key-exchange", 1)
    }

    /// Context for session keys
    pub fn session_keys() -> Vec<u8> {
        KeyDerivation::create_context("session", 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_key_derivation_deterministic() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context = contexts::bridge_transaction();
        
        // Two calls with same parameters should give identical results
        let key1 = KeyDerivation::derive_encryption_key(&shared_secret, &context).unwrap();
        let key2 = KeyDerivation::derive_encryption_key(&shared_secret, &context).unwrap();
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_contexts_different_keys() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context1 = contexts::bridge_transaction();
        let context2 = contexts::key_exchange();
        
        let key1 = KeyDerivation::derive_encryption_key(&shared_secret, &context1).unwrap();
        let key2 = KeyDerivation::derive_encryption_key(&shared_secret, &context2).unwrap();
        
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_multiple_keys_derivation() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context = contexts::session_keys();
        let derived = KeyDerivation::derive_multiple_keys(&shared_secret, &context).unwrap();
        
        // Keys should be different
        assert_ne!(derived.encryption_key, derived.authentication_key);
    }

    #[test]
    fn test_context_generation() {
        let context1 = KeyDerivation::create_context("test", 1);
        let context2 = KeyDerivation::create_context("test", 2);
        let context3 = KeyDerivation::create_context("other", 1);
        
        assert_ne!(context1, context2); // Different versions
        assert_ne!(context1, context3); // Different purposes
        
        // Check that context is readable
        let context_str = String::from_utf8(context1).unwrap();
        assert!(context_str.contains("KEMBridge"));
        assert!(context_str.contains("test"));
    }
}