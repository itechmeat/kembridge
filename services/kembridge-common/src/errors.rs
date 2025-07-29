use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Authorization failed: {reason}")]
    AuthorizationFailed { reason: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Timeout: {operation}")]
    Timeout { operation: String },
}

impl ServiceError {
    pub fn to_status_code(&self) -> u16 {
        match self {
            ServiceError::InvalidRequest { .. } => 400,
            ServiceError::AuthenticationFailed { .. } => 401,
            ServiceError::AuthorizationFailed { .. } => 403,
            ServiceError::NotFound { .. } => 404,
            ServiceError::RateLimitExceeded => 429,
            ServiceError::ServiceUnavailable { .. } => 503,
            ServiceError::Timeout { .. } => 504,
            ServiceError::Validation { .. } => 422,
            _ => 500,
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;