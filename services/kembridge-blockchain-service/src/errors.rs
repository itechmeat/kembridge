use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kembridge_common::ServiceResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainServiceError {
    #[error("Ethereum RPC error: {message}")]
    EthereumRpc { message: String },

    #[error("NEAR RPC error: {message}")]
    NearRpc { message: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Chain not supported: {chain_id}")]
    ChainNotSupported { chain_id: u64 },

    #[error("Gas estimation failed: {reason}")]
    GasEstimationFailed { reason: String },

    #[error("Signature verification failed: {reason}")]
    SignatureVerificationFailed { reason: String },

    #[error("Chain signature error: {message}")]
    ChainSignatureError { message: String },

    #[error("Bridge operation failed: {reason}")]
    BridgeOperationFailed { reason: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Network timeout: {operation}")]
    NetworkTimeout { operation: String },

    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Internal service error: {message}")]
    Internal { message: String },
}

impl BlockchainServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            BlockchainServiceError::InvalidAddress { .. } => StatusCode::BAD_REQUEST,
            BlockchainServiceError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            BlockchainServiceError::InsufficientBalance { .. } => StatusCode::CONFLICT,
            BlockchainServiceError::ChainNotSupported { .. } => StatusCode::NOT_FOUND,
            BlockchainServiceError::SignatureVerificationFailed { .. } => StatusCode::UNAUTHORIZED,
            BlockchainServiceError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            BlockchainServiceError::NetworkTimeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            BlockchainServiceError::EthereumRpc { .. } => StatusCode::BAD_GATEWAY,
            BlockchainServiceError::NearRpc { .. } => StatusCode::BAD_GATEWAY,
            BlockchainServiceError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            BlockchainServiceError::EthereumRpc { .. } => "ETHEREUM_RPC_ERROR",
            BlockchainServiceError::NearRpc { .. } => "NEAR_RPC_ERROR",
            BlockchainServiceError::InvalidAddress { .. } => "INVALID_ADDRESS",
            BlockchainServiceError::TransactionFailed { .. } => "TRANSACTION_FAILED",
            BlockchainServiceError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            BlockchainServiceError::ChainNotSupported { .. } => "CHAIN_NOT_SUPPORTED",
            BlockchainServiceError::GasEstimationFailed { .. } => "GAS_ESTIMATION_FAILED",
            BlockchainServiceError::SignatureVerificationFailed { .. } => "SIGNATURE_VERIFICATION_FAILED",
            BlockchainServiceError::ChainSignatureError { .. } => "CHAIN_SIGNATURE_ERROR",
            BlockchainServiceError::BridgeOperationFailed { .. } => "BRIDGE_OPERATION_FAILED",
            BlockchainServiceError::DatabaseError { .. } => "DATABASE_ERROR",
            BlockchainServiceError::CacheError { .. } => "CACHE_ERROR",
            BlockchainServiceError::ConfigError { .. } => "CONFIG_ERROR",
            BlockchainServiceError::NetworkTimeout { .. } => "NETWORK_TIMEOUT",
            BlockchainServiceError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            BlockchainServiceError::ValidationError { .. } => "VALIDATION_ERROR",
            BlockchainServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for BlockchainServiceError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        let error_response = ServiceResponse::<()>::error(format!("{}", self));
        
        tracing::error!(
            error_code = self.error_code(),
            status_code = status_code.as_u16(),
            error = %self,
            "Blockchain service error occurred"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common service errors
impl From<kembridge_common::ServiceError> for BlockchainServiceError {
    fn from(err: kembridge_common::ServiceError) -> Self {
        match err {
            kembridge_common::ServiceError::InvalidRequest { message } => {
                BlockchainServiceError::ValidationError { 
                    field: "request".to_string(), 
                    message 
                }
            }
            kembridge_common::ServiceError::Network { message } => {
                BlockchainServiceError::NetworkTimeout { 
                    operation: message 
                }
            }
            kembridge_common::ServiceError::ExternalService { service, message } => {
                if service.contains("ethereum") {
                    BlockchainServiceError::EthereumRpc { message }
                } else if service.contains("near") {
                    BlockchainServiceError::NearRpc { message }
                } else {
                    BlockchainServiceError::Internal { message }
                }
            }
            kembridge_common::ServiceError::RateLimitExceeded => {
                BlockchainServiceError::RateLimitExceeded { retry_after: 60 }
            }
            kembridge_common::ServiceError::Timeout { operation } => {
                BlockchainServiceError::NetworkTimeout { operation }
            }
            kembridge_common::ServiceError::Validation { field, message } => {
                BlockchainServiceError::ValidationError { field, message }
            }
            _ => BlockchainServiceError::Internal { 
                message: err.to_string() 
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, BlockchainServiceError>;