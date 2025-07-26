// Nonce management for Web3 authentication

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use rand::Rng;
use chrono::{DateTime, Utc, Duration};
use crate::chains::ChainType;
use crate::errors::AuthError;
use crate::models::{NonceResponse};

const NONCE_EXPIRY_MINUTES: i64 = 5;
const NONCE_LENGTH: usize = 32;

#[derive(Clone)]
pub struct NonceManager {
    redis: ConnectionManager,
}

impl NonceManager {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Generate a new nonce for wallet authentication
    pub async fn generate_nonce(
        &self,
        wallet_address: &str,
        chain_type: ChainType,
    ) -> Result<NonceResponse, AuthError> {
        let nonce = self.generate_random_nonce();
        let expires_at = Utc::now() + Duration::minutes(NONCE_EXPIRY_MINUTES);
        
        // Create message for signing
        let message = format!(
            "KEMBridge Authentication\n\nWallet: {}\nChain: {}\nNonce: {}\nExpires: {}",
            wallet_address,
            chain_type.to_string(),
            nonce,
            expires_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        // Store nonce in Redis
        let key = self.nonce_key(wallet_address, &nonce);
        let nonce_data = serde_json::json!({
            "wallet_address": wallet_address,
            "chain_type": chain_type.to_string(),
            "message": message,
            "created_at": Utc::now(),
            "expires_at": expires_at
        });

        let mut conn = self.redis.clone();
        let _: () = conn.set_ex(
            &key,
            serde_json::to_string(&nonce_data)?,
            (NONCE_EXPIRY_MINUTES * 60) as u64,
        )
        .await?;

        Ok(NonceResponse {
            nonce,
            message,
            expires_at,
        })
    }

    /// Verify nonce exists and is valid
    pub async fn verify_nonce(
        &self,
        wallet_address: &str,
        nonce: &str,
        chain_type: ChainType,
    ) -> Result<(), AuthError> {
        let key = self.nonce_key(wallet_address, nonce);
        let mut conn = self.redis.clone();
        
        let nonce_data: Option<String> = conn.get(&key).await?;
        
        let nonce_data = nonce_data.ok_or(AuthError::InvalidNonce)?;
        let parsed_data: serde_json::Value = serde_json::from_str(&nonce_data)?;

        // Verify wallet address matches
        let stored_address = parsed_data["wallet_address"]
            .as_str()
            .ok_or(AuthError::InvalidNonce)?;
        
        if stored_address != wallet_address {
            return Err(AuthError::InvalidNonce);
        }

        // Verify chain type matches
        let stored_chain = parsed_data["chain_type"]
            .as_str()
            .ok_or(AuthError::InvalidNonce)?;
        
        if stored_chain != chain_type.to_string() {
            return Err(AuthError::InvalidNonce);
        }

        // Verify expiration
        let expires_at_str = parsed_data["expires_at"]
            .as_str()
            .ok_or(AuthError::InvalidNonce)?;
        
        let expires_at: DateTime<Utc> = expires_at_str
            .parse()
            .map_err(|_| AuthError::InvalidNonce)?;

        if Utc::now() > expires_at {
            return Err(AuthError::NonceExpired);
        }

        Ok(())
    }

    /// Consume (delete) nonce after successful authentication
    pub async fn consume_nonce(
        &self,
        wallet_address: &str,
        nonce: &str,
    ) -> Result<(), AuthError> {
        let key = self.nonce_key(wallet_address, nonce);
        let mut conn = self.redis.clone();
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// Clean up expired nonces (called periodically)
    pub async fn cleanup_expired_nonces(&self) -> Result<u64, AuthError> {
        // Redis TTL automatically handles cleanup
        // This method is for additional cleanup if needed
        Ok(0)
    }

    fn generate_random_nonce(&self) -> String {
        let mut rng = rand::thread_rng();
        let nonce_bytes: Vec<u8> = (0..NONCE_LENGTH).map(|_| rng.r#gen()).collect();
        hex::encode(nonce_bytes)
    }

    fn nonce_key(&self, wallet_address: &str, nonce: &str) -> String {
        format!("kembridge:auth:nonce:{}:{}", wallet_address, nonce)
    }
}