// JWT token management for Web3 authentication

use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use crate::chains::ChainType;
use crate::errors::AuthError;

const JWT_ALGORITHM: Algorithm = Algorithm::HS256;
const JWT_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,           // User ID
    pub wallet_address: String,
    pub chain_type: String,
    pub user_id: String,
    pub iat: i64,             // Issued at
    pub exp: i64,             // Expiry
    pub iss: String,          // Issuer
    pub aud: String,          // Audience
}

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
}

impl JwtManager {
    pub fn new(secret: String) -> Result<Self, AuthError> {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        Ok(Self {
            encoding_key,
            decoding_key,
            issuer: "kembridge-auth".to_string(),
            audience: "kembridge-api".to_string(),
        })
    }

    pub async fn generate_token(
        &self,
        user_id: Uuid,
        wallet_address: &str,
        chain_type: ChainType,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(JWT_EXPIRY_HOURS);

        let claims = JwtClaims {
            sub: user_id.to_string(),
            wallet_address: wallet_address.to_string(),
            chain_type: chain_type.to_string(),
            user_id: user_id.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let header = Header::new(JWT_ALGORITHM);
        let token = encode(&header, &claims, &self.encoding_key)?;
        
        Ok(token)
    }

    pub async fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let mut validation = Validation::new(JWT_ALGORITHM);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)?;
        
        // Additional validation
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AuthError::JwtError(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
            )));
        }

        Ok(token_data.claims)
    }

    pub async fn refresh_token(&self, token: &str) -> Result<String, AuthError> {
        let claims = self.verify_token(token).await?;
        
        // Generate new token with same user data but new expiry
        let user_id = Uuid::parse_str(&claims.user_id)
            .map_err(|_| AuthError::JwtError(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken
            )))?;
        
        let chain_type = ChainType::from_str(&claims.chain_type)?;
        
        self.generate_token(user_id, &claims.wallet_address, chain_type).await
    }

    pub fn extract_user_id(&self, claims: &JwtClaims) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&claims.user_id)
            .map_err(|_| AuthError::JwtError(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken
            )))
    }
}