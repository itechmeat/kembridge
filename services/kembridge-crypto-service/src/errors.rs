use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kembridge_common::ServiceResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoServiceError {
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Invalid key format: {message}")]
    InvalidKeyFormat { message: String },

    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Decryption failed: {reason}")]
    DecryptionFailed { reason: String },

    #[error("Signature generation failed: {reason}")]
    SignatureFailed { reason: String },

    #[error("Signature verification failed: {reason}")]
    SignatureVerificationFailed { reason: String },

    #[error("Key derivation failed: {reason}")]
    KeyDerivationFailed { reason: String },

    #[error("Algorithm not supported: {algorithm}")]
    AlgorithmNotSupported { algorithm: String },

    #[error("Key size invalid: {size}")]
    InvalidKeySize { size: usize },

    #[error("Random number generation failed: {reason}")]
    RandomGenerationFailed { reason: String },

    #[error("Post-quantum algorithm error: {message}")]
    PostQuantumError { message: String },

    #[error("HSM operation failed: {operation}")]
    HsmOperationFailed { operation: String },

    #[error("Key rotation failed: {reason}")]
    KeyRotationFailed { reason: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Internal service error: {message}")]
    Internal { message: String },
}

impl CryptoServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CryptoServiceError::KeyNotFound { .. } => StatusCode::NOT_FOUND,
            CryptoServiceError::InvalidKeyFormat { .. } => StatusCode::BAD_REQUEST,
            CryptoServiceError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            CryptoServiceError::AlgorithmNotSupported { .. } => StatusCode::NOT_IMPLEMENTED,
            CryptoServiceError::InvalidKeySize { .. } => StatusCode::BAD_REQUEST,
            CryptoServiceError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            CryptoServiceError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            CryptoServiceError::KeyGenerationFailed { .. } => "KEY_GENERATION_FAILED",
            CryptoServiceError::KeyNotFound { .. } => "KEY_NOT_FOUND",
            CryptoServiceError::InvalidKeyFormat { .. } => "INVALID_KEY_FORMAT",
            CryptoServiceError::EncryptionFailed { .. } => "ENCRYPTION_FAILED",
            CryptoServiceError::DecryptionFailed { .. } => "DECRYPTION_FAILED",
            CryptoServiceError::SignatureFailed { .. } => "SIGNATURE_FAILED",
            CryptoServiceError::SignatureVerificationFailed { .. } => "SIGNATURE_VERIFICATION_FAILED",
            CryptoServiceError::KeyDerivationFailed { .. } => "KEY_DERIVATION_FAILED",
            CryptoServiceError::AlgorithmNotSupported { .. } => "ALGORITHM_NOT_SUPPORTED",
            CryptoServiceError::InvalidKeySize { .. } => "INVALID_KEY_SIZE",
            CryptoServiceError::RandomGenerationFailed { .. } => "RANDOM_GENERATION_FAILED",
            CryptoServiceError::PostQuantumError { .. } => "POST_QUANTUM_ERROR",
            CryptoServiceError::HsmOperationFailed { .. } => "HSM_OPERATION_FAILED",
            CryptoServiceError::KeyRotationFailed { .. } => "KEY_ROTATION_FAILED",
            CryptoServiceError::DatabaseError { .. } => "DATABASE_ERROR",
            CryptoServiceError::CacheError { .. } => "CACHE_ERROR",
            CryptoServiceError::ConfigError { .. } => "CONFIG_ERROR",
            CryptoServiceError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            CryptoServiceError::ValidationError { .. } => "VALIDATION_ERROR",
            CryptoServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for CryptoServiceError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        let error_response = ServiceResponse::<()>::error(format!("{}", self));
        
        tracing::error!(
            error_code = self.error_code(),
            status_code = status_code.as_u16(),
            error = %self,
            "Crypto service error occurred"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common service errors
impl From<kembridge_common::ServiceError> for CryptoServiceError {
    fn from(err: kembridge_common::ServiceError) -> Self {
        match err {
            kembridge_common::ServiceError::InvalidRequest { message } => {
                CryptoServiceError::ValidationError { 
                    field: "request".to_string(), 
                    message 
                }
            }
            kembridge_common::ServiceError::ExternalService { service, message } => {
                if service.contains("hsm") {
                    CryptoServiceError::HsmOperationFailed { operation: message }
                } else {
                    CryptoServiceError::Internal { message }
                }
            }
            kembridge_common::ServiceError::RateLimitExceeded => {
                CryptoServiceError::RateLimitExceeded { retry_after: 60 }
            }
            kembridge_common::ServiceError::Validation { field, message } => {
                CryptoServiceError::ValidationError { field, message }
            }
            _ => CryptoServiceError::Internal { 
                message: err.to_string() 
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, CryptoServiceError>;