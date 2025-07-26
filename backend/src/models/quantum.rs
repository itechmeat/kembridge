// src/models/quantum.rs - Quantum cryptography data models (Phase 3 placeholder)
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct QuantumKeyPair {
    pub key_id: Uuid,
    pub public_key: String,
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub key_strength: String,
    pub usage_category: String,
}