// src/handlers/jwt_validation.rs - Enhanced JWT validation handlers
use axum::{extract::State, Json, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::{middleware::error_handler::ApiError, state::AppState};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub claims: Option<Value>,
    pub errors: Vec<String>,
    pub token_age_seconds: Option<u64>,
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
pub async fn validate_jwt_token(
    State(state): State<AppState>,
    Json(payload): Json<JwtValidationRequest>,
) -> Result<Json<JwtValidationResponse>, ApiError> {
    let mut response = JwtValidationResponse {
        valid: false,
        expired: false,
        signature_valid: false,
        claims: None,
        errors: Vec::new(),
        token_age_seconds: None,
    };
    
    // Basic token format validation
    if payload.token.is_empty() {
        response.errors.push("Token is empty".to_string());
        return Ok(Json(response));
    }
    
    let token_parts: Vec<&str> = payload.token.split('.').collect();
    if token_parts.len() != 3 {
        response.errors.push("Invalid JWT format - must have 3 parts".to_string());
        return Ok(Json(response));
    }
    
    // Decode and validate token
    let jwt_secret = state.config.jwt_secret.as_bytes();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = payload.check_expiry.unwrap_or(true);
    
    match decode::<Claims>(
        &payload.token,
        &DecodingKey::from_secret(jwt_secret),
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
            
            // Validate issuer if configured
            if let Some(expected_iss) = &state.config.jwt_issuer {
                if claims.iss.as_ref() != Some(expected_iss) {
                    response.errors.push("Invalid token issuer".to_string());
                }
            }
            
            // Validate wallet address format if present
            if let Some(wallet) = &claims.wallet_address {
                if !is_valid_wallet_address(wallet) {
                    response.errors.push("Invalid wallet address format in token".to_string());
                }
            }
            
            response.claims = Some(json!({
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
        }
    }
    
    Ok(Json(response))
}

/// Extract and validate JWT from Authorization header
pub async fn validate_auth_header(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<JwtValidationResponse>, ApiError> {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;
    
    let token = if auth_header.starts_with("Bearer ") {
        auth_header.strip_prefix("Bearer ").unwrap_or("")
    } else {
        return Err(ApiError::unauthorized("Invalid Authorization header format"));
    };
    
    let request = JwtValidationRequest {
        token: token.to_string(),
        check_expiry: Some(true),
        check_signature: Some(true),
    };
    
    validate_jwt_token(State(state), Json(request)).await
}

/// Check if error is critical (affects token validity)
fn is_critical_error(error: &str) -> bool {
    error.contains("Invalid token issuer") ||
    error.contains("Invalid wallet address format")
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
    fn test_is_valid_wallet_address() {
        // Valid Ethereum addresses
        assert!(is_valid_wallet_address("0x1234567890123456789012345678901234567890"));
        assert!(is_valid_wallet_address("0xabcdefABCDEF1234567890123456789012345678"));
        
        // Valid NEAR addresses
        assert!(is_valid_wallet_address("alice.near"));
        assert!(is_valid_wallet_address("test-account.testnet"));
        assert!(is_valid_wallet_address("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
        
        // Invalid addresses
        assert!(!is_valid_wallet_address("invalid"));
        assert!(!is_valid_wallet_address("0x123"));
        assert!(!is_valid_wallet_address("alice.invalid"));
    }
    
    #[test]
    fn test_is_critical_error() {
        assert!(is_critical_error("Invalid token issuer"));
        assert!(is_critical_error("Invalid wallet address format in token"));
        assert!(!is_critical_error("Token is older than 24 hours"));
        assert!(!is_critical_error("Token has expired"));
    }
}