//! KEMBridge Post-Quantum Cryptography Module
//! 
//! This crate implements FIPS 203 ML-KEM-1024 for post-quantum key encapsulation
//! and hybrid cryptography combining ML-KEM with AES-256-GCM.
//!
//! ## Phase 3.1 - ML-KEM-1024 Implementation
//! Core post-quantum key encapsulation functionality
//!
//! ## Phase 3.3 - Hybrid Cryptography 
//! Combines ML-KEM with classical AES-256-GCM for optimal security and performance

// Core ML-KEM implementation (Phase 3.1)
pub mod ml_kem;
pub mod key_management;
pub mod error;

// Hybrid cryptography modules (Phase 3.3)
pub mod aes_gcm;
pub mod kdf;
pub mod integrity;
pub mod hybrid_crypto;

// Transaction-specific cryptography (Phase 8.1.2)
pub mod transaction_crypto;

// Operation-specific key derivation (Phase 8.1.2)
pub mod operation_keys;

// Cross-chain quantum authentication (Phase 8.1.2)
pub mod cross_chain_auth;

// Re-export main types for convenience
pub use ml_kem::{MlKemCrypto, MlKemKeyPair, AlgorithmInfo};
pub use key_management::{QuantumKeyManager, QuantumKeyPair, EncapsulationResult};
pub use error::QuantumCryptoError;

// Hybrid cryptography exports (Phase 3.3)
pub use aes_gcm::{AesGcmCrypto, EncryptedData, SecureAesKey};
pub use kdf::{KeyDerivation, DerivedKeys};
pub use integrity::{IntegrityProtection, IntegrityProof};
pub use hybrid_crypto::{HybridCrypto, HybridEncryptedData, TransactionCrypto, SecureHybridData};

// Transaction cryptography exports (Phase 8.1.2)
pub use transaction_crypto::{
    QuantumTransactionCrypto, QuantumTransaction, SensitiveTransactionData,
    QuantumProtectedAddresses, TransactionEncryptionMetadata, TransactionDataType
};

// Operation key derivation exports (Phase 8.1.2)
pub use operation_keys::{
    OperationKeyManager, OperationKeys, OperationType, OperationKeyContext
};

// Cross-chain authentication exports (Phase 8.1.2)
pub use cross_chain_auth::{
    CrossChainAuthenticator, QuantumAuthenticatedMessage, QuantumMessageSignature,
    MessageVerificationResult, CrossChainMessageType, AlertSeverity
};

// Legacy modules removed in Phase 3.4 - hybrid cryptography is now production ready

// Test modules
#[cfg(test)]
pub mod tests {
    pub mod transaction_crypto_tests;
    pub mod operation_keys_tests;
    pub mod cross_chain_auth_tests;
    pub mod quantum_integration_tests;
}