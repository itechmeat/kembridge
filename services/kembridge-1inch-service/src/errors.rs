use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kembridge_common::ServiceResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OneinchServiceError {
    #[error("1inch API error: {message}")]
    OneinchApi { message: String },

    #[error("Invalid quote: {reason}")]
    InvalidQuote { reason: String },

    #[error("Quote expired: {quote_id}")]
    QuoteExpired { quote_id: String },

    #[error("Insufficient liquidity for amount: {amount}")]
    InsufficientLiquidity { amount: String },

    #[error("Token not supported: {token} on chain {chain_id}")]
    TokenNotSupported { token: String, chain_id: u64 },

    #[error("Chain not supported: {chain_id}")]
    ChainNotSupported { chain_id: u64 },

    #[error("Invalid slippage: {slippage}% (must be between 0.1% and 50%)")]
    InvalidSlippage { slippage: String },

    #[error("Price impact too high: {impact}% (maximum allowed: {max}%)")]
    PriceImpactTooHigh { impact: String, max: String },

    #[error("Gas estimation failed: {reason}")]
    GasEstimationFailed { reason: String },

    #[error("Swap execution failed: {reason}")]
    SwapExecutionFailed { reason: String },

    #[error("Order not found: {order_hash}")]
    OrderNotFound { order_hash: String },

    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Network timeout: {operation}")]
    NetworkTimeout { operation: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Cross-chain operation failed: {reason}")]
    CrossChainError { reason: String },

    #[error("Price oracle error: {message}")]
    PriceOracleError { message: String },

    #[error("Internal service error: {message}")]
    Internal { message: String },
}

impl OneinchServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            OneinchServiceError::InvalidQuote { .. } => StatusCode::BAD_REQUEST,
            OneinchServiceError::InvalidSlippage { .. } => StatusCode::BAD_REQUEST,
            OneinchServiceError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            OneinchServiceError::QuoteExpired { .. } => StatusCode::GONE,
            OneinchServiceError::TokenNotSupported { .. } => StatusCode::NOT_FOUND,
            OneinchServiceError::ChainNotSupported { .. } => StatusCode::NOT_FOUND,
            OneinchServiceError::OrderNotFound { .. } => StatusCode::NOT_FOUND,
            OneinchServiceError::InsufficientLiquidity { .. } => StatusCode::CONFLICT,
            OneinchServiceError::PriceImpactTooHigh { .. } => StatusCode::CONFLICT,
            OneinchServiceError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            OneinchServiceError::NetworkTimeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            OneinchServiceError::OneinchApi { .. } => StatusCode::BAD_GATEWAY,
            OneinchServiceError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            OneinchServiceError::OneinchApi { .. } => "ONEINCH_API_ERROR",
            OneinchServiceError::InvalidQuote { .. } => "INVALID_QUOTE",
            OneinchServiceError::QuoteExpired { .. } => "QUOTE_EXPIRED",
            OneinchServiceError::InsufficientLiquidity { .. } => "INSUFFICIENT_LIQUIDITY",
            OneinchServiceError::TokenNotSupported { .. } => "TOKEN_NOT_SUPPORTED",
            OneinchServiceError::ChainNotSupported { .. } => "CHAIN_NOT_SUPPORTED",
            OneinchServiceError::InvalidSlippage { .. } => "INVALID_SLIPPAGE",
            OneinchServiceError::PriceImpactTooHigh { .. } => "PRICE_IMPACT_TOO_HIGH",
            OneinchServiceError::GasEstimationFailed { .. } => "GAS_ESTIMATION_FAILED",
            OneinchServiceError::SwapExecutionFailed { .. } => "SWAP_EXECUTION_FAILED",
            OneinchServiceError::OrderNotFound { .. } => "ORDER_NOT_FOUND",
            OneinchServiceError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            OneinchServiceError::NetworkTimeout { .. } => "NETWORK_TIMEOUT",
            OneinchServiceError::CacheError { .. } => "CACHE_ERROR",
            OneinchServiceError::ConfigError { .. } => "CONFIG_ERROR",
            OneinchServiceError::ValidationError { .. } => "VALIDATION_ERROR",
            OneinchServiceError::CrossChainError { .. } => "CROSS_CHAIN_ERROR",
            OneinchServiceError::PriceOracleError { .. } => "PRICE_ORACLE_ERROR",
            OneinchServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for OneinchServiceError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();

        let error_response = ServiceResponse::<()>::error(format!("{}", self));

        tracing::error!(
            error_code = self.error_code(),
            status_code = status_code.as_u16(),
            error = %self,
            "1inch service error occurred"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common service errors
impl From<kembridge_common::ServiceError> for OneinchServiceError {
    fn from(err: kembridge_common::ServiceError) -> Self {
        match err {
            kembridge_common::ServiceError::InvalidRequest { message } => {
                OneinchServiceError::ValidationError {
                    field: "request".to_string(),
                    message,
                }
            }
            kembridge_common::ServiceError::Network { message } => {
                OneinchServiceError::NetworkTimeout { operation: message }
            }
            kembridge_common::ServiceError::ExternalService { service, message } => {
                if service.contains("1inch") {
                    OneinchServiceError::OneinchApi { message }
                } else {
                    OneinchServiceError::Internal { message }
                }
            }
            kembridge_common::ServiceError::RateLimitExceeded => {
                OneinchServiceError::RateLimitExceeded { retry_after: 60 }
            }
            kembridge_common::ServiceError::Timeout { operation } => {
                OneinchServiceError::NetworkTimeout { operation }
            }
            kembridge_common::ServiceError::Validation { field, message } => {
                OneinchServiceError::ValidationError { field, message }
            }
            _ => OneinchServiceError::Internal {
                message: err.to_string(),
            },
        }
    }
}

// Helper for creating errors
impl OneinchServiceError {
    pub fn oneinch_api<S: Into<String>>(message: S) -> Self {
        Self::OneinchApi {
            message: message.into(),
        }
    }

    pub fn invalid_quote<S: Into<String>>(reason: S) -> Self {
        Self::InvalidQuote {
            reason: reason.into(),
        }
    }

    pub fn quote_expired<S: Into<String>>(quote_id: S) -> Self {
        Self::QuoteExpired {
            quote_id: quote_id.into(),
        }
    }

    pub fn token_not_supported<S: Into<String>>(token: S, chain_id: u64) -> Self {
        Self::TokenNotSupported {
            token: token.into(),
            chain_id,
        }
    }

    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, OneinchServiceError>;
