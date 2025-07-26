// src/integrity.rs - Data integrity protection using HMAC-SHA256
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
// use zeroize::Zeroize;  // Будет использоваться в полной реализации

use crate::error::QuantumCryptoError;

type HmacSha256 = Hmac<Sha256>;

/// Data integrity protection utilities
pub struct IntegrityProtection;

impl IntegrityProtection {
    /// Генерация HMAC для проверки целостности зашифрованных данных
    pub fn generate_mac(
        key: &[u8],
        data: &[u8]
    ) -> Result<Vec<u8>, QuantumCryptoError> {
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| QuantumCryptoError::MacError(e.to_string()))?;
        
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Верификация целостности данных
    pub fn verify_integrity(
        key: &[u8],
        data: &[u8],
        expected_mac: &[u8]
    ) -> Result<bool, QuantumCryptoError> {
        let computed_mac = Self::generate_mac(key, data)?;
        
        // Constant-time comparison для защиты от timing attacks
        Ok(constant_time_eq(&computed_mac, expected_mac))
    }

    /// Генерация SHA-256 хеша для дополнительной проверки
    pub fn hash_data(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Комбинированная защита: HMAC + hash
    pub fn create_integrity_proof(
        key: &[u8],
        data: &[u8]
    ) -> Result<IntegrityProof, QuantumCryptoError> {
        let mac = Self::generate_mac(key, data)?;
        let hash = Self::hash_data(data);
        
        Ok(IntegrityProof {
            hmac: mac,
            sha256_hash: hash,
        })
    }

    /// Верификация комбинированной защиты
    pub fn verify_integrity_proof(
        key: &[u8],
        data: &[u8],
        proof: &IntegrityProof
    ) -> Result<bool, QuantumCryptoError> {
        // Проверка HMAC
        let hmac_valid = Self::verify_integrity(key, data, &proof.hmac)?;
        
        // Проверка hash
        let computed_hash = Self::hash_data(data);
        let hash_valid = constant_time_eq(&computed_hash, &proof.sha256_hash);
        
        Ok(hmac_valid && hash_valid)
    }
}

/// Структура для хранения доказательства целостности
#[derive(Debug, Clone)]
pub struct IntegrityProof {
    pub hmac: Vec<u8>,
    pub sha256_hash: [u8; 32],
}

impl IntegrityProof {
    /// Сериализация в байты для хранения
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(4 + self.hmac.len() + 32);
        
        // Длина HMAC (4 bytes)
        result.extend_from_slice(&(self.hmac.len() as u32).to_le_bytes());
        
        // HMAC данные
        result.extend_from_slice(&self.hmac);
        
        // SHA-256 hash (32 bytes)
        result.extend_from_slice(&self.sha256_hash);
        
        result
    }

    /// Десериализация из байтов
    pub fn from_bytes(data: &[u8]) -> Result<Self, QuantumCryptoError> {
        if data.len() < 4 + 32 {
            return Err(QuantumCryptoError::InvalidData(
                "Insufficient data for integrity proof".to_string()
            ));
        }

        // Чтение длины HMAC
        let hmac_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        
        if data.len() != 4 + hmac_len + 32 {
            return Err(QuantumCryptoError::InvalidData(
                "Invalid integrity proof format".to_string()
            ));
        }

        // Извлечение HMAC
        let hmac = data[4..4 + hmac_len].to_vec();
        
        // Извлечение SHA-256 hash
        let mut sha256_hash = [0u8; 32];
        sha256_hash.copy_from_slice(&data[4 + hmac_len..4 + hmac_len + 32]);
        
        Ok(IntegrityProof {
            hmac,
            sha256_hash,
        })
    }
}

/// Constant-time comparison для защиты от timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_hmac_generation_and_verification() {
        let mut key = [0u8; 32];
        thread_rng().fill(&mut key);
        
        let data = b"Test data for HMAC verification";
        
        // Генерация MAC
        let mac = IntegrityProtection::generate_mac(&key, data).unwrap();
        
        // Верификация
        let is_valid = IntegrityProtection::verify_integrity(&key, data, &mac).unwrap();
        assert!(is_valid);
        
        // Проверка с неправильными данными
        let wrong_data = b"Wrong data";
        let is_invalid = IntegrityProtection::verify_integrity(&key, wrong_data, &mac).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_integrity_proof_round_trip() {
        let mut key = [0u8; 32];
        thread_rng().fill(&mut key);
        
        let data = b"Test data for integrity proof";
        
        // Создание proof
        let proof = IntegrityProtection::create_integrity_proof(&key, data).unwrap();
        
        // Верификация
        let is_valid = IntegrityProtection::verify_integrity_proof(&key, data, &proof).unwrap();
        assert!(is_valid);
        
        // Сериализация и десериализация
        let serialized = proof.to_bytes();
        let deserialized = IntegrityProof::from_bytes(&serialized).unwrap();
        
        // Верификация после десериализации
        let is_still_valid = IntegrityProtection::verify_integrity_proof(&key, data, &deserialized).unwrap();
        assert!(is_still_valid);
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"Consistent test data";
        
        let hash1 = IntegrityProtection::hash_data(data);
        let hash2 = IntegrityProtection::hash_data(data);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_constant_time_comparison() {
        let data1 = vec![1, 2, 3, 4];
        let data2 = vec![1, 2, 3, 4];
        let data3 = vec![1, 2, 3, 5];
        let data4 = vec![1, 2, 3]; // Разная длина
        
        assert!(constant_time_eq(&data1, &data2));
        assert!(!constant_time_eq(&data1, &data3));
        assert!(!constant_time_eq(&data1, &data4));
    }

    #[test]
    fn test_integrity_proof_serialization() {
        let proof = IntegrityProof {
            hmac: vec![1, 2, 3, 4, 5],
            sha256_hash: [42; 32],
        };
        
        let serialized = proof.to_bytes();
        let deserialized = IntegrityProof::from_bytes(&serialized).unwrap();
        
        assert_eq!(proof.hmac, deserialized.hmac);
        assert_eq!(proof.sha256_hash, deserialized.sha256_hash);
    }
}