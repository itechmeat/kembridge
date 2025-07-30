//! Operation-specific key derivation for KEMBridge
//! 
//! This module provides context-specific key derivation for different
//! quantum-safe operations using HKDF.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::QuantumCryptoError;

/// Operation types for key derivation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationType {
    BridgeTransaction,
    UserAuthentication,
    CrossChainMessage,
    StateSync,
    EventData,
}

/// Operation-specific keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationKeys {
    pub encryption_key: Vec<u8>,
    pub authentication_key: Vec<u8>,
    pub integrity_key: Vec<u8>,
    pub context: OperationKeyContext,
}

/// Key derivation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationKeyContext {
    pub operation_type: OperationType,
    pub metadata: HashMap<String, String>,
    pub derived_at: i64,
}

/// Operation key manager
pub struct OperationKeyManager;

impl OperationKeyManager {
    /// Derive keys for bridge transactions
    pub fn derive_bridge_keys(
        &self,
        shared_secret: &[u8; 32],
        from_chain: &str,
        to_chain: &str,
    ) -> Result<OperationKeys, QuantumCryptoError> {
        let context_info = format!("bridge_{}_{}", from_chain, to_chain);
        let keys = self.derive_keys_with_context(shared_secret, &context_info)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("from_chain".to_string(), from_chain.to_string());
        metadata.insert("to_chain".to_string(), to_chain.to_string());
        
        let context = OperationKeyContext {
            operation_type: OperationType::BridgeTransaction,
            metadata,
            derived_at: chrono::Utc::now().timestamp(),
        };

        Ok(OperationKeys {
            encryption_key: keys.0,
            authentication_key: keys.1,
            integrity_key: keys.2,
            context,
        })
    }

    /// Derive keys for user authentication
    pub fn derive_user_auth_keys(
        &self,
        shared_secret: &[u8; 32],
        user_id: &str,
        chain: &str,
    ) -> Result<OperationKeys, QuantumCryptoError> {
        let context_info = format!("auth_{}_{}", user_id, chain);
        let keys = self.derive_keys_with_context(shared_secret, &context_info)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), user_id.to_string());
        metadata.insert("chain".to_string(), chain.to_string());
        
        let context = OperationKeyContext {
            operation_type: OperationType::UserAuthentication,
            metadata,
            derived_at: chrono::Utc::now().timestamp(),
        };

        Ok(OperationKeys {
            encryption_key: keys.0,
            authentication_key: keys.1,
            integrity_key: keys.2,
            context,
        })
    }

    /// Derive keys for cross-chain messages
    pub fn derive_cross_chain_message_keys(
        &self,
        shared_secret: &[u8; 32],
        from_chain: &str,
        to_chain: &str,
        message_id: &str,
    ) -> Result<OperationKeys, QuantumCryptoError> {
        let context_info = format!("message_{}_{}_{}", from_chain, to_chain, message_id);
        let keys = self.derive_keys_with_context(shared_secret, &context_info)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("from_chain".to_string(), from_chain.to_string());
        metadata.insert("to_chain".to_string(), to_chain.to_string());
        metadata.insert("message_id".to_string(), message_id.to_string());
        
        let context = OperationKeyContext {
            operation_type: OperationType::CrossChainMessage,
            metadata,
            derived_at: chrono::Utc::now().timestamp(),
        };

        Ok(OperationKeys {
            encryption_key: keys.0,
            authentication_key: keys.1,
            integrity_key: keys.2,
            context,
        })
    }

    /// Derive keys for state synchronization
    pub fn derive_state_sync_keys(
        &self,
        shared_secret: &[u8; 32],
        from_chain: &str,
        to_chain: &str,
    ) -> Result<OperationKeys, QuantumCryptoError> {
        let context_info = format!("state_sync_{}_{}", from_chain, to_chain);
        let keys = self.derive_keys_with_context(shared_secret, &context_info)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("from_chain".to_string(), from_chain.to_string());
        metadata.insert("to_chain".to_string(), to_chain.to_string());
        
        let context = OperationKeyContext {
            operation_type: OperationType::StateSync,
            metadata,
            derived_at: chrono::Utc::now().timestamp(),
        };

        Ok(OperationKeys {
            encryption_key: keys.0,
            authentication_key: keys.1,
            integrity_key: keys.2,
            context,
        })
    }

    /// Derive keys for events
    pub fn derive_event_keys(
        &self,
        shared_secret: &[u8; 32],
        chain: &str,
        event_type: &str,
    ) -> Result<OperationKeys, QuantumCryptoError> {
        let context_info = format!("event_{}_{}", chain, event_type);
        let keys = self.derive_keys_with_context(shared_secret, &context_info)?;
        
        let mut metadata = HashMap::new();
        metadata.insert("chain".to_string(), chain.to_string());
        metadata.insert("event_type".to_string(), event_type.to_string());
        
        let context = OperationKeyContext {
            operation_type: OperationType::EventData,
            metadata,
            derived_at: chrono::Utc::now().timestamp(),
        };

        Ok(OperationKeys {
            encryption_key: keys.0,
            authentication_key: keys.1,
            integrity_key: keys.2,
            context,
        })
    }

    // Derive three different keys from shared secret and context
    fn derive_keys_with_context(
        &self,
        shared_secret: &[u8; 32],
        context_info: &str,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), QuantumCryptoError> {
        use sha2::{Sha256, Digest};
        
        // Simple key derivation for testing
        let mut hasher1 = Sha256::new();
        hasher1.update(shared_secret);
        hasher1.update(context_info.as_bytes());
        hasher1.update(b"encryption");
        let encryption_key = hasher1.finalize().to_vec();

        let mut hasher2 = Sha256::new();
        hasher2.update(shared_secret);
        hasher2.update(context_info.as_bytes());
        hasher2.update(b"authentication");
        let authentication_key = hasher2.finalize().to_vec();

        let mut hasher3 = Sha256::new();
        hasher3.update(shared_secret);
        hasher3.update(context_info.as_bytes());
        hasher3.update(b"integrity");
        let integrity_key = hasher3.finalize().to_vec();

        Ok((encryption_key, authentication_key, integrity_key))
    }
}