// Minimal 1inch service for architecture testing
pub mod config;
pub mod errors;

// Simplified types - only what we need for testing
pub mod types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SimpleQuoteRequest {
        pub from_token: String,
        pub to_token: String,
        pub amount: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SimpleQuoteResponse {
        pub from_amount: String,
        pub to_amount: String,
        pub quote_id: String,
    }
}

// Simplified handlers - minimal endpoints
pub mod handlers {
    use axum::{extract::Query, Json};
    use crate::types::{SimpleQuoteRequest, SimpleQuoteResponse};
    use kembridge_common::ServiceResponse;
    
    pub async fn simple_quote(
        Query(request): Query<SimpleQuoteRequest>,
    ) -> Result<Json<ServiceResponse<SimpleQuoteResponse>>, crate::errors::OneinchServiceError> {
        let response = SimpleQuoteResponse {
            from_amount: request.amount.clone(),
            to_amount: format!("{}00", request.amount), // Mock 2x conversion
            quote_id: uuid::Uuid::new_v4().to_string(),
        };
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn health() -> Json<ServiceResponse<String>> {
        Json(ServiceResponse::success("1inch Service OK".to_string()))
    }
}

// Re-export for easy access
pub use types::*;
pub use errors::*;