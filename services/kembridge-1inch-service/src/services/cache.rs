use crate::errors::{OneinchServiceError, Result};
use deadpool_redis::{Pool as RedisPool, Connection};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct CacheService {
    redis_pool: RedisPool,
}

impl CacheService {
    pub fn new(redis_pool: RedisPool) -> Self {
        Self { redis_pool }
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.get_connection().await?;
        
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(data)) => {
                match serde_json::from_str::<T>(&data) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        warn!("Failed to deserialize cached data for key {}: {}", key, e);
                        // Remove corrupted data
                        let _ = conn.del::<_, ()>(key).await;
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Redis GET error for key {}: {}", key, e);
                Err(OneinchServiceError::CacheError {
                    message: format!("GET failed: {}", e),
                })
            }
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        
        let data = serde_json::to_string(value)
            .map_err(|e| OneinchServiceError::CacheError {
                message: format!("Serialization failed: {}", e),
            })?;

        conn.set_ex::<_, _, ()>(key, data, ttl.as_secs())
            .await
            .map_err(|e| {
                error!("Redis SET error for key {}: {}", key, e);
                OneinchServiceError::CacheError {
                    message: format!("SET failed: {}", e),
                }
            })?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| {
                error!("Redis DEL error for key {}: {}", key, e);
                OneinchServiceError::CacheError {
                    message: format!("DEL failed: {}", e),
                }
            })?;

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        
        let exists: bool = conn.exists(key)
            .await
            .map_err(|e| {
                error!("Redis EXISTS error for key {}: {}", key, e);
                OneinchServiceError::CacheError {
                    message: format!("EXISTS failed: {}", e),
                }
            })?;

        Ok(exists)
    }

    pub async fn set_if_not_exists<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<bool>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        
        let data = serde_json::to_string(value)
            .map_err(|e| OneinchServiceError::CacheError {
                message: format!("Serialization failed: {}", e),
            })?;

        let set: bool = conn.set_nx(key, &data)
            .await
            .map_err(|e| {
                error!("Redis SETNX error for key {}: {}", key, e);
                OneinchServiceError::CacheError {
                    message: format!("SETNX failed: {}", e),
                }
            })?;

        if set {
            conn.expire::<_, ()>(key, ttl.as_secs() as i64)
                .await
                .map_err(|e| {
                    error!("Redis EXPIRE error for key {}: {}", key, e);
                    OneinchServiceError::CacheError {
                        message: format!("EXPIRE failed: {}", e),
                    }
                })?;
        }

        Ok(set)
    }

    pub async fn increment(&self, key: &str, ttl: Option<Duration>) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        
        let value: i64 = conn.incr(key, 1)
            .await
            .map_err(|e| {
                error!("Redis INCR error for key {}: {}", key, e);
                OneinchServiceError::CacheError {
                    message: format!("INCR failed: {}", e),
                }
            })?;

        if let Some(ttl) = ttl {
            conn.expire::<_, ()>(key, ttl.as_secs() as i64)
                .await
                .map_err(|e| {
                    error!("Redis EXPIRE error for key {}: {}", key, e);
                    OneinchServiceError::CacheError {
                        message: format!("EXPIRE failed: {}", e),
                    }
                })?;
        }

        Ok(value)
    }

    async fn get_connection(&self) -> Result<Connection> {
        self.redis_pool
            .get()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                OneinchServiceError::CacheError {
                    message: format!("Connection pool error: {}", e),
                }
            })
    }

    // Cache key builders
    pub fn quote_key(&self, chain_id: u64, from: &str, to: &str, amount: &str) -> String {
        format!("quote:{}:{}:{}:{}", chain_id, from, to, amount)
    }

    pub fn token_price_key(&self, chain_id: u64, token: &str) -> String {
        format!("price:{}:{}", chain_id, token)
    }

    pub fn tokens_key(&self, chain_id: u64) -> String {
        format!("tokens:{}", chain_id)
    }

    pub fn protocols_key(&self, chain_id: u64) -> String {
        format!("protocols:{}", chain_id)
    }

    pub fn rate_limit_key(&self, user_id: &str, endpoint: &str) -> String {
        format!("rate_limit:{}:{}", user_id, endpoint)
    }

    pub fn liquidity_key(&self, chain_id: u64, from: &str, to: &str) -> String {
        format!("liquidity:{}:{}:{}", chain_id, from, to)
    }

    // Batch operations
    pub async fn get_many<T>(&self, keys: &[String]) -> Result<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.get_connection().await?;
        
        let values: Vec<Option<String>> = conn.mget(keys)
            .await
            .map_err(|e| {
                error!("Redis MGET error: {}", e);
                OneinchServiceError::CacheError {
                    message: format!("MGET failed: {}", e),
                }
            })?;

        let mut results = Vec::with_capacity(values.len());
        
        for (i, value) in values.into_iter().enumerate() {
            match value {
                Some(data) => {
                    match serde_json::from_str::<T>(&data) {
                        Ok(parsed) => results.push(Some(parsed)),
                        Err(e) => {
                            warn!("Failed to deserialize cached data for key {}: {}", keys[i], e);
                            // Remove corrupted data
                            let _ = conn.del::<_, ()>(&keys[i]).await;
                            results.push(None);
                        }
                    }
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    pub async fn set_many<T>(&self, items: &[(String, T, Duration)]) -> Result<()>
    where
        T: Serialize,
    {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;

        // Use pipeline for better performance
        let mut pipe = redis::Pipeline::new();
        
        for (key, value, ttl) in items {
            let data = serde_json::to_string(value)
                .map_err(|e| OneinchServiceError::CacheError {
                    message: format!("Serialization failed: {}", e),
                })?;
            
            pipe.set_ex(key, data, ttl.as_secs()).ignore();
        }

        pipe.query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis pipeline error: {}", e);
                OneinchServiceError::CacheError {
                    message: format!("Pipeline failed: {}", e),
                }
            })?;

        Ok(())
    }

    // Health check
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis health check failed: {}", e);
                OneinchServiceError::CacheError {
                    message: format!("Health check failed: {}", e),
                }
            })?;

        if pong == "PONG" {
            info!("Redis health check passed");
            Ok(())
        } else {
            Err(OneinchServiceError::CacheError {
                message: format!("Unexpected PING response: {}", pong),
            })
        }
    }
}