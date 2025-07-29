use axum::{
    routing::get,
    Router,
    Json,
};
use kembridge_crypto_service::{config::ServiceConfig, handlers};
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

    info!("🚀 Starting KEMBridge Crypto Service (Minimal)...");

    // Load configuration
    let config = Arc::new(ServiceConfig::new()?);
    let port = config.port;

    // Create router with minimal routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/keys/generate", get(handlers::simple_generate_key))
        .route("/encrypt", get(handlers::simple_encrypt))
        .layer(CorsLayer::permissive());

    // Start server
    info!("🔐 Crypto Service listening on port {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ServiceResponse<serde_json::Value>> {
    Json(ServiceResponse::success(serde_json::json!({
        "service": "crypto-service",
        "status": "healthy",
        "supported_algorithms": ["ML-KEM-1024", "Dilithium-5", "AES-GCM"]
    })))
}