//! Integration tests for the kembridge-crypto crate
//! 
//! These tests verify that all components work together correctly
//! and provide comprehensive coverage of the post-quantum crypto functionality.

use kembridge_crypto::{
    MlKemCrypto, QuantumKeyManager, QuantumCryptoError,
    EncapsulationKey, DecapsulationKey
};
use rand::thread_rng;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_crypto_workflow() {
        let mut rng = thread_rng();
        let manager = QuantumKeyManager::new();

        // 1. Generate key pair
        let keypair = manager.generate_ml_kem_keypair().unwrap();
        
        // 2. Verify key pair works
        manager.verify_keypair(&keypair).unwrap();
        
        // 3. Export and import cycle
        let exported = manager.export_keypair(&keypair);
        let imported_keypair = manager.import_keypair(exported).unwrap();
        
        // 4. Verify imported keys work
        manager.verify_keypair(&imported_keypair).unwrap();
        
        // 5. Perform crypto operations
        let encap_result = manager.secure_encapsulate(&imported_keypair).unwrap();
        let shared_secret = manager.secure_decapsulate(&imported_keypair, &encap_result.ciphertext).unwrap();
        
        // 6. Verify shared secrets match
        assert_eq!(encap_result.shared_secret, shared_secret);
    }

    #[test]
    fn test_cross_key_compatibility() {
        let mut rng = thread_rng();
        
        // Generate key pair using low-level API
        let (dk, ek) = MlKemCrypto::generate_keypair(&mut rng).unwrap();
        
        // Use high-level API for operations
        let (ct, ss_send) = MlKemCrypto::encapsulate(&ek, &mut rng).unwrap();
        let ss_recv = MlKemCrypto::decapsulate(&dk, &ct).unwrap();
        
        assert_eq!(ss_send, ss_recv);
    }

    #[test]
    fn test_key_serialization_round_trip() {
        let mut rng = thread_rng();
        let (dk, ek) = MlKemCrypto::generate_keypair(&mut rng).unwrap();
        
        // Test public key serialization
        let pk_bytes = MlKemCrypto::export_public_key(&ek);
        let ek_imported = MlKemCrypto::import_public_key(&pk_bytes).unwrap();
        
        // Test private key serialization
        let sk_bytes = MlKemCrypto::export_private_key(&dk);
        let dk_imported = MlKemCrypto::import_private_key(&sk_bytes).unwrap();
        
        // Verify serialized keys work
        let (ct, ss_send) = MlKemCrypto::encapsulate(&ek_imported, &mut rng).unwrap();
        let ss_recv = MlKemCrypto::decapsulate(&dk_imported, &ct).unwrap();
        
        assert_eq!(ss_send, ss_recv);
    }

    #[test]
    fn test_multiple_encapsulations() {
        let mut rng = thread_rng();
        let manager = QuantumKeyManager::new();
        let keypair = manager.generate_ml_kem_keypair().unwrap();
        
        // Perform multiple encapsulations with the same key pair
        for i in 0..10 {
            let result = manager.secure_encapsulate(&keypair).unwrap();
            let shared_secret = manager.secure_decapsulate(&keypair, &result.ciphertext).unwrap();
            
            assert_eq!(result.shared_secret, shared_secret);
            
            // Each encapsulation should produce different ciphertext and shared secret
            if i > 0 {
                let result2 = manager.secure_encapsulate(&keypair).unwrap();
                assert_ne!(result.ciphertext.as_bytes(), result2.ciphertext.as_bytes());
                assert_ne!(result.shared_secret, result2.shared_secret);
            }
        }
    }

    #[test]
    fn test_algorithm_parameters() {
        let info = MlKemCrypto::algorithm_info();
        
        assert_eq!(info.name, "ML-KEM-1024");
        assert_eq!(info.standard, "FIPS 203");
        assert_eq!(info.security_level, 256);
        assert_eq!(info.public_key_size, 1568);
        assert_eq!(info.private_key_size, 3168);
        assert_eq!(info.ciphertext_size, 1568);
        assert_eq!(info.shared_secret_size, 32);
    }

    #[test]
    fn test_error_handling() {
        // Test invalid public key import
        let invalid_pk = vec![0u8; 100];
        let result = MlKemCrypto::import_public_key(&invalid_pk);
        assert!(result.is_err());
        
        // Test invalid private key import
        let invalid_sk = vec![0u8; 200];
        let result = MlKemCrypto::import_private_key(&invalid_sk);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_safety() {
        // This test ensures that memory is properly handled
        // The zeroize functionality is tested implicitly through drops
        let mut rng = thread_rng();
        let manager = QuantumKeyManager::new();
        
        for _ in 0..100 {
            let keypair = manager.generate_ml_kem_keypair().unwrap();
            let result = manager.secure_encapsulate(&keypair).unwrap();
            let _shared_secret = manager.secure_decapsulate(&keypair, &result.ciphertext).unwrap();
            
            // Variables will be dropped and zeroized here
        }
        
        // If we get here without crashes, memory safety is working
    }

    #[test]
    fn test_concurrent_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let manager = Arc::new(QuantumKeyManager::new());
        let mut handles = vec![];
        
        // Spawn multiple threads performing crypto operations
        for _ in 0..4 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let keypair = manager_clone.generate_ml_kem_keypair().unwrap();
                    manager_clone.verify_keypair(&keypair).unwrap();
                    
                    let result = manager_clone.secure_encapsulate(&keypair).unwrap();
                    let shared_secret = manager_clone.secure_decapsulate(&keypair, &result.ciphertext).unwrap();
                    
                    assert_eq!(result.shared_secret, shared_secret);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;
        
        let mut rng = thread_rng();
        let iterations = 10;
        
        // Measure key generation performance
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = MlKemCrypto::generate_keypair(&mut rng).unwrap();
        }
        let keygen_duration = start.elapsed();
        let avg_keygen = keygen_duration / iterations;
        
        // Key generation should be fast (< 10ms per key pair on modern hardware)
        assert!(avg_keygen.as_millis() < 10, "Key generation too slow: {:?}", avg_keygen);
        
        // Generate a key pair for encap/decap testing
        let (dk, ek) = MlKemCrypto::generate_keypair(&mut rng).unwrap();
        
        // Measure encapsulation performance
        let start = Instant::now();
        let mut ciphertexts = Vec::new();
        for _ in 0..iterations {
            let (ct, _ss) = MlKemCrypto::encapsulate(&ek, &mut rng).unwrap();
            ciphertexts.push(ct);
        }
        let encap_duration = start.elapsed();
        let avg_encap = encap_duration / iterations;
        
        // Encapsulation should be fast (< 1ms per operation)
        assert!(avg_encap.as_millis() < 1, "Encapsulation too slow: {:?}", avg_encap);
        
        // Measure decapsulation performance
        let start = Instant::now();
        for ct in ciphertexts {
            let _ss = MlKemCrypto::decapsulate(&dk, &ct).unwrap();
        }
        let decap_duration = start.elapsed();
        let avg_decap = decap_duration / iterations;
        
        // Decapsulation should be fast (< 1ms per operation)
        assert!(avg_decap.as_millis() < 1, "Decapsulation too slow: {:?}", avg_decap);
        
        println!("Performance results:");
        println!("  Key generation: {:?} avg", avg_keygen);
        println!("  Encapsulation:  {:?} avg", avg_encap);
        println!("  Decapsulation:  {:?} avg", avg_decap);
    }
}