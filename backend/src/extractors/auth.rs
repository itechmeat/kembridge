// src/extractors/auth.rs - Authentication extractors
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;
use crate::middleware::auth::UserTier;
use kembridge_auth::ChainType;

/// Authentication context extracted from request headers
/// Set by the auth middleware after JWT validation
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub chain_type: ChainType,
    pub session_id: String,
    pub user_tier: UserTier,
    pub is_quantum_protected: bool,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract user ID
        let user_id = parts.headers
            .get("x-user-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or(AuthError::MissingUserId)?;

        // Extract wallet address  
        let wallet_address = parts.headers
            .get("x-wallet-address")
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingWalletAddress)?
            .to_string();

        // Extract chain type
        let chain_type = parts.headers
            .get("x-chain-type")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
            .ok_or(AuthError::MissingChainType)?;

        // Extract session ID
        let session_id = parts.headers
            .get("x-session-id")
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingSessionId)?
            .to_string();

        // Extract user tier
        let user_tier = parts.headers
            .get("x-user-tier")
            .and_then(|h| h.to_str().ok())
            .map(|tier| match tier {
                "admin" => UserTier::Admin,
                "premium" => UserTier::Premium,
                _ => UserTier::Free,
            })
            .unwrap_or(UserTier::Free);

        // Check quantum protection
        let is_quantum_protected = parts.headers
            .get("x-quantum-protected")
            .and_then(|h| h.to_str().ok())
            .map(|s| s == "true")
            .unwrap_or(false);

        Ok(AuthUser {
            user_id,
            wallet_address,
            chain_type,
            session_id,
            user_tier,
            is_quantum_protected,
        })
    }
}

/// Optional authentication - returns None if not authenticated
/// Useful for endpoints that can work with or without auth
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuth(AuthUser::from_request_parts(parts, state).await.ok()))
    }
}

/// Admin-only authentication extractor
/// Automatically rejects non-admin users
#[derive(Debug, Clone)]
pub struct AdminAuth(pub AuthUser);

impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        
        if auth_user.user_tier != UserTier::Admin {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(AdminAuth(auth_user))
    }
}

/// Premium-or-higher authentication extractor
#[derive(Debug, Clone)]
pub struct PremiumAuth(pub AuthUser);

impl<S> FromRequestParts<S> for PremiumAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        
        if !matches!(auth_user.user_tier, UserTier::Premium | UserTier::Admin) {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(PremiumAuth(auth_user))
    }
}

impl AuthUser {
    /// Check if user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.user_tier == UserTier::Admin
    }

    /// Check if user has premium features
    pub fn is_premium(&self) -> bool {
        matches!(self.user_tier, UserTier::Premium | UserTier::Admin)
    }

    /// Check if user is using quantum-protected session
    pub fn is_quantum_protected(&self) -> bool {
        self.is_quantum_protected
    }
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingUserId,
    MissingWalletAddress,
    MissingChainType,
    MissingSessionId,
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingUserId => (StatusCode::UNAUTHORIZED, "Missing user ID"),
            AuthError::MissingWalletAddress => (StatusCode::UNAUTHORIZED, "Missing wallet address"),
            AuthError::MissingChainType => (StatusCode::UNAUTHORIZED, "Missing chain type"),
            AuthError::MissingSessionId => (StatusCode::UNAUTHORIZED, "Missing session ID"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };

        let body = Json(json!({
            "error": "AuthenticationError",
            "message": message,
            "status": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}