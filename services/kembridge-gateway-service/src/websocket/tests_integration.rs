#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use crate::websocket::{message::*, WebSocketBroadcaster, WebSocketConnection, WebSocketRegistry, create_websocket_services};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Integration test for complete WebSocket workflow
    #[tokio::test]
    async fn test_complete_websocket_workflow() {
        // Create WebSocket services
        let (registry, broadcaster) = create_websocket_services();
        
        // Create connections
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        
        // Authenticate connections
        conn1.set_user_id("user1".to_string()).await;
        conn2.set_user_id("user2".to_string()).await;
        
        // Subscribe to events
        conn1.subscribe(EventType::TransactionStatus).await;
        conn1.subscribe(EventType::RiskAlerts).await;
        
        conn2.subscribe(EventType::TransactionStatus).await;
        conn2.subscribe(EventType::SystemNotifications).await;
        
        // Add connections to registry
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        
        // Verify registry state
        assert_eq!(registry.connection_count().await, 2);
        assert_eq!(registry.authenticated_connection_count().await, 2);
        assert_eq!(registry.unique_user_count().await, 2);
        
        // Test broadcasting to subscribers
        let transaction_event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
            transaction_id: "tx_integration_test".to_string(),
            user_id: "user1".to_string(),
            status: TransactionStatus::Pending,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "1000.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: None,
        });
        
        let broadcast_result = broadcaster.broadcast_to_subscribers(transaction_event).await;
        assert!(broadcast_result.is_ok());
        
        // Test user-specific broadcasting
        let user_specific_event = RealTimeEvent::RiskAlert(RiskAlertEvent {
            alert_id: "alert_integration".to_string(),
            user_id: Some("user1".to_string()),
            transaction_id: Some("tx_integration_test".to_string()),
            risk_level: RiskLevel::Medium,
            risk_score: 0.7,
            alert_type: RiskAlertType::AnomalyDetected,
            message: "Unusual transaction pattern detected".to_string(),
            timestamp: chrono::Utc::now(),
            requires_action: false,
        });
        
        let user_broadcast_result = broadcaster.broadcast_to_user("user1", user_specific_event).await;
        assert!(user_broadcast_result.is_ok());
        
        // Test system notification
        let system_notify_result = broadcaster.broadcast_system_notification(
            NotificationLevel::Info,
            "Integration Test",
            "Testing system notification functionality",
            false
        ).await;
        assert!(system_notify_result.is_ok());
        
        // Test crypto service event
        let crypto_event_result = broadcaster.broadcast_crypto_event(
            CryptoEventType::KeyGenerated,
            "crypto-service",
            "success",
            "Integration test key generated",
            Some(serde_json::json!({
                "key_id": "integration_test_key",
                "algorithm": "ML-KEM-1024"
            }))
        ).await;
        assert!(crypto_event_result.is_ok());
        
        // Test cleanup operations
        let cleanup_result = broadcaster.cleanup_inactive_connections().await;
        assert!(cleanup_result.is_ok());
        
        let heartbeat_result = broadcaster.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());
        
        // Verify final state
        let final_stats = broadcaster.get_stats().await;
        assert_eq!(final_stats.total_connections, 2);
        assert_eq!(final_stats.active_connections, 2);
        assert_eq!(final_stats.authenticated_connections, 2);
        assert_eq!(final_stats.unique_users, 2);
    }

    /// Test event filtering and subscription system
    #[tokio::test]
    async fn test_event_filtering_system() {
        let (registry, broadcaster) = create_websocket_services();
        
        // Create connections with different subscription patterns
        let tx_subscriber = Arc::new(WebSocketConnection::new());
        let risk_subscriber = Arc::new(WebSocketConnection::new());
        let multi_subscriber = Arc::new(WebSocketConnection::new());
        let no_subscriber = Arc::new(WebSocketConnection::new());
        
        // Set up users
        tx_subscriber.set_user_id("tx_user".to_string()).await;
        risk_subscriber.set_user_id("risk_user".to_string()).await;
        multi_subscriber.set_user_id("multi_user".to_string()).await;
        no_subscriber.set_user_id("no_sub_user".to_string()).await;
        
        // Set up subscriptions
        tx_subscriber.subscribe(EventType::TransactionStatus).await;
        
        risk_subscriber.subscribe(EventType::RiskAlerts).await;
        
        multi_subscriber.subscribe(EventType::TransactionStatus).await;
        multi_subscriber.subscribe(EventType::RiskAlerts).await;
        multi_subscriber.subscribe(EventType::SystemNotifications).await;
        
        // no_subscriber has no subscriptions
        
        // Add to registry
        registry.add_connection(tx_subscriber.clone()).await;
        registry.add_connection(risk_subscriber.clone()).await;
        registry.add_connection(multi_subscriber.clone()).await;
        registry.add_connection(no_subscriber.clone()).await;
        
        // Test different event types
        let events = vec![
            RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
                transaction_id: "tx_filter_test".to_string(),
                user_id: "multi_user".to_string(),
                status: TransactionStatus::Confirmed,
                from_chain: "ethereum".to_string(),
                to_chain: "near".to_string(),
                amount: "500.0".to_string(),
                token_symbol: "USDC".to_string(),
                timestamp: chrono::Utc::now(),
                confirmation_blocks: Some(6),
                estimated_completion: None,
            }),
            RealTimeEvent::RiskAlert(RiskAlertEvent {
                alert_id: "risk_filter_test".to_string(),
                user_id: Some("risk_user".to_string()),
                transaction_id: Some("tx_filter_test".to_string()),
                risk_level: RiskLevel::Low,
                risk_score: 0.3,
                alert_type: RiskAlertType::ThresholdExceeded,
                message: "Transaction threshold exceeded".to_string(),
                timestamp: chrono::Utc::now(),
                requires_action: false,
            }),
            RealTimeEvent::PriceUpdate(PriceUpdateEvent {
                token_symbol: "ETH".to_string(),
                price_usd: "2600.00".to_string(),
                price_change_24h: 3.5,
                volume_24h: "2000000".to_string(),
                timestamp: chrono::Utc::now(),
                source: "coinbase".to_string(),
            }),
        ];
        
        // Broadcast each event and verify it works
        for event in events {
            let result = broadcaster.broadcast_to_subscribers(event).await;
            assert!(result.is_ok(), "Failed to broadcast event");
        }
        
        // Test filtered broadcasting
        let filtered_event = RealTimeEvent::SystemNotification(SystemNotificationEvent {
            notification_id: "filtered_test".to_string(),
            user_id: None,
            level: NotificationLevel::Info,
            title: "Filtered Test".to_string(),
            message: "Testing filtered broadcast".to_string(),
            timestamp: chrono::Utc::now(),
            action_required: false,
            expires_at: None,
        });
        
        // Broadcast to specific users only
        let target_users = vec!["multi_user".to_string(), "no_sub_user".to_string()];
        let filtered_result = broadcaster.broadcast_filtered(filtered_event, Some(target_users)).await;
        assert!(filtered_result.is_ok());
    }

    /// Test connection lifecycle management
    #[tokio::test]
    async fn test_connection_lifecycle_management() {
        let (registry, broadcaster) = create_websocket_services();
        
        // Phase 1: Connection creation and authentication
        let conn1 = Arc::new(WebSocketConnection::new());
        let conn2 = Arc::new(WebSocketConnection::new());
        let conn3 = Arc::new(WebSocketConnection::new());
        
        registry.add_connection(conn1.clone()).await;
        registry.add_connection(conn2.clone()).await;
        registry.add_connection(conn3.clone()).await;
        
        // Initially all unauthenticated
        assert_eq!(registry.authenticated_connection_count().await, 0);
        assert_eq!(registry.unique_user_count().await, 0);
        
        // Phase 2: Authentication
        registry.authenticate_connection(&conn1.id(), "user1".to_string()).await.unwrap();
        registry.authenticate_connection(&conn2.id(), "user2".to_string()).await.unwrap();
        // conn3 remains unauthenticated
        
        assert_eq!(registry.authenticated_connection_count().await, 2);
        assert_eq!(registry.unique_user_count().await, 2);
        
        // Phase 3: Subscription management
        conn1.subscribe(EventType::TransactionStatus).await;
        conn1.subscribe(EventType::RiskAlerts).await;
        
        conn2.subscribe(EventType::SystemNotifications).await;
        
        conn3.subscribe(EventType::PriceUpdates).await; // Unauthenticated but can subscribe
        
        // Phase 4: Activity simulation
        sleep(Duration::from_millis(10)).await;
        
        conn1.update_activity().await;
        conn2.update_activity().await;
        // conn3 doesn't update activity - will be idle
        
        sleep(Duration::from_millis(10)).await;
        
        // Phase 5: Connection closing
        conn2.close().await;
        
        // Verify state
        assert_eq!(registry.connection_count().await, 3);
        assert_eq!(registry.active_connection_count().await, 2); // conn2 is inactive
        assert_eq!(registry.authenticated_connection_count().await, 1); // only conn1
        
        // Phase 6: Cleanup operations
        let inactive_cleaned = broadcaster.cleanup_inactive_connections().await.unwrap();
        assert!(inactive_cleaned > 0); // Should clean conn2
        
        let remaining_connections = registry.active_connection_count().await;
        assert!(remaining_connections < 3); // Some connections should be cleaned
        
        // Phase 7: User disconnection
        let disconnected = broadcaster.disconnect_user("user1", "Test disconnect").await.unwrap();
        assert!(disconnected > 0);
        
        // Final verification
        let final_stats = broadcaster.get_stats().await;
        // Exact numbers depend on cleanup implementation, but should be consistent
        assert!(final_stats.total_connections <= 3);
        assert!(final_stats.active_connections <= final_stats.total_connections);
    }

    /// Test error handling and edge cases
    #[tokio::test]
    async fn test_error_handling_and_edge_cases() {
        let (registry, broadcaster) = create_websocket_services();
        
        // Test operations on empty registry
        let empty_broadcast = broadcaster.broadcast_to_user("nonexistent", 
            RealTimeEvent::PriceUpdate(PriceUpdateEvent {
                token_symbol: "BTC".to_string(),
                price_usd: "50000.00".to_string(),
                price_change_24h: 1.5,
                volume_24h: "800000".to_string(),
                timestamp: chrono::Utc::now(),
                source: "kraken".to_string(),
            })
        ).await;
        assert!(empty_broadcast.is_ok()); // Should not error, just deliver to 0 connections
        
        // Test authentication of non-existent connection
        let fake_id = uuid::Uuid::new_v4();
        let fake_auth = registry.authenticate_connection(&fake_id, "user".to_string()).await;
        assert!(fake_auth.is_err());
        
        // Test urgent broadcast with non-urgent event
        let non_urgent = RealTimeEvent::PriceUpdate(PriceUpdateEvent {
            token_symbol: "ETH".to_string(),
            price_usd: "2500.00".to_string(),
            price_change_24h: 0.5,
            volume_24h: "1500000".to_string(),
            timestamp: chrono::Utc::now(),
            source: "binance".to_string(),
        });
        
        let urgent_result = broadcaster.broadcast_urgent(non_urgent).await;
        assert!(urgent_result.is_err());
        
        // Test with closed connections
        let conn = Arc::new(WebSocketConnection::new());
        registry.add_connection(conn.clone()).await;
        conn.close().await;
        
        // Broadcasting to closed connection should not error
        let closed_broadcast = broadcaster.broadcast_to_all(
            RealTimeEvent::SystemNotification(SystemNotificationEvent {
                notification_id: "test_closed".to_string(),
                user_id: None,
                level: NotificationLevel::Info,
                title: "Test".to_string(),
                message: "Testing broadcast to closed connection".to_string(),
                timestamp: chrono::Utc::now(),
                action_required: false,
                expires_at: None,
            })
        ).await;
        assert!(closed_broadcast.is_ok());
        
        // Test cleanup operations
        let cleanup_result = broadcaster.cleanup_inactive_connections().await;
        assert!(cleanup_result.is_ok());
        
        let idle_cleanup = broadcaster.cleanup_idle_connections(0).await; // Immediate cleanup
        assert!(idle_cleanup.is_ok());
    }

    /// Test message serialization and deserialization in context
    #[tokio::test]
    async fn test_message_serialization_integration() {
        // Test all event types can be serialized and embedded in WebSocket messages
        let events = vec![
            RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
                transaction_id: "tx_serial_test".to_string(),
                user_id: "user_serial".to_string(),
                status: TransactionStatus::Processing,
                from_chain: "ethereum".to_string(),
                to_chain: "near".to_string(),
                amount: "250.0".to_string(),
                token_symbol: "DAI".to_string(),
                timestamp: chrono::Utc::now(),
                confirmation_blocks: Some(3),
                estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
            }),
            RealTimeEvent::RiskAlert(RiskAlertEvent {
                alert_id: "serial_risk".to_string(),
                user_id: Some("user_serial".to_string()),
                transaction_id: Some("tx_serial_test".to_string()),
                risk_level: RiskLevel::High,
                risk_score: 0.85,
                alert_type: RiskAlertType::HighRiskTransaction,
                message: "High risk transaction detected".to_string(),
                timestamp: chrono::Utc::now(),
                requires_action: true,
            }),
            RealTimeEvent::BridgeOperation(BridgeOperationEvent {
                operation_id: "bridge_serial".to_string(),
                user_id: "user_serial".to_string(),
                operation_type: BridgeOperationType::EthToNear,
                status: BridgeOperationStatus::Bridging,
                progress: 0.65,
                current_step: "Validating cross-chain signature".to_string(),
                estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(2)),
                timestamp: chrono::Utc::now(),
            }),
            RealTimeEvent::QuantumKeyEvent(QuantumKeyEvent {
                key_id: "qkey_serial".to_string(),
                user_id: "user_serial".to_string(),
                event_type: QuantumKeyEventType::Generated,
                algorithm: "ML-KEM-1024".to_string(),
                timestamp: chrono::Utc::now(),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::days(90)),
            }),
            RealTimeEvent::CryptoServiceEvent(CryptoServiceEvent {
                event_id: "crypto_serial".to_string(),
                event_type: CryptoEventType::SystemHealthCheck,
                service_name: "crypto-service".to_string(),
                status: "healthy".to_string(),
                message: "All quantum cryptography systems operational".to_string(),
                timestamp: chrono::Utc::now(),
                metadata: Some(serde_json::json!({
                    "cpu_usage": 45.2,
                    "memory_usage": 67.8,
                    "active_keys": 156
                })),
            }),
        ];
        
        // Test each event can be wrapped in WebSocket message and serialized
        for event in events {
            let ws_message = WebSocketMessage::event(event.clone());
            
            // Test serialization
            let json = ws_message.to_json();
            assert!(json.is_ok(), "Failed to serialize WebSocket message with event: {:?}", event.event_type());
            
            // Test deserialization
            let deserialized = WebSocketMessage::from_json(&json.unwrap());
            assert!(deserialized.is_ok(), "Failed to deserialize WebSocket message with event: {:?}", event.event_type());
            
            // Verify the deserialized message contains the event
            if let Ok(WebSocketMessage::Event { event: deserialized_event }) = deserialized {
                assert_eq!(deserialized_event.event_type(), event.event_type());
                assert_eq!(deserialized_event.user_id(), event.user_id());
            } else {
                panic!("Deserialized message is not an Event message");
            }
        }
    }

    /// Test concurrent operations and thread safety
    #[tokio::test]
    async fn test_concurrent_operations() {
        let (registry, broadcaster) = create_websocket_services();
        
        // Create multiple connections concurrently
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let registry_clone = registry.clone();
            let broadcaster_clone = broadcaster.clone();
            
            let handle = tokio::spawn(async move {
                // Create connection
                let conn = Arc::new(WebSocketConnection::new());
                conn.set_user_id(format!("concurrent_user_{}", i)).await;
                conn.subscribe(EventType::TransactionStatus).await;
                
                // Add to registry
                registry_clone.add_connection(conn.clone()).await;
                
                // Broadcast some events
                let event = RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
                    transaction_id: format!("tx_concurrent_{}", i),
                    user_id: format!("concurrent_user_{}", i),
                    status: TransactionStatus::Pending,
                    from_chain: "ethereum".to_string(),
                    to_chain: "near".to_string(),
                    amount: "100.0".to_string(),
                    token_symbol: "ETH".to_string(),
                    timestamp: chrono::Utc::now(),
                    confirmation_blocks: Some(1),
                    estimated_completion: None,
                });
                
                let _ = broadcaster_clone.broadcast_to_subscribers(event).await;
                
                // Return connection ID for verification
                conn.id()
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let connection_ids: Vec<_> = futures::future::join_all(handles).await
            .into_iter()
            .map(|result| result.unwrap())
            .collect();
        
        // Verify all connections were added
        assert_eq!(registry.connection_count().await, 10);
        assert_eq!(registry.authenticated_connection_count().await, 10);
        assert_eq!(registry.unique_user_count().await, 10);
        
        // Verify all connections exist
        for conn_id in connection_ids {
            let conn = registry.get_connection(&conn_id).await;
            assert!(conn.is_some(), "Connection {} should exist", conn_id);
        }
        
        // Test concurrent cleanup
        let cleanup_result = broadcaster.cleanup_inactive_connections().await;
        assert!(cleanup_result.is_ok());
        
        // Test concurrent broadcasting
        let broadcast_result = broadcaster.broadcast_system_notification(
            NotificationLevel::Info,
            "Concurrent Test",
            "Testing concurrent broadcasting",
            false
        ).await;
        assert!(broadcast_result.is_ok());
    }
}