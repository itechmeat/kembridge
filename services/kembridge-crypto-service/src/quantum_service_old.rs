/**
 * Quantum Cryptography Service
 * Full implementation matching old backend QuantumService functionality
 */

use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

use kembridge_crypto::{
    MlKemCrypto, QuantumKeyManager, QuantumCryptoError
};

use crate::types::*;
use crate::errors::CryptoServiceError;

/// Service for managing quantum cryptographic keys and operations
pub struct QuantumService {
    db: PgPool,
    key_manager: QuantumKeyManager,
}

impl QuantumService {
    /// Create a new QuantumService instance
    pub async fn new(db: PgPool) -> Result<Self> {
        Ok(Self {
            db,
            key_manager: QuantumKeyManager::new(),
        })
    }

    /// Get quantum cryptography system status
    pub async fn get_status(&self) -> Result<CryptoStatus, CryptoServiceError> {
        tracing::info!("🔐 Getting quantum crypto system status");

        // Get active keys count
        let active_keys: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM quantum_keys WHERE is_active = true"
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .unwrap_or(0);

        // Get next rotation due date  
        let next_rotation: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT MIN(created_at + INTERVAL '30 days') FROM quantum_keys WHERE is_active = true"
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .flatten();

        let status = CryptoStatus {
            quantum_protection: QuantumProtectionStatus {
                is_active: active_keys > 0,
                algorithm: "ML-KEM-1024".to_string(),
                key_rotation_date: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                next_rotation_due: next_rotation
                    .unwrap_or_else(|| Utc::now() + Duration::days(30))
                    .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                    .to_string(),
                encryption_strength: 1024,
            },
            overall: if active_keys > 0 { "SECURE".to_string() } else { "WARNING".to_string() },
            is_online: true,
            last_update: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            system_health: SystemHealth {
                backend: true,
                ai_engine: true,   // TODO: Check actual AI engine health
                blockchain: true,  // TODO: Check actual blockchain health
            },
        };

        Ok(status)
    }

    /// Generate a new ML-KEM-1024 key pair
    pub async fn generate_keypair(
        &self,
        user_id: Option<Uuid>,
        request: CreateQuantumKeyRequest,
    ) -> Result<QuantumKeyResponse, CryptoServiceError> {
        tracing::info!("🔑 Generating quantum keypair: {}", request.key_type);

        // Validate key type
        if request.key_type != "ml_kem_1024" {
            return Err(CryptoServiceError::AlgorithmNotSupported { 
                algorithm: request.key_type 
            });
        }

        // Generate the key pair using kembridge-crypto
        let keypair = MlKemCrypto::generate_keypair()
            .map_err(|e| CryptoServiceError::PostQuantumError { message: e.to_string() })?;

        // Extract keys as bytes
        let private_key_bytes = keypair.private_key_bytes();
        let public_key_bytes = keypair.public_key_bytes().to_vec();

        // For now, store private key base64 encoded (TODO: proper encryption)
        let private_key_encrypted = general_purpose::STANDARD.encode(private_key_bytes).into_bytes();

        // Calculate expiration date
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });

        // Create security metadata
        let security_metadata = serde_json::json!({
            "algorithm": request.key_type,
            "security_level": 256,
            "quantum_resistant": true,
            "generated_at": Utc::now(),
            "usage_category": request.usage_category.unwrap_or_else(|| "general".to_string())
        });

        let key_id = Uuid::new_v4();
        let now = Utc::now();

        // Store in database
        sqlx::query!(
            r#"
            INSERT INTO quantum_keys (
                id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                encryption_algorithm, security_metadata, created_at, expires_at,
                is_active, is_compromised, rotation_generation, usage_category
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            key_id,
            user_id,
            "ML-KEM-1024",
            request.key_type,
            public_key_bytes,
            private_key_encrypted,
            "base64",
            security_metadata,
            now,
            expires_at,
            true,
            false,
            1i32,
            request.usage_category.unwrap_or_else(|| "general".to_string())
        )
        .execute(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        Ok(QuantumKeyResponse {
            key_id,
            key_type: request.key_type,
            public_key: general_purpose::STANDARD.encode(&public_key_bytes),
            algorithm: "ML-KEM-1024".to_string(),
            created_at: now,
            expires_at,
            encryption_strength: 1024,
        })
    }

    /// List user's quantum keys
    pub async fn list_user_keys(&self, user_id: Option<Uuid>) -> Result<QuantumKeysListResponse, CryptoServiceError> {
        tracing::info!("📋 Listing quantum keys for user: {:?}", user_id);

        let keys = if let Some(uid) = user_id {
            sqlx::query_as!(
                QuantumKey,
                r#"
                SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                       encryption_algorithm, security_metadata, created_at, expires_at,
                       rotated_at, is_active, is_compromised, rotation_generation, usage_category
                FROM quantum_keys 
                WHERE user_id = $1 AND is_active = true
                ORDER BY created_at DESC
                "#,
                uid
            )
        } else {
            // System keys (user_id is NULL)
            sqlx::query_as!(
                QuantumKey,
                r#"
                SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                       encryption_algorithm, security_metadata, created_at, expires_at,
                       rotated_at, is_active, is_compromised, rotation_generation, usage_category  
                FROM quantum_keys
                WHERE user_id IS NULL AND is_active = true
                ORDER BY created_at DESC
                "#
            )
        }
        .fetch_all(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        let key_responses: Vec<QuantumKeyResponse> = keys
            .into_iter()
            .map(|key| QuantumKeyResponse {
                key_id: key.id,
                key_type: key.key_type,
                public_key: general_purpose::STANDARD.encode(&key.public_key),
                algorithm: key.algorithm,
                created_at: key.created_at,
                expires_at: key.expires_at,
                encryption_strength: 1024,
            })
            .collect();

        let active_keys = key_responses.len();

        Ok(QuantumKeysListResponse {
            keys: key_responses,
            total_count: active_keys,
            active_keys,
        })
    }

    /// Encapsulate using ML-KEM-1024
    pub async fn encapsulate(
        &self,
        request: EncapsulateRequest,
    ) -> Result<EncapsulateResponse, CryptoServiceError> {
        tracing::info!("🔐 Performing ML-KEM encapsulation for key: {}", request.public_key_id);

        // Get the public key from database
        let key = sqlx::query_as!(
            QuantumKey,
            r#"
            SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                   encryption_algorithm, security_metadata, created_at, expires_at,
                   rotated_at, is_active, is_compromised, rotation_generation, usage_category
            FROM quantum_keys 
            WHERE id = $1 AND is_active = true
            "#,
            request.public_key_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .ok_or_else(|| CryptoServiceError::KeyNotFound { key_id: "unknown".to_string() })?;

        // Perform ML-KEM encapsulation
        let (ciphertext, shared_secret) = MlKemCrypto::encapsulate_with_bytes(&key.public_key)
            .map_err(|e| CryptoServiceError::PostQuantumError { message: e.to_string() })?;

        // Create SHA-256 hash of shared secret (don't expose the secret)
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let shared_secret_hash = format!("{:x}", hasher.finalize());

        Ok(EncapsulateResponse {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            shared_secret_hash,
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        })
    }

    /// Decapsulate using ML-KEM-1024
    pub async fn decapsulate(
        &self,
        request: DecapsulateRequest,
    ) -> Result<DecapsulateResponse, CryptoServiceError> {
        tracing::info!("🔓 Performing ML-KEM decapsulation for key: {}", request.private_key_id);

        // Get the private key from database
        let key = sqlx::query_as!(
            QuantumKey,
            r#"
            SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                   encryption_algorithm, security_metadata, created_at, expires_at,
                   rotated_at, is_active, is_compromised, rotation_generation, usage_category
            FROM quantum_keys 
            WHERE id = $1 AND is_active = true
            "#,
            request.private_key_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .ok_or_else(|| CryptoServiceError::KeyNotFound { key_id: "unknown".to_string() })?;

        // Decode the stored private key (currently base64 encoded)
        let private_key_encoded = String::from_utf8(key.encrypted_private_key)
            .map_err(|e| CryptoServiceError::InvalidKeyFormat { message: format!("Invalid private key encoding: {}", e) })?;
        let private_key_bytes = general_purpose::STANDARD.decode(private_key_encoded)
            .map_err(|e| CryptoServiceError::InvalidKeyFormat { message: format!("Failed to decode private key: {}", e) })?;

        // Decode the ciphertext from base64
        let ciphertext_bytes = general_purpose::STANDARD.decode(&request.ciphertext)
            .map_err(|e| CryptoServiceError::InvalidKeyFormat { message: format!("Invalid ciphertext encoding: {}", e) })?;

        // Perform ML-KEM decapsulation
        let shared_secret = MlKemCrypto::decapsulate_with_bytes(&private_key_bytes, &ciphertext_bytes)
            .map_err(|e| CryptoServiceError::PostQuantumError { message: e.to_string() })?;

        // Create SHA-256 hash of shared secret
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let shared_secret_hash = format!("{:x}", hasher.finalize());

        Ok(DecapsulateResponse {
            shared_secret_hash,
            success: true,
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        })
    }

    /// Check which keys need rotation
    pub async fn check_rotation_needed(
        &self,
        request: CheckRotationRequest,
    ) -> Result<CheckRotationResponse, CryptoServiceError> {
        tracing::info!("🔄 Checking key rotation status");

        let threshold_days = request.threshold_days.unwrap_or(30) as i64;
        let rotation_threshold = Utc::now() - Duration::days(threshold_days);

        let keys_needing_rotation = sqlx::query_scalar!(
            "SELECT id FROM quantum_keys WHERE is_active = true AND created_at < $1",
            rotation_threshold
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        let total_keys = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM quantum_keys WHERE is_active = true"
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .unwrap_or(0) as usize;

        let next_rotation_date = sqlx::query_scalar!(
            "SELECT MIN(created_at + INTERVAL '30 days') FROM quantum_keys WHERE is_active = true"
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .flatten();

        Ok(CheckRotationResponse {
            rotation_due: !keys_needing_rotation.is_empty(),
            keys_needing_rotation,
            total_keys,
            next_rotation_date,
        })
    }

    /// Trigger manual key rotation
    pub async fn rotate_key(
        &self,
        request: RotateKeyRequest,
    ) -> Result<RotateKeyResponse, CryptoServiceError> {
        tracing::info!("🔄 Rotating key: {} reason: {}", request.key_id, request.reason);

        // Start transaction
        let mut tx = self.db.begin().await
            .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        // Get the old key
        let old_key = sqlx::query_as!(
            QuantumKey,
            r#"
            SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                   encryption_algorithm, security_metadata, created_at, expires_at,
                   rotated_at, is_active, is_compromised, rotation_generation, usage_category
            FROM quantum_keys 
            WHERE id = $1 AND is_active = true
            "#,
            request.key_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .ok_or_else(|| CryptoServiceError::KeyNotFound { key_id: "unknown".to_string() })?;

        // Generate new key pair
        let keypair = MlKemCrypto::generate_keypair()
            .map_err(|e| CryptoServiceError::PostQuantumError { message: e.to_string() })?;

        let new_key_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert new key
        let new_private_key_encrypted = general_purpose::STANDARD.encode(keypair.private_key_bytes()).into_bytes();
        let new_public_key_bytes = keypair.public_key_bytes().to_vec();

        sqlx::query!(
            r#"
            INSERT INTO quantum_keys (
                id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                encryption_algorithm, security_metadata, created_at, expires_at,
                is_active, is_compromised, rotation_generation, usage_category
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            new_key_id,
            old_key.user_id,
            old_key.algorithm,
            old_key.key_type,
            new_public_key_bytes,
            new_private_key_encrypted,
            "base64",
            old_key.security_metadata,
            now,
            old_key.expires_at,
            true,
            false,
            old_key.rotation_generation + 1,
            old_key.usage_category
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        // Deactivate old key
        sqlx::query!(
            "UPDATE quantum_keys SET is_active = false, rotated_at = $1 WHERE id = $2",
            now,
            request.key_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?;

        Ok(RotateKeyResponse {
            old_key_id: request.key_id,
            new_key_id,
            rotation_completed_at: now,
            algorithm: old_key.algorithm,
            success: true,
        })
    }

    /// Export public key in Base64 format
    pub async fn export_public_key(
        &self,
        key_id: Uuid,
    ) -> Result<ExportPublicKeyResponse, CryptoServiceError> {
        tracing::info!("📤 Exporting public key: {}", key_id);

        let key = sqlx::query_as!(
            QuantumKey,
            r#"
            SELECT id, user_id, algorithm, key_type, public_key, encrypted_private_key,
                   encryption_algorithm, security_metadata, created_at, expires_at,
                   rotated_at, is_active, is_compromised, rotation_generation, usage_category
            FROM quantum_keys 
            WHERE id = $1 AND is_active = true
            "#,
            key_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| CryptoServiceError::DatabaseError { message: e.to_string() })?
        .ok_or_else(|| CryptoServiceError::KeyNotFound { key_id: "unknown".to_string() })?;

        Ok(ExportPublicKeyResponse {
            key_id,
            public_key: general_purpose::STANDARD.encode(&key.public_key),
            algorithm: key.algorithm,
            format: "base64".to_string(),
            created_at: key.created_at,
            usage: "This public key can be used for ML-KEM-1024 encapsulation operations".to_string(),
        })
    }
}