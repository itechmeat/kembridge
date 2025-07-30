/**
 * Crypto Service Handlers
 * Full quantum cryptography API endpoints
 */

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use kembridge_common::ServiceResponse;
use crate::quantum_service::QuantumService;
use crate::types::*;
use crate::errors::CryptoServiceError;

/// Get quantum cryptography system status
pub async fn get_crypto_status(
    State(service): State<Arc<QuantumService>>,
) -> Result<Json<ServiceResponse<CryptoStatus>>, CryptoServiceError> {
    let status = service.get_status().await?;
    Ok(Json(ServiceResponse::success(status)))
}

/// Generate a new ML-KEM-1024 key pair
pub async fn generate_keypair(
    State(service): State<Arc<QuantumService>>,
    Json(request): Json<CreateQuantumKeyRequest>,
) -> Result<Json<ServiceResponse<QuantumKeyResponse>>, CryptoServiceError> {
    let response = service.generate_keypair(None, request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Generate a key pair for a specific user
pub async fn generate_user_keypair(
    State(service): State<Arc<QuantumService>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<CreateQuantumKeyRequest>,
) -> Result<Json<ServiceResponse<QuantumKeyResponse>>, CryptoServiceError> {
    let response = service.generate_keypair(Some(user_id), request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// List quantum keys (system keys)
pub async fn list_keys(
    State(service): State<Arc<QuantumService>>,
) -> Result<Json<ServiceResponse<QuantumKeysListResponse>>, CryptoServiceError> {
    let response = service.list_user_keys(None).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// List user's quantum keys
pub async fn list_user_keys(
    State(service): State<Arc<QuantumService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServiceResponse<QuantumKeysListResponse>>, CryptoServiceError> {
    let response = service.list_user_keys(Some(user_id)).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Encapsulate using ML-KEM-1024
pub async fn encapsulate(
    State(service): State<Arc<QuantumService>>,
    Json(request): Json<EncapsulateRequest>,
) -> Result<Json<ServiceResponse<EncapsulateResponse>>, CryptoServiceError> {
    let response = service.encapsulate(request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Decapsulate using ML-KEM-1024
pub async fn decapsulate(
    State(service): State<Arc<QuantumService>>,
    Json(request): Json<DecapsulateRequest>,
) -> Result<Json<ServiceResponse<DecapsulateResponse>>, CryptoServiceError> {
    let response = service.decapsulate(request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Check which keys need rotation
pub async fn check_rotation(
    State(service): State<Arc<QuantumService>>,
    Json(request): Json<CheckRotationRequest>,
) -> Result<Json<ServiceResponse<CheckRotationResponse>>, CryptoServiceError> {
    let response = service.check_rotation_needed(request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Trigger manual key rotation
pub async fn rotate_key(
    State(service): State<Arc<QuantumService>>,
    Json(request): Json<RotateKeyRequest>,
) -> Result<Json<ServiceResponse<RotateKeyResponse>>, CryptoServiceError> {
    let response = service.rotate_key(request).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Export public key in Base64 format
pub async fn export_public_key(
    State(service): State<Arc<QuantumService>>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<ServiceResponse<ExportPublicKeyResponse>>, CryptoServiceError> {
    let response = service.export_public_key(key_id).await?;
    Ok(Json(ServiceResponse::success(response)))
}

/// Health check endpoint
pub async fn health() -> Json<ServiceResponse<serde_json::Value>> {
    Json(ServiceResponse::success(serde_json::json!({
        "service": "kembridge-crypto-service",
        "status": "healthy",
        "quantum_ready": true,
        "algorithms": ["ML-KEM-1024", "AES-256-GCM", "HKDF", "SHA-256"],
        "capabilities": [
            "key_generation",
            "encapsulation",
            "decapsulation", 
            "key_rotation",
            "hybrid_cryptography"
        ]
    })))
}

// Legacy endpoints for backward compatibility
pub async fn simple_generate_key(
    Query(request): Query<GenerateKeyRequest>,
    State(service): State<Arc<QuantumService>>,
) -> Result<Json<ServiceResponse<GenerateKeyResponse>>, CryptoServiceError> {
    let crypto_request = CreateQuantumKeyRequest {
        key_type: request.algorithm.clone(),
        expires_in_days: None,
        usage_category: Some("legacy".to_string()),
    };

    match service.generate_keypair(None, crypto_request).await {
        Ok(response) => {
            let legacy_response = GenerateKeyResponse {
                key_id: response.key_id.to_string(),
                public_key: response.public_key,
                algorithm: response.algorithm,
            };
            Ok(Json(ServiceResponse::success(legacy_response)))
        }
        Err(e) => Err(e),
    }
}

pub async fn simple_encrypt(
    Query(_request): Query<EncryptRequest>,
) -> Result<Json<ServiceResponse<EncryptResponse>>, CryptoServiceError> {
    // TODO: Implement hybrid encryption when needed
    let response = EncryptResponse {
        encrypted_data: "hybrid-encrypted-data-placeholder".to_string(),
        nonce: "crypto-nonce-123".to_string(),
        key_id: "system-key".to_string(),
    };
    
    Ok(Json(ServiceResponse::success(response)))
}

// Legacy types for compatibility
#[derive(serde::Deserialize)]
pub struct GenerateKeyRequest {
    pub key_type: String,
    pub algorithm: String,
}

#[derive(serde::Serialize)]
pub struct GenerateKeyResponse {
    pub key_id: String,
    pub public_key: String,
    pub algorithm: String,
}

#[derive(serde::Deserialize)]
pub struct EncryptRequest {
    pub data: String,
    pub key_id: String,
}

#[derive(serde::Serialize)]
pub struct EncryptResponse {
    pub encrypted_data: String,
    pub nonce: String,
    pub key_id: String,
}