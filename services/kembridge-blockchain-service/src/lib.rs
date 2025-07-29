// Minimal blockchain service for architecture testing
pub mod config;
pub mod errors;

// Simplified types - only what we need for testing
pub mod types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EthBalanceRequest {
        pub address: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EthBalanceResponse {
        pub address: String,
        pub balance: String,
        pub chain_id: u64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NearAccountRequest {
        pub account_id: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NearAccountResponse {
        pub account_id: String,
        pub balance: String,
        pub storage_usage: u64,
    }
}

// Simplified handlers - minimal endpoints
pub mod handlers {
    use axum::{extract::Query, Json};
    use crate::types::{EthBalanceRequest, EthBalanceResponse, NearAccountRequest, NearAccountResponse};
    use kembridge_common::ServiceResponse;
    
    pub async fn simple_eth_balance(
        Query(request): Query<EthBalanceRequest>,
    ) -> Result<Json<ServiceResponse<EthBalanceResponse>>, crate::errors::BlockchainServiceError> {
        let response = EthBalanceResponse {
            address: request.address.clone(),
            balance: "1000000000000000000".to_string(), // Mock 1 ETH
            chain_id: 1,
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn simple_near_account(
        Query(request): Query<NearAccountRequest>,
    ) -> Result<Json<ServiceResponse<NearAccountResponse>>, crate::errors::BlockchainServiceError> {
        let response = NearAccountResponse {
            account_id: request.account_id.clone(),
            balance: "1000000000000000000000000".to_string(), // Mock 1 NEAR
            storage_usage: 1000,
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn health() -> Json<ServiceResponse<String>> {
        Json(ServiceResponse::success("Blockchain Service OK".to_string()))
    }
}

// Re-export for easy access
pub use types::*;
pub use errors::*;