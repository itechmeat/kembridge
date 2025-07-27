// src/routes/oneinch.rs - Routes for 1inch Fusion+ integration

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::oneinch::*,
    state::AppState,
};

/// Create 1inch Fusion+ routes
pub fn create_oneinch_routes() -> Router<AppState> {
    Router::new()
        // Quote endpoints
        .route("/quote", post(get_quote))
        .route("/quote/enhanced", post(get_enhanced_quote))
        
        // Intelligent routing endpoints
        .route("/routing/intelligent", post(get_intelligent_routing))
        
        // Swap execution endpoints  
        .route("/execute", post(execute_swap))
        .route("/execute-signed", post(execute_signed_swap))
        
        // Order management endpoints
        .route("/order/{order_hash}", get(get_order_status))
        
        // Token information endpoints
        .route("/tokens", get(get_supported_tokens))
        
        // Health and status endpoints
        .route("/health", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;
    use std::sync::Arc;
    
    use crate::{
        config::AppConfig,
        oneinch::{OneinchService},
        constants::*,
    };

    async fn create_test_app() -> TestServer {
        let config = Arc::new(AppConfig::default());
        
        // Create test 1inch service
        let oneinch_service = Arc::new(OneinchService::new(
            "test_api_key".to_string(),
            ONEINCH_SEPOLIA_CHAIN_ID,
        ));
        
        let state = AppState {
            config,
            oneinch_service,
            // Other services would be initialized here in real implementation
            ..Default::default()
        };
        
        let app = create_oneinch_routes().with_state(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = create_test_app().await;
        
        let response = server.get("/health").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body = response.json::<serde_json::Value>();
        assert!(body["status"].is_string());
        assert!(body["api_available"].is_boolean());
        assert!(body["supported_chains"].is_array());
    }

    #[tokio::test]
    async fn test_tokens_endpoint() {
        let server = create_test_app().await;
        
        let response = server.get("/tokens").await;
        
        // This might fail with external API, but we're testing the route structure
        assert!(response.status_code() == StatusCode::OK || response.status_code() == StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_quote_endpoint_requires_auth() {
        let server = create_test_app().await;
        
        let quote_request = json!({
            "from_token": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D",
            "to_token": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "amount": "1000000000000000000",
            "slippage": 0.5
        });
        
        let response = server.post("/quote").json(&quote_request).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_enhanced_quote_endpoint_requires_auth() {
        let server = create_test_app().await;
        
        let enhanced_quote_request = json!({
            "from_token": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D",
            "to_token": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "amount": "1000000000000000000",
            "slippages": [0.5, 1.0, 1.5],
            "selection_criteria": "best_efficiency",
            "compare_with_oracle": true
        });
        
        let response = server.post("/quote/enhanced").json(&enhanced_quote_request).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_execute_endpoint_requires_auth() {
        let server = create_test_app().await;
        
        let execute_request = json!({
            "from_token": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D",
            "to_token": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "amount": "1000000000000000000",
            "slippage": 0.5,
            "deadline_minutes": 5
        });
        
        let response = server.post("/execute").json(&execute_request).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_order_status_endpoint_requires_auth() {
        let server = create_test_app().await;
        
        let test_order_hash = "0x1234567890abcdef1234567890abcdef12345678";
        let response = server.get(&format!("/order/{}", test_order_hash)).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_order_hash_format() {
        let server = create_test_app().await;
        
        let invalid_order_hash = "invalid_hash";
        let response = server.get(&format!("/order/{}", invalid_order_hash)).await;
        
        // Should require authentication first, then might validate hash format
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}