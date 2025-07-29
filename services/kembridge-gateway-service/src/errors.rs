use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kembridge_common::ServiceResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayServiceError {
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Upstream service error: {service} - {message}")]
    UpstreamServiceError { service: String, message: String },

    #[error("Routing failed: {path}")]
    RoutingFailed { path: String },

    #[error("Load balancing failed: {reason}")]
    LoadBalancingFailed { reason: String },

    #[error("Circuit breaker open for service: {service}")]
    CircuitBreakerOpen { service: String },

    #[error("Timeout calling service: {service}")]
    ServiceTimeout { service: String },

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Authorization failed: {reason}")]
    AuthorizationFailed { reason: String },

    #[error("Rate limit exceeded for client: {client_id}")]
    RateLimitExceeded { client_id: String },

    #[error("Request too large: {size} bytes")]
    RequestTooLarge { size: usize },

    #[error("Invalid request format: {message}")]
    InvalidRequestFormat { message: String },

    #[error("Service discovery failed: {service}")]
    ServiceDiscoveryFailed { service: String },

    #[error("Health check failed: {service}")]
    HealthCheckFailed { service: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Internal gateway error: {message}")]
    Internal { message: String },
}

impl GatewayServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            GatewayServiceError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            GatewayServiceError::UpstreamServiceError { .. } => StatusCode::BAD_GATEWAY,
            GatewayServiceError::RoutingFailed { .. } => StatusCode::NOT_FOUND,
            GatewayServiceError::LoadBalancingFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,
            GatewayServiceError::CircuitBreakerOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
            GatewayServiceError::ServiceTimeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            GatewayServiceError::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            GatewayServiceError::AuthorizationFailed { .. } => StatusCode::FORBIDDEN,
            GatewayServiceError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            GatewayServiceError::RequestTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            GatewayServiceError::InvalidRequestFormat { .. } => StatusCode::BAD_REQUEST,
            GatewayServiceError::ServiceDiscoveryFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,
            GatewayServiceError::HealthCheckFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,
            GatewayServiceError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            GatewayServiceError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            GatewayServiceError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            GatewayServiceError::UpstreamServiceError { .. } => "UPSTREAM_SERVICE_ERROR",
            GatewayServiceError::RoutingFailed { .. } => "ROUTING_FAILED",
            GatewayServiceError::LoadBalancingFailed { .. } => "LOAD_BALANCING_FAILED",
            GatewayServiceError::CircuitBreakerOpen { .. } => "CIRCUIT_BREAKER_OPEN",
            GatewayServiceError::ServiceTimeout { .. } => "SERVICE_TIMEOUT",
            GatewayServiceError::AuthenticationRequired => "AUTHENTICATION_REQUIRED",
            GatewayServiceError::AuthorizationFailed { .. } => "AUTHORIZATION_FAILED",
            GatewayServiceError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            GatewayServiceError::RequestTooLarge { .. } => "REQUEST_TOO_LARGE",
            GatewayServiceError::InvalidRequestFormat { .. } => "INVALID_REQUEST_FORMAT",
            GatewayServiceError::ServiceDiscoveryFailed { .. } => "SERVICE_DISCOVERY_FAILED",
            GatewayServiceError::HealthCheckFailed { .. } => "HEALTH_CHECK_FAILED",
            GatewayServiceError::ConfigError { .. } => "CONFIG_ERROR",
            GatewayServiceError::ValidationError { .. } => "VALIDATION_ERROR",
            GatewayServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for GatewayServiceError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        let error_response = ServiceResponse::<()>::error(format!("{}", self));
        
        tracing::error!(
            error_code = self.error_code(),
            status_code = status_code.as_u16(),
            error = %self,
            "Gateway service error occurred"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common service errors
impl From<kembridge_common::ServiceError> for GatewayServiceError {
    fn from(err: kembridge_common::ServiceError) -> Self {
        match err {
            kembridge_common::ServiceError::InvalidRequest { message } => {
                GatewayServiceError::ValidationError { 
                    field: "request".to_string(), 
                    message 
                }
            }
            kembridge_common::ServiceError::AuthenticationFailed { reason } => {
                GatewayServiceError::AuthorizationFailed { reason }
            }
            kembridge_common::ServiceError::ServiceUnavailable { service } => {
                GatewayServiceError::ServiceUnavailable { service }
            }
            kembridge_common::ServiceError::ExternalService { service, message } => {
                GatewayServiceError::UpstreamServiceError { service, message }
            }
            kembridge_common::ServiceError::Timeout { operation } => {
                GatewayServiceError::ServiceTimeout { service: operation }
            }
            kembridge_common::ServiceError::RateLimitExceeded => {
                GatewayServiceError::RateLimitExceeded { client_id: "unknown".to_string() }
            }
            kembridge_common::ServiceError::Validation { field, message } => {
                GatewayServiceError::ValidationError { field, message }
            }
            _ => GatewayServiceError::Internal { 
                message: err.to_string() 
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, GatewayServiceError>;