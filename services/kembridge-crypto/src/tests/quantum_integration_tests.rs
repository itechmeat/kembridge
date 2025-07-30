//! Integration tests for Phase 8.1.2 Quantum Cryptography Full Integration
//! 
//! These tests verify that all quantum cryptography components work together
//! correctly in realistic scenarios and provide comprehensive coverage of
//! the post-quantum crypto functionality for KEMBridge operations.

use super::super::*;
use uuid::Uuid;
use std::collections::HashMap;

// Integration test constants - using values from backend constants
const TEST_ETH_CHAIN: &str = "ethereum";
const TEST_NEAR_CHAIN: &str = "near";
const TEST_BSC_CHAIN: &str = "bsc";
const TEST_TRANSACTION_ID: &str = "0xabcdef1234567890abcdef1234567890abcdef12";
const TEST_USER_WALLET_ETH: &str = "0x742d35Cc6664C0532C14f59Ba3Dc27C4E2E4e6e8";
const TEST_USER_WALLET_NEAR: &str = "alice.testnet";
const TEST_AMOUNT_ETH: &str = "1.5";
const TEST_AMOUNT_NEAR: &str = "150.0";
const TEST_BRIDGE_FEE_PERCENTAGE: f64 = 0.15; // From constants.rs

#[cfg(test)]
mod quantum_integration_tests {
    use super::*;

    fn create_realistic_transaction_data() -> SensitiveTransactionData {
        let mut metadata = HashMap::new();
        metadata.insert("gas_limit".to_string(), "150000".to_string());
        metadata.insert("gas_price_gwei".to_string(), "25".to_string());
        metadata.insert("nonce".to_string(), "42".to_string());
        metadata.insert("bridge_fee_percentage".to_string(), TEST_BRIDGE_FEE_PERCENTAGE.to_string());
        metadata.insert("slippage_percentage".to_string(), "0.5".to_string());

        SensitiveTransactionData {
            from_address: TEST_USER_WALLET_ETH.to_string(),
            to_address: TEST_USER_WALLET_NEAR.to_string(),
            from_amount: TEST_AMOUNT_ETH.to_string(),
            to_amount: TEST_AMOUNT_NEAR.to_string(),
            from_token: "ETH".to_string(),
            to_token: "NEAR".to_string(),
            metadata,
        }
    }

    #[test]
    fn test_full_bridge_transaction_quantum_protection() {
        // Phase 1: Generate quantum key pair
        let mut key_manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let ml_kem_public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();
        let quantum_key_id = Uuid::new_v4();

        // Phase 2: Derive operation-specific keys for bridge transaction
        let operation_manager = OperationKeyManager;
        let bridge_keys = operation_manager.derive_bridge_keys(
            &[0u8; 32], // Simplified shared secret for test
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
        ).unwrap();

        // Phase 3: Encrypt transaction data
        let transaction_data = create_realistic_transaction_data();
        let quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
            &ml_kem_public_key,
            &transaction_data,
            quantum_key_id,
        ).unwrap();

        // Phase 4: Create authenticated cross-chain message for transaction confirmation
        let authenticator = CrossChainAuthenticator;
        let confirmation_payload = format!(
            "{{\"transaction_id\":\"{}\",\"quantum_key_id\":\"{}\",\"status\":\"pending\"}}",
            TEST_TRANSACTION_ID, quantum_key_id
        );

        let auth_message = authenticator.create_transaction_confirmation(
            &ml_kem_public_key,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            TEST_TRANSACTION_ID,
            confirmation_payload.as_bytes(),
        ).unwrap();

        // Phase 5: Verify all components work together
        assert_eq!(quantum_transaction.quantum_key_id, quantum_key_id);
        assert_eq!(bridge_keys.context.operation_type, OperationType::BridgeTransaction);
        assert_eq!(auth_message.message_type, CrossChainMessageType::TransactionConfirmation);
        assert_eq!(auth_message.metadata["transaction_id"], TEST_TRANSACTION_ID);

        // Phase 6: Verify message integrity
        let verification_result = authenticator.verify_message_integrity(&auth_message).unwrap();
        assert!(verification_result.is_valid);
    }

    #[test]
    fn test_multi_chain_quantum_workflow() {
        let mut key_manager = QuantumKeyManager::new();
        let operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;

        // Test multiple chain combinations
        let chain_pairs = vec![
            (TEST_ETH_CHAIN, TEST_NEAR_CHAIN),
            (TEST_ETH_CHAIN, TEST_BSC_CHAIN),
            (TEST_NEAR_CHAIN, TEST_BSC_CHAIN),
        ];

        for (from_chain, to_chain) in chain_pairs {
            // Generate unique keypair for each chain pair
            let user_id = Uuid::new_v4();
            let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
            let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();
            let shared_secret = [0u8; 32]; // Simplified for test

            // Derive keys for specific chain pair
            let bridge_keys = operation_manager.derive_bridge_keys(
                &shared_secret,
                from_chain,
                to_chain,
            ).unwrap();

            // Create cross-chain message
            let message_payload = format!("Bridge operation from {} to {}", from_chain, to_chain);
            let auth_message = authenticator.create_authenticated_message(
                &public_key,
                CrossChainMessageType::StateSync,
                message_payload.as_bytes(),
                Some(300), // 5 minutes
            ).unwrap();

            // Verify uniqueness across chains
            assert!(bridge_keys.context.metadata.contains_key("from_chain"));
            assert!(bridge_keys.context.metadata.contains_key("to_chain"));
            assert_eq!(bridge_keys.context.metadata["from_chain"], from_chain);
            assert_eq!(bridge_keys.context.metadata["to_chain"], to_chain);

            // Verify message integrity
            let verification = authenticator.verify_message_integrity(&auth_message).unwrap();
            assert!(verification.is_valid);
        }
    }

    #[test]
    fn test_user_authentication_to_transaction_flow() {
        let mut key_manager = QuantumKeyManager::new();
        let operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;

        // Phase 1: User authentication
        let user_id = Uuid::new_v4();
        let auth_key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let _auth_public_key = key_manager.get_quantum_public_key(auth_key_id).unwrap().to_vec();
        let shared_secret = [1u8; 32]; // Different secret for auth

        let user_auth_keys = operation_manager.derive_user_auth_keys(
            &shared_secret,
            TEST_USER_WALLET_ETH,
            TEST_ETH_CHAIN,
        ).unwrap();

        // Phase 2: Bridge transaction with different keys
        let tx_key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let tx_public_key = key_manager.get_quantum_public_key(tx_key_id).unwrap().to_vec();
        let quantum_key_id = Uuid::new_v4();

        let transaction_data = create_realistic_transaction_data();
        let quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
            &tx_public_key,
            &transaction_data,
            quantum_key_id,
        ).unwrap();

        // Phase 3: Cross-chain confirmation
        let confirmation_message = authenticator.create_transaction_confirmation(
            &tx_public_key,
            TEST_ETH_CHAIN,
            TEST_NEAR_CHAIN,
            TEST_TRANSACTION_ID,
            b"Transaction completed successfully",
        ).unwrap();

        // Verify different operation types have different keys
        assert_eq!(user_auth_keys.context.operation_type, OperationType::UserAuthentication);
        assert_ne!(user_auth_keys.encryption_key, quantum_transaction.encrypted_ciphertext[..32]);
        
        // Verify transaction and confirmation are linked
        assert_eq!(confirmation_message.metadata["transaction_id"], TEST_TRANSACTION_ID);
        
        let verification = authenticator.verify_message_integrity(&confirmation_message).unwrap();
        assert!(verification.is_valid);
    }

    #[test]
    fn test_security_alert_quantum_workflow() {
        let mut key_manager = QuantumKeyManager::new();
        let authenticator = CrossChainAuthenticator;

        // Generate keypair for security alerts
        let user_id = Uuid::new_v4();
        let security_key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let security_public_key = key_manager.get_quantum_public_key(security_key_id).unwrap().to_vec();

        // Test different severity levels
        let alert_scenarios = vec![
            (AlertSeverity::Low, "Unusual transaction pattern detected"),
            (AlertSeverity::Medium, "Multiple failed authentication attempts"),
            (AlertSeverity::High, "Suspicious cross-chain activity"),
            (AlertSeverity::Critical, "Potential quantum attack detected"),
        ];

        for (severity, alert_message) in alert_scenarios {
            let security_alert = authenticator.create_security_alert(
                &security_public_key,
                TEST_ETH_CHAIN,
                severity.clone(),
                alert_message.as_bytes(),
            ).unwrap();

            // Verify alert properties
            assert_eq!(security_alert.message_type, CrossChainMessageType::SecurityAlert);
            assert!(security_alert.metadata.contains_key("alert_severity"));
            assert_eq!(security_alert.metadata["alert_severity"], format!("{:?}", severity));

            // Critical alerts should be verified immediately
            let verification = authenticator.verify_message_integrity(&security_alert).unwrap();
            assert!(verification.is_valid);
        }
    }

    #[test]
    fn test_wallet_address_protection_workflow() {
        let mut key_manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();
        let quantum_key_id = Uuid::new_v4();

        // Test protecting various wallet address formats
        let wallet_addresses = vec![
            TEST_USER_WALLET_ETH.to_string(),
            TEST_USER_WALLET_NEAR.to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "bob.testnet".to_string(),
            "alice.near".to_string(),
        ];

        let protected_addresses = QuantumTransactionCrypto::encrypt_wallet_addresses(
            &public_key,
            &wallet_addresses,
            quantum_key_id,
        ).unwrap();

        // Verify all addresses are protected
        assert_eq!(protected_addresses.encrypted_addresses.len(), wallet_addresses.len());
        assert_eq!(protected_addresses.quantum_key_id, quantum_key_id);
        
        // Verify each encrypted address is non-empty and different
        let mut encrypted_values = std::collections::HashSet::new();
        for encrypted_addr in &protected_addresses.encrypted_addresses {
            assert!(!encrypted_addr.encrypted_address.is_empty());
            assert!(encrypted_values.insert(encrypted_addr.encrypted_address.clone()));
        }
    }

    #[test]
    fn test_transaction_amount_protection_workflow() {
        let mut key_manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();
        let quantum_key_id = Uuid::new_v4();

        // Test protecting various transaction amounts
        let transaction_amounts = vec![
            ("ETH".to_string(), "0.001".to_string()),
            ("NEAR".to_string(), "100.0".to_string()),
            ("USDT".to_string(), "1500.50".to_string()),
            ("BTC".to_string(), "0.05".to_string()),
        ];

        let encrypted_amounts = QuantumTransactionCrypto::encrypt_transaction_amounts(
            &public_key,
            &transaction_amounts,
            quantum_key_id,
        ).unwrap();

        // Verify all amounts are protected
        assert_eq!(encrypted_amounts.len(), transaction_amounts.len());
        
        for (i, encrypted_amount) in encrypted_amounts.iter().enumerate() {
            assert_eq!(encrypted_amount.token_symbol, transaction_amounts[i].0);
            assert!(!encrypted_amount.encrypted_amount.is_empty());
            assert!(encrypted_amount.timestamp > 0);
        }
    }

    #[test]
    fn test_event_data_quantum_protection() {
        let mut key_manager = QuantumKeyManager::new();
        let operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;

        let shared_secret = [2u8; 32]; // Different secret for events
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();

        // Test different event types
        let event_types = vec![
            "transaction_confirmed",
            "bridge_locked",
            "bridge_unlocked",
            "user_authenticated",
            "security_alert_triggered",
        ];

        for event_type in event_types {
            // Derive event-specific keys
            let event_keys = operation_manager.derive_event_keys(
                &shared_secret,
                TEST_ETH_CHAIN,
                event_type,
            ).unwrap();

            // Create authenticated event message
            let event_payload = format!(
                "{{\"event_type\":\"{}\",\"chain\":\"{}\",\"timestamp\":{}}}",
                event_type,
                TEST_ETH_CHAIN,
                chrono::Utc::now().timestamp()
            );

            let event_message = authenticator.create_authenticated_message(
                &public_key,
                CrossChainMessageType::EventNotification,
                event_payload.as_bytes(),
                Some(60), // 1 minute validity
            ).unwrap();

            // Verify event-specific properties
            assert_eq!(event_keys.context.operation_type, OperationType::EventData);
            assert_eq!(event_keys.context.metadata["event_type"], event_type);
            assert_eq!(event_message.message_type, CrossChainMessageType::EventNotification);
            
            let verification = authenticator.verify_message_integrity(&event_message).unwrap();
            assert!(verification.is_valid);
        }
    }

    #[test]
    fn test_state_synchronization_quantum_workflow() {
        let mut key_manager = QuantumKeyManager::new();
        let operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;

        let shared_secret = [3u8; 32]; // Different secret for state sync
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();

        // Test state sync between different chains
        let chain_combinations = vec![
            (TEST_ETH_CHAIN, TEST_NEAR_CHAIN),
            (TEST_NEAR_CHAIN, TEST_ETH_CHAIN),
            (TEST_ETH_CHAIN, TEST_BSC_CHAIN),
        ];

        for (from_chain, to_chain) in chain_combinations {
            // Derive state sync keys
            let sync_keys = operation_manager.derive_state_sync_keys(
                &shared_secret,
                from_chain,
                to_chain,
            ).unwrap();

            // Create state sync message
            let sync_payload = format!(
                "{{\"from_chain\":\"{}\",\"to_chain\":\"{}\",\"block_height\":12345,\"state_root\":\"0xabc123\"}}",
                from_chain, to_chain
            );

            let sync_message = authenticator.create_authenticated_message(
                &public_key,
                CrossChainMessageType::StateSync,
                sync_payload.as_bytes(),
                Some(120), // 2 minutes validity
            ).unwrap();

            // Verify state sync properties
            assert_eq!(sync_keys.context.operation_type, OperationType::StateSync);
            assert_eq!(sync_keys.context.metadata["from_chain"], from_chain);
            assert_eq!(sync_keys.context.metadata["to_chain"], to_chain);
            assert_eq!(sync_message.message_type, CrossChainMessageType::StateSync);
            
            let verification = authenticator.verify_message_integrity(&sync_message).unwrap();
            assert!(verification.is_valid);
        }
    }

    #[test]
    fn test_concurrent_quantum_operations() {
        use std::thread;
        use std::sync::Arc;

        let key_manager = Arc::new(std::sync::Mutex::new(QuantumKeyManager::new()));
        let operation_manager = Arc::new(OperationKeyManager);
        let authenticator = Arc::new(CrossChainAuthenticator);
        let mut handles = vec![];

        // Spawn multiple threads performing different quantum operations
        for i in 0..8 {
            let km_clone = Arc::clone(&key_manager);
            let om_clone = Arc::clone(&operation_manager);
            let auth_clone = Arc::clone(&authenticator);
            
            let handle = thread::spawn(move || {
                // Each thread performs a complete quantum workflow
                let user_id = Uuid::new_v4();
                let key_id = km_clone.lock().unwrap().generate_quantum_keypair(user_id).unwrap();
                let public_key = km_clone.lock().unwrap().get_quantum_public_key(key_id).unwrap().to_vec();
                let shared_secret = [(i as u8); 32];
                let quantum_key_id = Uuid::new_v4();

                // Operation-specific key derivation
                let user_id = format!("user_{}", i);
                let auth_keys = om_clone.derive_user_auth_keys(
                    &shared_secret,
                    &user_id,
                    TEST_ETH_CHAIN,
                ).unwrap();

                // Transaction encryption
                let mut transaction_data = create_realistic_transaction_data();
                transaction_data.from_address = format!("0x{:040x}", i);
                
                let quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
                    &public_key,
                    &transaction_data,
                    quantum_key_id,
                ).unwrap();

                // Cross-chain authentication
                let message_payload = format!("Concurrent operation {}", i);
                let auth_message = auth_clone.create_authenticated_message(
                    &public_key,
                    CrossChainMessageType::TransactionConfirmation,
                    message_payload.as_bytes(),
                    Some(300),
                ).unwrap();

                // Verify everything works
                assert_eq!(auth_keys.context.metadata["user_id"], user_id);
                assert_eq!(quantum_transaction.quantum_key_id, quantum_key_id);
                
                let verification = auth_clone.verify_message_integrity(&auth_message).unwrap();
                assert!(verification.is_valid);

                (quantum_key_id, user_id)
            });
            handles.push(handle);
        }

        // Collect results and verify uniqueness
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Verify all operations produced unique results
        let mut key_ids = std::collections::HashSet::new();
        let mut user_ids = std::collections::HashSet::new();
        
        for (key_id, user_id) in results {
            assert!(key_ids.insert(key_id), "Quantum key IDs should be unique");
            assert!(user_ids.insert(user_id), "User IDs should be unique");
        }
    }

    #[test]
    fn test_performance_full_quantum_workflow() {
        use std::time::Instant;

        let mut key_manager = QuantumKeyManager::new();
        let operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;
        let iterations = 50;

        let start = Instant::now();
        
        for i in 0..iterations {
            // Complete quantum workflow
            let user_id = Uuid::new_v4();
            let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
            let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();
            let shared_secret = [(i as u8); 32];
            let quantum_key_id = Uuid::new_v4();

            // Key derivation
            let _bridge_keys = operation_manager.derive_bridge_keys(
                &shared_secret,
                TEST_ETH_CHAIN,
                TEST_NEAR_CHAIN,
            ).unwrap();

            // Transaction encryption
            let transaction_data = create_realistic_transaction_data();
            let _quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
                &public_key,
                &transaction_data,
                quantum_key_id,
            ).unwrap();

            // Message authentication
            let message = format!("Performance test {}", i);
            let auth_message = authenticator.create_authenticated_message(
                &public_key,
                CrossChainMessageType::TransactionConfirmation,
                message.as_bytes(),
                Some(300),
            ).unwrap();

            // Verification
            let verification = authenticator.verify_message_integrity(&auth_message).unwrap();
            assert!(verification.is_valid);
        }

        let duration = start.elapsed();
        let avg_workflow_time = duration / iterations;

        // Full quantum workflow should complete reasonably fast (< 50ms)
        assert!(
            avg_workflow_time.as_millis() < 50,
            "Full quantum workflow too slow: {:?}",
            avg_workflow_time
        );

        println!("Full quantum workflow performance: {:?} avg", avg_workflow_time);
    }

    #[test]
    fn test_quantum_crypto_error_handling_integration() {
        let mut key_manager = QuantumKeyManager::new();
        let _operation_manager = OperationKeyManager;
        let authenticator = CrossChainAuthenticator;

        // Test with invalid ML-KEM key
        let invalid_key = vec![0u8; 10];
        let quantum_key_id = Uuid::new_v4();
        let transaction_data = create_realistic_transaction_data();

        let result = QuantumTransactionCrypto::encrypt_transaction_data(
            &invalid_key,
            &transaction_data,
            quantum_key_id,
        );
        assert!(result.is_err());

        let result = authenticator.create_authenticated_message(
            &invalid_key,
            CrossChainMessageType::TransactionConfirmation,
            b"test",
            Some(300),
        );
        assert!(result.is_err());

        // Test with valid keys but corrupted data
        let user_id = Uuid::new_v4();
        let key_id = key_manager.generate_quantum_keypair(user_id).unwrap();
        let public_key = key_manager.get_quantum_public_key(key_id).unwrap().to_vec();

        let mut auth_message = authenticator.create_authenticated_message(
            &public_key,
            CrossChainMessageType::TransactionConfirmation,
            b"test message",
            Some(300),
        ).unwrap();

        // Corrupt the signature
        if !auth_message.signature.quantum_signature.is_empty() {
            auth_message.signature.quantum_signature[0] = 
                auth_message.signature.quantum_signature[0].wrapping_add(1);
        }

        let verification = authenticator.verify_message_integrity(&auth_message).unwrap();
        assert!(!verification.is_valid);
        assert!(verification.error_message.is_some());
    }
}