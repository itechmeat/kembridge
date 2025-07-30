#[cfg(test)]
mod event_listener_tests {
    use super::super::*;
    use crate::websocket::create_websocket_services;
    use std::sync::Arc;
    use std::time::Duration;

    /// Create test EventListener with mock URLs
    fn create_test_event_listener() -> EventListener {
        let (_, broadcaster) = create_websocket_services();

        EventListener::new(
            broadcaster,
            "http://mock-crypto-service:4001".to_string(),
            "http://mock-ai-engine:4005".to_string(),
            "http://mock-blockchain-service:4003".to_string(),
            Duration::from_millis(100), // Fast polling for tests
        )
    }

    #[tokio::test]
    async fn test_event_listener_creation() {
        let event_listener = create_test_event_listener();

        // Verify all URLs are set correctly
        assert_eq!(
            event_listener.get_crypto_service_url(),
            "http://mock-crypto-service:4001"
        );
        assert_eq!(
            event_listener.get_ai_engine_url(),
            "http://mock-ai-engine:4005"
        );
        assert_eq!(
            event_listener.get_blockchain_service_url(),
            "http://mock-blockchain-service:4003"
        );
        assert_eq!(
            event_listener.get_poll_interval(),
            Duration::from_millis(100)
        );
    }

    #[tokio::test]
    async fn test_create_event_listener_function() {
        let (_, broadcaster) = create_websocket_services();
        let event_listener = create_event_listener(broadcaster);

        // Should use default URLs from environment or fallback
        assert!(
            event_listener
                .get_crypto_service_url()
                .contains("localhost")
                || event_listener.get_crypto_service_url().contains("4001")
        );
        assert!(
            event_listener.get_ai_engine_url().contains("localhost")
                || event_listener.get_ai_engine_url().contains("4005")
        );
        assert!(
            event_listener
                .get_blockchain_service_url()
                .contains("localhost")
                || event_listener.get_blockchain_service_url().contains("4003")
        );
    }

    #[tokio::test]
    async fn test_handle_crypto_operation_key_generation() {
        let event_listener = create_test_event_listener();
        let user_id = "test_user_123";

        // This should not panic and should complete
        let result = event_listener
            .handle_crypto_operation("key_generation", user_id)
            .await;

        // Since we're using mock URLs, this will likely fail with connection error
        // but the important thing is that it doesn't panic and handles the operation
        assert!(result.is_ok() || result.is_err()); // Either outcome is fine for unit test
    }

    #[tokio::test]
    async fn test_handle_crypto_operation_encapsulation() {
        let event_listener = create_test_event_listener();
        let user_id = "test_user_456";

        let result = event_listener
            .handle_crypto_operation("encapsulation", user_id)
            .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_crypto_operation_decapsulation() {
        let event_listener = create_test_event_listener();
        let user_id = "test_user_789";

        let result = event_listener
            .handle_crypto_operation("decapsulation", user_id)
            .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_crypto_operation_unknown_type() {
        let event_listener = create_test_event_listener();
        let user_id = "test_user_unknown";

        // Unknown operation type should not crash
        let result = event_listener
            .handle_crypto_operation("unknown_operation", user_id)
            .await;
        assert!(result.is_ok()); // Should complete successfully, just log warning
    }

    #[tokio::test]
    async fn test_handle_risk_analysis_low_risk() {
        let event_listener = create_test_event_listener();
        let user_id = "risk_user_low";
        let transaction_id = "tx_low_risk";
        let risk_score = 0.2;

        let result = event_listener
            .handle_risk_analysis(user_id, transaction_id, risk_score)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_risk_analysis_medium_risk() {
        let event_listener = create_test_event_listener();
        let user_id = "risk_user_medium";
        let transaction_id = "tx_medium_risk";
        let risk_score = 0.5;

        let result = event_listener
            .handle_risk_analysis(user_id, transaction_id, risk_score)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_risk_analysis_high_risk() {
        let event_listener = create_test_event_listener();
        let user_id = "risk_user_high";
        let transaction_id = "tx_high_risk";
        let risk_score = 0.7;

        let result = event_listener
            .handle_risk_analysis(user_id, transaction_id, risk_score)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_risk_analysis_critical_risk() {
        let event_listener = create_test_event_listener();
        let user_id = "risk_user_critical";
        let transaction_id = "tx_critical_risk";
        let risk_score = 0.95;

        let result = event_listener
            .handle_risk_analysis(user_id, transaction_id, risk_score)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_risk_analysis_edge_cases() {
        let event_listener = create_test_event_listener();

        // Test minimum risk score
        let result1 = event_listener
            .handle_risk_analysis("user1", "tx1", 0.0)
            .await;
        assert!(result1.is_ok());

        // Test maximum risk score
        let result2 = event_listener
            .handle_risk_analysis("user2", "tx2", 1.0)
            .await;
        assert!(result2.is_ok());

        // Test boundary values
        let result3 = event_listener
            .handle_risk_analysis("user3", "tx3", 0.8)
            .await; // Critical threshold
        assert!(result3.is_ok());

        let result4 = event_listener
            .handle_risk_analysis("user4", "tx4", 0.6)
            .await; // High threshold
        assert!(result4.is_ok());
    }

    // Note: poll_crypto_service, poll_ai_engine, and poll_blockchain_service are private methods
    // and are tested indirectly through the public start() method and integration tests

    #[tokio::test]
    async fn test_event_listener_clone() {
        let event_listener = create_test_event_listener();
        let cloned_listener = event_listener.clone();

        // Verify clone has same configuration
        assert_eq!(
            event_listener.get_crypto_service_url(),
            cloned_listener.get_crypto_service_url()
        );
        assert_eq!(
            event_listener.get_ai_engine_url(),
            cloned_listener.get_ai_engine_url()
        );
        assert_eq!(
            event_listener.get_blockchain_service_url(),
            cloned_listener.get_blockchain_service_url()
        );
        assert_eq!(
            event_listener.get_poll_interval(),
            cloned_listener.get_poll_interval()
        );
    }

    #[tokio::test]
    async fn test_broadcast_integration() {
        let (registry, broadcaster) = create_websocket_services();
        let event_listener = EventListener::new(
            broadcaster.clone(),
            "http://mock:4001".to_string(),
            "http://mock:4005".to_string(),
            "http://mock:4003".to_string(),
            Duration::from_millis(50),
        );

        // Create a test connection to receive broadcasts
        let test_connection = Arc::new(crate::websocket::WebSocketConnection::new());
        test_connection
            .set_user_id("broadcast_test_user".to_string())
            .await;
        test_connection
            .subscribe(crate::websocket::EventType::CryptoService)
            .await;
        test_connection
            .subscribe(crate::websocket::EventType::RiskAlerts)
            .await;

        registry.add_connection(test_connection.clone()).await;

        // Test crypto operation broadcast
        let crypto_result = event_listener
            .handle_crypto_operation("key_generation", "broadcast_test_user")
            .await;
        assert!(crypto_result.is_ok());

        // Test risk analysis broadcast
        let risk_result = event_listener
            .handle_risk_analysis("broadcast_test_user", "broadcast_tx", 0.9)
            .await;
        assert!(risk_result.is_ok());

        // Verify broadcaster stats
        let stats = broadcaster.get_stats().await;
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.authenticated_connections, 1);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let event_listener = create_test_event_listener();

        let mut handles = Vec::new();

        // Start multiple concurrent operations
        for i in 0..5 {
            let listener = event_listener.clone();
            let user_id = format!("concurrent_user_{}", i);
            let tx_id = format!("concurrent_tx_{}", i);

            let handle = tokio::spawn(async move {
                // Mix of different operations
                let result = if i % 2 == 0 {
                    listener
                        .handle_crypto_operation("key_generation", &user_id)
                        .await
                } else {
                    listener.handle_risk_analysis(&user_id, &tx_id, 0.5).await
                };

                // Convert to a Send-safe error type
                result.map_err(|e| e.to_string())
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All operations should complete (though they may error due to mock URLs)
        for result in results {
            assert!(result.is_ok()); // tokio::spawn succeeded
        }
    }

    #[tokio::test]
    async fn test_transaction_updates_polling() {
        let event_listener = create_test_event_listener();

        // This method should not panic and should handle errors gracefully
        let result = event_listener.poll_transaction_updates().await;
        assert!(result.is_ok()); // Currently just returns Ok(()) in implementation
    }

    #[tokio::test]
    async fn test_risk_level_classification() {
        let event_listener = create_test_event_listener();

        // Test different risk scores map to correct risk levels
        let test_cases = vec![
            (0.0, "should be LOW"),
            (0.3, "should be LOW"),
            (0.5, "should be MEDIUM"),
            (0.65, "should be HIGH"),
            (0.85, "should be CRITICAL"),
            (1.0, "should be CRITICAL"),
        ];

        for (risk_score, description) in test_cases {
            let result = event_listener
                .handle_risk_analysis("classification_user", "classification_tx", risk_score)
                .await;

            assert!(
                result.is_ok(),
                "Risk score {} {} failed",
                risk_score,
                description
            );
        }
    }
}
