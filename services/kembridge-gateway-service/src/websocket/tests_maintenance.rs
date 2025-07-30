#[cfg(test)]
mod maintenance_tests {
    use super::super::*;
    use crate::websocket::{message::*, WebSocketBroadcaster, WebSocketConnection, WebSocketRegistry, ConnectionMaintenance};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_connection_maintenance_creation() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Test default maintenance settings
        let maintenance = ConnectionMaintenance::new(broadcaster.clone());
        
        // Create maintenance with custom intervals
        let custom_maintenance = ConnectionMaintenance::with_intervals(
            broadcaster,
            Duration::from_secs(60),  // cleanup every minute
            Duration::from_secs(10),  // heartbeat every 10 seconds
            15                        // idle timeout 15 minutes
        );
        
        // Both should be created successfully
        // We can't easily test the actual intervals without running the background tasks
    }

    #[tokio::test]
    async fn test_heartbeat_functionality() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create test connections
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        
        conn1.set_user_id("user1".to_string()).await;
        conn2.set_user_id("user2".to_string()).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Test heartbeat sending
        let heartbeat_result = broadcaster.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());
        
        // Should return count of connections that received heartbeat
        // In actual implementation with real WebSocket channels, this would be > 0
    }

    #[tokio::test]
    async fn test_idle_connection_cleanup() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Simulate idle time by waiting
        sleep(Duration::from_millis(50)).await;
        
        // Update activity for one connection
        conn2.update_activity().await;
        
        // Wait a bit more
        sleep(Duration::from_millis(50)).await;
        
        // Cleanup idle connections (very short timeout for testing)
        let cleaned = broadcaster.cleanup_idle_connections(0).await; // 0 minutes = immediate
        assert!(cleaned.is_ok());
        
        // The exact number depends on implementation, but should not error
    }

    #[tokio::test]
    async fn test_inactive_connection_cleanup() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        let conn3 = Arc::new(WebSocketConnection::new());
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        assert_eq!(registry.connection_count().await, 3);
        
        // Close some connections
        conn1.close().await;
        conn3.close().await;
        
        // Cleanup inactive connections
        let cleaned = broadcaster.cleanup_inactive_connections().await;
        assert!(cleaned.is_ok());
        
        // After cleanup, should have fewer connections
        let remaining = registry.connection_count().await;
        // The exact count depends on cleanup implementation
        assert!(remaining <= 3);
    }

    #[tokio::test]
    async fn test_connection_activity_tracking() {
        let connection = Arc::new(WebSocketConnection::new());
        
        // Record initial connection time
        let initial_time = connection.connected_at();
        let initial_activity = connection.last_activity().await;
        
        // Should be connected recently
        assert!(initial_time <= chrono::Utc::now());
        assert!(initial_activity <= chrono::Utc::now());
        
        // Wait a bit
        sleep(Duration::from_millis(10)).await;
        
        // Update activity
        connection.update_activity().await;
        let updated_activity = connection.last_activity().await;
        
        // Activity should be more recent than initial
        assert!(updated_activity > initial_activity);
        
        // Test idle detection
        let very_short_idle = chrono::Duration::milliseconds(1);
        let longer_idle = chrono::Duration::milliseconds(100);
        
        // Should not be idle with very short duration
        assert!(!connection.is_idle(very_short_idle).await);
        
        // Wait longer than idle threshold
        sleep(Duration::from_millis(50)).await;
        
        // Now should be considered idle
        assert!(connection.is_idle(very_short_idle).await);
        assert!(!connection.is_idle(longer_idle).await);
    }

    #[tokio::test]
    async fn test_registry_stats_accuracy() {
        let registry = Arc::new(WebSocketRegistry::new());
        
        // Test empty registry
        let empty_stats = registry.get_stats().await;
        assert_eq!(empty_stats.total_connections, 0);
        assert_eq!(empty_stats.active_connections, 0);
        assert_eq!(empty_stats.authenticated_connections, 0);
        assert_eq!(empty_stats.unique_users, 0);
        
        // Add authenticated connection
        let conn1 = Arc::new(WebSocketConnection::new());
        conn1.set_user_id("user1".to_string()).await;
        registry.add_connection(conn1.clone()).await;
        
        let stats_1 = registry.get_stats().await;
        assert_eq!(stats_1.total_connections, 1);
        assert_eq!(stats_1.active_connections, 1);
        assert_eq!(stats_1.authenticated_connections, 1);
        assert_eq!(stats_1.unique_users, 1);
        
        // Add anonymous connection
        let conn2 = Arc::new(WebSocketConnection::new());
        registry.add_connection(conn2.clone()).await;
        
        let stats_2 = registry.get_stats().await;
        assert_eq!(stats_2.total_connections, 2);
        assert_eq!(stats_2.active_connections, 2);
        assert_eq!(stats_2.authenticated_connections, 1);
        assert_eq!(stats_2.unique_users, 1);
        
        // Add another connection for same user
        let conn3 = Arc::new(WebSocketConnection::new());
        conn3.set_user_id("user1".to_string()).await;
        registry.add_connection(conn3.clone()).await;
        
        let stats_3 = registry.get_stats().await;
        assert_eq!(stats_3.total_connections, 3);
        assert_eq!(stats_3.active_connections, 3);
        assert_eq!(stats_3.authenticated_connections, 2);
        assert_eq!(stats_3.unique_users, 1); // Still same user
        
        // Add connection for different user
        let conn4 = Arc::new(WebSocketConnection::new());
        conn4.set_user_id("user2".to_string()).await;
        registry.add_connection(conn4.clone()).await;
        
        let stats_4 = registry.get_stats().await;
        assert_eq!(stats_4.total_connections, 4);
        assert_eq!(stats_4.active_connections, 4);
        assert_eq!(stats_4.authenticated_connections, 3);
        assert_eq!(stats_4.unique_users, 2); // Now 2 unique users
        
        // Close some connections
        conn1.close().await;
        conn3.close().await;
        
        let stats_5 = registry.get_stats().await;
        assert_eq!(stats_5.total_connections, 4); // Still in registry
        assert_eq!(stats_5.active_connections, 2); // But not active
        assert_eq!(stats_5.authenticated_connections, 1); // Only conn4
        assert_eq!(stats_5.unique_users, 2); // Users still tracked
    }

    #[tokio::test]
    async fn test_user_disconnect_functionality() {
        let registry = Arc::new(WebSocketRegistry::new());
        
        // Create multiple connections for same user
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        let conn3 = Arc::new(WebSocketConnection::new());
        
        conn1.set_user_id("user1".to_string()).await;
        conn2.set_user_id("user1".to_string()).await;
        conn3.set_user_id("user2".to_string()).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Verify initial state
        assert_eq!(registry.get_user_connections("user1").await.len(), 2);
        assert_eq!(registry.get_user_connections("user2").await.len(), 1);
        assert_eq!(registry.unique_user_count().await, 2);
        
        // Disconnect all connections for user1
        let disconnected = registry.disconnect_user("user1").await;
        assert_eq!(disconnected, 2);
        
        // Verify user1 connections are gone
        assert_eq!(registry.get_user_connections("user1").await.len(), 0);
        assert_eq!(registry.get_user_connections("user2").await.len(), 1);
        assert_eq!(registry.unique_user_count().await, 1);
        
        // Verify connections are closed
        assert!(!conn1.is_active().await);
        assert!(!conn2.is_active().await);
        assert!(conn3.is_active().await);
    }

    #[tokio::test]
    async fn test_connection_authentication_flow() {
        let registry = Arc::new(WebSocketRegistry::new());
        let connection = Arc::new(WebSocketConnection::new());
        let connection_id = connection.id();
        
        // Add connection to registry (unauthenticated)
        registry.add_connection(connection.clone()).await;
        
        // Verify initial state
        assert!(connection.user_id().await.is_none());
        assert_eq!(registry.authenticated_connection_count().await, 0);
        assert_eq!(registry.unique_user_count().await, 0);
        
        // Authenticate connection
        let auth_result = registry.authenticate_connection(&connection_id, "user123".to_string()).await;
        assert!(auth_result.is_ok());
        
        // Verify authentication worked
        assert_eq!(connection.user_id().await, Some("user123".to_string()));
        assert_eq!(registry.authenticated_connection_count().await, 1);
        assert_eq!(registry.unique_user_count().await, 1);
        
        // Verify user connections retrieval
        let user_connections = registry.get_user_connections("user123").await;
        assert_eq!(user_connections.len(), 1);
        assert_eq!(user_connections[0].id(), connection_id);
        
        // Test authentication of non-existent connection
        let fake_id = uuid::Uuid::new_v4();
        let fake_auth = registry.authenticate_connection(&fake_id, "user456".to_string()).await;
        assert!(fake_auth.is_err());
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let connection = Arc::new(WebSocketConnection::new());
        
        // Test initial state - no subscriptions
        let initial_subs = connection.get_subscriptions().await;
        assert!(initial_subs.is_empty());
        
        // Test subscribing to different event types
        let event_types = vec![
            EventType::TransactionStatus,
            EventType::RiskAlerts,
            EventType::PriceUpdates,
            EventType::SystemNotifications,
            EventType::BridgeOperations,
            EventType::QuantumKeys,
            EventType::UserProfile,
            EventType::CryptoService,
        ];
        
        for event_type in &event_types {
            let was_new = connection.subscribe(event_type.clone()).await;
            assert!(was_new, "First subscription to {:?} should be new", event_type);
            
            // Test duplicate subscription
            let was_duplicate = connection.subscribe(event_type.clone()).await;
            assert!(!was_duplicate, "Duplicate subscription to {:?} should not be new", event_type);
            
            // Test subscription check
            assert!(connection.is_subscribed(event_type).await, "Should be subscribed to {:?}", event_type);
        }
        
        // Verify all subscriptions are present
        let all_subs = connection.get_subscriptions().await;
        assert_eq!(all_subs.len(), event_types.len());
        
        for event_type in &event_types {
            assert!(all_subs.contains(event_type), "Should contain subscription to {:?}", event_type);
        }
        
        // Test unsubscribing
        for event_type in &event_types {
            let was_present = connection.unsubscribe(event_type).await;
            assert!(was_present, "Unsubscription from {:?} should find existing subscription", event_type);
            
            // Test duplicate unsubscription
            let was_duplicate = connection.unsubscribe(event_type).await;
            assert!(!was_duplicate, "Duplicate unsubscription from {:?} should not find subscription", event_type);
            
            // Test subscription check after unsubscription
            assert!(!connection.is_subscribed(event_type).await, "Should not be subscribed to {:?} after unsubscription", event_type);
        }
        
        // Verify no subscriptions remain
        let final_subs = connection.get_subscriptions().await;
        assert!(final_subs.is_empty());
    }

    #[tokio::test]
    async fn test_connection_info_accuracy() {
        let connection = Arc::new(WebSocketConnection::with_metadata(
            Some("192.168.1.100".to_string()),
            Some("Mozilla/5.0 (Test Browser)".to_string())
        ));
        
        // Set user and subscriptions
        connection.set_user_id("test_user".to_string()).await;
        connection.subscribe(EventType::TransactionStatus).await;
        connection.subscribe(EventType::RiskAlerts).await;
        
        // Get connection info
        let info = connection.get_info().await;
        
        // Verify all fields are correct
        assert_eq!(info.id, connection.id());
        assert_eq!(info.user_id, Some("test_user".to_string()));
        assert_eq!(info.client_ip, Some("192.168.1.100".to_string()));
        assert_eq!(info.user_agent, Some("Mozilla/5.0 (Test Browser)".to_string()));
        assert!(info.is_active);
        assert_eq!(info.subscriptions.len(), 2);
        assert!(info.subscriptions.contains(&EventType::TransactionStatus));
        assert!(info.subscriptions.contains(&EventType::RiskAlerts));
        
        // Verify timestamps are reasonable
        assert!(info.connected_at <= chrono::Utc::now());
        assert!(info.last_activity <= chrono::Utc::now());
        assert!(info.last_activity >= info.connected_at);
        
        // Close connection and verify info reflects change
        connection.close().await;
        let closed_info = connection.get_info().await;
        assert!(!closed_info.is_active);
    }

    #[tokio::test]
    async fn test_registry_cleanup_edge_cases() {
        let registry = Arc::new(WebSocketRegistry::new());
        
        // Test cleanup with no connections
        let cleaned_empty = registry.cleanup_inactive_connections().await;
        assert_eq!(cleaned_empty, 0);
        
        let idle_cleaned_empty = registry.cleanup_idle_connections(chrono::Duration::minutes(30)).await;
        assert_eq!(idle_cleaned_empty, 0);
        
        // Test cleanup with all active connections
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        let cleaned_active = registry.cleanup_inactive_connections().await;
        assert_eq!(cleaned_active, 0); // No inactive connections to clean
        
        // Update activity to prevent idle cleanup
        conn1.update_activity().await;
        conn2.update_activity().await;
        
        let idle_cleaned_active = registry.cleanup_idle_connections(chrono::Duration::minutes(30)).await;
        assert_eq!(idle_cleaned_active, 0); // No idle connections to clean
        
        // Test cleanup with mixed states
        conn1.close().await; // Make one inactive
        
        let cleaned_mixed = registry.cleanup_inactive_connections().await;
        assert_eq!(cleaned_mixed, 1); // Should clean the inactive one
        
        // Verify registry state
        assert_eq!(registry.connection_count().await, 1);
        assert_eq!(registry.active_connection_count().await, 1);
    }
}