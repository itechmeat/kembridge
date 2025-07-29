use axum::{
    routing::get,
    Router,
    Json,
};
use kembridge_auth_service::{config::ServiceConfig, handlers};
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

    info!("🚀 Starting KEMBridge Auth Service (Minimal)...");

    // Load configuration
    let config = Arc::new(ServiceConfig::new()?);
    let port = config.port;

    // Create router with minimal routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", get(handlers::simple_login))
        .route("/auth/validate", get(handlers::simple_validate))
        .layer(CorsLayer::permissive());

    // Start server
    info!("🔐 Auth Service listening on port {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ServiceResponse<serde_json::Value>> {
    Json(ServiceResponse::success(serde_json::json!({
        "service": "kembridge-auth-service",
        "status": "healthy",
        "supported_wallets": ["MetaMask", "NEAR Wallet", "WalletConnect"]
    })))
}