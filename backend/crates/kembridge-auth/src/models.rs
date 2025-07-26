// Data models for Web3 authentication

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::chains::ChainType;

#[derive(Debug, Deserialize, Validate)]
pub struct NonceRequest {
    #[validate(length(min = 1, max = 100))]
    pub wallet_address: String,
    pub chain_type: ChainType,
}

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
    pub message: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 1, max = 100))]
    pub wallet_address: String,
    pub chain_type: ChainType,
    #[validate(length(min = 1, max = 1000))]
    pub message: String,
    #[validate(length(min = 1, max = 500))]
    pub signature: String,
    #[validate(length(min = 1, max = 100))]
    pub nonce: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user_id: Uuid,
    pub wallet_address: String,
    pub chain_type: ChainType,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}