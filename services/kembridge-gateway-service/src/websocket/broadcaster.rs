// WebSocket event broadcasting - Enhanced from old backend
use super::{
    registry::WebSocketRegistry,
    message::{RealTimeEvent, WebSocketMessage}
};
use std::sync::Arc;
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
        let connections = self.registry.get_active_connections().await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        info!("📢 Broadcasting {:?} event to all connections", event_type);
        
        for connection in connections {
            match connection.send_event(event.clone()).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    warn!("❌ Failed to send event to connection {}: {}", connection.id(), e);
                }
            }
        }
        
        info!("✅ Broadcasted {:?} event to {} connections", event_type, success_count);
        Ok(success_count)
    }

    /// Broadcast event to specific user
    pub async fn broadcast_to_user(&self, user_id: &str, event: RealTimeEvent) -> Result<usize, String> {
        let connections = self.registry.get_user_connections(user_id).await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        debug!("📤 Broadcasting {:?} event to user {}", event_type, user_id);
        
        for connection in connections {
            if connection.is_active().await {
                match connection.send_event(event.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("❌ Failed to send event to user {} connection {}: {}", 
                              user_id, connection.id(), e);
                    }
                }
            }
        }
        
        debug!("✅ Broadcasted {:?} event to {} connections for user {}", 
               event_type, success_count, user_id);
        Ok(success_count)
    }

    /// Broadcast event to users with specific subscription
    pub async fn broadcast_to_subscribers(&self, event: RealTimeEvent) -> Result<usize, String> {
        let connections = self.registry.get_active_connections().await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        debug!("🎯 Broadcasting {:?} event to subscribers", event_type);
        
        for connection in connections {
            if connection.is_subscribed(&event_type).await {
                match connection.send_event(event.clone()).await {
                    Ok(()) => success_count += 1,
                    Err(e) => {
                        warn!("❌ Failed to send event to connection {}: {}", connection.id(), e);
                    }
                }
            }
        }
        
        debug!("✅ Broadcasted {:?} event to {} subscribed connections", event_type, success_count);
        Ok(success_count)
    }

    /// Broadcast urgent events (automatically to all active connections)
    pub async fn broadcast_urgent(&self, event: RealTimeEvent) -> Result<usize, String> {
        if !event.is_urgent() {
            return Err("Event is not marked as urgent".to_string());
        }

        let connections = self.registry.get_active_connections().await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        warn!("🚨 Broadcasting URGENT {:?} event to all active connections", event_type);
        
        for connection in connections {
            // For urgent events, ignore subscription status
            match connection.send_message(WebSocketMessage::event(event.clone())).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error!("❌ Failed to send urgent event to connection {}: {}", connection.id(), e);
                }
            }
        }
        
        warn!("🚨 Broadcasted URGENT {:?} event to {} connections", event_type, success_count);
        Ok(success_count)
    }

    /// Send message to specific connection
    pub async fn send_to_connection(&self, connection_id: uuid::Uuid, message: WebSocketMessage) -> Result<(), String> {
        if let Some(connection) = self.registry.get_connection(&connection_id).await {
            connection.send_message(message).await
        } else {
            Err(format!("Connection {} not found", connection_id))
        }
    }

    /// Send message to all connections of a user
    pub async fn send_to_user(&self, user_id: &str, message: WebSocketMessage) -> Result<usize, String> {
        self.registry.send_to_user(user_id, message).await
    }

    /// Broadcast system notification to all users
    pub async fn broadcast_system_notification(
        &self, 
        level: super::message::NotificationLevel,
        title: &str,
        message: &str,
        action_required: bool
    ) -> Result<usize, String> {
        let event = RealTimeEvent::SystemNotification(super::message::SystemNotificationEvent {
            notification_id: uuid::Uuid::new_v4().to_string(),
            user_id: None, // Global notification
            level: level.clone(),
            title: title.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
            action_required,
            expires_at: None,
        });

        // Use urgent broadcast for critical notifications
        if matches!(level, super::message::NotificationLevel::Critical | super::message::NotificationLevel::Error) {
            self.broadcast_urgent(event).await
        } else {
            self.broadcast_to_all(event).await
        }
    }

    /// Broadcast crypto service event
    pub async fn broadcast_crypto_event(
        &self,
        event_type: super::message::CryptoEventType,
        service: &str,
        status: &str,
        message: &str,
        metadata: Option<serde_json::Value>
    ) -> Result<usize, String> {
        let event = RealTimeEvent::CryptoServiceEvent(super::message::CryptoServiceEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            service_name: service.to_string(),
            status: status.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
        });

        self.broadcast_to_subscribers(event).await
    }

    /// Get broadcaster statistics
    pub async fn get_stats(&self) -> BroadcasterStats {
        let registry_stats = self.registry.get_stats().await;
        
        BroadcasterStats {
            total_connections: registry_stats.total_connections,
            active_connections: registry_stats.active_connections,
            authenticated_connections: registry_stats.authenticated_connections,
            unique_users: registry_stats.unique_users,
        }
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self) -> Result<usize, String> {
        let cleaned_count = self.registry.cleanup_inactive_connections().await;
        Ok(cleaned_count)
    }

    /// Clean up idle connections
    pub async fn cleanup_idle_connections(&self, idle_minutes: u64) -> Result<usize, String> {
        let idle_duration = chrono::Duration::minutes(idle_minutes as i64);
        let cleaned_count = self.registry.cleanup_idle_connections(idle_duration).await;
        Ok(cleaned_count)
    }

    /// Send heartbeat to all connections
    pub async fn send_heartbeat(&self) -> Result<usize, String> {
        let connections = self.registry.get_active_connections().await;
        let mut success_count = 0;
        
        debug!("💓 Sending heartbeat to {} active connections", connections.len());
        
        for connection in connections {
            match connection.send_message(WebSocketMessage::Ping).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    warn!("❌ Failed to send heartbeat to connection {}: {}", connection.id(), e);
                }
            }
        }
        
        debug!("💓 Sent heartbeat to {} connections", success_count);
        Ok(success_count)
    }

    /// Disconnect all connections for a user (admin action)
    pub async fn disconnect_user(&self, user_id: &str, reason: &str) -> Result<usize, String> {
        // Send close message before disconnecting
        let close_message = WebSocketMessage::close(format!("Disconnected by admin: {}", reason));
        let _ = self.send_to_user(user_id, close_message).await;
        
        // Give a moment for the message to be sent
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Disconnect all user connections
        let disconnected_count = self.registry.disconnect_user(user_id).await;
        
        Ok(disconnected_count)
    }

    /// Get detailed connection information for debugging
    pub async fn get_connection_details(&self) -> Vec<super::connection::ConnectionInfo> {
        self.registry.get_connection_details().await
    }

    /// Broadcast event with filtering by subscription and user targeting
    pub async fn broadcast_filtered(&self, event: RealTimeEvent, target_users: Option<Vec<String>>) -> Result<usize, String> {
        let connections = self.registry.get_active_connections().await;
        let mut success_count = 0;
        let event_type = event.event_type();
        
        debug!("🎯 Broadcasting filtered {:?} event", event_type);
        
        for connection in connections {
            // Check if user is in target list (if specified)
            if let Some(ref target_users) = target_users {
                if let Some(user_id) = connection.user_id().await {
                    if !target_users.contains(&user_id) {
                        continue; // Skip this connection
                    }
                } else {
                    continue; // Skip unauthenticated connections when targeting specific users
                }
            }
            
            // Use the connection's send_event method which handles subscription filtering
            match connection.send_event(event.clone()).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    warn!("❌ Failed to send filtered event to connection {}: {}", connection.id(), e);
                }
            }
        }
        
        debug!("✅ Broadcasted filtered {:?} event to {} connections", event_type, success_count);
        Ok(success_count)
    }
}

/// Broadcaster statistics
#[derive(Debug, Clone)]
pub struct BroadcasterStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub authenticated_connections: usize,
    pub unique_users: usize,
}

/// Background task for connection maintenance
pub struct ConnectionMaintenance {
    broadcaster: WebSocketBroadcaster,
    cleanup_interval: std::time::Duration,
    heartbeat_interval: std::time::Duration,
    idle_timeout_minutes: u64,
}

impl ConnectionMaintenance {
    /// Create new maintenance task
    pub fn new(broadcaster: WebSocketBroadcaster) -> Self {
        Self {
            broadcaster,
            cleanup_interval: std::time::Duration::from_secs(300), // 5 minutes
            heartbeat_interval: std::time::Duration::from_secs(30), // 30 seconds
            idle_timeout_minutes: 30, // 30 minutes
        }
    }

    /// Create with custom intervals
    pub fn with_intervals(
        broadcaster: WebSocketBroadcaster,
        cleanup_interval: std::time::Duration,
        heartbeat_interval: std::time::Duration,
        idle_timeout_minutes: u64,
    ) -> Self {
        Self {
            broadcaster,
            cleanup_interval,
            heartbeat_interval,
            idle_timeout_minutes,
        }
    }

    /// Start maintenance tasks
    pub async fn start(&self) {
        let broadcaster_cleanup = self.broadcaster.clone();
        let cleanup_interval = self.cleanup_interval;
        let idle_timeout = self.idle_timeout_minutes;
        
        let broadcaster_heartbeat = self.broadcaster.clone();
        let heartbeat_interval = self.heartbeat_interval;
        
        // Spawn cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                
                // Clean up inactive connections
                if let Err(e) = broadcaster_cleanup.cleanup_inactive_connections().await {
                    error!("❌ Failed to cleanup inactive connections: {}", e);
                }
                
                // Clean up idle connections
                if let Err(e) = broadcaster_cleanup.cleanup_idle_connections(idle_timeout).await {
                    error!("❌ Failed to cleanup idle connections: {}", e);
                }
                
                // Log connection stats
                let stats = broadcaster_cleanup.get_stats().await;
                info!("📊 Connection stats: {} total, {} active, {} authenticated, {} unique users", 
                      stats.total_connections, stats.active_connections, 
                      stats.authenticated_connections, stats.unique_users);
            }
        });
        
        // Spawn heartbeat task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            loop {
                interval.tick().await;
                if let Err(e) = broadcaster_heartbeat.send_heartbeat().await {
                    error!("❌ Failed to send heartbeat: {}", e);
                }
            }
        });
        
        info!("🔧 WebSocket connection maintenance tasks started (cleanup: {:?}, heartbeat: {:?})", 
              cleanup_interval, heartbeat_interval);
    }
}

// Include broadcaster-specific tests
#[path = "tests_broadcaster.rs"]
#[cfg(test)]
mod tests_broadcaster;