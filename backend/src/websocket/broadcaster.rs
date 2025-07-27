// src/websocket/broadcaster.rs - WebSocket event broadcasting
use super::{WebSocketRegistry, message::{RealTimeEvent, WebSocketMessage}};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// WebSocket event broadcaster
#[derive(Clone)]
pub struct WebSocketBroadcaster {
    registry: Arc<WebSocketRegistry>,
}

impl WebSocketBroadcaster {
    /// Create new broadcaster
    pub fn new(registry: Arc<WebSocketRegistry>) -> Self {
        Self { registry }
    }

    /// Broadcast event to all connected clients
    pub async fn broadcast_to_all(&self, event: RealTimeEvent) -> Result<usize, String> {
        let connections = self.registry.get_all_connections().await;
        let mut success_count = 0;
        
        for connection in connections {
            if connection.is_active().await {
                match connection.send_event(event.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("Failed to send event to connection {}: {}", connection.id(), e);
                    }
                }
            }
        }
        
        info!("Broadcasted event to {} connections", success_count);
        Ok(success_count)
    }

    /// Broadcast event to specific user
    pub async fn broadcast_to_user(&self, user_id: Uuid, event: RealTimeEvent) -> Result<usize, String> {
        let connections = self.registry.get_user_connections(&user_id).await;
        let mut success_count = 0;
        
        for connection in connections {
            if connection.is_active().await {
                match connection.send_event(event.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("Failed to send event to user {} connection {}: {}", 
                              user_id, connection.id(), e);
                    }
                }
            }
        }
        
        debug!("Broadcasted event to {} connections for user {}", success_count, user_id);
        Ok(success_count)
    }

    /// Broadcast event to users with specific subscription
    pub async fn broadcast_to_subscribers(&self, event: RealTimeEvent) -> Result<usize, String> {
        let connections = self.registry.get_all_connections().await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        for connection in connections {
            if connection.is_active().await && connection.is_subscribed(&event_type).await {
                match connection.send_event(event.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("Failed to send event to connection {}: {}", connection.id(), e);
                    }
                }
            }
        }
        
        debug!("Broadcasted event to {} subscribed connections", success_count);
        Ok(success_count)
    }

    /// Send message to specific connection
    pub async fn send_to_connection(&self, connection_id: Uuid, message: WebSocketMessage) -> Result<(), String> {
        if let Some(connection) = self.registry.get_connection(&connection_id).await {
            connection.send_message(message).await
        } else {
            Err(format!("Connection {} not found", connection_id))
        }
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> BroadcasterStats {
        let total_connections = self.registry.connection_count().await;
        let connections = self.registry.get_all_connections().await;
        
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
        
        BroadcasterStats {
            total_connections,
            active_connections,
            authenticated_connections,
        }
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self) -> Result<usize, String> {
        let connections = self.registry.get_all_connections().await;
        let mut cleaned_count = 0;
        
        for connection in connections {
            if !connection.is_active().await {
                self.registry.remove_connection(&connection.id()).await;
                cleaned_count += 1;
            }
        }
        
        if cleaned_count > 0 {
            info!("Cleaned up {} inactive connections", cleaned_count);
        }
        
        Ok(cleaned_count)
    }

    /// Send heartbeat to all connections
    pub async fn send_heartbeat(&self) -> Result<usize, String> {
        let connections = self.registry.get_all_connections().await;
        let mut success_count = 0;
        
        for connection in connections {
            if connection.is_active().await {
                match connection.send_message(WebSocketMessage::Ping).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("Failed to send heartbeat to connection {}: {}", connection.id(), e);
                    }
                }
            }
        }
        
        debug!("Sent heartbeat to {} connections", success_count);
        Ok(success_count)
    }
}

/// Broadcaster statistics
#[derive(Debug, Clone)]
pub struct BroadcasterStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub authenticated_connections: usize,
}

/// Background task for connection maintenance
pub struct ConnectionMaintenance {
    broadcaster: WebSocketBroadcaster,
    cleanup_interval: std::time::Duration,
    heartbeat_interval: std::time::Duration,
}

impl ConnectionMaintenance {
    /// Create new maintenance task
    pub fn new(broadcaster: WebSocketBroadcaster) -> Self {
        Self {
            broadcaster,
            cleanup_interval: std::time::Duration::from_secs(300), // 5 minutes
            heartbeat_interval: std::time::Duration::from_secs(30), // 30 seconds
        }
    }

    /// Start maintenance tasks
    pub async fn start(&self) {
        let broadcaster_cleanup = self.broadcaster.clone();
        let cleanup_interval = self.cleanup_interval;
        
        let broadcaster_heartbeat = self.broadcaster.clone();
        let heartbeat_interval = self.heartbeat_interval;
        
        // Spawn cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = broadcaster_cleanup.cleanup_inactive_connections().await {
                    error!("Failed to cleanup inactive connections: {}", e);
                }
            }
        });
        
        // Spawn heartbeat task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            loop {
                interval.tick().await;
                if let Err(e) = broadcaster_heartbeat.send_heartbeat().await {
                    error!("Failed to send heartbeat: {}", e);
                }
            }
        });
        
        info!("WebSocket connection maintenance tasks started");
    }
}