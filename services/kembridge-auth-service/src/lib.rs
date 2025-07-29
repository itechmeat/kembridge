// Minimal auth service for architecture testing
pub mod config;
pub mod errors;

// Simplified types - only what we need for testing
pub mod types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginRequest {
        pub wallet_address: String,
        pub signature: String,
        pub message: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoginResponse {
        pub token: String,
        pub user_id: String,
        pub wallet_address: String,
        pub expires_at: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValidateTokenRequest {
        pub token: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValidateTokenResponse {
        pub valid: bool,
        pub user_id: String,
        pub wallet_address: String,
    }
}

// Simplified handlers - minimal endpoints
pub mod handlers {
    use axum::{extract::Query, Json};
    use crate::types::{LoginRequest, LoginResponse, ValidateTokenRequest, ValidateTokenResponse};
    use kembridge_common::ServiceResponse;
    
    pub async fn simple_login(
        Query(request): Query<LoginRequest>,
    ) -> Result<Json<ServiceResponse<LoginResponse>>, crate::errors::AuthServiceError> {
        let response = LoginResponse {
            token: "mock-jwt-token-123".to_string(),
            user_id: "user-456".to_string(),
            wallet_address: request.wallet_address.clone(),
            expires_at: "2025-08-26T21:00:00Z".to_string(),
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn simple_validate(
        Query(request): Query<ValidateTokenRequest>,
    ) -> Result<Json<ServiceResponse<ValidateTokenResponse>>, crate::errors::AuthServiceError> {
        let response = ValidateTokenResponse {
            valid: true,
            user_id: "user-456".to_string(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn health() -> Json<ServiceResponse<String>> {
        Json(ServiceResponse::success("Auth Service OK".to_string()))
    }
}

// Re-export for easy access
pub use types::*;
pub use errors::*;