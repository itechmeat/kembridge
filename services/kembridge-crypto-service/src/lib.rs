// Minimal crypto service for architecture testing
pub mod config;
pub mod errors;

// Simplified types - only what we need for testing
pub mod types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GenerateKeyRequest {
        pub key_type: String,
        pub algorithm: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GenerateKeyResponse {
        pub key_id: String,
        pub public_key: String,
        pub algorithm: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EncryptRequest {
        pub data: String,
        pub key_id: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EncryptResponse {
        pub encrypted_data: String,
        pub nonce: String,
        pub key_id: String,
    }
}

// Simplified handlers - minimal endpoints
pub mod handlers {
    use axum::{extract::Query, Json};
    use crate::types::{GenerateKeyRequest, GenerateKeyResponse, EncryptRequest, EncryptResponse};
    use kembridge_common::ServiceResponse;
    
    pub async fn simple_generate_key(
        Query(request): Query<GenerateKeyRequest>,
    ) -> Result<Json<ServiceResponse<GenerateKeyResponse>>, crate::errors::CryptoServiceError> {
        let response = GenerateKeyResponse {
            key_id: "mock-key-123".to_string(),
            public_key: "mock-public-key-data".to_string(),
            algorithm: request.algorithm.clone(),
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn simple_encrypt(
        Query(request): Query<EncryptRequest>,
    ) -> Result<Json<ServiceResponse<EncryptResponse>>, crate::errors::CryptoServiceError> {
        let response = EncryptResponse {
            encrypted_data: "mock-encrypted-data".to_string(),
            nonce: "mock-nonce-123".to_string(),
            key_id: request.key_id.clone(),
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn health() -> Json<ServiceResponse<String>> {
        Json(ServiceResponse::success("🔥⚡ CRYPTO HOT RELOAD TESTED! ⚡🔥".to_string()))
    }
}

// Re-export for easy access
pub use types::*;
pub use errors::*;