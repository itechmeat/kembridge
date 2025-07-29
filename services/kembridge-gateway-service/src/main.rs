use axum::{
    routing::{get, post},
    Router,
    Json,
};
use kembridge_gateway_service::{
    config::ServiceConfig, 
    handlers,
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig}
};
use kembridge_common::ServiceResponse;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting KEMBridge Gateway Service with Circuit Breaker...");

    // Load configuration
    let config = Arc::new(ServiceConfig::new()?);
    let port = config.port;

    // Initialize circuit breaker
    let circuit_breaker_config = CircuitBreakerConfig::default();
    let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_breaker_config));
    info!("🛡️ Circuit breaker initialized with 5 failure threshold and 30s timeout");

    // Create router with gateway routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/gateway/proxy", get(handlers::simple_proxy))
        .route("/gateway/services", get(handlers::services_status))
        .route("/gateway/circuit-breaker", get(handlers::circuit_breaker_status))
        // API routes
        .route("/api/v1/auth/nonce", get(handlers::get_nonce))
        .route("/api/v1/auth/verify-wallet", post(handlers::verify_wallet))
        // Bridge routes
        .route("/api/v1/bridge/tokens", get(handlers::get_bridge_tokens))
        .route("/api/v1/bridge/history", get(handlers::get_bridge_history))
        .with_state(circuit_breaker)
        .layer(CorsLayer::permissive());

    // Start server
    info!("🌐 Gateway Service listening on port {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ServiceResponse<serde_json::Value>> {
    Json(ServiceResponse::success(serde_json::json!({
        "service": "kembridge-gateway-service",
        "status": "healthy",
        "upstream_services": ["1inch-service", "blockchain-service", "crypto-service", "auth-service"]
    })))
}