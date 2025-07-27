// src/websocket/handler.rs - WebSocket HTTP handler
use super::{WebSocketRegistry, WebSocketConnection, WebSocketHandler};
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    http::StatusCode,
};
use std::sync::Arc;
use tracing::{info, warn};

/// WebSocket upgrade handler
pub async fn websocket_upgrade_handler(
    ws: WebSocketUpgrade,
    State(registry): State<Arc<WebSocketRegistry>>,
) -> Response {
    info!("WebSocket upgrade request received");
    
    ws.on_upgrade(move |socket| async move {
        // Create new connection
        let connection = Arc::new(WebSocketConnection::new());
        
        // Create handler
        let handler = WebSocketHandler::new(connection, registry);
        
        // Handle connection
        handler.handle_connection(socket).await;
    })
}

/// WebSocket status handler (for monitoring)
pub async fn websocket_status_handler(
    State(registry): State<Arc<WebSocketRegistry>>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let total_connections = registry.connection_count().await;
    let connections = registry.get_all_connections().await;
    
    let mut active_connections = 0;
    let mut authenticated_connections = 0;
    
    for connection in connections {
        if connection.is_active().await {
            active_connections += 1;
            if connection.user_id().is_some() {
                authenticated_connections += 1;
            }
        }
    }
    
    Ok(axum::Json(serde_json::json!({
        "status": "operational",
        "connections": {
            "total": total_connections,
            "active": active_connections,
            "authenticated": authenticated_connections
        },
        "features": {
            "real_time_events": true,
            "authentication": true,
            "subscriptions": true,
            "heartbeat": true
        }
    })))
}