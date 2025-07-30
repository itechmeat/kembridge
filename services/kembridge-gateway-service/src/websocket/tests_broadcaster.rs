#[cfg(test)]
mod broadcaster_tests {
    use super::super::*;
    use crate::websocket::{message::*, WebSocketBroadcaster, WebSocketConnection, WebSocketRegistry};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Test helper to create a test connection with subscription
    async fn create_test_connection_with_subscription(
        user_id: Option<&str>, 
        event_type: EventType
    ) -> Arc<WebSocketConnection> {
        let connection = Arc::new(WebSocketConnection::new());
        
        if let Some(uid) = user_id {
            connection.set_user_id(uid.to_string()).await;
        }
        
        connection.subscribe(event_type).await;
        connection
    }

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        let stats = broadcaster.get_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.authenticated_connections, 0);
        assert_eq!(stats.unique_users, 0);
    }

    #[tokio::test]
    async fn test_broadcast_to_subscribers() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections with different subscriptions
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::RiskAlerts
        ).await;
        let conn3 = create_test_connection_with_subscription(
            Some("user3"), 
            EventType::TransactionStatus
        ).await;
        
        // Add connections to registry
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Create a transaction status event
        let event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_123".to_string(),
            user_id: "user1".to_string(),
            status: TransactionStatus::Pending,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: None,
        });
        
        // Broadcast to subscribers - should reach conn1 and conn3 (both subscribed to TransactionStatus)
        let result = broadcaster.broadcast_to_subscribers(event).await;
        
        // Only connections subscribed to TransactionStatus should receive the event
        // But since we don't have actual WebSocket channels in tests, we can't verify delivery
        // We can verify that the method executes without error
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_to_specific_user() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections for different users
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::TransactionStatus
        ).await;
        let conn3 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::RiskAlerts
        ).await; // Same user, different connection
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Create event for specific user
        let event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_123".to_string(),
            user_id: "user1".to_string(),
            status: TransactionStatus::Completed,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(12),
            estimated_completion: None,
        });
        
        // Broadcast to specific user - should reach conn1 and conn3 (both belong to user1)
        let result = broadcaster.broadcast_to_user("user1", event).await;
        assert!(result.is_ok());
        
        // Verify that broadcasting to non-existent user works without error
        let event2 = RealTimeEvent::PriceUpdate(PriceUpdateEvent {
            token_symbol: "ETH".to_string(),
            price_usd: "2500.00".to_string(),
            price_change_24h: 5.2,
            volume_24h: "1000000".to_string(),
            timestamp: chrono::Utc::now(),
            source: "coinbase".to_string(),
        });
        
        let result2 = broadcaster.broadcast_to_user("non_existent_user", event2).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_urgent_events() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::RiskAlerts
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::TransactionStatus
        ).await; // Not subscribed to risk alerts
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Create urgent risk alert
        let urgent_event = RealTimeEvent::RiskAlert(RiskAlertEvent {
            alert_id: "alert_critical".to_string(),
            user_id: Some("user1".to_string()),
            transaction_id: Some("tx_danger".to_string()),
            risk_level: RiskLevel::Critical,
            risk_score: 0.98,
            alert_type: RiskAlertType::BlacklistMatch,
            message: "CRITICAL: Blacklisted address detected".to_string(),
            timestamp: chrono::Utc::now(),
            requires_action: true,
        });
        
        // Verify event is marked as urgent
        assert!(urgent_event.is_urgent());
        
        // Broadcast urgent event - should reach ALL active connections regardless of subscription
        let result = broadcaster.broadcast_urgent(urgent_event).await;
        assert!(result.is_ok());
        
        // Test non-urgent event should fail with broadcast_urgent
        let normal_event = RealTimeEvent::PriceUpdate(PriceUpdateEvent {
            token_symbol: "ETH".to_string(),
            price_usd: "2500.00".to_string(),
            price_change_24h: 2.1,
            volume_24h: "500000".to_string(),
            timestamp: chrono::Utc::now(),
            source: "binance".to_string(),
        });
        
        assert!(!normal_event.is_urgent());
        let result2 = broadcaster.broadcast_urgent(normal_event).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("not marked as urgent"));
    }

    #[tokio::test]
    async fn test_broadcast_system_notifications() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::SystemNotifications
        ).await;
        let conn2 = create_test_connection_with_subscription(
            None, // Anonymous connection
            EventType::SystemNotifications
        ).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Test info notification (should use regular broadcast)
        let result1 = broadcaster.broadcast_system_notification(
            NotificationLevel::Info,
            "System Update",
            "System has been updated to version 1.2.3",
            false
        ).await;
        assert!(result1.is_ok());
        
        // Test critical notification (should use urgent broadcast)
        let result2 = broadcaster.broadcast_system_notification(
            NotificationLevel::Critical,
            "System Emergency",
            "Critical system failure detected",
            true
        ).await;
        assert!(result2.is_ok());
        
        // Test error notification (should use urgent broadcast)
        let result3 = broadcaster.broadcast_system_notification(
            NotificationLevel::Error,
            "Database Error",
            "Database connection lost",
            false
        ).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_crypto_events() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connection subscribed to crypto service events
        let conn = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::CryptoService
        ).await;
        
        registry.add_connection(conn.clone()).await;
        
        // Test various crypto event types
        let test_cases = vec![
            (CryptoEventType::ServiceStatusChange, "crypto-service", "healthy", "Service is operational"),
            (CryptoEventType::KeyGenerated, "crypto-service", "success", "New ML-KEM-1024 key generated"),
            (CryptoEventType::KeyRotated, "crypto-service", "success", "Key rotation completed"),
            (CryptoEventType::EncapsulationCompleted, "crypto-service", "success", "Encapsulation completed successfully"),
            (CryptoEventType::DecapsulationCompleted, "crypto-service", "success", "Decapsulation completed successfully"),
            (CryptoEventType::SystemHealthCheck, "crypto-service", "healthy", "System health check passed"),
        ];
        
        for (event_type, service, status, message) in test_cases {
            let metadata = Some(serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "version": "1.0.0"
            }));
            
            let result = broadcaster.broadcast_crypto_event(
                event_type.clone(),
                service,
                status,
                message,
                metadata
            ).await;
            
            assert!(result.is_ok(), "Failed to broadcast crypto event: {:?}", event_type);
        }
    }

    #[tokio::test]
    async fn test_broadcaster_stats_with_connections() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Initial stats should be zero
        let initial_stats = broadcaster.get_stats().await;
        assert_eq!(initial_stats.total_connections, 0);
        assert_eq!(initial_stats.active_connections, 0);
        assert_eq!(initial_stats.authenticated_connections, 0);
        assert_eq!(initial_stats.unique_users, 0);
        
        // Add authenticated connection
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        registry.add_connection(conn1.clone()).await;
        
        let stats_after_auth = broadcaster.get_stats().await;
        assert_eq!(stats_after_auth.total_connections, 1);
        assert_eq!(stats_after_auth.active_connections, 1);
        assert_eq!(stats_after_auth.authenticated_connections, 1);
        assert_eq!(stats_after_auth.unique_users, 1);
        
        // Add anonymous connection
        let conn2 = create_test_connection_with_subscription(
            None, 
            EventType::RiskAlerts
        ).await;
        registry.add_connection(conn2.clone()).await;
        
        let stats_with_anon = broadcaster.get_stats().await;
        assert_eq!(stats_with_anon.total_connections, 2);
        assert_eq!(stats_with_anon.active_connections, 2);
        assert_eq!(stats_with_anon.authenticated_connections, 1); // Still only 1 authenticated
        assert_eq!(stats_with_anon.unique_users, 1); // Still only 1 unique user
        
        // Add another connection for same user
        let conn3 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::SystemNotifications
        ).await;
        registry.add_connection(conn3.clone()).await;
        
        let stats_same_user = broadcaster.get_stats().await;
        assert_eq!(stats_same_user.total_connections, 3);
        assert_eq!(stats_same_user.active_connections, 3);
        assert_eq!(stats_same_user.authenticated_connections, 2);
        assert_eq!(stats_same_user.unique_users, 1); // Still only 1 unique user
        
        // Close one connection
        conn1.close().await;
        
        let stats_after_close = broadcaster.get_stats().await;
        assert_eq!(stats_after_close.total_connections, 3); // Connection still in registry
        assert_eq!(stats_after_close.active_connections, 2); // But not active
    }

    #[tokio::test]
    async fn test_connection_cleanup_operations() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::RiskAlerts
        ).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Close one connection
        conn1.close().await;
        
        // Test cleanup of inactive connections
        let cleaned = broadcaster.cleanup_inactive_connections().await;
        assert!(cleaned.is_ok());
        
        // Test cleanup of idle connections (very short timeout for testing)
        let idle_cleaned = broadcaster.cleanup_idle_connections(0).await; // 0 minutes = immediate cleanup
        assert!(idle_cleaned.is_ok());
        
        // Test heartbeat sending
        let heartbeat_result = broadcaster.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());
    }

    #[tokio::test]
    async fn test_disconnect_user() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create multiple connections for same user
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::RiskAlerts
        ).await;
        let conn3 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::SystemNotifications
        ).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Verify initial state
        assert_eq!(registry.connection_count().await, 3);
        assert_eq!(registry.unique_user_count().await, 2);
        
        // Disconnect all connections for user1
        let disconnected = broadcaster.disconnect_user("user1", "Test disconnection").await;
        assert!(disconnected.is_ok());
        
        // Give some time for cleanup
        sleep(Duration::from_millis(10)).await;
        
        // Verify user1 connections are gone but user2 remains
        let user1_connections = registry.get_user_connections("user1").await;
        assert_eq!(user1_connections.len(), 0);
        
        let user2_connections = registry.get_user_connections("user2").await;
        assert_eq!(user2_connections.len(), 1);
    }

    #[tokio::test]
    async fn test_broadcast_filtered() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections for different users
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            Some("user2"), 
            EventType::TransactionStatus
        ).await;
        let conn3 = create_test_connection_with_subscription(
            Some("user3"), 
            EventType::TransactionStatus
        ).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Create event
        let event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_filtered".to_string(),
            user_id: "user1".to_string(),
            status: TransactionStatus::Pending,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "50.0".to_string(),
            token_symbol: "USDC".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: None,
        });
        
        // Test broadcast to all (no filtering)
        let result1 = broadcaster.broadcast_filtered(event.clone(), None).await;
        assert!(result1.is_ok());
        
        // Test broadcast to specific users only
        let target_users = vec!["user1".to_string(), "user3".to_string()];
        let result2 = broadcaster.broadcast_filtered(event.clone(), Some(target_users)).await;
        assert!(result2.is_ok());
        
        // Test broadcast to non-existent users
        let non_existent_users = vec!["user_404".to_string()];
        let result3 = broadcaster.broadcast_filtered(event, Some(non_existent_users)).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_connection_details_retrieval() {
        let registry = Arc::new(WebSocketRegistry::new());
        let broadcaster = WebSocketBroadcaster::new(registry.clone());
        
        // Create connections with different states
        let conn1 = create_test_connection_with_subscription(
            Some("user1"), 
            EventType::TransactionStatus
        ).await;
        let conn2 = create_test_connection_with_subscription(
            None, // Anonymous
            EventType::RiskAlerts
        ).await;
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Get connection details
        let details = broadcaster.get_connection_details().await;
        assert_eq!(details.len(), 2);
        
        // Verify details contain expected information
        let authenticated_detail = details.iter()
            .find(|d| d.user_id.is_some())
            .expect("Should have authenticated connection");
        
        assert_eq!(authenticated_detail.user_id, Some("user1".to_string()));
        assert!(authenticated_detail.is_active);
        assert!(!authenticated_detail.subscriptions.is_empty());
        
        let anonymous_detail = details.iter()
            .find(|d| d.user_id.is_none())
            .expect("Should have anonymous connection");
        
        assert!(anonymous_detail.user_id.is_none());
        assert!(anonymous_detail.is_active);
        assert!(!anonymous_detail.subscriptions.is_empty());
    }
}