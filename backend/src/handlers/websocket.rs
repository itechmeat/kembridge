// src/handlers/websocket.rs - WebSocket handler (Phase 5.3 - Real-time Monitoring)
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    http::StatusCode,
};
use serde_json::{json, Value};
use crate::AppState;

/// WebSocket connection upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    crate::websocket::handler::websocket_upgrade_handler(ws, State(state.websocket_registry)).await
}

/// WebSocket status handler (for monitoring)
pub async fn websocket_status_handler(
    State(state): State<AppState>,
) -> Result<axum::Json<Value>, StatusCode> {
    crate::websocket::handler::websocket_status_handler(State(state.websocket_registry)).await
}