// src/jwt_validation.rs - JWT validation module for WebSocket authentication
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtValidationRequest {
    pub token: String,
    pub check_expiry: Option<bool>,
    pub check_signature: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtValidationResponse {
    pub valid: bool,
    pub expired: bool,
    pub signature_valid: bool,
    pub claims: Option<serde_json::Value>,
    pub errors: Vec<String>,
    pub token_age_seconds: Option<u64>,
    pub user_id: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: Option<String>,
    aud: Option<String>,
    wallet_address: Option<String>,
    user_tier: Option<String>,
}

/// Validate JWT token with comprehensive checks
pub fn validate_jwt_token(token: &str, jwt_secret: &str) -> JwtValidationResponse {
    let mut response = JwtValidationResponse {
        valid: false,
        expired: false,
        signature_valid: false,
        claims: None,
        errors: Vec::new(),
        token_age_seconds: None,
        user_id: None,
        wallet_address: None,
    };
    
    // Basic token format validation
    if token.is_empty() {
        response.errors.push("Token is empty".to_string());
        return response;
    }
    
    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 3 {
        response.errors.push("Invalid JWT format - must have 3 parts".to_string());
        return response;
    }
    
    // Decode and validate token
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_aud = false; // Disable audience validation for now
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            response.signature_valid = true;
            
            let claims = &token_data.claims;
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize;
            
            // Check expiration
            if claims.exp < current_time {
                response.expired = true;
                response.errors.push("Token has expired".to_string());
            }
            
            // Calculate token age
            if claims.iat <= current_time {
                response.token_age_seconds = Some((current_time - claims.iat) as u64);
            }
            
            // Additional security checks
            if let Some(token_age) = response.token_age_seconds {
                // Warn if token is very old (more than 24 hours)
                if token_age > 86400 {
                    response.errors.push("Token is older than 24 hours".to_string());
                }
            }
            
            // Validate user ID (sub) is not empty
            if claims.sub.trim().is_empty() {
                response.errors.push("User ID (sub) cannot be empty in token".to_string());
            }
            
            // Validate wallet address format if present
            if let Some(wallet) = &claims.wallet_address {
                if !is_valid_wallet_address(wallet) {
                    response.errors.push("Invalid wallet address format in token".to_string());
                }
            }
            
            // Set user information
            response.user_id = Some(claims.sub.clone());
            response.wallet_address = claims.wallet_address.clone();
            
            response.claims = Some(serde_json::json!({
                "sub": claims.sub,
                "exp": claims.exp,
                "iat": claims.iat,
                "iss": claims.iss,
                "aud": claims.aud,
                "wallet_address": claims.wallet_address,
                "user_tier": claims.user_tier
            }));
            
            // Token is valid if signature is valid, not expired, and no critical errors
            response.valid = response.signature_valid && !response.expired && 
                           !response.errors.iter().any(|e| is_critical_error(e));
                           
            debug!("JWT validation successful for user: {}", claims.sub);
        }
        Err(err) => {
            match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    response.expired = true;
                    response.errors.push("Token signature has expired".to_string());
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    response.errors.push("Invalid token signature".to_string());
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    response.errors.push("Invalid token format".to_string());
                }
                _ => {
                    response.errors.push(format!("Token validation error: {}", err));
                }
            }
            warn!("JWT validation failed: {:?}", err);
        }
    }
    
    response
}

/// Check if error is critical (affects token validity)
fn is_critical_error(error: &str) -> bool {
    error.contains("Invalid token issuer") ||
    error.contains("Invalid wallet address format") ||
    error.contains("User ID (sub) cannot be empty")
}

/// Validate wallet address format
fn is_valid_wallet_address(address: &str) -> bool {
    // Ethereum address format
    if address.len() == 42 && address.starts_with("0x") {
        return address[2..].chars().all(|c| c.is_ascii_hexdigit());
    }
    
    // NEAR address format
    if address.ends_with(".near") || address.ends_with(".testnet") {
        return address.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
    }
    
    // NEAR implicit account (64 hex chars)
    if address.len() == 64 {
        return address.chars().all(|c| c.is_ascii_hexdigit());
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_critical_error() {
        assert!(is_critical_error("Invalid token issuer"));
        assert!(is_critical_error("Invalid wallet address format in token"));
        assert!(!is_critical_error("Token is older than 24 hours"));
    }
    
    #[test]
    fn test_is_valid_wallet_address() {
        // Valid Ethereum address
        assert!(is_valid_wallet_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b"));
        
        // Valid NEAR address
        assert!(is_valid_wallet_address("alice.near"));
        assert!(is_valid_wallet_address("test.testnet"));
        
        // Valid NEAR implicit account
        assert!(is_valid_wallet_address("abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234"));
        
        // Invalid addresses
        assert!(!is_valid_wallet_address("invalid"));
        assert!(!is_valid_wallet_address("0xinvalid"));
        assert!(!is_valid_wallet_address(""));
    }
    
    #[test]
    fn test_empty_token() {
        let result = validate_jwt_token("", "secret");
        assert!(!result.valid);
        assert!(result.errors.contains(&"Token is empty".to_string()));
    }
    
    #[test]
    fn test_invalid_format() {
        let result = validate_jwt_token("invalid.format", "secret");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid JWT format")));
    }
}