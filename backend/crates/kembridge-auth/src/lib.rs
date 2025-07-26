// KEMBridge Web3 Authentication Service
// Rust 1.88+ with modern crypto libraries for multi-chain signature verification

pub mod chains;
pub mod jwt;
pub mod models;
pub mod nonce;
pub mod signature;
pub mod errors;
pub mod handlers;
pub mod routes;

use axum::Router;
use sqlx::PgPool;
use redis::aio::ConnectionManager;

use crate::models::*;
use crate::errors::AuthError;
use sha2::Digest;

// Re-exports for convenience
pub use chains::{ChainType, ChainVerifier, EthereumVerifier, NearVerifier, MultiChainVerifier};
pub use jwt::{JwtManager, JwtClaims};
pub use nonce::NonceManager;
pub use signature::SignatureVerifier;
pub use models::{AuthRequest, AuthResponse, NonceRequest, NonceResponse};
pub use handlers::*;
pub use routes::create_auth_routes;

/// Authentication service state
#[derive(Clone)]
pub struct AuthService {
    pub db_pool: PgPool,
    pub redis_manager: ConnectionManager,
    pub jwt_manager: JwtManager,
    pub nonce_manager: NonceManager,
    pub signature_verifier: SignatureVerifier,
    pub multi_chain_verifier: MultiChainVerifier,
}

impl AuthService {
    /// Create new authentication service with all dependencies
    pub async fn new(
        db_pool: PgPool,
        redis_manager: ConnectionManager,
        jwt_secret: String,
    ) -> Result<Self, AuthError> {
        let jwt_manager = JwtManager::new(jwt_secret)?;
        let nonce_manager = NonceManager::new(redis_manager.clone());
        let signature_verifier = SignatureVerifier::new();
        let multi_chain_verifier = MultiChainVerifier::new();

        Ok(Self {
            db_pool,
            redis_manager,
            jwt_manager,
            nonce_manager,
            signature_verifier,
            multi_chain_verifier,
        })
    }

    /// Generate nonce for wallet signature
    pub async fn generate_nonce(
        &self,
        wallet_address: &str,
        chain_type: ChainType,
    ) -> Result<NonceResponse, AuthError> {
        self.nonce_manager
            .generate_nonce(wallet_address, chain_type)
            .await
    }

    /// Verify wallet signature and create JWT token
    pub async fn verify_wallet_signature(
        &self,
        request: AuthRequest,
    ) -> Result<AuthResponse, AuthError> {
        // Verify nonce
        self.nonce_manager
            .verify_nonce(&request.wallet_address, &request.nonce, request.chain_type)
            .await?;

        // Verify signature
        let is_valid = self
            .multi_chain_verifier
            .verify_signature(
                request.chain_type,
                &request.message,
                &request.signature,
                &request.wallet_address,
            )
            .await?;

        if !is_valid {
            return Err(AuthError::InvalidSignature);
        }

        // Create or get user
        let user_id = self
            .get_or_create_user(&request.wallet_address, request.chain_type)
            .await?;

        // Generate JWT token
        let jwt_token = self
            .jwt_manager
            .generate_token(user_id, &request.wallet_address, request.chain_type)
            .await?;

        // Save session to database
        self.save_session(user_id, &jwt_token, &request).await?;

        // Consume nonce
        self.nonce_manager
            .consume_nonce(&request.wallet_address, &request.nonce)
            .await?;

        Ok(AuthResponse {
            access_token: jwt_token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 60 * 60, // 24 hours
            user_id,
            wallet_address: request.wallet_address,
            chain_type: request.chain_type,
        })
    }

    /// Get or create user in database
    async fn get_or_create_user(
        &self,
        wallet_address: &str,
        chain_type: ChainType,
    ) -> Result<uuid::Uuid, AuthError> {
        // Check if user exists
        let existing_user: Option<(uuid::Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id 
            FROM users u
            JOIN user_auth_methods uam ON u.id = uam.user_id
            WHERE uam.wallet_address = $1 AND uam.chain_type = $2 AND uam.auth_type = 'web3_wallet'
            "#,
        )
        .bind(wallet_address)
        .bind(chain_type.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((user_id,)) = existing_user {
            return Ok(user_id);
        }

        // Create new user
        let mut tx = self.db_pool.begin().await?;

        let user_id: uuid::Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO users (username, profile_data, risk_profile)
            VALUES (NULL, '{}', '{"score": 0.0, "level": "new_user"}')
            RETURNING id
            "#
        )
        .fetch_one(&mut *tx)
        .await?;

        // Create auth method
        sqlx::query(
            r#"
            INSERT INTO user_auth_methods (
                user_id, auth_type, chain_type, wallet_address, is_primary, is_verified, first_used_at
            ) VALUES ($1, 'web3_wallet', $2, $3, true, true, NOW())
            "#,
        )
        .bind(user_id)
        .bind(chain_type.to_string())
        .bind(wallet_address)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user_id)
    }

    /// Save session to database
    async fn save_session(
        &self,
        user_id: uuid::Uuid,
        jwt_token: &str,
        request: &AuthRequest,
    ) -> Result<(), AuthError> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(jwt_token.as_bytes());
        let token_hash = hasher.finalize();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        sqlx::query(
            r#"
            INSERT INTO user_sessions (
                user_id, jwt_token_hash, wallet_address, chain_type,
                session_metadata, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(user_id)
        .bind(token_hash.as_slice())
        .bind(&request.wallet_address)
        .bind(request.chain_type.to_string())
        .bind(serde_json::json!({
            "auth_method": "direct_wallet",
            "signature": request.signature,
            "message": request.message
        }))
        .bind(expires_at)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}