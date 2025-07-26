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

// Re-export main types for convenience
pub use ml_kem::{MlKemCrypto, MlKemKeyPair, AlgorithmInfo};
pub use key_management::{QuantumKeyManager, QuantumKeyPair, EncapsulationResult};
pub use error::QuantumCryptoError;

// Hybrid cryptography exports (Phase 3.3)
pub use aes_gcm::{AesGcmCrypto, EncryptedData, SecureAesKey};
pub use kdf::{KeyDerivation, DerivedKeys};
pub use integrity::{IntegrityProtection, IntegrityProof};
pub use hybrid_crypto::{HybridCrypto, HybridEncryptedData, TransactionCrypto, SecureHybridData};

// Legacy modules removed in Phase 3.4 - hybrid cryptography is now production ready