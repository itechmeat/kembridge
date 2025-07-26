// src/services/quantum.rs - Quantum Key Management Service for Phase 3.2
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

use kembridge_crypto::{MlKemCrypto, QuantumKeyManager, QuantumCryptoError};
use crate::config::AppConfig;
use crate::models::quantum::{
    QuantumKey, CreateQuantumKeyRequest, QuantumKeyResponse,
    EncapsulateRequest, EncapsulateResponse,
    DecapsulateRequest, DecapsulateResponse,
    QuantumKeysListResponse,
};

/// Service for managing quantum cryptographic keys and operations
pub struct QuantumService {
    db: PgPool,
    key_manager: QuantumKeyManager,
    #[allow(dead_code)]
    config: AppConfig,
}

impl QuantumService {
    /// Create a new QuantumService instance
    pub async fn new(db: PgPool, config: &AppConfig) -> Result<Self> {
        Ok(Self {
            db,
            key_manager: QuantumKeyManager::new(),
            config: config.clone(),
        })
    }

    /// Generate a new ML-KEM-1024 key pair for a user
    pub async fn generate_keypair(
        &self,
        user_id: Uuid,
        request: CreateQuantumKeyRequest,
    ) -> Result<QuantumKeyResponse, QuantumServiceError> {
        // Validate key type
        if request.key_type != "ml_kem_1024" {
            return Err(QuantumServiceError::UnsupportedKeyType(request.key_type));
        }

        // Generate the key pair using kembridge-crypto
        let keypair = MlKemCrypto::generate_keypair()
            .map_err(QuantumServiceError::CryptoError)?;

        // For Phase 3.2, we'll use a simple placeholder encryption for private keys
        // TODO: Phase 3.3 - Implement proper AES-256-GCM encryption for private keys
        let private_key_encrypted = b"placeholder_encrypted_private_key".to_vec();
        
        // Extract public key as bytes (placeholder implementation)
        // TODO: Phase 3.2 - Implement proper public key serialization
        let public_key_bytes = b"placeholder_public_key_bytes".to_vec();

        // Calculate expiration date
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });

        // Create security metadata
        let security_metadata = serde_json::json!({
            "algorithm": request.key_type,
            "security_level": 256,
            "key_size": {
                "public": 1568,
                "private": 3168,
                "ciphertext": 1568
            },
            "version": "1.0",
            "generation_method": "ml_kem_crypto",
            "entropy_source": "system_random"
        });

        let key_derivation_params = serde_json::json!({});

        // Store in database using the schema from Phase 1.2
        let key_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO quantum_keys (
                id, user_id, algorithm, key_type, 
                public_key, encrypted_private_key,
                encryption_algorithm, key_derivation_params, security_metadata,
                expires_at, is_active, validation_status, rotation_generation
            ) VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            key_id,
            user_id,
            request.key_type,      // algorithm field 
            "key_encapsulation",   // key_type field (fixed value for ML-KEM)
            public_key_bytes,
            private_key_encrypted,
            "aes-256-gcm",         // encryption_algorithm
            key_derivation_params,
            security_metadata,
            expires_at,
            true,                  // is_active
            "pending",             // validation_status
            1                      // rotation_generation
        )
        .execute(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Return response
        Ok(QuantumKeyResponse {
            id: key_id,
            public_key: general_purpose::STANDARD.encode(&public_key_bytes),
            algorithm: request.key_type,
            key_metadata: security_metadata,
            created_at: now,
            expires_at,
            is_active: true,
            usage_count: 0, // This field doesn't exist in schema, so we return 0
        })
    }

    /// List all quantum keys for a user
    pub async fn list_user_keys(&self, user_id: Uuid) -> Result<QuantumKeysListResponse, QuantumServiceError> {
        let keys = sqlx::query_as!(
            QuantumKey,
            "SELECT id::uuid as id, user_id, algorithm, key_type, public_key, encrypted_private_key, 
                    encryption_algorithm, encryption_iv, encryption_salt, key_derivation_params, 
                    security_metadata, created_at, expires_at, rotated_at, is_active, is_compromised, 
                    validation_status, previous_key_id, rotation_reason, rotation_generation, 
                    hsm_key_id, hsm_provider, key_status, key_strength, usage_category
             FROM quantum_keys WHERE user_id = $1 AND is_active = true ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        let key_responses: Vec<QuantumKeyResponse> = keys
            .into_iter()
            .map(|key| QuantumKeyResponse {
                id: key.id.unwrap_or_else(|| Uuid::new_v4()), // Handle nullable id
                public_key: general_purpose::STANDARD.encode(&key.public_key),
                algorithm: key.algorithm.unwrap_or_else(|| "ml-kem-1024".to_string()), // Handle nullable algorithm
                key_metadata: key.security_metadata.unwrap_or_else(|| serde_json::json!({})), // Handle nullable metadata
                created_at: key.created_at.unwrap_or_else(|| Utc::now()), // Handle nullable created_at
                expires_at: key.expires_at,
                is_active: key.is_active.unwrap_or(true), // Handle nullable is_active with default
                usage_count: 0, // Not in schema, return 0
            })
            .collect();

        Ok(QuantumKeysListResponse {
            total: key_responses.len(),
            keys: key_responses,
        })
    }

    /// Encapsulate using a public key
    pub async fn encapsulate(
        &self,
        user_id: Uuid,
        request: EncapsulateRequest,
    ) -> Result<EncapsulateResponse, QuantumServiceError> {
        // Verify that the key belongs to the user
        let _key = self.get_user_key(user_id, request.public_key_id).await?;

        // For Phase 3.2, return a placeholder response
        // TODO: Phase 3.2 - Implement real ML-KEM encapsulation
        Ok(EncapsulateResponse {
            ciphertext: general_purpose::STANDARD.encode(b"placeholder_ciphertext"),
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        })
    }

    /// Decapsulate using a private key
    pub async fn decapsulate(
        &self,
        user_id: Uuid,
        request: DecapsulateRequest,
    ) -> Result<DecapsulateResponse, QuantumServiceError> {
        // Verify that the key belongs to the user
        let _key = self.get_user_key(user_id, request.private_key_id).await?;

        // For Phase 3.2, return a placeholder response
        // TODO: Phase 3.2 - Implement real ML-KEM decapsulation
        Ok(DecapsulateResponse {
            shared_secret_hash: "placeholder_secret_hash".to_string(),
            success: true,
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        })
    }

    /// Get a specific key for a user (helper method)
    async fn get_user_key(&self, user_id: Uuid, key_id: Uuid) -> Result<QuantumKey, QuantumServiceError> {
        sqlx::query_as!(
            QuantumKey,
            "SELECT id::uuid as id, user_id, algorithm, key_type, public_key, encrypted_private_key, 
                    encryption_algorithm, encryption_iv, encryption_salt, key_derivation_params, 
                    security_metadata, created_at, expires_at, rotated_at, is_active, is_compromised, 
                    validation_status, previous_key_id, rotation_reason, rotation_generation, 
                    hsm_key_id, hsm_provider, key_status, key_strength, usage_category
             FROM quantum_keys WHERE id = $1 AND user_id = $2 AND is_active = true",
            key_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?
        .ok_or(QuantumServiceError::KeyNotFound)
    }
}

/// Errors that can occur in the QuantumService
#[derive(Debug, thiserror::Error)]
pub enum QuantumServiceError {
    #[error("Unsupported key type: {0}")]
    UnsupportedKeyType(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] QuantumCryptoError),

    #[error("Key not found or access denied")]
    KeyNotFound,

    #[error("Key has expired")]
    KeyExpired,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}