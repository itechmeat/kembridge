// HTTP handlers for Web3 authentication endpoints

use axum::{
    extract::{State, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use validator::Validate;
use crate::{
    AuthService,
    models::{NonceRequest, NonceResponse, AuthRequest, AuthResponse, ErrorResponse},
    errors::AuthError,
};

pub async fn generate_nonce_handler(
    State(auth_service): State<AuthService>,
    Json(request): Json<NonceRequest>,
) -> Result<ResponseJson<NonceResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        let error_message = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors.iter().map(|e| e.message.as_ref().unwrap_or(&"invalid".into()).to_string()).collect();
                format!("{}: {}", field, error_messages.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");
        
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse::new("validation_error", &error_message)),
        ));
    }

    match auth_service
        .generate_nonce(&request.wallet_address, request.chain_type)
        .await
    {
        Ok(response) => Ok(ResponseJson(response)),
        Err(error) => {
            let error_message = format!("Failed to generate nonce: {}", error);
            let status_code = StatusCode::from(error);
            let error_response = ErrorResponse::new("nonce_generation_failed", &error_message);
            Err((status_code, ResponseJson(error_response)))
        }
    }
}

pub async fn verify_wallet_handler(
    State(auth_service): State<AuthService>,
    Json(request): Json<AuthRequest>,
) -> Result<ResponseJson<AuthResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        let error_message = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors.iter().map(|e| e.message.as_ref().unwrap_or(&"invalid".into()).to_string()).collect();
                format!("{}: {}", field, error_messages.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");
        
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse::new("validation_error", &error_message)),
        ));
    }

    match auth_service.verify_wallet_signature(request).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(error) => {
            let error_response = match &error {
                AuthError::InvalidSignature => ErrorResponse::new(
                    "invalid_signature",
                    "The provided signature is invalid or does not match the wallet address",
                ),
                AuthError::InvalidNonce | AuthError::NonceExpired => ErrorResponse::new(
                    "invalid_nonce",
                    "The nonce is invalid or has expired. Please request a new nonce",
                ),
                AuthError::InvalidWalletAddress => ErrorResponse::new(
                    "invalid_wallet_address",
                    "The wallet address format is invalid",
                ),
                AuthError::UnsupportedChainType(chain) => ErrorResponse::new(
                    "unsupported_chain",
                    &format!("Chain type '{}' is not supported", chain),
                ),
                _ => ErrorResponse::new(
                    "authentication_failed",
                    "Authentication failed due to internal error",
                ),
            };
            let status_code = StatusCode::from(error);
            Err((status_code, ResponseJson(error_response)))
        }
    }
}

pub async fn health_check_handler() -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    Ok(ResponseJson(serde_json::json!({
        "status": "healthy",
        "service": "kembridge-auth",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}