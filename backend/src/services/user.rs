// src/services/user.rs - User management service (Phase 2.3 implementation)
use crate::models::user::{
    UserProfile, UserProfileResponse, UserWallet, UserWalletInfo, 
    UserStats, UpdateUserRequest, AddWalletRequest
};
use crate::middleware::error_handler::ApiError;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct UserService {
    db_pool: PgPool,
}

impl UserService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get user profile by user ID with wallets and stats
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserProfile>, ApiError> {
        let user = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT 
                id, username, profile_data, risk_profile, 
                created_at, updated_at, is_active, account_status,
                last_login_at, profile_completeness, risk_category
            FROM users 
            WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user by ID {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to fetch user profile")
        })?;

        Ok(user)
    }

    /// Get full user profile response with wallets and statistics
    pub async fn get_user_profile_response(&self, user_id: Uuid) -> Result<UserProfileResponse, ApiError> {
        // Get user profile
        let user = self.get_user_by_id(user_id).await?
            .ok_or_else(|| ApiError::not_found("User not found"))?;

        // Get user's wallets
        let wallets = self.get_user_wallets(user_id).await?;

        // Get user statistics
        let stats = self.get_user_stats(user_id).await?;

        // Extract email and name from profile_data
        let email = user.profile_data.get("email").and_then(|v| v.as_str()).map(String::from);
        let name = user.profile_data.get("display_name").and_then(|v| v.as_str()).map(String::from);

        Ok(UserProfileResponse {
            id: user.id,
            username: user.username,
            email,
            name,
            profile_data: user.profile_data,
            risk_profile: user.risk_profile,
            wallets,
            stats,
            created_at: user.created_at.unwrap_or_default(),
            updated_at: user.updated_at.unwrap_or_default(),
            is_active: user.is_active.unwrap_or(true),
            account_status: user.account_status.unwrap_or_default(),
            profile_completeness: user.profile_completeness,
            risk_category: user.risk_category,
        })
    }

    /// Get user wallets with chain information
    pub async fn get_user_wallets(&self, user_id: Uuid) -> Result<Vec<UserWalletInfo>, ApiError> {
        let wallets = sqlx::query_as!(
            UserWallet,
            r#"
            SELECT 
                id, user_id, auth_type, chain_type, 
                wallet_address, is_primary, signature_params,
                first_used_at, last_used_at, is_verified
            FROM user_auth_methods 
            WHERE user_id = $1 AND auth_type = 'web3_wallet'
            ORDER BY is_primary DESC, first_used_at ASC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch wallets for user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to fetch user wallets")
        })?;

        let wallet_infos = wallets
            .into_iter()
            .filter_map(|wallet| {
                if let (Some(address), Some(chain)) = (wallet.wallet_address, wallet.chain_type) {
                    Some(UserWalletInfo {
                        wallet_address: address,
                        chain_type: chain,
                        is_primary: wallet.is_primary.unwrap_or(false),
                        verified_at: if wallet.is_verified.unwrap_or(false) { wallet.first_used_at } else { None },
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(wallet_infos)
    }

    /// Get user statistics from database
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserStats, ApiError> {
        // Get wallet count
        let wallet_count = sqlx::query_scalar!(
            "SELECT COUNT(*)::INTEGER FROM user_auth_methods WHERE user_id = $1 AND auth_type = 'web3_wallet'",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count wallets for user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to get user stats")
        })?
        .unwrap_or(0);

        // Get transaction count (will be implemented in Phase 4.3)
        let transaction_count = 0;

        // Get total volume (will be implemented in Phase 4.3)
        let total_volume_usd = 0.0;

        // Get last activity from sessions
        let last_activity = sqlx::query_scalar!(
            "SELECT MAX(created_at) FROM user_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get last activity for user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to get user stats")
        })?
        .flatten();

        Ok(UserStats {
            wallet_count: wallet_count as u32,
            transaction_count,
            total_volume_usd,
            last_activity,
        })
    }

    /// Update user profile
    pub async fn update_user(&self, user_id: Uuid, request: UpdateUserRequest) -> Result<UserProfile, ApiError> {
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
            return Err(ApiError::bad_request(error_message));
        }

        let updated_user = sqlx::query_as!(
            UserProfile,
            r#"
            UPDATE users 
            SET 
                username = COALESCE($2, username),
                profile_data = COALESCE($3, profile_data),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING 
                id, username, profile_data, risk_profile, 
                created_at, updated_at, is_active, account_status,
                last_login_at, profile_completeness, risk_category
            "#,
            user_id,
            request.username,
            request.profile_data
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to update user profile")
        })?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

        Ok(updated_user)
    }

    /// Soft delete user (Phase 2.3.6)
    pub async fn soft_delete_user(&self, user_id: Uuid) -> Result<(), ApiError> {
        let result = sqlx::query!(
            "UPDATE users SET is_active = false, account_status = 'closed' WHERE id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to soft delete user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to delete user")
        })?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("User not found or already deleted"));
        }

        // Invalidate all sessions for the user
        if let Err(e) = sqlx::query!(
            "UPDATE user_sessions SET expires_at = NOW() WHERE user_id = $1 AND expires_at > NOW()",
            user_id
        )
        .execute(&self.db_pool)
        .await
        {
            tracing::warn!("Failed to invalidate sessions for deleted user {}: {}", user_id, e);
        }

        Ok(())
    }

    /// Add new wallet to user (Phase 2.3.4)
    pub async fn add_wallet(&self, user_id: Uuid, request: AddWalletRequest) -> Result<(), ApiError> {
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
            return Err(ApiError::bad_request(error_message));
        }

        // In a real implementation, we would verify the signature here
        // For now, we'll just add the wallet
        
        // Check if wallet already exists
        let existing_wallet = sqlx::query!(
            "SELECT id FROM user_auth_methods WHERE wallet_address = $1 AND chain_type = $2",
            request.wallet_address,
            request.chain_type
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check existing wallet: {}", e);
            ApiError::internal_server_error("Failed to verify wallet")
        })?;

        if existing_wallet.is_some() {
            return Err(ApiError::bad_request("Wallet already linked to another user"));
        }

        // Add the wallet
        sqlx::query!(
            r#"
            INSERT INTO user_auth_methods 
                (user_id, auth_type, chain_type, wallet_address, is_primary, signature_params, is_verified, first_used_at)
            VALUES ($1, 'web3_wallet', $2, $3, FALSE, $4, TRUE, NOW())
            "#,
            user_id,
            request.chain_type,
            request.wallet_address,
            serde_json::json!({
                "signature": request.signature,
                "message": request.message,
                "algorithm": if request.chain_type == "ethereum" { "secp256k1" } else { "ed25519" }
            })
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add wallet for user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to add wallet")
        })?;

        Ok(())
    }

    /// Remove wallet from user
    pub async fn remove_wallet(&self, user_id: Uuid, wallet_address: &str) -> Result<(), ApiError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_auth_methods 
            WHERE user_id = $1 AND wallet_address = $2 AND auth_type = 'web3_wallet'
            "#,
            user_id,
            wallet_address
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to remove wallet {} for user {}: {}", wallet_address, user_id, e);
            ApiError::internal_server_error("Failed to remove wallet")
        })?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("Wallet not found or not owned by user"));
        }

        Ok(())
    }

    /// Set primary wallet for user
    pub async fn set_primary_wallet(&self, user_id: Uuid, wallet_address: &str) -> Result<(), ApiError> {
        // Start transaction to ensure atomicity
        let mut tx = self.db_pool.begin().await
            .map_err(|e| {
                tracing::error!("Failed to start transaction: {}", e);
                ApiError::internal_server_error("Database transaction failed")
            })?;

        // First, unset all primary flags for this user
        sqlx::query!(
            "UPDATE user_auth_methods SET is_primary = FALSE WHERE user_id = $1 AND auth_type = 'web3_wallet'",
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to unset primary wallets for user {}: {}", user_id, e);
            ApiError::internal_server_error("Failed to update primary wallet")
        })?;

        // Then set the specified wallet as primary
        let result = sqlx::query!(
            r#"
            UPDATE user_auth_methods 
            SET is_primary = TRUE 
            WHERE user_id = $1 AND wallet_address = $2 AND auth_type = 'web3_wallet'
            "#,
            user_id,
            wallet_address
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set primary wallet {} for user {}: {}", wallet_address, user_id, e);
            ApiError::internal_server_error("Failed to update primary wallet")
        })?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("Wallet not found or not owned by user"));
        }

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                tracing::error!("Failed to commit primary wallet transaction: {}", e);
                ApiError::internal_server_error("Failed to update primary wallet")
            })?;

        Ok(())
    }
}