// src/routes/websocket.rs - WebSocket routes (Phase 5.3.1)
use axum::{routing::get, Router};
use crate::{handlers::websocket, state::AppState};

/// WebSocket routes
pub fn websocket_routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(websocket::websocket_handler))
        .route("/ws/status", get(websocket::websocket_status_handler))
}