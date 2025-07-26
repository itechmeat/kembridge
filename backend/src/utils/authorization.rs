// src/utils/authorization.rs - Authorization rules for user operations
use crate::extractors::auth::AuthUser;
use crate::middleware::error_handler::ApiError;
use uuid::Uuid;

/// Authorization levels for user operations
pub enum AuthLevel {
    /// User can only access their own data
    Self_,
    /// Admin can access any user data
    Admin,
    /// System operations (automated processes)
    System,
}

/// Check if user can access target user's data
pub async fn require_self_or_admin(
    auth_user: &AuthUser,
    target_user_id: Uuid,
) -> Result<(), ApiError> {
    if auth_user.user_id == target_user_id || auth_user.is_admin() {
        Ok(())
    } else {
        Err(ApiError::unauthorized("Access denied: can only access your own profile"))
    }
}

/// Check if user has admin privileges
pub async fn require_admin(auth_user: &AuthUser) -> Result<(), ApiError> {
    if auth_user.is_admin() {
        Ok(())
    } else {
        Err(ApiError::unauthorized("Admin access required"))
    }
}

/// Check if user has premium privileges
pub async fn require_premium_or_admin(auth_user: &AuthUser) -> Result<(), ApiError> {
    if auth_user.is_premium() || auth_user.is_admin() {
        Ok(())
    } else {
        Err(ApiError::unauthorized("Premium or admin access required"))
    }
}

/// Check if user can perform system-level operations
pub async fn require_system(auth_user: &AuthUser) -> Result<(), ApiError> {
    // For now, system access is equivalent to admin access
    // TODO: Phase 3+ - Implement proper system-level authentication
    if auth_user.is_admin() {
        Ok(())
    } else {
        Err(ApiError::unauthorized("System access required"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractors::auth::{AuthUser, UserTier};
    use kembridge_auth::ChainType;

    fn create_test_user(user_id: Uuid, tier: UserTier) -> AuthUser {
        AuthUser {
            user_id,
            wallet_address: "0x123".to_string(),
            chain_type: ChainType::Ethereum,
            session_id: "test_session".to_string(),
            user_tier: tier,
            is_quantum_protected: false,
        }
    }

    #[tokio::test]
    async fn test_require_self_or_admin_self_access() {
        let user_id = Uuid::new_v4();
        let auth_user = create_test_user(user_id, UserTier::Standard);
        
        let result = require_self_or_admin(&auth_user, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_require_self_or_admin_admin_access() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let auth_user = create_test_user(user_id, UserTier::Admin);
        
        let result = require_self_or_admin(&auth_user, other_user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_require_self_or_admin_denied() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let auth_user = create_test_user(user_id, UserTier::Standard);
        
        let result = require_self_or_admin(&auth_user, other_user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_require_admin_success() {
        let user_id = Uuid::new_v4();
        let auth_user = create_test_user(user_id, UserTier::Admin);
        
        let result = require_admin(&auth_user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_require_admin_denied() {
        let user_id = Uuid::new_v4();
        let auth_user = create_test_user(user_id, UserTier::Standard);
        
        let result = require_admin(&auth_user).await;
        assert!(result.is_err());
    }
}