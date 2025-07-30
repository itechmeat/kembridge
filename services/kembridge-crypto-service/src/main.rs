use axum::{
    routing::{get, post},
    Router,
};
use kembridge_crypto_service::{config::ServiceConfig, handlers, QuantumService};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, error, Level};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting KEMBridge Crypto Service (Full Quantum Implementation)...");

    // Load configuration
    let config = Arc::new(ServiceConfig::new()?);
    let port = config.port;

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:dev_password@postgres:5432/kembridge_dev".to_string());
    
    info!("📊 Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .map_err(|e| {
            error!("❌ Failed to connect to database: {}", e);
            e
        })?;

    // Run database migrations
    info!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .map_err(|e| {
            error!("❌ Failed to run migrations: {}", e);
            e
        })?;

    // Initialize QuantumService
    info!("🔐 Initializing Quantum Service...");
    let quantum_service = Arc::new(QuantumService::new(db_pool).await?);

    // Create router with full quantum cryptography endpoints
    let app = Router::new()
        // Health and status
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::get_crypto_status))
        
        // Key management
        .route("/keys/generate", post(handlers::generate_keypair))
        .route("/keys/user/{user_id}/generate", post(handlers::generate_user_keypair))
        .route("/keys", get(handlers::list_keys))
        .route("/keys/user/{user_id}", get(handlers::list_user_keys))
        .route("/keys/{key_id}/public", get(handlers::export_public_key))
        
        // ML-KEM operations
        .route("/encapsulate", post(handlers::encapsulate))
        .route("/decapsulate", post(handlers::decapsulate))
        
        // Key rotation
        .route("/keys/check-rotation", post(handlers::check_rotation))
        .route("/keys/rotate", post(handlers::rotate_key))
        
        // Legacy endpoints for backward compatibility
        .route("/keys/generate-legacy", get(handlers::simple_generate_key))
        .route("/encrypt-legacy", get(handlers::simple_encrypt))
        
        .with_state(quantum_service)
        .layer(CorsLayer::permissive());

    // Start server
    info!("🔐 Quantum Crypto Service listening on port {} with full ML-KEM-1024 support", port);
    info!("📋 Available endpoints:");
    info!("   GET  /health - Health check");
    info!("   GET  /status - Quantum system status");
    info!("   POST /keys/generate - Generate ML-KEM-1024 keypair");
    info!("   POST /encapsulate - ML-KEM encapsulation");
    info!("   POST /decapsulate - ML-KEM decapsulation");
    info!("   POST /keys/rotate - Key rotation");
    
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}