use axum::{
    routing::{get, post},
    Router,
    Json,
    middleware,
};
use kembridge_gateway_service::{
    config::ServiceConfig, 
    handlers,
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    websocket::{ws_handler, create_websocket_services, start_maintenance_tasks, broadcast_system_startup, ConnectionManager},
    event_listener::create_event_listener,
    event_api::{EventApiState, trigger_crypto_event, trigger_risk_analysis, trigger_system_notification, 
                get_websocket_stats, cleanup_connections, send_heartbeat, disconnect_user, test_user_broadcast},
    middleware::security_headers
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

    // Initialize enhanced WebSocket services
    let (registry, broadcaster) = create_websocket_services();
    let connections: ConnectionManager = registry.clone();
    
    // Start WebSocket maintenance tasks (heartbeat, cleanup)
    start_maintenance_tasks(broadcaster.clone()).await;
    info!("🌐 WebSocket services initialized with maintenance tasks");
    
    // Broadcast system startup notification
    if let Ok(notified_count) = broadcast_system_startup(&broadcaster).await {
        info!("📢 System startup notification sent to {} connections", notified_count);
    }

    // Initialize and start EventListener for microservices integration
    let event_listener = create_event_listener(broadcaster.clone());
    let event_listener_clone = event_listener.clone();
    
    tokio::spawn(async move {
        event_listener_clone.start().await;
    });
    info!("🎧 EventListener started for microservices integration");

    // Prepare state for event API
    let event_api_state: EventApiState = (broadcaster.clone(), Arc::new(event_listener));

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
        // Crypto routes (matching old backend)
        .route("/api/v1/crypto/status", get(handlers::get_crypto_status))  
        .route("/api/v1/crypto/keys/check-rotation", get(handlers::check_key_rotation))
        .route("/api/v1/crypto/keys/rotate", post(handlers::trigger_key_rotation))
        // WebSocket route
        .route("/ws", get(ws_handler))
        .with_state((circuit_breaker, connections))
        // Event API routes need separate state
        .nest("/api/v1/events", Router::new()
            .route("/crypto/trigger", post(trigger_crypto_event))
            .route("/risk/trigger", post(trigger_risk_analysis))
            .route("/system/notification/{level}", post(trigger_system_notification))
            .route("/websocket/stats", get(get_websocket_stats))
            .route("/websocket/cleanup", post(cleanup_connections))
            .route("/websocket/heartbeat", post(send_heartbeat))
            .route("/websocket/disconnect/{user_id}", post(disconnect_user))
            .route("/websocket/test/{user_id}", post(test_user_broadcast))
            .with_state(event_api_state)
        )
        .layer(middleware::from_fn(security_headers))
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
