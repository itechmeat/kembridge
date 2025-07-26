// src/models/user.rs - User data models (Phase 2.3 implementation)
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::str::FromStr;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
pub enum UserType {
    Individual,
    Business,
    Institutional,
}

impl FromStr for UserType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "individual" => Ok(UserType::Individual),
            "business" => Ok(UserType::Business),
            "institutional" => Ok(UserType::Institutional),
            _ => Err(format!("Invalid user type: {}", s))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "kyc_status", rename_all = "snake_case")]
pub enum KycStatus {
    Pending,
    InProgress,
    Verified,
    Rejected,
}

impl FromStr for KycStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(KycStatus::Pending),
            "in_progress" => Ok(KycStatus::InProgress),
            "verified" => Ok(KycStatus::Verified),
            "rejected" => Ok(KycStatus::Rejected),
            _ => Err(format!("Invalid KYC status: {}", s))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: Option<String>,
    pub profile_data: serde_json::Value,
    pub risk_profile: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub account_status: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub profile_completeness: Option<i32>,
    pub risk_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_type: String,
    pub chain_type: Option<String>,
    pub wallet_address: Option<String>,
    pub is_primary: Option<bool>,
    pub signature_params: serde_json::Value,
    pub first_used_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub profile_data: serde_json::Value,
    pub risk_profile: serde_json::Value,
    pub wallets: Vec<UserWalletInfo>,
    pub stats: UserStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub account_status: String,
    pub profile_completeness: Option<i32>,
    pub risk_category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWalletInfo {
    pub wallet_address: String,
    pub chain_type: String,
    pub is_primary: bool,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub wallet_count: u32,
    pub transaction_count: u32,
    pub total_volume_usd: f64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: Option<String>,
    
    #[validate(custom(function = "validate_profile_data"))]
    pub profile_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddWalletRequest {
    #[validate(custom(function = "validate_wallet_address"))]
    pub wallet_address: String,
    
    #[validate(custom(function = "validate_chain_type"))]
    pub chain_type: String,
    
    #[validate(length(min = 10, message = "Signature is too short"))]
    pub signature: String,
    
    #[validate(length(min = 10, message = "Message is too short"))]
    pub message: String,
}

// Validation functions
fn validate_profile_data(profile_data: &serde_json::Value) -> Result<(), validator::ValidationError> {
    // Ensure it's an object
    if !profile_data.is_object() {
        return Err(validator::ValidationError::new("must_be_object"));
    }
    
    let obj = profile_data.as_object().unwrap();
    
    // Validate email if present
    if let Some(email) = obj.get("email") {
        if let Some(email_str) = email.as_str() {
            if !email_str.contains('@') || email_str.len() > 254 {
                return Err(validator::ValidationError::new("invalid_email"));
            }
        }
    }
    
    // Validate display_name if present
    if let Some(name) = obj.get("display_name") {
        if let Some(name_str) = name.as_str() {
            if name_str.len() > 100 {
                return Err(validator::ValidationError::new("name_too_long"));
            }
        }
    }
    
    Ok(())
}

fn validate_wallet_address(address: &str) -> Result<(), validator::ValidationError> {
    // Basic wallet address validation
    if address.starts_with("0x") {
        // Ethereum address validation
        if address.len() != 42 {
            return Err(validator::ValidationError::new("invalid_ethereum_address"));
        }
        if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(validator::ValidationError::new("invalid_ethereum_address"));
        }
    } else if address.ends_with(".near") || address.len() == 64 {
        // NEAR address validation (account.near or 64-char hex)
        if address.ends_with(".near") && address.len() > 64 {
            return Err(validator::ValidationError::new("invalid_near_address"));
        }
    } else {
        return Err(validator::ValidationError::new("unsupported_address_format"));
    }
    
    Ok(())
}

fn validate_chain_type(chain: &str) -> Result<(), validator::ValidationError> {
    match chain {
        "ethereum" | "near" | "polygon" | "bsc" => Ok(()),
        _ => Err(validator::ValidationError::new("unsupported_chain_type"))
    }
}