// src/routes/bridge_oneinch.rs - Routes for 1inch Bridge Integration

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::bridge_oneinch::*,
    state::AppState,
};

/// Create bridge-1inch integration routes
pub fn create_bridge_oneinch_routes() -> Router<AppState> {
    Router::new()
        // Optimized bridge swap endpoints
        .route("/swap/optimized", post(execute_optimized_bridge_swap))
        .route("/swap/calculate-savings", post(calculate_bridge_swap_savings))
        
        // Bridge swap status endpoints
        .route("/swap/:swap_id/status", get(get_bridge_swap_status))
        
        // Information endpoints
        .route("/supported-chains", get(get_supported_bridge_chains))
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
        oneinch::OneinchService,
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
            // Bridge service would be added here in production
            ..Default::default()
        };
        
        let app = create_bridge_oneinch_routes().with_state(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_supported_chains_endpoint() {
        let server = create_test_app().await;
        
        let response = server.get("/supported-chains").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let chains = response.json::<Vec<String>>();
        assert!(chains.contains(&"ethereum".to_string()));
        assert!(chains.contains(&"near".to_string()));
    }

    #[tokio::test]
    async fn test_calculate_savings_requires_auth() {
        let server = create_test_app().await;
        
        let savings_request = json!({
            "from_chain": "ethereum",
            "to_chain": "near",
            "from_token": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D",
            "to_token": "wrap.near",
            "amount": "1.0",
            "recipient": "user.near"
        });
        
        let response = server.post("/swap/calculate-savings").json(&savings_request).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_optimized_swap_requires_auth() {
        let server = create_test_app().await;
        
        let swap_request = json!({
            "from_chain": "ethereum",
            "to_chain": "near",
            "from_token": "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D",
            "to_token": "wrap.near",
            "amount": "1.0",
            "recipient": "user.near",
            "optimization_strategy": "balanced"
        });
        
        let response = server.post("/swap/optimized").json(&swap_request).await;
        
        // Should require authentication
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_swap_id_format() {
        let server = create_test_app().await;
        
        let response = server.get("/swap/invalid_uuid/status").await;
        
        // Should require authentication first, then validate UUID
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}