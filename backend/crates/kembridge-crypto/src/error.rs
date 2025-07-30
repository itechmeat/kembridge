//! Error handling for post-quantum cryptography operations

use thiserror::Error;

/// Errors that can occur during quantum cryptographic operations
#[derive(Debug, Error)]
pub enum QuantumCryptoError {
    /// Failed to generate cryptographic keys
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    
    /// Failed to encapsulate shared secret
    #[error("Encapsulation failed")]
    EncapsulationFailed,
    
    /// Failed to decapsulate shared secret
    #[error("Decapsulation failed")]
    DecapsulationFailed,
    
    /// Invalid key format or corrupted key data
    #[error("Invalid key")]
    InvalidKey,
    
    /// Invalid input data format
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Serialization or deserialization error
    #[error("Serialization error")]
    SerializationError,
    
    /// Random number generation failed
    #[error("Random number generation failed: {0}")]
    RandomGenerationFailed(String),
    
    /// Memory allocation or zeroization failed
    #[error("Memory operation failed: {0}")]
    MemoryError(String),
    
    /// Verification failed
    #[error("Verification failed")]
    VerificationFailed,
    
    /// Generic internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    // === Hybrid Cryptography Errors (Phase 3.3) ===
    
    /// AES encryption failed
    #[error("AES encryption failed: {0}")]
    EncryptionFailed(String),
    
    /// AES decryption failed
    #[error("AES decryption failed: {0}")]
    DecryptionFailed(String),
    
    /// Key derivation function failed
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    /// HMAC generation or verification failed
    #[error("MAC operation failed: {0}")]
    MacError(String),
    
    /// Invalid key size for cryptographic operation
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },
    
    /// Invalid data format or structure
    #[error("Invalid data format: {0}")]
    InvalidData(String),
    
    /// Hybrid scheme operation failed
    #[error("Hybrid cryptography operation failed: {0}")]
    HybridOperationFailed(String),
}

impl QuantumCryptoError {
    /// Create a new key generation error
    pub fn key_generation(msg: impl Into<String>) -> Self {
        Self::KeyGenerationFailed(msg.into())
    }
    
    /// Create a new encapsulation error
    pub fn encapsulation() -> Self {
        Self::EncapsulationFailed
    }
    
    /// Create a new decapsulation error
    pub fn decapsulation() -> Self {
        Self::DecapsulationFailed
    }
    
    /// Create a new invalid key error
    pub fn invalid_key(_msg: impl Into<String>) -> Self {
        Self::InvalidKey
    }
    
    /// Create a new invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

/// Result type for quantum cryptographic operations
pub type QuantumResult<T> = Result<T, QuantumCryptoError>;