// src/aes_gcm.rs - AES-256-GCM wrapper for hybrid cryptography
use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use aes_gcm::aead::consts::U12;
use rand::{Rng, thread_rng};
use zeroize::ZeroizeOnDrop;

use crate::error::QuantumCryptoError;

/// Encrypted data with AES-256-GCM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// AES-256-GCM cryptographic operations for hybrid scheme
pub struct AesGcmCrypto;

impl AesGcmCrypto {
    /// Generate random AES-256 key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        thread_rng().fill(&mut key);
        key
    }

    /// Generate random nonce for AES-GCM
    fn generate_nonce() -> GenericArray<u8, U12> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill(&mut nonce_bytes);
        *GenericArray::from_slice(&nonce_bytes)
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt(
        key: &[u8; 32],
        data: &[u8]
    ) -> Result<EncryptedData, QuantumCryptoError> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        let nonce = Self::generate_nonce();
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| QuantumCryptoError::EncryptionFailed(e.to_string()))?;
            
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
        })
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(
        key: &[u8; 32],
        encrypted_data: &EncryptedData
    ) -> Result<Vec<u8>, QuantumCryptoError> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        let nonce = GenericArray::from_slice(&encrypted_data.nonce);
        
        let plaintext = cipher.decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| QuantumCryptoError::DecryptionFailed(e.to_string()))?;
            
        Ok(plaintext)
    }
}

/// Secure AES key wrapper with automatic zeroing
#[derive(ZeroizeOnDrop)]
pub struct SecureAesKey {
    key: [u8; 32],
}

impl SecureAesKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, QuantumCryptoError> {
        if slice.len() != 32 {
            return Err(QuantumCryptoError::InvalidKeySize {
                expected: 32,
                actual: slice.len(),
            });
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Ok(Self { key })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_round_trip() {
        let key = AesGcmCrypto::generate_key();
        let data = b"Hello, KEMBridge!";

        // Encryption
        let encrypted = AesGcmCrypto::encrypt(&key, data).unwrap();
        
        // Decryption
        let decrypted = AesGcmCrypto::decrypt(&key, &encrypted).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_secure_key_zeroize() {
        let key_data = [1u8; 32];
        let secure_key = SecureAesKey::new(key_data);
        
        // Key should be accessible
        assert_eq!(secure_key.as_bytes(), &key_data);
        
        // After drop, memory should be zeroed (can only check conceptually)
        drop(secure_key);
    }

    #[test]
    fn test_different_nonces() {
        let key = AesGcmCrypto::generate_key();
        let data = b"Test data";

        let encrypted1 = AesGcmCrypto::encrypt(&key, data).unwrap();
        let encrypted2 = AesGcmCrypto::encrypt(&key, data).unwrap();

        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        
        // But decryption should work for both
        let decrypted1 = AesGcmCrypto::decrypt(&key, &encrypted1).unwrap();
        let decrypted2 = AesGcmCrypto::decrypt(&key, &encrypted2).unwrap();
        
        assert_eq!(decrypted1, data);
        assert_eq!(decrypted2, data);
    }
}