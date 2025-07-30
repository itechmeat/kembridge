/**
 * Crypto Service Types
 * Full quantum cryptography API types matching old backend
 */

use serde::{Deserialize, Serialize};use uuid::Uuid;
use chrono::{DateTime, Utc};

// Status and Health Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumProtectionStatus {
    pub is_active: bool,
    pub algorithm: String,           // "ML-KEM-1024"  
    pub key_rotation_date: String,
    pub next_rotation_due: String,
    pub encryption_strength: u32,    // 1024
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub backend: bool,
    pub ai_engine: bool,
    pub blockchain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoStatus {
    pub quantum_protection: QuantumProtectionStatus,
    pub overall: String,             // "SECURE" | "WARNING" | "DANGER"
    pub is_online: bool,
    pub last_update: String,
    pub system_health: SystemHealth,
}

// Key Management Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuantumKeyRequest {
    pub key_type: String,            // "ml_kem_1024"
    pub expires_in_days: Option<u32>,
    pub usage_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyResponse {
    pub key_id: Uuid,
    pub key_type: String,
    pub public_key: String,          // Base64 encoded
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub encryption_strength: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeysListResponse {
    pub keys: Vec<QuantumKeyResponse>,
    pub total_count: usize,
    pub active_keys: usize,
}

// Encapsulation/Decapsulation Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncapsulateRequest {
    pub public_key_id: Uuid,
    pub context: Option<String>,     // Optional context for key derivation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncapsulateResponse {
    pub ciphertext: String,          // Base64 encoded
    pub shared_secret_hash: String,  // SHA-256 hash (don't expose secret)
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecapsulateRequest {
    pub private_key_id: Uuid,
    pub ciphertext: String,          // Base64 encoded
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecapsulateResponse {
    pub shared_secret_hash: String,  // SHA-256 hash
    pub success: bool,
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Key Rotation Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateKeyRequest {
    pub key_id: Uuid,
    pub reason: String,              // "scheduled", "manual", "compromised"
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateKeyResponse {
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub rotation_completed_at: DateTime<Utc>,
    pub algorithm: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRotationRequest {
    pub threshold_days: Option<u32>, // Default: 30 days
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRotationResponse {
    pub rotation_due: bool,
    pub keys_needing_rotation: Vec<Uuid>,
    pub total_keys: usize,
    pub next_rotation_date: Option<DateTime<Utc>>,
}

// Hybrid Cryptography Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptRequest {
    pub data: String,                // Data to encrypt (can be JSON)
    pub key_id: Option<Uuid>,        // If not provided, uses default key
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptResponse {
    pub encrypted_data: String,      // Base64 encoded
    pub key_id: Uuid,
    pub nonce: String,               // Base64 encoded
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridDecryptRequest {
    pub encrypted_data: String,      // Base64 encoded  
    pub key_id: Uuid,
    pub nonce: String,               // Base64 encoded
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridDecryptResponse {
    pub decrypted_data: String,
    pub success: bool,
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Export public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPublicKeyResponse {
    pub key_id: Uuid,
    pub public_key: String,          // Base64 encoded
    pub algorithm: String,
    pub format: String,              // "base64"
    pub created_at: DateTime<Utc>,
    pub usage: String,
}

// Database model for quantum keys
#[derive(Debug, Clone)]
pub struct QuantumKey {
    pub id: Uuid,
    pub user_id: Option<Uuid>,       // None for system keys
    pub algorithm: String,
    pub key_type: String,
    pub public_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub encryption_algorithm: String,
    pub security_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_compromised: bool,
    pub rotation_generation: i32,
    pub usage_category: String,
}