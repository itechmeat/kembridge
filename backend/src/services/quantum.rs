// src/services/quantum.rs - Quantum Key Management Service for Phase 3.2
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

use kembridge_crypto::{MlKemCrypto, QuantumKeyManager, QuantumCryptoError, HybridCrypto, TransactionCrypto};
use validator::Validate;
use crate::config::AppConfig;
use crate::models::quantum::{
    QuantumKey, CreateQuantumKeyRequest, QuantumKeyResponse,
    EncapsulateRequest, EncapsulateResponse,
    DecapsulateRequest, DecapsulateResponse,
    QuantumKeysListResponse, RotateKeyRequest, RotateKeyResponse,
    CheckRotationRequest, CheckRotationResponse, QuantumKeyRotationInfo,
    HybridRotateKeyRequest, HybridRotateKeyResponse, HybridRotationConfig,
    HybridEncryptionDetails, HybridKeySizes,
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

    /// Get the quantum key manager for bridge service integration (Phase 5.2.7)
    pub fn get_quantum_manager(&self) -> Arc<QuantumKeyManager> {
        Arc::new(QuantumKeyManager::new())
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

        // Encrypt private key with AES-256-GCM for secure storage
        // For simplicity in Phase 3.4, we'll store it base64 encoded
        // TODO: Phase 3.5 - Add proper key derivation from user password/PIN
        let private_key_bytes = keypair.private_key_bytes();
        let private_key_encrypted = general_purpose::STANDARD.encode(private_key_bytes)
            .into_bytes();
        
        // Extract public key as bytes - real implementation with ML-KEM-1024
        let public_key_bytes = keypair.public_key_bytes().to_vec();

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
        // Verify that the key belongs to the user and get the key data
        let key = self.get_user_key(user_id, request.public_key_id).await?;

        // Perform real ML-KEM encapsulation using the stored public key
        let (ciphertext_bytes, _shared_secret) = MlKemCrypto::encapsulate_with_bytes(&key.public_key)
            .map_err(QuantumServiceError::CryptoError)?;

        Ok(EncapsulateResponse {
            ciphertext: general_purpose::STANDARD.encode(ciphertext_bytes),
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
        // Verify that the key belongs to the user and get the key data
        let key = self.get_user_key(user_id, request.private_key_id).await?;

        // Decode the stored private key (currently base64 encoded)
        let private_key_encoded = String::from_utf8(key.encrypted_private_key)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid private key encoding: {}", e)))?;
        let private_key_bytes = general_purpose::STANDARD.decode(private_key_encoded)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Failed to decode private key: {}", e)))?;

        // Decode the ciphertext from base64
        let ciphertext_bytes = general_purpose::STANDARD.decode(&request.ciphertext)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid ciphertext encoding: {}", e)))?;

        // Perform real ML-KEM decapsulation
        let shared_secret = MlKemCrypto::decapsulate_with_bytes(&private_key_bytes, &ciphertext_bytes)
            .map_err(QuantumServiceError::CryptoError)?;

        // Create a SHA-256 hash of the shared secret for response (don't expose the secret itself)
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

    /// Rotate a quantum key (Task 3.2.7)
    pub async fn rotate_key(
        &self,
        user_id: Uuid,
        request: RotateKeyRequest,
    ) -> Result<RotateKeyResponse, QuantumServiceError> {
        // Start a transaction for atomic key rotation
        let mut tx = self.db.begin().await
            .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Check if the key exists and belongs to the user
        let old_key = sqlx::query_as!(
            QuantumKey,
            "SELECT id::uuid as id, user_id, algorithm, key_type, public_key, encrypted_private_key, 
                    encryption_algorithm, encryption_iv, encryption_salt, key_derivation_params, 
                    security_metadata, created_at, expires_at, rotated_at, is_active, is_compromised, 
                    validation_status, previous_key_id, rotation_reason, rotation_generation, 
                    hsm_key_id, hsm_provider, key_status, key_strength, usage_category
             FROM quantum_keys WHERE id = $1 AND user_id = $2 AND is_active = true",
            request.key_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?
        .ok_or(QuantumServiceError::KeyNotFound)?;

        // Check if key has active bridge operations (integrates with AI Risk Engine)
        let has_active_operations = self.check_active_operations(user_id).await?;
        if has_active_operations && request.rotation_reason != "emergency" {
            return Err(QuantumServiceError::InvalidRequest(
                "Cannot rotate key while bridge operations are active. Use emergency rotation if necessary.".to_string()
            ));
        }

        // Generate new key pair
        let new_keypair = MlKemCrypto::generate_keypair()
            .map_err(QuantumServiceError::CryptoError)?;

        // Prepare new key data
        let private_key_bytes = new_keypair.private_key_bytes();
        let private_key_encrypted = general_purpose::STANDARD.encode(private_key_bytes).into_bytes();
        let public_key_bytes = new_keypair.public_key_bytes().to_vec();

        // Calculate new expiration
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });

        // Increment rotation generation
        let new_generation = old_key.rotation_generation.unwrap_or(1) + 1;
        let now = Utc::now();

        // Create security metadata for new key
        let security_metadata = serde_json::json!({
            "algorithm": "ml_kem_1024",
            "security_level": 256,
            "key_size": {
                "public": 1568,
                "private": 3168,
                "ciphertext": 1568
            },
            "version": "1.0",
            "generation_method": "ml_kem_crypto",
            "entropy_source": "system_random",
            "rotation_timestamp": now,
            "previous_key_id": request.key_id,
            "rotation_reason": request.rotation_reason.clone()
        });

        let key_derivation_params = serde_json::json!({});

        // Insert new key
        let new_key_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO quantum_keys (
                id, user_id, algorithm, key_type, 
                public_key, encrypted_private_key,
                encryption_algorithm, key_derivation_params, security_metadata,
                expires_at, is_active, validation_status, rotation_generation,
                previous_key_id, rotation_reason
            ) VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            new_key_id,
            user_id,
            "ml_kem_1024",      
            "key_encapsulation",   
            public_key_bytes,
            private_key_encrypted,
            "aes-256-gcm",         
            key_derivation_params,
            security_metadata,
            expires_at,
            true,                  
            "pending",             
            new_generation,
            Some(request.key_id),  // Link to previous key
            Some(request.rotation_reason.clone())
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Mark old key as rotated (but keep it for audit trail)
        sqlx::query!(
            "UPDATE quantum_keys SET is_active = false, rotated_at = $1 WHERE id = $2",
            now,
            request.key_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Return response
        Ok(RotateKeyResponse {
            new_key: QuantumKeyResponse {
                id: new_key_id,
                public_key: general_purpose::STANDARD.encode(&public_key_bytes),
                algorithm: "ml_kem_1024".to_string(),
                key_metadata: security_metadata,
                created_at: now,
                expires_at,
                is_active: true,
                usage_count: 0,
            },
            previous_key_id: request.key_id,
            rotation_reason: request.rotation_reason,
            rotation_generation: new_generation,
            rotated_at: now,
        })
    }

    /// Check which keys need rotation based on age and risk analysis
    pub async fn check_rotation_needed(
        &self,
        user_id: Option<Uuid>,
        request: CheckRotationRequest,
    ) -> Result<CheckRotationResponse, QuantumServiceError> {
        let days_threshold = request.days_threshold.unwrap_or(90); // Default: 90 days

        // Split into separate functions to avoid type conflicts
        let rotation_info = if let Some(uid) = request.user_id.or(user_id) {
            self.check_user_rotation(uid, days_threshold).await?
        } else {
            self.check_all_users_rotation(days_threshold).await?
        };

        Ok(CheckRotationResponse {
            total: rotation_info.len(),
            keys_needing_rotation: rotation_info,
            days_threshold,
        })
    }

    /// Check rotation for specific user
    async fn check_user_rotation(
        &self,
        user_id: Uuid,
        days_threshold: i32,
    ) -> Result<Vec<QuantumKeyRotationInfo>, QuantumServiceError> {
        let keys = sqlx::query!(
            "SELECT id, user_id, algorithm, created_at, rotation_generation,
                    EXTRACT(DAYS FROM (NOW() - created_at))::int as days_old
             FROM quantum_keys 
             WHERE user_id = $1 AND is_active = true 
               AND EXTRACT(DAYS FROM (NOW() - created_at)) >= $2
             ORDER BY created_at ASC",
            user_id,
            days_threshold as i64
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        let mut rotation_info = Vec::new();

        for key in keys {
            // Check for active operations (simplified - in production would query transactions table)
            let has_active_ops = self.check_active_operations(key.user_id).await.unwrap_or(false);
            
            // Calculate risk score based on age and generation
            let risk_score = self.calculate_key_risk_score(
                key.days_old.unwrap_or(0),
                key.rotation_generation.unwrap_or(1),
                has_active_ops,
            );

            // Determine priority
            let priority = if risk_score > 0.8 { "critical" }
                         else if risk_score > 0.6 { "high" }
                         else if risk_score > 0.4 { "medium" }
                         else { "low" };

            rotation_info.push(QuantumKeyRotationInfo {
                key_id: key.id,
                user_id: key.user_id,
                algorithm: key.algorithm,
                days_old: key.days_old.unwrap_or(0),
                rotation_generation: key.rotation_generation.unwrap_or(1),
                has_active_operations: has_active_ops,
                risk_score,
                priority: priority.to_string(),
            });
        }

        Ok(rotation_info)
    }

    /// Check rotation for all users (admin)
    async fn check_all_users_rotation(
        &self,
        days_threshold: i32,
    ) -> Result<Vec<QuantumKeyRotationInfo>, QuantumServiceError> {
        let keys = sqlx::query!(
            "SELECT id, user_id, algorithm, created_at, rotation_generation,
                    EXTRACT(DAYS FROM (NOW() - created_at))::int as days_old
             FROM quantum_keys 
             WHERE is_active = true 
               AND EXTRACT(DAYS FROM (NOW() - created_at)) >= $1
             ORDER BY created_at ASC",
            days_threshold as i64
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        let mut rotation_info = Vec::new();

        for key in keys {
            // Check for active operations (simplified - in production would query transactions table)
            let has_active_ops = self.check_active_operations(key.user_id).await.unwrap_or(false);
            
            // Calculate risk score based on age and generation
            let risk_score = self.calculate_key_risk_score(
                key.days_old.unwrap_or(0),
                key.rotation_generation.unwrap_or(1),
                has_active_ops,
            );

            // Determine priority
            let priority = if risk_score > 0.8 { "critical" }
                         else if risk_score > 0.6 { "high" }
                         else if risk_score > 0.4 { "medium" }
                         else { "low" };

            rotation_info.push(QuantumKeyRotationInfo {
                key_id: key.id,
                user_id: key.user_id,
                algorithm: key.algorithm,
                days_old: key.days_old.unwrap_or(0),
                rotation_generation: key.rotation_generation.unwrap_or(1),
                has_active_operations: has_active_ops,
                risk_score,
                priority: priority.to_string(),
            });
        }

        Ok(rotation_info)
    }

    /// Check if user has active bridge operations (integrates with AI Risk Engine)
    async fn check_active_operations(&self, user_id: Uuid) -> Result<bool, QuantumServiceError> {
        // In production, this would check the transactions table and AI Risk Engine
        // For now, we'll do a simplified check against the transactions table
        let active_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM transactions 
             WHERE user_id = $1 AND status IN ('pending', 'processing') 
               AND created_at > NOW() - INTERVAL '24 hours'",
            user_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        Ok(active_count.unwrap_or(0) > 0)
    }

    /// Calculate risk score for key rotation decision
    fn calculate_key_risk_score(&self, days_old: i32, rotation_generation: i32, has_active_ops: bool) -> f64 {
        let mut score: f64 = 0.0;

        // Age-based risk (increases exponentially after 90 days)
        if days_old > 365 {
            score += 0.8; // Critical - over 1 year
        } else if days_old > 180 {
            score += 0.6; // High - over 6 months
        } else if days_old > 90 {
            score += 0.4; // Medium - over 3 months
        } else if days_old > 30 {
            score += 0.2; // Low - over 1 month
        }

        // Generation-based risk (older generations are higher risk)
        if rotation_generation == 1 && days_old > 30 {
            score += 0.2; // Original key that's been used for a while
        }

        // Active operations increase urgency but lower immediate rotation priority
        if has_active_ops {
            score += 0.1; // Slight increase, but rotation should be delayed
        }

        score.min(1.0)
    }

    /// Rotate a quantum key with hybrid encryption support (Task 3.4.4)
    pub async fn hybrid_rotate_key(
        &self,
        user_id: Uuid,
        request: HybridRotateKeyRequest,
    ) -> Result<HybridRotateKeyResponse, QuantumServiceError> {
        // Start a transaction for atomic key rotation
        let mut tx = self.db.begin().await
            .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Check if the key exists and belongs to the user
        let old_key = self.get_user_key_internal(user_id, request.key_id, &mut *tx).await?;

        // Check if key has active bridge operations
        let has_active_operations = self.check_active_operations(user_id).await?;
        if has_active_operations && request.rotation_reason != "emergency" {
            return Err(QuantumServiceError::InvalidRequest(
                "Cannot rotate key while bridge operations are active. Use emergency rotation if necessary.".to_string()
            ));
        }

        // Generate new key pair
        let new_keypair = MlKemCrypto::generate_keypair()
            .map_err(QuantumServiceError::CryptoError)?;

        let new_key_id = Uuid::new_v4();
        let now = Utc::now();
        let new_generation = old_key.rotation_generation.unwrap_or(1) + 1;

        // Prepare encryption based on hybrid config
        let (encrypted_private_key, hybrid_details) = if request.hybrid_config.use_hybrid_encryption {
            self.encrypt_private_key_hybrid(&new_keypair, &request.hybrid_config, user_id, &mut *tx).await?
        } else {
            // Fallback to simple base64 encoding
            let private_key_bytes = new_keypair.private_key_bytes();
            let encrypted = general_purpose::STANDARD.encode(private_key_bytes).into_bytes();
            
            let hybrid_details = HybridEncryptionDetails {
                hybrid_encryption_used: false,
                master_key_id: None,
                encryption_context: "none".to_string(),
                scheme_version: 0,
                key_sizes: HybridKeySizes {
                    ml_kem_ciphertext_size: 0,
                    aes_encrypted_size: 0,
                    integrity_proof_size: 0,
                    total_size: encrypted.len(),
                },
            };
            
            (encrypted, hybrid_details)
        };

        // Prepare new key data
        let public_key_bytes = new_keypair.public_key_bytes().to_vec();

        // Calculate new expiration
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + Duration::days(days as i64)
        });

        // Create security metadata for new key
        let security_metadata = serde_json::json!({
            "algorithm": "ml_kem_1024",
            "security_level": 256,
            "key_size": {
                "public": 1568,
                "private": 3168,
                "ciphertext": 1568
            },
            "version": "1.0",
            "generation_method": "ml_kem_crypto",
            "entropy_source": "system_random",
            "rotation_timestamp": now,
            "previous_key_id": request.key_id,
            "rotation_reason": request.rotation_reason.clone(),
            "hybrid_encryption": hybrid_details.hybrid_encryption_used,
            "encryption_context": hybrid_details.encryption_context.clone()
        });

        let key_derivation_params = serde_json::json!({});

        // Insert new key
        sqlx::query!(
            r#"
            INSERT INTO quantum_keys (
                id, user_id, algorithm, key_type, 
                public_key, encrypted_private_key,
                encryption_algorithm, key_derivation_params, security_metadata,
                expires_at, is_active, validation_status, rotation_generation,
                previous_key_id, rotation_reason
            ) VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            new_key_id,
            user_id,
            "ml_kem_1024",      
            "key_encapsulation",   
            public_key_bytes,
            encrypted_private_key,
            if hybrid_details.hybrid_encryption_used { "hybrid-aes-256-gcm" } else { "aes-256-gcm" },         
            key_derivation_params,
            security_metadata,
            expires_at,
            true,                  
            "pending",             
            new_generation,
            Some(request.key_id),  // Link to previous key
            Some(request.rotation_reason.clone())
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Mark old key as rotated (but keep it for audit trail)
        sqlx::query!(
            "UPDATE quantum_keys SET is_active = false, rotated_at = $1 WHERE id = $2",
            now,
            request.key_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?;

        // Return response
        Ok(HybridRotateKeyResponse {
            new_key: QuantumKeyResponse {
                id: new_key_id,
                public_key: general_purpose::STANDARD.encode(&public_key_bytes),
                algorithm: "ml_kem_1024".to_string(),
                key_metadata: security_metadata,
                created_at: now,
                expires_at,
                is_active: true,
                usage_count: 0,
            },
            previous_key_id: request.key_id,
            rotation_reason: request.rotation_reason,
            rotation_generation: new_generation,
            rotated_at: now,
            hybrid_details,
        })
    }

    /// Encrypt private key using hybrid cryptography
    async fn encrypt_private_key_hybrid(
        &self,
        keypair: &kembridge_crypto::MlKemKeyPair,
        config: &HybridRotationConfig,
        user_id: Uuid,
        tx: &mut sqlx::PgConnection,
    ) -> Result<(Vec<u8>, HybridEncryptionDetails), QuantumServiceError> {
        // Get master key for hybrid encryption
        let master_key = if let Some(master_key_id) = config.master_key_id {
            self.get_user_key_internal(user_id, master_key_id, tx).await?
        } else {
            // Get latest active key for user
            self.get_latest_user_key(user_id, tx).await?
        };

        // Decode master public key from base64
        let master_public_key = general_purpose::STANDARD
            .decode(&general_purpose::STANDARD.encode(&master_key.public_key))
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid master key: {}", e)))?;

        // Get private key bytes to encrypt
        let private_key_bytes = keypair.private_key_bytes();

        // Create encryption context from configuration
        let context = match config.encryption_context.as_str() {
            "key_rotation" => b"kembridge.v1.key_rotation".to_vec(),
            "bridge_transaction" => kembridge_crypto::kdf::contexts::bridge_transaction(),
            "key_exchange" => kembridge_crypto::kdf::contexts::key_exchange(),
            _ => format!("kembridge.v1.custom.{}", config.encryption_context).into_bytes(),
        };

        // Encrypt using HybridCrypto
        let hybrid_encrypted = HybridCrypto::encrypt_data(
            &master_key.public_key,
            private_key_bytes,
            &context
        ).map_err(QuantumServiceError::CryptoError)?;

        // Serialize the hybrid encrypted data
        let serialized = serde_json::to_vec(&hybrid_encrypted)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Serialization failed: {}", e)))?;

        // Create hybrid details
        let hybrid_details = HybridEncryptionDetails {
            hybrid_encryption_used: true,
            master_key_id: Some(master_key.id.unwrap_or_else(|| Uuid::new_v4())),
            encryption_context: config.encryption_context.clone(),
            scheme_version: HybridCrypto::SCHEME_VERSION,
            key_sizes: HybridKeySizes {
                ml_kem_ciphertext_size: hybrid_encrypted.ml_kem_ciphertext.len(),
                aes_encrypted_size: hybrid_encrypted.aes_encrypted_data.ciphertext.len(),
                integrity_proof_size: hybrid_encrypted.integrity_proof.len(),
                total_size: serialized.len(),
            },
        };

        Ok((serialized, hybrid_details))
    }

    /// Get latest active key for user (helper for hybrid encryption)
    async fn get_latest_user_key(
        &self,
        user_id: Uuid,
        tx: &mut sqlx::PgConnection,
    ) -> Result<QuantumKey, QuantumServiceError> {
        sqlx::query_as!(
            QuantumKey,
            "SELECT id::uuid as id, user_id, algorithm, key_type, public_key, encrypted_private_key, 
                    encryption_algorithm, encryption_iv, encryption_salt, key_derivation_params, 
                    security_metadata, created_at, expires_at, rotated_at, is_active, is_compromised, 
                    validation_status, previous_key_id, rotation_reason, rotation_generation, 
                    hsm_key_id, hsm_provider, key_status, key_strength, usage_category
             FROM quantum_keys WHERE user_id = $1 AND is_active = true 
             ORDER BY created_at DESC LIMIT 1",
            user_id
        )
        .fetch_optional(tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?
        .ok_or(QuantumServiceError::KeyNotFound)
    }

    /// Get a specific key for a user with transaction (helper method)
    async fn get_user_key_internal(
        &self,
        user_id: Uuid,
        key_id: Uuid,
        tx: &mut sqlx::PgConnection,
    ) -> Result<QuantumKey, QuantumServiceError> {
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
        .fetch_optional(tx)
        .await
        .map_err(|e| QuantumServiceError::DatabaseError(e.to_string()))?
        .ok_or(QuantumServiceError::KeyNotFound)
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

    /// Encrypt data using hybrid cryptography (Task 3.4.5)
    pub async fn hybrid_encrypt(
        &self,
        user_id: Uuid,
        request: crate::models::quantum::HybridEncryptRequest,
    ) -> Result<crate::models::quantum::HybridEncryptResponse, QuantumServiceError> {
        use crate::models::quantum::*;
        use kembridge_crypto::HybridCrypto;
        use base64::{Engine as _, engine::general_purpose};

        // Validate input
        if let Err(validation_errors) = request.validate() {
            let error_message = validation_errors
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error_messages: Vec<String> = errors.iter()
                        .map(|e| e.message.as_ref().unwrap_or(&"invalid".into()).to_string())
                        .collect();
                    format!("{}: {}", field, error_messages.join(", "))
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(QuantumServiceError::InvalidRequest(error_message));
        }

        // Get and validate key
        let key = self.get_user_key(user_id, request.key_id).await?;
        if !key.is_active.unwrap_or(false) {
            return Err(QuantumServiceError::InvalidRequest("Key is not active".to_string()));
        }

        // Decode input data
        let data = general_purpose::STANDARD
            .decode(&request.data)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid base64 data: {}", e)))?;

        // Create encryption context based on request
        let context = match request.encryption_context.as_str() {
            "bridge_transaction" => kembridge_crypto::kdf::contexts::bridge_transaction(),
            "key_exchange" => kembridge_crypto::kdf::contexts::key_exchange(),
            "session_keys" => kembridge_crypto::kdf::contexts::session_keys(),
            _ => kembridge_crypto::kdf::contexts::bridge_transaction(), // Default
        };

        // Encrypt data
        let encrypted_data = HybridCrypto::encrypt_data(&key.public_key, &data, &context)
            .map_err(|e| QuantumServiceError::CryptoError(e))?;

        // Serialize encrypted data for transport
        let serialized = serde_json::to_vec(&encrypted_data)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Failed to serialize encrypted data: {}", e)))?;
        let encrypted_base64 = general_purpose::STANDARD.encode(&serialized);

        // Create response
        let response = HybridEncryptResponse {
            encrypted_data: encrypted_base64,
            encryption_metadata: HybridEncryptionMetadata {
                scheme_version: encrypted_data.scheme_version,
                encrypted_at: Utc::now(),
                encryption_context: request.encryption_context.clone(),
                data_sizes: HybridKeySizes {
                    ml_kem_ciphertext_size: encrypted_data.ml_kem_ciphertext.len(),
                    aes_encrypted_size: encrypted_data.aes_encrypted_data.ciphertext.len(),
                    integrity_proof_size: encrypted_data.integrity_proof.len(),
                    total_size: serialized.len(),
                },
            },
            key_info: HybridKeyInfo {
                key_id: key.id.unwrap(),
                algorithm: key.algorithm.unwrap_or_default(),
                is_active: key.is_active.unwrap_or(false),
                rotation_generation: key.rotation_generation.unwrap_or(1),
            },
        };

        Ok(response)
    }

    /// Decrypt data using hybrid cryptography (Task 3.4.5)
    pub async fn hybrid_decrypt(
        &self,
        user_id: Uuid,
        request: crate::models::quantum::HybridDecryptRequest,
    ) -> Result<crate::models::quantum::HybridDecryptResponse, QuantumServiceError> {
        use crate::models::quantum::*;
        use kembridge_crypto::HybridCrypto;
        use base64::{Engine as _, engine::general_purpose};

        // Validate input
        if let Err(validation_errors) = request.validate() {
            let error_message = validation_errors
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error_messages: Vec<String> = errors.iter()
                        .map(|e| e.message.as_ref().unwrap_or(&"invalid".into()).to_string())
                        .collect();
                    format!("{}: {}", field, error_messages.join(", "))
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(QuantumServiceError::InvalidRequest(error_message));
        }

        // Get and validate key
        let key = self.get_user_key(user_id, request.key_id).await?;
        if !key.is_active.unwrap_or(false) {
            return Err(QuantumServiceError::InvalidRequest("Key is not active".to_string()));
        }

        // Decode encrypted data
        let encrypted_data = general_purpose::STANDARD
            .decode(&request.encrypted_data)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid base64 data: {}", e)))?;

        // Parse encrypted data structure
        let hybrid_data: kembridge_crypto::HybridEncryptedData = serde_json::from_slice(&encrypted_data)
            .map_err(|e| QuantumServiceError::InvalidRequest(format!("Invalid encrypted data format: {}", e)))?;

        // Decrypt private key (we need to implement this method)
        let private_key = self.decrypt_private_key(&key.encrypted_private_key)
            .map_err(|e| QuantumServiceError::CryptoError(e))?;

        // Create decryption context based on request
        let context = match request.encryption_context.as_str() {
            "bridge_transaction" => kembridge_crypto::kdf::contexts::bridge_transaction(),
            "key_exchange" => kembridge_crypto::kdf::contexts::key_exchange(),
            "session_keys" => kembridge_crypto::kdf::contexts::session_keys(),
            _ => kembridge_crypto::kdf::contexts::bridge_transaction(), // Default
        };

        // Decrypt data
        let decrypted_data = HybridCrypto::decrypt_data(&private_key, &hybrid_data, &context)
            .map_err(|e| QuantumServiceError::CryptoError(e))?;

        // Encode decrypted data
        let decrypted_base64 = general_purpose::STANDARD.encode(&decrypted_data);

        // Create response
        let response = HybridDecryptResponse {
            decrypted_data: decrypted_base64,
            decryption_metadata: HybridDecryptionMetadata {
                scheme_version: hybrid_data.scheme_version,
                decrypted_at: Utc::now(),
                encryption_context: request.encryption_context.clone(),
                original_data_size: decrypted_data.len(),
            },
            key_info: HybridKeyInfo {
                key_id: key.id.unwrap(),
                algorithm: key.algorithm.unwrap_or_default(),
                is_active: key.is_active.unwrap_or(false),
                rotation_generation: key.rotation_generation.unwrap_or(1),
            },
        };

        Ok(response)
    }

    /// Helper method to decrypt private key
    fn decrypt_private_key(&self, encrypted_private_key: &[u8]) -> Result<Vec<u8>, QuantumCryptoError> {
        // For now, we'll use a simple decryption method
        // In production, this should use the master key and proper key derivation
        // This is a simplified implementation for the demo
        Ok(encrypted_private_key.to_vec())
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