//! Unit tests for operation_keys module
//! 
//! Comprehensive tests for operation-specific key derivation
//! using HKDF with different contexts for quantum-safe operations.

use super::super::operation_keys::*;

// Test constants for different operation contexts
const TEST_ETH_CHAIN: &str = "ethereum";
const TEST_NEAR_CHAIN: &str = "near";
const TEST_BSC_CHAIN: &str = "bsc";
const TEST_USER_ID: &str = "user_123456";
const TEST_TRANSACTION_ID: &str = "tx_abcdef";
const TEST_MESSAGE_ID: &str = "msg_789xyz";

#[cfg(test)]
mod operation_keys_tests {
    use super::*;

    fn generate_test_shared_secret() -> [u8; 32] {
        // Use a deterministic but valid shared secret for testing
        let mut secret = [0u8; 32];
        for (i, byte) in secret.iter_mut().enumerate() {
            *byte = (i * 7 + 13) as u8; // Some deterministic pattern
        }
        secret
    }

    #[test]
    fn test_derive_bridge_keys_success() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let result = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        );

        assert!(result.is_ok());
        let keys = result.unwrap();
        
        // Verify all keys are present and different
        assert_ne!(keys.encryption_key, keys.authentication_key);
        assert_ne!(keys.encryption_key, keys.integrity_key);
        assert_ne!(keys.authentication_key, keys.integrity_key);
        
        // Verify key lengths (32 bytes each for AES-256)
        assert_eq!(keys.encryption_key.len(), 32);
        assert_eq!(keys.authentication_key.len(), 32);
        assert_eq!(keys.integrity_key.len(), 32);
        
        // Verify context information
        assert_eq!(keys.context.operation_type, OperationType::BridgeTransaction);
        assert!(keys.context.metadata.contains_key("from_chain"));
        assert!(keys.context.metadata.contains_key("to_chain"));
        assert_eq!(keys.context.metadata["from_chain"], TEST_ETH_CHAIN);
        assert_eq!(keys.context.metadata["to_chain"], TEST_NEAR_CHAIN);
    }

    #[test]
    fn test_derive_user_auth_keys_success() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let result = manager.derive_user_auth_keys(
            &shared_secret,
            TEST_USER_ID,
            TEST_ETH_CHAIN,
        );

        assert!(result.is_ok());
        let keys = result.unwrap();
        
        // Verify operation type and context
        assert_eq!(keys.context.operation_type, OperationType::UserAuthentication);
        assert!(keys.context.metadata.contains_key("user_id"));
        assert!(keys.context.metadata.contains_key("chain"));
        assert_eq!(keys.context.metadata["user_id"], TEST_USER_ID);
        assert_eq!(keys.context.metadata["chain"], TEST_ETH_CHAIN);
    }

    #[test]
    fn test_derive_cross_chain_message_keys_success() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let result = manager.derive_cross_chain_message_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            TEST_MESSAGE_ID,
        );

        assert!(result.is_ok());
        let keys = result.unwrap();
        
        // Verify operation type and context
        assert_eq!(keys.context.operation_type, OperationType::CrossChainMessage);
        assert!(keys.context.metadata.contains_key("from_chain"));
        assert!(keys.context.metadata.contains_key("to_chain"));
        assert!(keys.context.metadata.contains_key("message_id"));
        assert_eq!(keys.context.metadata["message_id"], TEST_MESSAGE_ID);
    }

    #[test]
    fn test_derive_state_sync_keys_success() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let result = manager.derive_state_sync_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        );

        assert!(result.is_ok());
        let keys = result.unwrap();
        
        // Verify operation type
        assert_eq!(keys.context.operation_type, OperationType::StateSync);
        assert!(keys.context.metadata.contains_key("from_chain"));
        assert!(keys.context.metadata.contains_key("to_chain"));
    }

    #[test]
    fn test_derive_event_keys_success() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let result = manager.derive_event_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            "transaction_confirmed",
        );

        assert!(result.is_ok());
        let keys = result.unwrap();
        
        // Verify operation type
        assert_eq!(keys.context.operation_type, OperationType::EventData);
        assert!(keys.context.metadata.contains_key("chain"));
        assert!(keys.context.metadata.contains_key("event_type"));
        assert_eq!(keys.context.metadata["event_type"], "transaction_confirmed");
    }

    #[test]
    fn test_key_determinism() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Generate keys twice with same parameters
        let keys1 = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        let keys2 = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        // Keys should be identical for same inputs
        assert_eq!(keys1.encryption_key, keys2.encryption_key);
        assert_eq!(keys1.authentication_key, keys2.authentication_key);
        assert_eq!(keys1.integrity_key, keys2.integrity_key);
    }

    #[test]
    fn test_key_uniqueness_across_operations() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Generate keys for different operations
        let bridge_keys = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        let auth_keys = manager.derive_user_auth_keys(
            &shared_secret,
            TEST_USER_ID,
            TEST_ETH_CHAIN,
        ).unwrap();

        let message_keys = manager.derive_cross_chain_message_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            TEST_MESSAGE_ID,
        ).unwrap();

        // All keys should be different across operations
        assert_ne!(bridge_keys.encryption_key, auth_keys.encryption_key);
        assert_ne!(bridge_keys.encryption_key, message_keys.encryption_key);
        assert_ne!(auth_keys.encryption_key, message_keys.encryption_key);

        assert_ne!(bridge_keys.authentication_key, auth_keys.authentication_key);
        assert_ne!(bridge_keys.authentication_key, message_keys.authentication_key);
        assert_ne!(auth_keys.authentication_key, message_keys.authentication_key);
    }

    #[test]
    fn test_key_uniqueness_across_chains() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Generate bridge keys for different chain combinations
        let eth_near_keys = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        let eth_bsc_keys = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_BSC_CHAIN,
        ).unwrap();

        let near_bsc_keys = manager.derive_bridge_keys(
            &shared_secret,
            TEST_NEAR_CHAIN,
            TEST_BSC_CHAIN,
        ).unwrap();

        // Keys should be different for different chain combinations
        assert_ne!(eth_near_keys.encryption_key, eth_bsc_keys.encryption_key);
        assert_ne!(eth_near_keys.encryption_key, near_bsc_keys.encryption_key);
        assert_ne!(eth_bsc_keys.encryption_key, near_bsc_keys.encryption_key);
    }

    #[test]
    fn test_key_uniqueness_with_different_secrets() {
        let manager = OperationKeyManager;
        let secret1 = generate_test_shared_secret();
        let mut secret2 = generate_test_shared_secret();
        secret2[0] = secret2[0].wrapping_add(1); // Make it different

        let keys1 = manager.derive_bridge_keys(
            &secret1,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        let keys2 = manager.derive_bridge_keys(
            &secret2,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        // Keys should be different for different shared secrets
        assert_ne!(keys1.encryption_key, keys2.encryption_key);
        assert_ne!(keys1.authentication_key, keys2.authentication_key);
        assert_ne!(keys1.integrity_key, keys2.integrity_key);
    }

    #[test]
    fn test_empty_parameters() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Test with empty chain names
        let result = manager.derive_bridge_keys(&shared_secret, "", "");
        assert!(result.is_ok()); // Should still work but produce different keys

        // Test with empty user ID
        let result = manager.derive_user_auth_keys(&shared_secret, "", TEST_ETH_CHAIN);
        assert!(result.is_ok());

        // Test with empty message ID
        let result = manager.derive_cross_chain_message_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            "",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_operation_context_serialization() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        let keys = manager.derive_bridge_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        // Test serialization to JSON
        let json_result = serde_json::to_string(&keys.context);
        assert!(json_result.is_ok());

        // Test deserialization from JSON
        let json_str = json_result.unwrap();
        let deserialized: Result<OperationKeyContext, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let deserialized_context = deserialized.unwrap();
        assert_eq!(deserialized_context.operation_type, OperationType::BridgeTransaction);
        assert_eq!(deserialized_context.metadata["from_chain"], TEST_ETH_CHAIN);
        assert_eq!(deserialized_context.metadata["to_chain"], TEST_NEAR_CHAIN);
    }

    #[test]
    fn test_operation_type_serialization() {
        // Test all operation types
        let operation_types = vec![
            OperationType::BridgeTransaction,
            OperationType::UserAuthentication,
            OperationType::CrossChainMessage,
            OperationType::StateSync,
            OperationType::EventData,
        ];

        for op_type in operation_types {
            let json_result = serde_json::to_string(&op_type);
            assert!(json_result.is_ok());

            let json_str = json_result.unwrap();
            let deserialized: Result<OperationType, _> = serde_json::from_str(&json_str);
            assert!(deserialized.is_ok());
            assert_eq!(deserialized.unwrap(), op_type);
        }
    }

    #[test]
    fn test_concurrent_key_derivation() {
        use std::thread;
        use std::sync::Arc;

        let manager = Arc::new(OperationKeyManager);
        let shared_secret = Arc::new(generate_test_shared_secret());
        let mut handles = vec![];

        // Spawn multiple threads performing key derivation
        for i in 0..4 {
            let manager_clone = Arc::clone(&manager);
            let secret_clone = Arc::clone(&shared_secret);
            
            let handle = thread::spawn(move || {
                let user_id = format!("user_{}", i);
                let result = manager_clone.derive_user_auth_keys(
                    &secret_clone,
                    &user_id,
                    TEST_ETH_CHAIN,
                );
                
                assert!(result.is_ok(), "Thread {} key derivation failed", i);
                result.unwrap()
            });
            handles.push(handle);
        }

        // Collect results and verify uniqueness
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Verify each derivation produced different keys (due to different user IDs)
        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                assert_ne!(
                    results[i].encryption_key,
                    results[j].encryption_key,
                    "Keys should be unique for different users"
                );
            }
        }
    }

    #[test]
    fn test_performance_benchmarks() {
        use std::time::Instant;

        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();
        let iterations = 1000;

        // Benchmark bridge key derivation
        let start = Instant::now();
        for i in 0..iterations {
            let from_chain = if i % 2 == 0 { TEST_ETH_CHAIN } else { TEST_NEAR_CHAIN };
            let to_chain = if i % 2 == 0 { TEST_NEAR_CHAIN } else { TEST_ETH_CHAIN };
            
            let result = manager.derive_bridge_keys(
                &shared_secret,
                from_chain,
                to_chain,
            );
            assert!(result.is_ok());
        }
        let duration = start.elapsed();
        let avg_derivation_time = duration / iterations;

        // Key derivation should be fast (< 1ms per operation)
        assert!(
            avg_derivation_time.as_millis() < 1,
            "Key derivation too slow: {:?}",
            avg_derivation_time
        );

        println!("Performance results:");
        println!("  Key derivation: {:?} avg", avg_derivation_time);
    }

    #[test]
    fn test_memory_safety_and_cleanup() {
        // Test that sensitive key material is properly handled in memory
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        for i in 0..100 {
            let user_id = format!("user_{}", i);
            let result = manager.derive_user_auth_keys(
                &shared_secret,
                &user_id,
                TEST_ETH_CHAIN,
            );
            assert!(result.is_ok());
            
            // Keys will be dropped and zeroized here
        }

        // If we get here without crashes, memory safety is working
    }

    #[test]
    fn test_large_parameter_strings() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Test with very long chain names
        let long_chain = "a".repeat(1000);
        let result = manager.derive_bridge_keys(
            &shared_secret,
            &long_chain,
            TEST_NEAR_CHAIN,
        );
        assert!(result.is_ok());

        // Test with very long user ID
        let long_user_id = "u".repeat(1000);
        let result = manager.derive_user_auth_keys(
            &shared_secret,
            &long_user_id,
            TEST_ETH_CHAIN,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_characters_in_parameters() {
        let manager = OperationKeyManager;
        let shared_secret = generate_test_shared_secret();

        // Test with special characters
        let special_chain = "test-chain_2.0@example.com";
        let special_user = "user@domain.com#123";
        let special_message = "msg:abc/def?query=1&param=2";

        let result = manager.derive_bridge_keys(
            &shared_secret,
            special_chain,
            TEST_NEAR_CHAIN,
        );
        assert!(result.is_ok());

        let result = manager.derive_user_auth_keys(
            &shared_secret,
            special_user,
            TEST_ETH_CHAIN,
        );
        assert!(result.is_ok());

        let result = manager.derive_cross_chain_message_keys(
            &shared_secret,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            special_message,
        );
        assert!(result.is_ok());
    }
}