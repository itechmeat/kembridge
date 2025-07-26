// src/models/quantum.rs - Quantum cryptography data models for Phase 3.2
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Database model for quantum_keys table (matches Phase 1.2 schema exactly)
#[derive(Debug, Clone, FromRow)]
pub struct QuantumKey {
    pub id: Option<Uuid>,      // Primary key, potentially nullable in sqlx mapping
    pub user_id: Option<Uuid>, // Foreign key, potentially nullable in sqlx mapping
    
    // Post-quantum algorithm specification (has defaults, nullable in sqlx)
    pub algorithm: Option<String>,  // DEFAULT 'ml-kem-1024'
    pub key_type: Option<String>,   // DEFAULT 'key_encapsulation'
    
    // Key storage (NOT NULL)
    pub public_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    
    // Encryption configuration (has default, nullable in sqlx)
    pub encryption_algorithm: Option<String>,  // DEFAULT 'aes-256-gcm'
    pub encryption_iv: Option<Vec<u8>>, // NULLABLE
    pub encryption_salt: Option<Vec<u8>>, // NULLABLE
    
    // Key derivation and security metadata (has defaults, nullable in sqlx)
    pub key_derivation_params: Option<serde_json::Value>, // JSONB DEFAULT '{}'
    pub security_metadata: Option<serde_json::Value>,     // JSONB DEFAULT '{}'
    
    // Key lifecycle timestamps  
    pub created_at: Option<DateTime<Utc>>, // transaction_timestamp type (nullable in sqlx mapping)
    pub expires_at: Option<DateTime<Utc>>, // NULLABLE
    pub rotated_at: Option<DateTime<Utc>>, // NULLABLE
    
    // Key status (has defaults, nullable in sqlx)
    pub is_active: Option<bool>,           // BOOLEAN DEFAULT true
    pub is_compromised: Option<bool>,      // BOOLEAN DEFAULT false  
    pub validation_status: Option<String>, // VARCHAR(20) DEFAULT 'pending'
    
    // Key rotation chain (has default, nullable in sqlx)
    pub previous_key_id: Option<Uuid>,    // NULLABLE REFERENCES quantum_keys(id)
    pub rotation_reason: Option<String>,  // NULLABLE VARCHAR(255)
    pub rotation_generation: Option<i32>, // DEFAULT 1
    
    // HSM integration (both nullable)
    pub hsm_key_id: Option<String>,    // NULLABLE VARCHAR(255)
    pub hsm_provider: Option<String>,  // NULLABLE VARCHAR(100)
    
    // Generated columns from schema (generated always as stored, potentially nullable in sqlx)
    pub key_status: Option<String>,      // GENERATED ALWAYS AS (...) STORED
    pub key_strength: Option<String>,    // GENERATED ALWAYS AS (...) STORED  
    pub usage_category: Option<String>,  // GENERATED ALWAYS AS (...) STORED
}

/// Request to create a new quantum key pair
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateQuantumKeyRequest {
    /// Key algorithm type (currently only "ml_kem_1024")
    #[schema(example = "ml_kem_1024")]
    pub key_type: String,
    /// Optional expiration in days (default: 365)
    #[schema(example = 365)]
    pub expires_in_days: Option<i32>,
}

/// Response containing quantum key information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuantumKeyResponse {
    /// Unique key identifier
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    /// Public key in Base64 format
    pub public_key: String,
    /// Algorithm type
    #[schema(example = "ml_kem_1024")]
    pub algorithm: String,
    /// Key metadata
    pub key_metadata: serde_json::Value,
    /// When the key was created
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    /// When the key expires (if set)
    #[schema(value_type = Option<String>, format = "date-time")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the key is active
    pub is_active: bool,
    /// Number of times this key was used
    pub usage_count: i32,
}

/// Request for encapsulation operation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EncapsulateRequest {
    /// ID of the public key to use for encapsulation
    #[schema(value_type = String, format = "uuid")]
    pub public_key_id: Uuid,
    /// Optional metadata for the operation
    pub metadata: Option<serde_json::Value>,
}

/// Response from encapsulation operation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EncapsulateResponse {
    /// Encrypted ciphertext in Base64 format
    pub ciphertext: String,
    /// Operation identifier for tracking
    #[schema(value_type = String, format = "uuid")]
    pub operation_id: Uuid,
    /// When the operation was performed
    #[schema(value_type = String, format = "date-time")]
    pub timestamp: DateTime<Utc>,
}

/// Request for decapsulation operation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DecapsulateRequest {
    /// ID of the private key to use for decapsulation
    #[schema(value_type = String, format = "uuid")]
    pub private_key_id: Uuid,
    /// Ciphertext to decrypt in Base64 format
    pub ciphertext: String,
}

/// Response from decapsulation operation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DecapsulateResponse {
    /// Hash of the shared secret (never return actual secret)
    pub shared_secret_hash: String,
    /// Whether the operation was successful
    pub success: bool,
    /// Operation identifier for tracking
    #[schema(value_type = String, format = "uuid")]
    pub operation_id: Uuid,
    /// When the operation was performed
    #[schema(value_type = String, format = "date-time")]
    pub timestamp: DateTime<Utc>,
}

/// Response for listing user's quantum keys
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuantumKeysListResponse {
    /// List of user's quantum keys
    pub keys: Vec<QuantumKeyResponse>,
    /// Total number of keys
    pub total: usize,
}