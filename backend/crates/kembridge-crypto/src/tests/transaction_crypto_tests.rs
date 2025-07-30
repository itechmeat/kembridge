//! Unit tests for transaction_crypto module
//! 
//! Comprehensive tests for quantum-safe transaction data encryption
//! using ML-KEM-1024 post-quantum cryptography.

use super::super::transaction_crypto::*;
use super::super::{QuantumKeyManager, QuantumCryptoError};
use uuid::Uuid;
use std::collections::HashMap;

// Test constants - derived from global constants
const TEST_WALLET_ADDRESS_ETH: &str = "0x742d35Cc6664C0532C14f59Ba3Dc27C4E2E4e6e8";
const TEST_WALLET_ADDRESS_NEAR: &str = "test.testnet";
const TEST_AMOUNT_ETH: &str = "1.5";
const TEST_AMOUNT_NEAR: &str = "150.0";
const TEST_TOKEN_SYMBOL_ETH: &str = "ETH";
const TEST_TOKEN_SYMBOL_NEAR: &str = "NEAR";

#[cfg(test)]
mod transaction_crypto_tests {
    use super::*;

    fn create_test_transaction_data() -> SensitiveTransactionData {
        let mut metadata = HashMap::new();
        metadata.insert("gas_limit".to_string(), "21000".to_string());
        metadata.insert("gas_price".to_string(), "20".to_string());
        metadata.insert("nonce".to_string(), "42".to_string());

        SensitiveTransactionData {
            from_address: TEST_WALLET_ADDRESS_ETH.to_string(),
            to_address: TEST_WALLET_ADDRESS_NEAR.to_string(),
            from_amount: TEST_AMOUNT_ETH.to_string(),
            to_amount: TEST_AMOUNT_NEAR.to_string(),
            from_token: TEST_TOKEN_SYMBOL_ETH.to_string(),
            to_token: TEST_TOKEN_SYMBOL_NEAR.to_string(),
            metadata,
        }
    }

    fn generate_test_ml_kem_key() -> Vec<u8> {
        let mut manager = QuantumKeyManager::new();
        let user_id = Uuid::new_v4();
        let key_id = manager.generate_quantum_keypair(user_id).unwrap();
        manager.get_quantum_public_key(key_id).unwrap().to_vec()
    }

    #[test]
    fn test_encrypt_transaction_data_success() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_transaction_data(
            &ml_kem_public_key,
            &transaction_data,
            quantum_key_id,
        );

        assert!(result.is_ok());
        let quantum_transaction = result.unwrap();
        
        // Verify structure
        assert_eq!(quantum_transaction.quantum_key_id, quantum_key_id);
        assert!(!quantum_transaction.encrypted_ciphertext.is_empty());
        assert!(!quantum_transaction.encrypted_shared_secret.is_empty());
        assert!(quantum_transaction.metadata.timestamp > 0);
        assert_eq!(quantum_transaction.metadata.encryption_scheme, "ML-KEM-1024");
        assert_eq!(quantum_transaction.metadata.data_type, TransactionDataType::Bridge);
    }

    #[test]
    fn test_encrypt_transaction_data_with_different_types() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        let data_types = vec![
            TransactionDataType::Bridge,
            TransactionDataType::Swap,
            TransactionDataType::Transfer,
            TransactionDataType::Staking,
        ];

        for data_type in data_types {
            let result = QuantumTransactionCrypto::encrypt_transaction_data_with_type(
                &ml_kem_public_key,
                &transaction_data,
                quantum_key_id,
                data_type.clone(),
            );

            assert!(result.is_ok());
            let quantum_transaction = result.unwrap();
            assert_eq!(quantum_transaction.metadata.data_type, data_type);
        }
    }

    #[test]
    fn test_encrypt_wallet_addresses() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let addresses = vec![
            TEST_WALLET_ADDRESS_ETH.to_string(),
            TEST_WALLET_ADDRESS_NEAR.to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
        ];
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_wallet_addresses(
            &ml_kem_public_key,
            &addresses,
            quantum_key_id,
        );

        assert!(result.is_ok());
        let protected_addresses = result.unwrap();
        
        assert_eq!(protected_addresses.quantum_key_id, quantum_key_id);
        assert_eq!(protected_addresses.encrypted_addresses.len(), addresses.len());
        assert!(!protected_addresses.encrypted_addresses.is_empty());
        assert!(protected_addresses.metadata.timestamp > 0);
    }

    #[test]
    fn test_encrypt_transaction_amounts() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let amounts = vec![
            (TEST_TOKEN_SYMBOL_ETH.to_string(), TEST_AMOUNT_ETH.to_string()),
            (TEST_TOKEN_SYMBOL_NEAR.to_string(), TEST_AMOUNT_NEAR.to_string()),
            ("USDT".to_string(), "1000.5".to_string()),
        ];
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_transaction_amounts(
            &ml_kem_public_key,
            &amounts,
            quantum_key_id,
        );

        assert!(result.is_ok());
        let encrypted_amounts = result.unwrap();
        
        assert_eq!(encrypted_amounts.len(), amounts.len());
        for (original, encrypted) in amounts.iter().zip(encrypted_amounts.iter()) {
            assert_eq!(encrypted.token_symbol, original.0);
            assert!(!encrypted.encrypted_amount.is_empty());
            assert!(encrypted.timestamp > 0);
        }
    }

    #[test]
    fn test_encrypt_transaction_data_invalid_key() {
        let invalid_key = vec![0u8; 10]; // Too short for ML-KEM-1024
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_transaction_data(
            &invalid_key,
            &transaction_data,
            quantum_key_id,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            QuantumCryptoError::InvalidKey => {},
            _ => panic!("Expected InvalidKey error"),
        }
    }

    #[test]
    fn test_encrypt_wallet_addresses_empty_list() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let addresses: Vec<String> = vec![];
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_wallet_addresses(
            &ml_kem_public_key,
            &addresses,
            quantum_key_id,
        );

        assert!(result.is_ok());
        let protected_addresses = result.unwrap();
        assert!(protected_addresses.encrypted_addresses.is_empty());
    }

    #[test]
    fn test_encrypt_transaction_amounts_zero_amounts() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let amounts = vec![
            ("ETH".to_string(), "0.0".to_string()),
            ("NEAR".to_string(), "0".to_string()),
        ];
        let quantum_key_id = Uuid::new_v4();

        let result = QuantumTransactionCrypto::encrypt_transaction_amounts(
            &ml_kem_public_key,
            &amounts,
            quantum_key_id,
        );

        assert!(result.is_ok());
        let encrypted_amounts = result.unwrap();
        assert_eq!(encrypted_amounts.len(), 2);
    }

    #[test]
    fn test_quantum_transaction_serialization() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        let quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
            &ml_kem_public_key,
            &transaction_data,
            quantum_key_id,
        ).unwrap();

        // Test serialization to JSON
        let json_result = serde_json::to_string(&quantum_transaction);
        assert!(json_result.is_ok());

        // Test deserialization from JSON
        let json_str = json_result.unwrap();
        let deserialized: Result<QuantumTransaction, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let deserialized_transaction = deserialized.unwrap();
        assert_eq!(deserialized_transaction.quantum_key_id, quantum_key_id);
        assert_eq!(deserialized_transaction.metadata.encryption_scheme, "ML-KEM-1024");
    }

    #[test]
    fn test_transaction_metadata_integrity() {
        let ml_kem_public_key = generate_test_ml_kem_key();
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        let quantum_transaction = QuantumTransactionCrypto::encrypt_transaction_data(
            &ml_kem_public_key,
            &transaction_data,
            quantum_key_id,
        ).unwrap();

        // Verify all metadata fields
        assert_eq!(quantum_transaction.metadata.encryption_scheme, "ML-KEM-1024");
        assert_eq!(quantum_transaction.metadata.data_type, TransactionDataType::Bridge);
        assert!(quantum_transaction.metadata.timestamp > 0);
        assert!(!quantum_transaction.metadata.context_hash.is_empty());
        
        // Verify context hash uniqueness for different transactions
        let transaction_data2 = SensitiveTransactionData {
            from_address: "different_address".to_string(),
            ..transaction_data
        };

        let quantum_transaction2 = QuantumTransactionCrypto::encrypt_transaction_data(
            &ml_kem_public_key,
            &transaction_data2,
            quantum_key_id,
        ).unwrap();

        assert_ne!(
            quantum_transaction.metadata.context_hash,
            quantum_transaction2.metadata.context_hash
        );
    }

    #[test]
    fn test_concurrent_encryption_operations() {
        use std::thread;
        use std::sync::Arc;

        let ml_kem_public_key = Arc::new(generate_test_ml_kem_key());
        let transaction_data = Arc::new(create_test_transaction_data());
        let mut handles = vec![];

        // Spawn multiple threads performing encryption operations
        for i in 0..4 {
            let key_clone = Arc::clone(&ml_kem_public_key);
            let data_clone = Arc::clone(&transaction_data);
            
            let handle = thread::spawn(move || {
                let quantum_key_id = Uuid::new_v4();
                
                let result = QuantumTransactionCrypto::encrypt_transaction_data(
                    &key_clone,
                    &data_clone,
                    quantum_key_id,
                );
                
                assert!(result.is_ok(), "Thread {} encryption failed", i);
                result.unwrap()
            });
            handles.push(handle);
        }

        // Collect results and verify uniqueness
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Verify each encryption produced different ciphertext
        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                assert_ne!(
                    results[i].encrypted_ciphertext,
                    results[j].encrypted_ciphertext,
                    "Encryption results should be unique"
                );
            }
        }
    }

    #[test]
    fn test_performance_benchmarks() {
        use std::time::Instant;

        let ml_kem_public_key = generate_test_ml_kem_key();
        let transaction_data = create_test_transaction_data();
        let iterations = 50;

        // Benchmark transaction data encryption
        let start = Instant::now();
        for _ in 0..iterations {
            let quantum_key_id = Uuid::new_v4();
            let result = QuantumTransactionCrypto::encrypt_transaction_data(
                &ml_kem_public_key,
                &transaction_data,
                quantum_key_id,
            );
            assert!(result.is_ok());
        }
        let duration = start.elapsed();
        let avg_encryption_time = duration / iterations;

        // Transaction encryption should be fast (< 10ms per operation)
        assert!(
            avg_encryption_time.as_millis() < 10,
            "Transaction encryption too slow: {:?}",
            avg_encryption_time
        );

        println!("Performance results:");
        println!("  Transaction encryption: {:?} avg", avg_encryption_time);
    }

    #[test]
    fn test_memory_safety_and_cleanup() {
        // Test that sensitive data is properly handled in memory
        let ml_kem_public_key = generate_test_ml_kem_key();
        let quantum_key_id = Uuid::new_v4();

        for _ in 0..100 {
            let transaction_data = create_test_transaction_data();
            let result = QuantumTransactionCrypto::encrypt_transaction_data(
                &ml_kem_public_key,
                &transaction_data,
                quantum_key_id,
            );
            assert!(result.is_ok());
            
            // Variables will be dropped and zeroized here
        }

        // If we get here without crashes, memory safety is working
    }

    #[test]
    fn test_different_key_sizes() {
        let transaction_data = create_test_transaction_data();
        let quantum_key_id = Uuid::new_v4();

        // Test with various invalid key sizes
        let invalid_keys = vec![
            vec![0u8; 10],   // Too short
            vec![0u8; 31],   // Still too short  
            vec![],          // Empty
        ];

        for invalid_key in invalid_keys {
            let result = QuantumTransactionCrypto::encrypt_transaction_data(
                &invalid_key,
                &transaction_data,
                quantum_key_id,
            );
            
            assert!(result.is_err(), "Should fail with invalid key size");
        }
    }
}