// Authentication error types for KEMBridge

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid wallet signature")]
    InvalidSignature,

    #[error("Invalid or expired nonce")]
    InvalidNonce,

    #[error("Nonce expired")]
    NonceExpired,

    #[error("Invalid wallet address format")]
    InvalidWalletAddress,

    #[error("Unsupported chain type: {0}")]
    UnsupportedChainType(String),

    #[error("JWT token error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AuthError> for axum::http::StatusCode {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidSignature
            | AuthError::InvalidNonce
            | AuthError::NonceExpired
            | AuthError::JwtError(_) => axum::http::StatusCode::UNAUTHORIZED,
            
            AuthError::InvalidWalletAddress
            | AuthError::UnsupportedChainType(_)
            | AuthError::Validation(_) => axum::http::StatusCode::BAD_REQUEST,

            AuthError::Database(_)
            | AuthError::Redis(_)
            | AuthError::Serialization(_)
            | AuthError::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}