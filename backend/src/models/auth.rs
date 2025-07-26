// src/models/auth.rs - Authentication data models (Phase 2 placeholder)
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 42, max = 42))] // Ethereum address length
    pub wallet_address: String,

    #[validate(length(min = 1))]
    pub signature: String,

    #[validate(length(min = 1))]
    pub message: String,

    #[validate(length(min = 1, max = 20))]
    pub chain_type: String, // "ethereum" or "near"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user_id: Uuid,
    pub wallet_address: String,
    pub quantum_keys_available: bool,
}