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
    middleware::{security_headers, error_handling_middleware}
};
use kembridge_common::ServiceResponse;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use chrono;
use std::time::{SystemTime, UNIX_EPOCH};

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
        .route("/api/v1/bridge/quote", get(handlers::get_bridge_quote))
        .route("/api/v1/bridge/history", get(handlers::get_bridge_history))
        // Crypto routes (matching old backend)
        .route("/api/v1/crypto/status", get(handlers::get_crypto_status))  
        .route("/api/v1/crypto/keys/check-rotation", get(handlers::check_key_rotation))
        .route("/api/v1/crypto/keys/rotate", post(handlers::trigger_key_rotation))
        // Error handling test endpoint
        .route("/api/v1/test/error-handling", get(handlers::test_error_handling))
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
        .layer(middleware::from_fn(error_handling_middleware))
        .layer(middleware::from_fn(security_headers))
        .layer(CorsLayer::permissive());

    // Start server
    info!("🌐 Gateway Service listening on port {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<ServiceResponse<serde_json::Value>> {
    // Get performance metrics
    let uptime_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Get memory usage if available
    let memory_info = get_memory_info();
    
    // Get process metrics
    let process_info = get_process_info();
    
    // Health check returns ServiceResponse with performance metrics
    let health_data = serde_json::json!({
        "status": "healthy",
        "service": "kembridge-gateway-service", 
        "upstream_services": ["1inch-service", "blockchain-service", "crypto-service", "auth-service"],
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "performance": {
            "uptime_seconds": uptime_seconds,
            "memory": memory_info,
            "process": process_info,
            "api_response_time_target": "< 500ms",
            "websocket_connections": 0 // TODO: Get actual WebSocket connection count
        }
    });
    
    Json(ServiceResponse::success(health_data))
}

fn get_memory_info() -> serde_json::Value {
    // Basic memory info - in production would use proper system metrics
    serde_json::json!({
        "status": "monitoring_available",
        "note": "Use Prometheus/Grafana for detailed metrics"
    })
}

fn get_process_info() -> serde_json::Value {
    // Basic process info
    serde_json::json!({
        "pid": std::process::id(),
        "rust_version": "1.88.0",
        "build_mode": if cfg!(debug_assertions) { "debug" } else { "release" }
    })
}
