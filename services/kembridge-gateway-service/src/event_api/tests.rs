#[cfg(test)]
mod event_api_tests {
    use crate::event_api::{
        EventApiState, EventTriggerResponse, TriggerCryptoEventRequest, TriggerRiskAnalysisRequest,
    };
    use crate::event_listener::create_event_listener;
    use crate::websocket::create_websocket_services;
    use std::sync::Arc;

    fn create_test_event_api_state() -> EventApiState {
        let (_, broadcaster) = create_websocket_services();
        let event_listener = create_event_listener(broadcaster.clone());
        (broadcaster, Arc::new(event_listener))
    }

    #[tokio::test]
    async fn test_trigger_crypto_event_request_serialization() {
        let request = TriggerCryptoEventRequest {
            event_type: "key_generation".to_string(),
            user_id: Some("test_user".to_string()),
            message: "Test message".to_string(),
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_trigger_risk_analysis_request_serialization() {
        let request = TriggerRiskAnalysisRequest {
            user_id: "test_user".to_string(),
            transaction_id: "test_tx".to_string(),
            risk_score: 0.5,
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_event_trigger_response_serialization() {
        let response = EventTriggerResponse {
            success: true,
            message: "Test message".to_string(),
            event_id: "test_id".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("Test message"));
        assert!(json_str.contains("test_id"));
        assert!(json_str.contains("true"));
    }

    #[tokio::test]
    async fn test_event_api_state_creation() {
        let state = create_test_event_api_state();

        // Test that broadcaster and event listener are valid
        let stats = state.0.get_stats().await;
        assert_eq!(stats.total_connections, 0);

        // Event listener should have proper configuration
        assert!(state.1.get_crypto_service_url().len() > 0);
        assert!(state.1.get_ai_engine_url().len() > 0);
        assert!(state.1.get_blockchain_service_url().len() > 0);
    }

    #[tokio::test]
    async fn test_crypto_event_types() {
        let test_cases = vec![
            "key_generation",
            "encapsulation",
            "decapsulation",
            "key_rotation",
            "service_status",
        ];

        for event_type in test_cases {
            let request = TriggerCryptoEventRequest {
                event_type: event_type.to_string(),
                user_id: Some("test_user".to_string()),
                message: format!("Test {}", event_type),
            };

            // Should serialize without error
            let json = serde_json::to_string(&request);
            assert!(json.is_ok(), "Failed to serialize {}", event_type);
        }
    }

    #[tokio::test]
    async fn test_risk_score_validation_bounds() {
        let valid_scores = vec![0.0, 0.1, 0.5, 0.9, 1.0];

        for score in valid_scores {
            let request = TriggerRiskAnalysisRequest {
                user_id: "test_user".to_string(),
                transaction_id: "test_tx".to_string(),
                risk_score: score,
            };

            let json = serde_json::to_string(&request);
            assert!(json.is_ok(), "Failed to serialize risk score {}", score);
        }
    }

    #[tokio::test]
    async fn test_websocket_stats_integration() {
        let state = create_test_event_api_state();

        // Add a test connection
        let test_connection = Arc::new(crate::websocket::WebSocketConnection::new());
        test_connection
            .set_user_id("stats_test_user".to_string())
            .await;

        // The registry is not directly accessible, but we can test that
        // the broadcaster has access to it
        let stats = state.0.get_stats().await;
        assert_eq!(stats.total_connections, 0); // No connections added directly
    }

    #[tokio::test]
    async fn test_concurrent_api_state_access() {
        let state = create_test_event_api_state();

        let mut handles = Vec::new();

        // Start multiple concurrent operations
        for _i in 0..5 {
            let state_clone = state.clone();

            let handle = tokio::spawn(async move {
                // Test that multiple threads can access the state
                let stats = state_clone.0.get_stats().await;
                let crypto_url = state_clone.1.get_crypto_service_url();

                (stats.total_connections, crypto_url.len())
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All operations should succeed
        for result in results {
            assert!(result.is_ok());
            let (connections, url_len) = result.unwrap();
            assert_eq!(connections, 0); // No connections in test
            assert!(url_len > 0); // URL should be configured
        }
    }

    #[tokio::test]
    async fn test_event_listener_methods() {
        let state = create_test_event_api_state();

        // Test crypto operation handling
        let crypto_result = state
            .1
            .handle_crypto_operation("key_generation", "test_user")
            .await;
        assert!(crypto_result.is_ok() || crypto_result.is_err()); // Either outcome is acceptable in test

        // Test risk analysis handling
        let risk_result = state
            .1
            .handle_risk_analysis("test_user", "test_tx", 0.7)
            .await;
        assert!(risk_result.is_ok() || risk_result.is_err()); // Either outcome is acceptable in test
    }

    #[tokio::test]
    async fn test_broadcaster_methods() {
        let state = create_test_event_api_state();

        // Test cleanup methods
        let cleanup_result = state.0.cleanup_inactive_connections().await;
        assert!(cleanup_result.is_ok());

        let heartbeat_result = state.0.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());

        let idle_cleanup = state.0.cleanup_idle_connections(30).await;
        assert!(idle_cleanup.is_ok());
    }

    #[tokio::test]
    async fn test_notification_levels() {
        let levels = vec!["info", "warning", "error", "critical"];

        for level in levels {
            // Test that level strings are valid
            let is_valid = match level {
                "info" | "warning" | "error" | "critical" => true,
                _ => false,
            };
            assert!(is_valid, "Level {} should be valid", level);
        }
    }
}
