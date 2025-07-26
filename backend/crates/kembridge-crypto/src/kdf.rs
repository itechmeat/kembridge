// src/kdf.rs - Key Derivation Functions for hybrid cryptography
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::QuantumCryptoError;

/// HKDF-SHA256 для расширения ML-KEM shared secret в AES ключи
pub struct KeyDerivation;

impl KeyDerivation {
    /// Расширение ML-KEM shared secret в AES-256 ключ
    pub fn derive_encryption_key(
        shared_secret: &[u8; 32], 
        context: &[u8]
    ) -> Result<[u8; 32], QuantumCryptoError> {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut aes_key = [0u8; 32];
        
        hk.expand(context, &mut aes_key)
            .map_err(|e| QuantumCryptoError::KeyDerivationFailed(e.to_string()))?;
            
        Ok(aes_key)
    }

    /// Расширение shared secret в несколько ключей (AES + HMAC)
    pub fn derive_multiple_keys(
        shared_secret: &[u8; 32],
        context: &[u8]
    ) -> Result<DerivedKeys, QuantumCryptoError> {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut output = [0u8; 64]; // 32 bytes для AES + 32 bytes для HMAC
        
        hk.expand(context, &mut output)
            .map_err(|e| QuantumCryptoError::KeyDerivationFailed(e.to_string()))?;
        
        let mut aes_key = [0u8; 32];
        let mut hmac_key = [0u8; 32];
        
        aes_key.copy_from_slice(&output[0..32]);
        hmac_key.copy_from_slice(&output[32..64]);
        
        // Очистка промежуточного буфера
        output.zeroize();
        
        Ok(DerivedKeys {
            encryption_key: aes_key,
            authentication_key: hmac_key,
        })
    }

    /// Генерация context строки для HKDF
    pub fn create_context(purpose: &str, version: u8) -> Vec<u8> {
        format!("KEMBridge-v{}-{}", version, purpose).into_bytes()
    }
}

/// Производные ключи для hybrid схемы
#[derive(Debug)]
pub struct DerivedKeys {
    pub encryption_key: [u8; 32],
    pub authentication_key: [u8; 32],
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.encryption_key.zeroize();
        self.authentication_key.zeroize();
    }
}

/// Предопределенные контексты для различных операций
pub mod contexts {
    use super::KeyDerivation;

    /// Контекст для шифрования bridge транзакций
    pub fn bridge_transaction() -> Vec<u8> {
        KeyDerivation::create_context("bridge-tx", 1)
    }

    /// Контекст для key exchange между чейнами
    pub fn key_exchange() -> Vec<u8> {
        KeyDerivation::create_context("key-exchange", 1)
    }

    /// Контекст для session ключей
    pub fn session_keys() -> Vec<u8> {
        KeyDerivation::create_context("session", 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_key_derivation_deterministic() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context = contexts::bridge_transaction();
        
        // Два вызова с одинаковыми параметрами должны дать одинаковый результат
        let key1 = KeyDerivation::derive_encryption_key(&shared_secret, &context).unwrap();
        let key2 = KeyDerivation::derive_encryption_key(&shared_secret, &context).unwrap();
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_contexts_different_keys() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context1 = contexts::bridge_transaction();
        let context2 = contexts::key_exchange();
        
        let key1 = KeyDerivation::derive_encryption_key(&shared_secret, &context1).unwrap();
        let key2 = KeyDerivation::derive_encryption_key(&shared_secret, &context2).unwrap();
        
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_multiple_keys_derivation() {
        let mut shared_secret = [0u8; 32];
        thread_rng().fill(&mut shared_secret);
        
        let context = contexts::session_keys();
        let derived = KeyDerivation::derive_multiple_keys(&shared_secret, &context).unwrap();
        
        // Ключи должны быть разными
        assert_ne!(derived.encryption_key, derived.authentication_key);
    }

    #[test]
    fn test_context_generation() {
        let context1 = KeyDerivation::create_context("test", 1);
        let context2 = KeyDerivation::create_context("test", 2);
        let context3 = KeyDerivation::create_context("other", 1);
        
        assert_ne!(context1, context2); // Разные версии
        assert_ne!(context1, context3); // Разные purpose
        
        // Проверим что контекст читабельный
        let context_str = String::from_utf8(context1).unwrap();
        assert!(context_str.contains("KEMBridge"));
        assert!(context_str.contains("test"));
    }
}