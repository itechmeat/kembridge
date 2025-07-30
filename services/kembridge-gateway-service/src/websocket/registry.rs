// WebSocket connection registry - Enhanced from old backend
use super::connection::{WebSocketConnection, ConnectionInfo};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// WebSocket connection registry
#[derive(Clone)]
pub struct WebSocketRegistry {
    connections: Arc<RwLock<HashMap<Uuid, Arc<WebSocketConnection>>>>,
    user_connections: Arc<RwLock<HashMap<String, Vec<Uuid>>>>, // user_id -> connection_ids
}

impl WebSocketRegistry {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add connection to registry
    pub async fn add_connection(&self, connection: Arc<WebSocketConnection>) {
        let connection_id = connection.id();
        
        // Add to main connections map
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection.clone());
        }
        
        // If user is authenticated, add to user connections map
        if let Some(user_id) = connection.user_id().await {
            let mut user_connections = self.user_connections.write().await;
            user_connections
                .entry(user_id.clone())
                .or_insert_with(Vec::new)
                .push(connection_id);
            
            info!("📝 Connection {} added for user {}", connection_id, user_id);
        } else {
            info!("📝 Anonymous connection {} added", connection_id);
        }
    }

    /// Remove connection from registry
    pub async fn remove_connection(&self, id: &Uuid) {
        // Get connection first to check if it has a user_id
        let connection = {
            let connections = self.connections.read().await;
            connections.get(id).cloned()
        };
        
        // Remove from main connections map
        {
            let mut connections = self.connections.write().await;
            connections.remove(id);
        }
        
        // Remove from user connections map if needed
        if let Some(connection) = connection {
            if let Some(user_id) = connection.user_id().await {
                let mut user_connections = self.user_connections.write().await;
                if let Some(user_conn_list) = user_connections.get_mut(&user_id) {
                    user_conn_list.retain(|&conn_id| conn_id != *id);
                    
                    // Remove user entry if no connections left
                    if user_conn_list.is_empty() {
                        user_connections.remove(&user_id);
                    }
                }
                info!("🗑️ Connection {} removed for user {}", id, user_id);
            } else {
                info!("🗑️ Anonymous connection {} removed", id);
            }
        }
    }

    /// Get specific connection
    pub async fn get_connection(&self, id: &Uuid) -> Option<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    /// Get all connections for a specific user
    pub async fn get_user_connections(&self, user_id: &str) -> Vec<Arc<WebSocketConnection>> {
        let user_connections = self.user_connections.read().await;
        let connections = self.connections.read().await;
        
        if let Some(connection_ids) = user_connections.get(user_id) {
            connection_ids
                .iter()
                .filter_map(|id| connections.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all active connections
    pub async fn get_all_connections(&self) -> Vec<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Get all active connections (only active ones)
    pub async fn get_active_connections(&self) -> Vec<Arc<WebSocketConnection>> {
        let connections = self.connections.read().await;
        let mut active_connections = Vec::new();
        
        for connection in connections.values() {
            if connection.is_active().await {
                active_connections.push(connection.clone());
            }
        }
        
        active_connections
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Get active connection count
    pub async fn active_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        let mut count = 0;
        
        for connection in connections.values() {
            if connection.is_active().await {
                count += 1;
            }
        }
        
        count
    }

    /// Get authenticated connection count
    pub async fn authenticated_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        let mut count = 0;
        
        for connection in connections.values() {
            if connection.is_active().await && connection.user_id().await.is_some() {
                count += 1;
            }
        }
        
        count
    }

    /// Get unique user count
    pub async fn unique_user_count(&self) -> usize {
        let user_connections = self.user_connections.read().await;
        user_connections.len()
    }

    /// Update user ID for a connection (for authentication after connection)
    pub async fn authenticate_connection(&self, connection_id: &Uuid, user_id: String) -> Result<(), String> {
        let connection = {
            let connections = self.connections.read().await;
            connections.get(connection_id).cloned()
        };
        
        if let Some(connection) = connection {
            // Set user ID on connection
            connection.set_user_id(user_id.clone()).await;
            
            // Add to user connections map
            {
                let mut user_connections = self.user_connections.write().await;
                user_connections
                    .entry(user_id.clone())
                    .or_insert_with(Vec::new)
                    .push(*connection_id);
            }
            
            info!("🔐 Connection {} authenticated as user {}", connection_id, user_id);
            Ok(())
        } else {
            Err(format!("Connection {} not found", connection_id))
        }
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self) -> usize {
        let mut to_remove = Vec::new();
        
        // Find inactive connections
        {
            let connections = self.connections.read().await;
            for (id, connection) in connections.iter() {
                if !connection.is_active().await {
                    to_remove.push(*id);
                }
            }
        }
        
        // Remove inactive connections
        for id in &to_remove {
            self.remove_connection(id).await;
        }
        
        if !to_remove.is_empty() {
            info!("🧹 Cleaned up {} inactive connections", to_remove.len());
        }
        
        to_remove.len()
    }

    /// Clean up idle connections
    pub async fn cleanup_idle_connections(&self, idle_duration: chrono::Duration) -> usize {
        let mut to_remove = Vec::new();
        
        // Find idle connections
        {
            let connections = self.connections.read().await;
            for (id, connection) in connections.iter() {
                if connection.is_active().await && connection.is_idle(idle_duration).await {
                    to_remove.push(*id);
                }
            }
        }
        
        // Close and remove idle connections
        for id in &to_remove {
            if let Some(connection) = self.get_connection(id).await {
                connection.close().await;
            }
            self.remove_connection(id).await;
        }
        
        if !to_remove.is_empty() {
            info!("⏰ Cleaned up {} idle connections", to_remove.len());
        }
        
        to_remove.len()
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let total_connections = self.connection_count().await;
        let active_connections = self.active_connection_count().await;
        let authenticated_connections = self.authenticated_connection_count().await;
        let unique_users = self.unique_user_count().await;

        RegistryStats {
            total_connections,
            active_connections,
            authenticated_connections,
            unique_users,
        }
    }

    /// Get detailed connection information for debugging
    pub async fn get_connection_details(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        let mut details = Vec::new();
        
        for connection in connections.values() {
            details.push(connection.get_info().await);
        }
        
        details
    }

    /// Disconnect all connections for a user
    pub async fn disconnect_user(&self, user_id: &str) -> usize {
        let user_connections = self.get_user_connections(user_id).await;
        let count = user_connections.len();
        
        for connection in user_connections {
            connection.close().await;
            self.remove_connection(&connection.id()).await;
        }
        
        if count > 0 {
            warn!("🚫 Disconnected {} connections for user {}", count, user_id);
        }
        
        count
    }

    /// Send message to all connections of a user
    pub async fn send_to_user(&self, user_id: &str, message: super::message::WebSocketMessage) -> Result<usize, String> {
        let connections = self.get_user_connections(user_id).await;
        let mut success_count = 0;
        
        let connection_count = connections.len();
        for connection in connections {
            if connection.is_active().await {
                match connection.send_message(message.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("❌ Failed to send message to user {} connection {}: {}", 
                              user_id, connection.id(), e);
                    }
                }
            }
        }
        
        debug!("📤 Sent message to {}/{} connections for user {}", success_count, connection_count, user_id);
        Ok(success_count)
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub authenticated_connections: usize,
    pub unique_users: usize,
}

impl Default for WebSocketRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Include maintenance-specific tests
#[path = "tests_maintenance.rs"]
#[cfg(test)]
mod tests_maintenance;