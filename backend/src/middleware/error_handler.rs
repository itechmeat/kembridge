// src/middleware/error_handler.rs - Comprehensive error handling
use axum::{
    http::{StatusCode, HeaderValue},
    response::{Json, Response, IntoResponse},
    middleware::Next,
    extract::Request,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Application-wide error types with detailed context
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: insufficient permissions")]
    Authorization,

    #[error("Quantum signature verification failed")]
    QuantumSignature,

    #[error("Bridge operation failed: {0}")]
    Bridge(String),

    #[error("Rate limit exceeded: {limit} requests per {window} seconds")]
    RateLimit { limit: u32, window: u64 },

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("AI Risk Engine error: {0}")]
    AiRiskEngine(String),

    #[error("Blockchain adapter error: {chain}: {message}")]
    Blockchain { chain: String, message: String },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("External service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Request timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ApiError {
    /// Create an unauthorized error (convenience method)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    /// Create a not found error (convenience method)
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound { resource: resource.into() }
    }

    /// Create an internal server error (convenience method)
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Create a bad request error (convenience method)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::Validation { 
            field: "request".to_string(), 
            message: message.into() 
        }
    }

    /// Convert error to HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Authentication(_) | Self::QuantumSignature => StatusCode::UNAUTHORIZED,
            Self::Authorization => StatusCode::FORBIDDEN,
            Self::Validation { .. } | Self::Bridge(_) | Self::Blockchain { .. } => StatusCode::BAD_REQUEST,
            Self::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Database(_) | Self::Redis(_) | Self::Internal(_) | 
            Self::AiRiskEngine(_) | Self::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Redis(_) => "CACHE_ERROR", 
            Self::Authentication(_) => "AUTH_FAILED",
            Self::Authorization => "ACCESS_DENIED",
            Self::QuantumSignature => "QUANTUM_SIGNATURE_INVALID",
            Self::Bridge(_) => "BRIDGE_ERROR",
            Self::RateLimit { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::AiRiskEngine(_) => "AI_ENGINE_ERROR",
            Self::Blockchain { .. } => "BLOCKCHAIN_ERROR",
            Self::Configuration(_) => "CONFIG_ERROR",
            Self::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Self::Timeout { .. } => "REQUEST_TIMEOUT",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Check if error should be logged as an error vs warning
    pub fn should_log_as_error(&self) -> bool {
        match self {
            // Client errors - log as warnings
            Self::Authentication(_) | Self::Authorization | Self::Validation { .. } |
            Self::RateLimit { .. } | Self::NotFound { .. } | Self::Conflict { .. } => false,
            
            // Server/infrastructure errors - log as errors
            Self::Database(_) | Self::Redis(_) | Self::Internal(_) |
            Self::AiRiskEngine(_) | Self::Configuration(_) |
            Self::ServiceUnavailable { .. } | Self::Timeout { .. } => true,
            
            // Context-dependent errors
            Self::QuantumSignature | Self::Bridge(_) | Self::Blockchain { .. } => false,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_id = Uuid::new_v4();
        
        // Log error with appropriate level
        if self.should_log_as_error() {
            tracing::error!(
                error_id = %error_id,
                error_code = self.error_code(),
                error = %self,
                "API Error occurred"
            );
        } else {
            tracing::warn!(
                error_id = %error_id,
                error_code = self.error_code(), 
                error = %self,
                "API Warning"
            );
        }

        let body = Json(json!({
            "error": {
                "id": error_id,
                "code": self.error_code(),
                "message": self.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "details": match &self {
                    Self::Validation { field, message } => Some(json!({
                        "field": field,
                        "validation_message": message
                    })),
                    Self::RateLimit { limit, window } => Some(json!({
                        "limit": limit,
                        "window_seconds": window,
                        "retry_after": window
                    })),
                    Self::Blockchain { chain, message } => Some(json!({
                        "chain": chain,
                        "blockchain_message": message
                    })),
                    Self::ServiceUnavailable { service } => Some(json!({
                        "service": service
                    })),
                    Self::Timeout { seconds } => Some(json!({
                        "timeout_seconds": seconds
                    })),
                    Self::NotFound { resource } => Some(json!({
                        "resource": resource
                    })),
                    Self::Conflict { message } => Some(json!({
                        "conflict_reason": message
                    })),
                    _ => None
                }
            }
        }));

        (status, body).into_response()
    }
}

/// Global error handling middleware
pub async fn handle_error(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string(); // Convert to owned string

    let method = request.method().clone();
    let uri = request.uri().clone();
    let start_time = std::time::Instant::now();

    let response = next.run(request).await;
    let elapsed = start_time.elapsed();

    // Add response headers
    let mut response = response;
    let headers = response.headers_mut();
    
    // Add request ID to response
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        headers.insert("x-request-id", header_value);
    }
    
    // Add response time
    if let Ok(response_time) = HeaderValue::from_str(&format!("{}ms", elapsed.as_millis())) {
        headers.insert("x-response-time", response_time);
    }

    // Log request completion
    if response.status().is_client_error() || response.status().is_server_error() {
        tracing::warn!(
            method = %method,
            uri = %uri,
            status = %response.status(),
            elapsed_ms = elapsed.as_millis(),
            request_id = request_id,
            "Request completed with error"
        );
    } else {
        tracing::debug!(
            method = %method,
            uri = %uri, 
            status = %response.status(),
            elapsed_ms = elapsed.as_millis(),
            request_id = request_id,
            "Request completed successfully"
        );
    }

    response
}

/// Convert common errors to ApiError
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to known error types for proper error conversion
        // Note: We can't clone these errors, so we create descriptive versions
        if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
            // Create a new error with the same message but different structure
            return Self::Internal(format!("Database error: {}", sqlx_err));
        }
        
        if let Some(redis_err) = err.downcast_ref::<redis::RedisError>() {
            // Create a new error with the same message but different structure  
            return Self::Internal(format!("Redis error: {}", redis_err));
        }

        // Default to internal error
        tracing::error!("Unhandled error converted to ApiError: {:#}", err);
        Self::Internal(format!("Unhandled error: {}", err))
    }
}

/// Helper macro for creating validation errors
#[macro_export]
macro_rules! validation_error {
    ($field:expr, $message:expr) => {
        ApiError::Validation {
            field: $field.to_string(),
            message: $message.to_string(),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::Authentication("test".to_string()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Authorization.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::NotFound { resource: "user".to_string() }.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::RateLimit { limit: 100, window: 60 }.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(ApiError::Internal.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ApiError::Authentication("test".to_string()).error_code(), "AUTH_FAILED");
        assert_eq!(ApiError::Validation { field: "email".to_string(), message: "invalid".to_string() }.error_code(), "VALIDATION_ERROR");
        assert_eq!(ApiError::RateLimit { limit: 100, window: 60 }.error_code(), "RATE_LIMIT_EXCEEDED");
    }

    #[test] 
    fn test_should_log_as_error() {
        // Client errors should not log as errors
        assert!(!ApiError::Authentication("test".to_string()).should_log_as_error());
        assert!(!ApiError::Validation { field: "test".to_string(), message: "test".to_string() }.should_log_as_error());
        
        // Server errors should log as errors
        assert!(ApiError::Database(sqlx::Error::RowNotFound).should_log_as_error());
        assert!(ApiError::Internal.should_log_as_error());
    }
}