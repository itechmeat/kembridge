// src/websocket/mod.rs - WebSocket module entry point
pub mod connection;
pub mod handler;
pub mod broadcaster;
pub mod auth;
pub mod message;

pub use connection::*;
pub use broadcaster::*;
pub use message::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;

/// WebSocket connection registry
#[derive(Clone)]
pub struct WebSocketRegistry {
    connections: Arc<RwLock<HashMap<Uuid, Arc<WebSocketConnection>>>>,
}

impl WebSocketRegistry {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, connection: Arc<WebSocketConnection>) {
        let mut connections = self.connections.write().await;
        connections.insert(connection.id(), connection);
    }

    pub async fn remove_connection(&self, id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
    }

    pub async fn get_connection(&self, id: &Uuid) -> Option<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    pub async fn get_user_connections(&self, user_id: &Uuid) -> Vec<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|conn| conn.user_id() == Some(*user_id))
            .cloned()
            .collect()
    }

    pub async fn get_all_connections(&self) -> Vec<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

impl Default for WebSocketRegistry {
    fn default() -> Self {
        Self::new()
    }
}