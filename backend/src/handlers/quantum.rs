// src/handlers/quantum.rs - Quantum cryptography handlers for Phase 3.2
use axum::{extract::{State, Path}, response::Json};
use serde_json::Value;
use utoipa;
use uuid;

use crate::AppState;
use crate::extractors::auth::AuthUser;
use crate::middleware::error_handler::ApiError;
use crate::models::quantum::{
    CreateQuantumKeyRequest, QuantumKeyResponse, QuantumKeysListResponse,
    EncapsulateRequest, EncapsulateResponse,
    DecapsulateRequest, DecapsulateResponse,
};
use crate::services::quantum::QuantumServiceError;

/// Generate ML-KEM-1024 keypair (Phase 3.2.4)
#[utoipa::path(
    post,
    path = "/api/v1/crypto/generate-keys",
    request_body = CreateQuantumKeyRequest,
    responses(
        (status = 201, description = "Quantum keypair generated successfully", body = QuantumKeyResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn generate_keypair(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<CreateQuantumKeyRequest>,
) -> Result<Json<QuantumKeyResponse>, ApiError> {
    let response = state.quantum_service
        .generate_keypair(user.user_id, request)
        .await
        .map_err(|e| match e {
            QuantumServiceError::UnsupportedKeyType(msg) => ApiError::Validation {
                field: "key_type".to_string(),
                message: msg,
            },
            QuantumServiceError::DatabaseError(msg) => ApiError::Internal(msg),
            QuantumServiceError::CryptoError(err) => ApiError::Internal(err.to_string()),
            _ => ApiError::Internal("Quantum key generation failed".to_string()),
        })?;

    Ok(Json(response))
}

/// Encapsulate data using ML-KEM-1024 (Phase 3.2.5)
#[utoipa::path(
    post,
    path = "/api/v1/crypto/encapsulate",
    request_body = EncapsulateRequest,
    responses(
        (status = 200, description = "Encapsulation successful", body = EncapsulateResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Key not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn encapsulate(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<EncapsulateRequest>,
) -> Result<Json<EncapsulateResponse>, ApiError> {
    let response = state.quantum_service
        .encapsulate(user.user_id, request)
        .await
        .map_err(|e| match e {
            QuantumServiceError::KeyNotFound => ApiError::NotFound { resource: "Quantum key".to_string() },
            QuantumServiceError::DatabaseError(msg) => ApiError::Internal(msg),
            QuantumServiceError::CryptoError(err) => ApiError::Internal(err.to_string()),
            _ => ApiError::Internal("Encapsulation failed".to_string()),
        })?;

    Ok(Json(response))
}

/// Decapsulate data using ML-KEM-1024 (Phase 3.2.6)
#[utoipa::path(
    post,
    path = "/api/v1/crypto/decapsulate",
    request_body = DecapsulateRequest,
    responses(
        (status = 200, description = "Decapsulation successful", body = DecapsulateResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Key not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn decapsulate(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<DecapsulateRequest>,
) -> Result<Json<DecapsulateResponse>, ApiError> {
    let response = state.quantum_service
        .decapsulate(user.user_id, request)
        .await
        .map_err(|e| match e {
            QuantumServiceError::KeyNotFound => ApiError::NotFound { resource: "Quantum key".to_string() },
            QuantumServiceError::DatabaseError(msg) => ApiError::Internal(msg),
            QuantumServiceError::CryptoError(err) => ApiError::Internal(err.to_string()),
            _ => ApiError::Internal("Decapsulation failed".to_string()),
        })?;

    Ok(Json(response))
}

/// List user's quantum keys (Phase 3.2.7)
#[utoipa::path(
    get,
    path = "/api/v1/crypto/keys",
    responses(
        (status = 200, description = "User quantum keys retrieved successfully", body = QuantumKeysListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_keys(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<QuantumKeysListResponse>, ApiError> {
    let response = state.quantum_service
        .list_user_keys(user.user_id)
        .await
        .map_err(|e| match e {
            QuantumServiceError::DatabaseError(msg) => ApiError::Internal(msg),
            _ => ApiError::Internal("Failed to list quantum keys".to_string()),
        })?;

    Ok(Json(response))
}

/// Export public key in Base64 format (Phase 3.2.8)
#[utoipa::path(
    get,
    path = "/api/v1/crypto/keys/{key_id}/public",
    params(
        ("key_id" = String, Path, description = "UUID of the quantum key")
    ),
    responses(
        (status = 200, description = "Public key exported successfully", body = Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Key not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn export_public_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key_uuid = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| ApiError::Validation {
            field: "key_id".to_string(),
            message: "Invalid UUID format".to_string(),
        })?;

    // Get the key from database to verify ownership and extract public key
    let key = sqlx::query!(
        "SELECT public_key, algorithm, created_at FROM quantum_keys WHERE id = $1 AND user_id = $2 AND is_active = true",
        key_uuid,
        user.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound { resource: "Quantum key".to_string() })?;

    use base64::{Engine as _, engine::general_purpose};
    
    Ok(Json(serde_json::json!({
        "key_id": key_id,
        "public_key": general_purpose::STANDARD.encode(&key.public_key),
        "algorithm": key.algorithm,
        "format": "base64",
        "created_at": key.created_at,
        "usage": "This public key can be used for ML-KEM-1024 encapsulation operations"
    })))
}