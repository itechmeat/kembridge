// src/hybrid_crypto.rs - Core hybrid cryptography implementation
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use zeroize::ZeroizeOnDrop;

use crate::{
    MlKemCrypto, AesGcmCrypto, EncryptedData, KeyDerivation, IntegrityProtection,
    QuantumCryptoError, kdf::contexts
};

/// Hybrid encrypted data combining ML-KEM and AES-GCM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptedData {
    /// ML-KEM ciphertext containing encrypted shared secret
    pub ml_kem_ciphertext: Vec<u8>,
    /// AES-GCM encrypted payload
    pub aes_encrypted_data: EncryptedData,
    /// ID of the quantum key used for encryption
    pub public_key_id: Uuid,
    /// HMAC for integrity verification
    pub integrity_proof: Vec<u8>,
    /// Timestamp of encryption
    pub encrypted_at: DateTime<Utc>,
    /// Version of hybrid scheme for forward compatibility
    pub scheme_version: u8,
}

/// Core hybrid cryptography implementation
pub struct HybridCrypto;

impl HybridCrypto {
    /// Current version of the hybrid scheme
    pub const SCHEME_VERSION: u8 = 1;

    /// Encrypt data using hybrid ML-KEM + AES-GCM scheme
    pub fn encrypt_data(
        ml_kem_public_key: &[u8],
        data: &[u8],
        context: &[u8]
    ) -> Result<HybridEncryptedData, QuantumCryptoError> {
        // 1. Generate shared secret via ML-KEM encapsulation
        let (ml_kem_ciphertext, shared_secret) = MlKemCrypto::encapsulate_with_bytes(ml_kem_public_key)?;
        
        // 2. Derive AES key from shared secret using HKDF
        let aes_key = KeyDerivation::derive_encryption_key(&shared_secret, context)?;
        
        // 3. Encrypt data with AES-256-GCM
        let aes_encrypted_data = AesGcmCrypto::encrypt(&aes_key, data)?;
        
        // 4. Generate integrity proof for the entire structure
        let integrity_data = Self::create_integrity_data(
            &ml_kem_ciphertext,
            &aes_encrypted_data
        );
        let integrity_proof = IntegrityProtection::generate_mac(&aes_key, &integrity_data)?;
        
        Ok(HybridEncryptedData {
            ml_kem_ciphertext,
            aes_encrypted_data,
            public_key_id: Uuid::new_v4(), // Will be set by caller
            integrity_proof,
            encrypted_at: Utc::now(),
            scheme_version: Self::SCHEME_VERSION,
        })
    }

    /// Decrypt hybrid encrypted data
    pub fn decrypt_data(
        ml_kem_private_key: &[u8],
        hybrid_data: &HybridEncryptedData,
        context: &[u8]
    ) -> Result<Vec<u8>, QuantumCryptoError> {
        // 1. Verify scheme version compatibility
        if hybrid_data.scheme_version != Self::SCHEME_VERSION {
            return Err(QuantumCryptoError::HybridOperationFailed(
                format!("Unsupported scheme version: {}", hybrid_data.scheme_version)
            ));
        }

        // 2. Recover shared secret via ML-KEM decapsulation
        let shared_secret = MlKemCrypto::decapsulate_with_bytes(
            ml_kem_private_key, 
            &hybrid_data.ml_kem_ciphertext
        )?;
        
        // 3. Derive AES key from shared secret
        let aes_key = KeyDerivation::derive_encryption_key(&shared_secret, context)?;
        
        // 4. Verify integrity before decryption
        let integrity_data = Self::create_integrity_data(
            &hybrid_data.ml_kem_ciphertext,
            &hybrid_data.aes_encrypted_data
        );
        
        let is_valid = IntegrityProtection::verify_integrity(
            &aes_key,
            &integrity_data,
            &hybrid_data.integrity_proof
        )?;
        
        if !is_valid {
            return Err(QuantumCryptoError::VerificationFailed);
        }
        
        // 5. Decrypt data with AES-256-GCM
        let plaintext = AesGcmCrypto::decrypt(&aes_key, &hybrid_data.aes_encrypted_data)?;
        
        Ok(plaintext)
    }

    /// Create integrity data for HMAC verification
    fn create_integrity_data(ml_kem_ciphertext: &[u8], aes_data: &EncryptedData) -> Vec<u8> {
        let mut integrity_data = Vec::new();
        
        // Include ML-KEM ciphertext length and data
        integrity_data.extend_from_slice(&(ml_kem_ciphertext.len() as u32).to_le_bytes());
        integrity_data.extend_from_slice(ml_kem_ciphertext);
        
        // Include AES nonce length and data
        integrity_data.extend_from_slice(&(aes_data.nonce.len() as u32).to_le_bytes());
        integrity_data.extend_from_slice(&aes_data.nonce);
        
        // Include AES ciphertext length and data
        integrity_data.extend_from_slice(&(aes_data.ciphertext.len() as u32).to_le_bytes());
        integrity_data.extend_from_slice(&aes_data.ciphertext);
        
        integrity_data
    }
}

/// High-level API for transaction data encryption
pub struct TransactionCrypto;

impl TransactionCrypto {
    /// Encrypt transaction data for bridge operations
    pub fn encrypt_bridge_transaction(
        ml_kem_public_key: &[u8],
        transaction_data: &[u8]
    ) -> Result<HybridEncryptedData, QuantumCryptoError> {
        let context = contexts::bridge_transaction();
        HybridCrypto::encrypt_data(ml_kem_public_key, transaction_data, &context)
    }

    /// Decrypt transaction data for bridge operations
    pub fn decrypt_bridge_transaction(
        ml_kem_private_key: &[u8],
        hybrid_data: &HybridEncryptedData
    ) -> Result<Vec<u8>, QuantumCryptoError> {
        let context = contexts::bridge_transaction();
        HybridCrypto::decrypt_data(ml_kem_private_key, hybrid_data, &context)
    }

    /// Encrypt session keys for key exchange
    pub fn encrypt_session_data(
        ml_kem_public_key: &[u8],
        session_data: &[u8]
    ) -> Result<HybridEncryptedData, QuantumCryptoError> {
        let context = contexts::key_exchange();
        HybridCrypto::encrypt_data(ml_kem_public_key, session_data, &context)
    }

    /// Decrypt session keys for key exchange
    pub fn decrypt_session_data(
        ml_kem_private_key: &[u8],
        hybrid_data: &HybridEncryptedData
    ) -> Result<Vec<u8>, QuantumCryptoError> {
        let context = contexts::key_exchange();
        HybridCrypto::decrypt_data(ml_kem_private_key, hybrid_data, &context)
    }
}

/// Secure memory container for sensitive hybrid data
#[derive(ZeroizeOnDrop)]
pub struct SecureHybridData {
    pub shared_secret: [u8; 32],
    pub aes_key: [u8; 32],
}

impl SecureHybridData {
    pub fn new(shared_secret: [u8; 32], aes_key: [u8; 32]) -> Self {
        Self { shared_secret, aes_key }
    }

    pub fn shared_secret(&self) -> &[u8; 32] {
        &self.shared_secret
    }

    pub fn aes_key(&self) -> &[u8; 32] {
        &self.aes_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MlKemCrypto;

    #[test]
    fn test_hybrid_round_trip() {
        // Generate ML-KEM keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        let public_key = keypair.public_key_bytes();
        let private_key = keypair.private_key_bytes();
        
        let test_data = b"Hello, hybrid cryptography!";
        let context = contexts::bridge_transaction();
        
        // Encrypt
        let encrypted = HybridCrypto::encrypt_data(public_key, test_data, &context).unwrap();
        
        // Decrypt
        let decrypted = HybridCrypto::decrypt_data(private_key, &encrypted, &context).unwrap();
        
        assert_eq!(test_data.as_slice(), decrypted.as_slice());
        assert_eq!(encrypted.scheme_version, HybridCrypto::SCHEME_VERSION);
    }

    #[test]
    fn test_transaction_crypto() {
        // Generate ML-KEM keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        let public_key = keypair.public_key_bytes();
        let private_key = keypair.private_key_bytes();
        
        let transaction_data = b"{'from': 'ethereum', 'to': 'near', 'amount': '100'}";
        
        // Encrypt transaction
        let encrypted = TransactionCrypto::encrypt_bridge_transaction(public_key, transaction_data).unwrap();
        
        // Decrypt transaction
        let decrypted = TransactionCrypto::decrypt_bridge_transaction(private_key, &encrypted).unwrap();
        
        assert_eq!(transaction_data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_integrity_verification_failure() {
        // Generate ML-KEM keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        let public_key = keypair.public_key_bytes();
        let private_key = keypair.private_key_bytes();
        
        let test_data = b"Test data for integrity";
        let context = contexts::bridge_transaction();
        
        // Encrypt
        let mut encrypted = HybridCrypto::encrypt_data(public_key, test_data, &context).unwrap();
        
        // Tamper with ciphertext
        encrypted.aes_encrypted_data.ciphertext[0] ^= 1;
        
        // Attempt to decrypt - should fail
        let result = HybridCrypto::decrypt_data(private_key, &encrypted, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_scheme_version_compatibility() {
        // Generate ML-KEM keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        let public_key = keypair.public_key_bytes();
        let private_key = keypair.private_key_bytes();
        
        let test_data = b"Version compatibility test";
        let context = contexts::bridge_transaction();
        
        // Encrypt
        let mut encrypted = HybridCrypto::encrypt_data(public_key, test_data, &context).unwrap();
        
        // Change version
        encrypted.scheme_version = 99;
        
        // Attempt to decrypt - should fail due to version mismatch
        let result = HybridCrypto::decrypt_data(private_key, &encrypted, &context);
        assert!(result.is_err());
        
        if let Err(QuantumCryptoError::HybridOperationFailed(msg)) = result {
            assert!(msg.contains("Unsupported scheme version"));
        } else {
            panic!("Expected HybridOperationFailed error");
        }
    }

    #[test]
    fn test_different_contexts() {
        // Generate ML-KEM keypair
        let keypair = MlKemCrypto::generate_keypair().unwrap();
        let public_key = keypair.public_key_bytes();
        let private_key = keypair.private_key_bytes();
        
        let test_data = b"Context separation test";
        
        // Encrypt with bridge context
        let encrypted_bridge = HybridCrypto::encrypt_data(public_key, test_data, &contexts::bridge_transaction()).unwrap();
        
        // Try to decrypt with session context - should fail
        let result = HybridCrypto::decrypt_data(private_key, &encrypted_bridge, &contexts::session_keys());
        assert!(result.is_err());
    }
}