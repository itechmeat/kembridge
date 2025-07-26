// src/models/user.rs - User data models (Phase 2.3 placeholder)
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub wallet_addresses: Vec<String>,
    pub risk_score: f64,
    pub transaction_count: u32,
    pub total_volume_usd: f64,
}