// src/middleware/auth.rs - JWT Authentication middleware
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::{middleware::error_handler::ApiError, AppState};

/// JWT Authentication middleware
/// Validates JWT tokens and extracts user context for protected endpoints
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Check if this is a public endpoint that doesn't require authentication
    if is_public_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..]; // Remove "Bearer " prefix
            
            // Validate JWT token
            match state.auth_service.jwt_manager.verify_token(token).await {
                Ok(claims) => {
                    tracing::debug!("JWT token validated for user: {}", claims.user_id);
                    
                    // Add user context to request headers
                    let headers = request.headers_mut();
                    headers.insert("x-user-id", claims.user_id.to_string().parse().unwrap());
                    headers.insert("x-wallet-address", claims.wallet_address.parse().unwrap());
                    headers.insert("x-chain-type", claims.chain_type.to_string().parse().unwrap());
                    headers.insert("x-session-id", claims.session_id.parse().unwrap());
                    
                    // Set user tier based on wallet type or other criteria
                    let user_tier = determine_user_tier(&claims.wallet_address);
                    headers.insert("x-user-tier", user_tier.parse().unwrap());
                },
                Err(e) => {
                    tracing::warn!("JWT token validation failed: {}", e);
                    return Err(ApiError::unauthorized("Invalid or expired token"));
                }
            }
        },
        Some(_) => {
            tracing::warn!("Invalid authorization header format");
            return Err(ApiError::unauthorized("Invalid authorization header format"));
        },
        None => {
            tracing::debug!("No authorization header for protected endpoint");
            return Err(ApiError::unauthorized("Authentication required"));
        }
    }

    // Check for quantum signature (will be implemented in Phase 3.1)
    if request.headers().contains_key("x-quantum-signature") {
        tracing::debug!("Quantum signature present, will validate in Phase 3.1");
        request.headers_mut().insert(
            "x-quantum-protected", 
            "true".parse().unwrap()
        );
    }

    Ok(next.run(request).await)
}

/// Check if endpoint is public and doesn't require authentication
fn is_public_endpoint(path: &str) -> bool {
    match path {
        // Health and status endpoints
        "/health" | "/ready" | "/metrics" => true,
        
        // WebSocket endpoint (auth handled separately)
        "/ws" => true,
        
        // Public bridge endpoints (quotes, status lookups)
        "/api/v1/bridge/quote" => true,
        
        // Pattern matching for path prefixes
        path if path.starts_with("/docs") => true,
        path if path.starts_with("/api/v1/auth") => true,
        path if path.starts_with("/api/v1/bridge/status/") => true,
        path if path.starts_with("/static") => true,
        
        _ => false,
    }
}

/// Determine user tier based on wallet address or other criteria
/// This is a simple implementation - in production you might check:
/// - Wallet balance, transaction history, KYC status, etc.
fn determine_user_tier(wallet_address: &str) -> &'static str {
    // For now, simple logic based on wallet address
    // In production, this would query the database for user preferences/tier
    if wallet_address.starts_with("0x000") || wallet_address.starts_with("admin") {
        "admin"
    } else if wallet_address.len() > 42 || wallet_address.ends_with("premium") {
        "premium" 
    } else {
        "free"
    }
}

/// Extract user ID from request headers (set by auth middleware)
pub fn extract_user_id(request: &Request) -> Option<String> {
    request.headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract wallet address from request headers
pub fn extract_wallet_address(request: &Request) -> Option<String> {
    request.headers()
        .get("x-wallet-address")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Check if request has quantum signature protection
pub fn is_quantum_protected(request: &Request) -> bool {
    request.headers()
        .get("x-quantum-protected")
        .and_then(|h| h.to_str().ok())
        .map(|s| s == "true")
        .unwrap_or(false)
}

/// Authentication context extracted from request
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub wallet_address: Option<String>,
    pub session_id: Option<String>,
    pub is_quantum_protected: bool,
    pub user_tier: UserTier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserTier {
    Free,
    Premium,
    Admin,
}

impl AuthContext {
    /// Extract authentication context from request headers
    pub fn from_request(request: &Request) -> Self {
        let user_tier = request.headers()
            .get("x-user-tier")
            .and_then(|h| h.to_str().ok())
            .map(|tier| match tier {
                "admin" => UserTier::Admin,
                "premium" => UserTier::Premium,
                _ => UserTier::Free,
            })
            .unwrap_or(UserTier::Free);

        Self {
            user_id: extract_user_id(request),
            wallet_address: extract_wallet_address(request),
            session_id: request.headers()
                .get("x-session-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            is_quantum_protected: is_quantum_protected(request),
            user_tier,
        }
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Check if user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.user_tier == UserTier::Admin
    }

    /// Check if user has premium features
    pub fn is_premium(&self) -> bool {
        matches!(self.user_tier, UserTier::Premium | UserTier::Admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn test_is_public_endpoint() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/api/v1/auth/login"));
        assert!(is_public_endpoint("/api/v1/bridge/quote"));
        assert!(is_public_endpoint("/docs/swagger"));
        
        assert!(!is_public_endpoint("/api/v1/user/profile"));
        assert!(!is_public_endpoint("/api/v1/bridge/swap"));
        assert!(!is_public_endpoint("/api/v1/quantum/generate"));
    }

    #[test]
    fn test_auth_context_extraction() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(())
            .unwrap();

        // No auth context
        let context = AuthContext::from_request(&request);
        assert!(!context.is_authenticated());
        assert_eq!(context.user_tier, UserTier::Free);

        // With auth context
        let headers = request.headers_mut();
        headers.insert("x-user-id", "user123".parse().unwrap());
        headers.insert("x-user-tier", "premium".parse().unwrap());
        headers.insert("x-wallet-address", "0x123".parse().unwrap());
        headers.insert("x-quantum-protected", "true".parse().unwrap());

        let context = AuthContext::from_request(&request);
        assert!(context.is_authenticated());
        assert!(context.is_premium());
        assert!(context.is_quantum_protected);
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.wallet_address, Some("0x123".to_string()));
    }

    #[test]
    fn test_user_tier_permissions() {
        let mut context = AuthContext {
            user_id: Some("test".to_string()),
            wallet_address: None,
            session_id: None,
            is_quantum_protected: false,
            user_tier: UserTier::Free,
        };

        assert!(!context.is_premium());
        assert!(!context.is_admin());

        context.user_tier = UserTier::Premium;
        assert!(context.is_premium());
        assert!(!context.is_admin());

        context.user_tier = UserTier::Admin;
        assert!(context.is_premium());
        assert!(context.is_admin());
    }
}