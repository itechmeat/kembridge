// WebSocket module - Enhanced architecture from old backend
pub mod message;
pub mod connection;
pub mod registry;
pub mod broadcaster;

// Re-export main types for convenience
pub use message::{
    WebSocketMessage, EventType, RealTimeEvent,
    TransactionStatusEvent, RiskAlertEvent, PriceUpdateEvent,
    SystemNotificationEvent, BridgeOperationEvent, QuantumKeyEvent,
    UserProfileEvent, CryptoServiceEvent, CryptoEventType,
    TransactionStatus, RiskLevel, NotificationLevel
};
pub use connection::{WebSocketConnection, ConnectionInfo, WebSocketHandler};
pub use registry::{WebSocketRegistry, RegistryStats};
pub use broadcaster::{WebSocketBroadcaster, BroadcasterStats, ConnectionMaintenance};

// Legacy exports for backward compatibility with existing code
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Query, State},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WSQuery {
    token: Option<String>,
}

/// Connection manager type alias for backward compatibility
pub type ConnectionManager = Arc<WebSocketRegistry>;

/// WebSocket handler for Axum integration - Enhanced version
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WSQuery>,
    State((_, registry)): State<(Arc<crate::circuit_breaker::CircuitBreaker>, ConnectionManager)>,
) -> Response {
    info!("🔌 WebSocket connection request received");
    
    // Create new connection with metadata
    let connection = Arc::new(WebSocketConnection::new());
    
    // Extract token for authentication if provided
    if let Some(token) = params.token {
        if !token.is_empty() {
            info!("🔑 WebSocket: Connection with authentication token");
            // Token will be processed during authentication handshake
        }
    }
    
    let handler = WebSocketHandler::new(connection, registry);
    
    ws.on_upgrade(move |socket| async move {
        handler.handle_connection(socket).await;
    })
}

/// Create a new WebSocket registry and broadcaster
pub fn create_websocket_services() -> (Arc<WebSocketRegistry>, Arc<WebSocketBroadcaster>) {
    let registry = Arc::new(WebSocketRegistry::new());
    let broadcaster = Arc::new(WebSocketBroadcaster::new(registry.clone()));
    
    info!("🔧 WebSocket services created: registry and broadcaster initialized");
    
    (registry, broadcaster)
}

/// Start WebSocket maintenance tasks
pub async fn start_maintenance_tasks(broadcaster: Arc<WebSocketBroadcaster>) {
    let maintenance = ConnectionMaintenance::new((*broadcaster).clone());
    maintenance.start().await;
    
    info!("🔧 WebSocket maintenance tasks started");
}

/// Broadcast system startup notification
pub async fn broadcast_system_startup(broadcaster: &WebSocketBroadcaster) -> Result<usize, String> {
    broadcaster.broadcast_system_notification(
        NotificationLevel::Info,
        "System Started",
        "KEMBridge Gateway WebSocket service is now online",
        false
    ).await
}

/// Broadcast system shutdown notification
pub async fn broadcast_system_shutdown(broadcaster: &WebSocketBroadcaster) -> Result<usize, String> {
    broadcaster.broadcast_system_notification(
        NotificationLevel::Warning,
        "System Shutdown",
        "KEMBridge Gateway WebSocket service is shutting down",
        false
    ).await
}

/// Create WebSocket event from crypto service status
pub fn create_crypto_status_event(
    service: &str,
    status: &str,
    message: &str,
    metadata: Option<serde_json::Value>
) -> RealTimeEvent {
    RealTimeEvent::CryptoServiceEvent(CryptoServiceEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: CryptoEventType::ServiceStatusChange,
        service_name: service.to_string(),
        status: status.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now(),
        metadata,
    })
}

/// Helper function to broadcast crypto service events
pub async fn broadcast_crypto_service_event(
    broadcaster: &WebSocketBroadcaster,
    event_type: CryptoEventType,
    service: &str,
    status: &str,
    message: &str,
    metadata: Option<serde_json::Value>
) -> Result<usize, String> {
    broadcaster.broadcast_crypto_event(
        event_type,
        service,
        status,
        message,
        metadata
    ).await
}

/// Legacy WebSocket handler function for backward compatibility
/// This maintains the same signature as the old implementation
pub async fn handle_socket(
    socket: WebSocket,
    _client_id: String,
    connections: ConnectionManager,
) {
    warn!("⚠️ Using legacy handle_socket function - consider upgrading to new WebSocketHandler");
    
    // Create connection and handler for legacy support
    let connection = Arc::new(WebSocketConnection::new());
    let handler = WebSocketHandler::new(connection, connections);
    
    // Handle the connection
    handler.handle_connection(socket).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::message::*;
    use tokio;
    use std::time::Duration;

    #[tokio::test]
    async fn test_websocket_services_creation() {
        let (registry, broadcaster) = create_websocket_services();
        
        assert_eq!(registry.connection_count().await, 0);
        
        let stats = broadcaster.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_crypto_event_creation() {
        let event = create_crypto_status_event(
            "crypto-service",
            "healthy",
            "Service is operational",
            Some(serde_json::json!({"version": "1.0.0"}))
        );
        
        match event {
            RealTimeEvent::CryptoServiceEvent(crypto_event) => {
                assert_eq!(crypto_event.service_name, "crypto-service");
                assert_eq!(crypto_event.status, "healthy");
                assert_eq!(crypto_event.message, "Service is operational");
                assert!(crypto_event.metadata.is_some());
            }
            _ => panic!("Expected CryptoServiceEvent"),
        }
    }

    #[tokio::test]
    async fn test_registry_basic_operations() {
        let registry = Arc::new(WebSocketRegistry::new());
        let connection = Arc::new(WebSocketConnection::new());
        let connection_id = connection.id();
        
        // Add connection
        registry.add_connection(connection.clone()).await;
        assert_eq!(registry.connection_count().await, 1);
        
        // Get connection
        let retrieved = registry.get_connection(&connection_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), connection_id);
        
        // Remove connection
        registry.remove_connection(&connection_id).await;
        assert_eq!(registry.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_subscription() {
        let connection = WebSocketConnection::new();
        
        // Test subscription
        let was_new = connection.subscribe(EventType::TransactionStatus).await;
        assert!(was_new);
        
        // Test duplicate subscription
        let was_new = connection.subscribe(EventType::TransactionStatus).await;
        assert!(!was_new);
        
        // Test subscription check
        assert!(connection.is_subscribed(&EventType::TransactionStatus).await);
        assert!(!connection.is_subscribed(&EventType::RiskAlerts).await);
        
        // Test unsubscription
        let was_present = connection.unsubscribe(&EventType::TransactionStatus).await;
        assert!(was_present);
        assert!(!connection.is_subscribed(&EventType::TransactionStatus).await);
    }

    #[tokio::test]
    async fn test_broadcaster_stats() {
        let (registry, broadcaster) = create_websocket_services();
        
        // Test initial stats
        let stats = broadcaster.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.authenticated_connections, 0);
        assert_eq!(stats.unique_users, 0);
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = WebSocketMessage::auth_success("test_user_123".to_string());
        
        // Test serialization
        let json = message.to_json();
        assert!(json.is_ok());
        
        // Test deserialization
        let deserialized = WebSocketMessage::from_json(&json.unwrap());
        assert!(deserialized.is_ok());
        
        match deserialized.unwrap() {
            WebSocketMessage::AuthSuccess { user_id } => {
                assert_eq!(user_id, "test_user_123");
            }
            _ => panic!("Expected AuthSuccess message"),
        }
    }

    #[tokio::test]
    async fn test_event_type_mapping() {
        let event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_123".to_string(),
            user_id: "user_123".to_string(),
            status: TransactionStatus::Pending,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: None,
        });
        
        assert_eq!(event.event_type(), EventType::TransactionStatus);
        assert_eq!(event.user_id(), Some("user_123".to_string()));
        assert!(!event.is_urgent());
    }

    #[tokio::test]
    async fn test_registry_user_specific_operations() {
        let registry = Arc::new(WebSocketRegistry::new());
        let connection1 = Arc::new(WebSocketConnection::new());
        let connection2 = Arc::new(WebSocketConnection::new());
        
        // Set user IDs
        connection1.set_user_id("user_1".to_string()).await;
        connection2.set_user_id("user_2".to_string()).await;
        
        // Add connections
        registry.add_connection(connection1.clone()).await;
        registry.add_connection(connection2.clone()).await;
        
        // Test user-specific retrieval
        let user1_connections = registry.get_user_connections("user_1").await;
        assert_eq!(user1_connections.len(), 1);
        assert_eq!(user1_connections[0].id(), connection1.id());
        
        let user2_connections = registry.get_user_connections("user_2").await;
        assert_eq!(user2_connections.len(), 1);
        assert_eq!(user2_connections[0].id(), connection2.id());
        
        // Test authentication count
        assert_eq!(registry.authenticated_connection_count().await, 2);
        assert_eq!(registry.unique_user_count().await, 2);
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let connection = Arc::new(WebSocketConnection::new());
        
        // Test initial state
        assert!(connection.is_active().await);
        assert!(connection.user_id().await.is_none());
        
        // Test authentication
        connection.set_user_id("test_user".to_string()).await;
        assert_eq!(connection.user_id().await, Some("test_user".to_string()));
        
        // Test close
        connection.close().await;
        assert!(!connection.is_active().await);
    }

    #[tokio::test]
    async fn test_event_urgency_detection() {
        // Test critical risk alert (should be urgent)
        let critical_alert = RealTimeEvent::RiskAlert(RiskAlertEvent {
            alert_id: "alert_1".to_string(),
            user_id: Some("user_1".to_string()),
            transaction_id: Some("tx_1".to_string()),
            risk_level: RiskLevel::Critical,
            risk_score: 0.95,
            alert_type: RiskAlertType::HighRiskTransaction,
            message: "Critical risk detected".to_string(),
            timestamp: chrono::Utc::now(),
            requires_action: true,
        });
        assert!(critical_alert.is_urgent());
        
        // Test normal transaction update (should not be urgent)
        let normal_update = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_123".to_string(),
            user_id: "user_123".to_string(),
            status: TransactionStatus::Confirmed,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(12),
            estimated_completion: None,
        });
        assert!(!normal_update.is_urgent());
        
        // Test critical system notification (should be urgent)
        let critical_notification = RealTimeEvent::SystemNotification(SystemNotificationEvent {
            notification_id: "notif_1".to_string(),
            user_id: None,
            level: NotificationLevel::Critical,
            title: "System Emergency".to_string(),
            message: "System requires immediate attention".to_string(),
            timestamp: chrono::Utc::now(),
            action_required: true,
            expires_at: None,
        });
        assert!(critical_notification.is_urgent());
    }

    #[tokio::test]
    async fn test_connection_idle_detection() {
        let connection = Arc::new(WebSocketConnection::new());
        
        // Should not be idle initially
        let short_duration = chrono::Duration::milliseconds(10);
        assert!(!connection.is_idle(short_duration).await);
        
        // Wait a bit and update activity
        tokio::time::sleep(Duration::from_millis(20)).await;
        connection.update_activity().await;
        
        // Should still not be idle after activity update
        assert!(!connection.is_idle(short_duration).await);
        
        // Wait longer than the idle duration
        tokio::time::sleep(Duration::from_millis(50)).await;
        let longer_duration = chrono::Duration::milliseconds(30);
        assert!(connection.is_idle(longer_duration).await);
    }

    #[tokio::test]
    async fn test_all_message_types_serialization() {
        let messages = vec![
            WebSocketMessage::Auth { token: "test_token".to_string() },
            WebSocketMessage::AuthSuccess { user_id: "user_123".to_string() },
            WebSocketMessage::AuthFailed { error: "Invalid token".to_string() },
            WebSocketMessage::Subscribe { event_type: EventType::TransactionStatus },
            WebSocketMessage::Unsubscribe { event_type: EventType::RiskAlerts },
            WebSocketMessage::Subscribed { event_type: EventType::PriceUpdates },
            WebSocketMessage::Ping,
            WebSocketMessage::Pong,
            WebSocketMessage::Error { message: "Test error".to_string(), code: Some(400) },
            WebSocketMessage::Close { reason: "Test close".to_string() },
        ];
        
        for message in messages {
            // Test serialization
            let json = message.to_json();
            assert!(json.is_ok(), "Failed to serialize message: {:?}", message);
            
            // Test deserialization
            let deserialized = WebSocketMessage::from_json(&json.unwrap());
            assert!(deserialized.is_ok(), "Failed to deserialize message: {:?}", message);
        }
    }

    #[tokio::test]
    async fn test_registry_connection_cleanup() {
        let registry = Arc::new(WebSocketRegistry::new());
        let connection1 = Arc::new(WebSocketConnection::new());
        let connection2 = Arc::new(WebSocketConnection::new());
        
        // Add connections
        registry.add_connection(connection1.clone()).await;
        registry.add_connection(connection2.clone()).await;
        assert_eq!(registry.connection_count().await, 2);
        
        // Close one connection
        connection1.close().await;
        
        // Cleanup should remove inactive connections
        let cleaned = registry.cleanup_inactive_connections().await;
        assert_eq!(cleaned, 1);
        assert_eq!(registry.connection_count().await, 1);
        
        // Active connection should remain
        let active_connections = registry.get_active_connections().await;
        assert_eq!(active_connections.len(), 1);
        assert_eq!(active_connections[0].id(), connection2.id());
    }

    #[tokio::test]
    async fn test_crypto_service_event_helpers() {
        // Test service status change event
        let status_event = CryptoServiceEvent::service_status_change(
            "crypto-service",
            "healthy",
            "Service is operational"
        );
        
        assert_eq!(status_event.service_name, "crypto-service");
        assert_eq!(status_event.status, "healthy");
        assert_eq!(status_event.message, "Service is operational");
        assert!(matches!(status_event.event_type, CryptoEventType::ServiceStatusChange));
        
        // Test key generated event
        let key_event = CryptoServiceEvent::key_generated("key_123", "ML-KEM-1024");
        
        assert_eq!(key_event.service_name, "crypto-service");
        assert_eq!(key_event.status, "success");
        assert!(key_event.message.contains("ML-KEM-1024"));
        assert!(matches!(key_event.event_type, CryptoEventType::KeyGenerated));
        assert!(key_event.metadata.is_some());
        
        if let Some(metadata) = key_event.metadata {
            assert_eq!(metadata["key_id"], "key_123");
            assert_eq!(metadata["algorithm"], "ML-KEM-1024");
        }
    }

    #[tokio::test]
    async fn test_connection_subscription_filtering() {
        let connection = Arc::new(WebSocketConnection::new());
        connection.set_user_id("user_123".to_string()).await;
        
        // Subscribe to specific event type
        connection.subscribe(EventType::TransactionStatus).await;
        
        // Create events for different types
        let transaction_event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_123".to_string(),
            user_id: "user_123".to_string(),
            status: TransactionStatus::Pending,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: None,
        });
        
        let risk_event = RealTimeEvent::RiskAlert(RiskAlertEvent {
            alert_id: "alert_1".to_string(),
            user_id: Some("user_123".to_string()),
            transaction_id: Some("tx_1".to_string()),
            risk_level: RiskLevel::Medium,
            risk_score: 0.6,
            alert_type: RiskAlertType::AnomalyDetected,
            message: "Anomaly detected".to_string(),
            timestamp: chrono::Utc::now(),
            requires_action: false,
        });
        
        // Test event filtering by subscription
        assert_eq!(transaction_event.event_type(), EventType::TransactionStatus);
        assert_eq!(risk_event.event_type(), EventType::RiskAlerts);
        
        // Connection should only be subscribed to TransactionStatus
        assert!(connection.is_subscribed(&EventType::TransactionStatus).await);
        assert!(!connection.is_subscribed(&EventType::RiskAlerts).await);
    }
}

// Include integration tests
#[path = "tests_integration.rs"]
#[cfg(test)]
mod tests_integration;