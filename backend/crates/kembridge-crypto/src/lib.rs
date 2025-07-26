//! KEMBridge Post-Quantum Cryptography Module
//! 
//! This crate implements FIPS 203 ML-KEM-1024 for post-quantum key encapsulation.
//! ML-KEM (Module-Lattice-Based Key-Encapsulation Mechanism) provides quantum-safe
//! cryptographic key exchange with 256-bit security level.
//!
//! ## Phase 3.1 Implementation
//! 
//! This phase focuses on the core ML-KEM-1024 cryptographic functionality:
//! - Basic key generation and round-trip verification
//! - Error handling and algorithm information
//! - Foundation for Phase 3.2 key management and serialization

pub mod ml_kem;
pub mod key_management;
pub mod error;

// Re-export main types for convenience
pub use ml_kem::{MlKemCrypto, MlKemKeyPair, AlgorithmInfo};
pub use key_management::{QuantumKeyManager, QuantumKeyPair, EncapsulationResult};
pub use error::QuantumCryptoError;

// Legacy compatibility (will be deprecated in Phase 3.3)
pub mod kyber;
pub mod dilithium;
pub mod sphincs;

pub use kyber::*;
pub use dilithium::*;
pub use sphincs::*;