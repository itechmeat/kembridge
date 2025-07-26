// TODO: These are mock implementations - Phase 3.1 will add real post-quantum crypto
#![allow(unused_variables)]

pub mod kyber;
pub mod dilithium;
pub mod sphincs;

pub use kyber::*;
pub use dilithium::*;
pub use sphincs::*;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
}