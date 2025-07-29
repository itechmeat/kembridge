use axum::{
    routing::get,
    Router,
    Json,
};
use kembridge_blockchain_service::{config::ServiceConfig, handlers};
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

    info!("🚀 Starting KEMBridge Blockchain Service (Minimal)...");

    // Load configuration
    let config = Arc::new(ServiceConfig::new()?);
    let port = config.port;

    // Create router with minimal routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ethereum/balance", get(handlers::simple_eth_balance))
        .route("/near/account", get(handlers::simple_near_account))
        .layer(CorsLayer::permissive());

    // Start server
    info!("🌐 Blockchain Service listening on port {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ServiceResponse<serde_json::Value>> {
    Json(ServiceResponse::success(serde_json::json!({
        "service": "kembridge-blockchain-service",
        "status": "healthy",
        "supported_chains": ["ethereum", "near"]
    })))
}