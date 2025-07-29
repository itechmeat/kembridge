use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kembridge_common::ServiceResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthServiceError {
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Invalid signature: {message}")]
    InvalidSignature { message: String },

    #[error("Token expired: {token_id}")]
    TokenExpired { token_id: String },

    #[error("Token invalid: {reason}")]
    TokenInvalid { reason: String },

    #[error("User not found: {identifier}")]
    UserNotFound { identifier: String },

    #[error("Wallet address invalid: {address}")]
    InvalidWalletAddress { address: String },

    #[error("Nonce already used: {nonce}")]
    NonceAlreadyUsed { nonce: String },

    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Rate limit exceeded for user: {user_id}")]
    RateLimitExceeded { user_id: String },

    #[error("JWT generation failed: {reason}")]
    JwtGenerationFailed { reason: String },

    #[error("JWT validation failed: {reason}")]
    JwtValidationFailed { reason: String },

    #[error("Wallet connection failed: {wallet_type}")]
    WalletConnectionFailed { wallet_type: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Internal service error: {message}")]
    Internal { message: String },
}

impl AuthServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AuthServiceError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
            AuthServiceError::InvalidSignature { .. } => StatusCode::UNAUTHORIZED,
            AuthServiceError::TokenExpired { .. } => StatusCode::UNAUTHORIZED,
            AuthServiceError::TokenInvalid { .. } => StatusCode::UNAUTHORIZED,
            AuthServiceError::UserNotFound { .. } => StatusCode::NOT_FOUND,
            AuthServiceError::InvalidWalletAddress { .. } => StatusCode::BAD_REQUEST,
            AuthServiceError::NonceAlreadyUsed { .. } => StatusCode::CONFLICT,
            AuthServiceError::SessionExpired { .. } => StatusCode::UNAUTHORIZED,
            AuthServiceError::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            AuthServiceError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            AuthServiceError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AuthServiceError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AuthServiceError::AuthenticationFailed { .. } => "AUTHENTICATION_FAILED",
            AuthServiceError::InvalidSignature { .. } => "INVALID_SIGNATURE",
            AuthServiceError::TokenExpired { .. } => "TOKEN_EXPIRED",
            AuthServiceError::TokenInvalid { .. } => "TOKEN_INVALID",
            AuthServiceError::UserNotFound { .. } => "USER_NOT_FOUND",
            AuthServiceError::InvalidWalletAddress { .. } => "INVALID_WALLET_ADDRESS",
            AuthServiceError::NonceAlreadyUsed { .. } => "NONCE_ALREADY_USED",
            AuthServiceError::SessionExpired { .. } => "SESSION_EXPIRED",
            AuthServiceError::PermissionDenied { .. } => "PERMISSION_DENIED",
            AuthServiceError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AuthServiceError::JwtGenerationFailed { .. } => "JWT_GENERATION_FAILED",
            AuthServiceError::JwtValidationFailed { .. } => "JWT_VALIDATION_FAILED",
            AuthServiceError::WalletConnectionFailed { .. } => "WALLET_CONNECTION_FAILED",
            AuthServiceError::DatabaseError { .. } => "DATABASE_ERROR",
            AuthServiceError::CacheError { .. } => "CACHE_ERROR",
            AuthServiceError::ConfigError { .. } => "CONFIG_ERROR",
            AuthServiceError::ValidationError { .. } => "VALIDATION_ERROR",
            AuthServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        let error_response = ServiceResponse::<()>::error(format!("{}", self));
        
        tracing::error!(
            error_code = self.error_code(),
            status_code = status_code.as_u16(),
            error = %self,
            "Auth service error occurred"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common service errors
impl From<kembridge_common::ServiceError> for AuthServiceError {
    fn from(err: kembridge_common::ServiceError) -> Self {
        match err {
            kembridge_common::ServiceError::InvalidRequest { message } => {
                AuthServiceError::ValidationError { 
                    field: "request".to_string(), 
                    message 
                }
            }
            kembridge_common::ServiceError::AuthenticationFailed { reason } => {
                AuthServiceError::AuthenticationFailed { reason }
            }
            kembridge_common::ServiceError::RateLimitExceeded => {
                AuthServiceError::RateLimitExceeded { user_id: "unknown".to_string() }
            }
            kembridge_common::ServiceError::Validation { field, message } => {
                AuthServiceError::ValidationError { field, message }
            }
            _ => AuthServiceError::Internal { 
                message: err.to_string() 
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, AuthServiceError>;