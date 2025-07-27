// src/monitoring/redis_cache.rs - Redis caching for monitoring data
use super::MonitoringError;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

/// Redis cache for monitoring data
#[derive(Clone)]
pub struct MonitoringRedisCache {
    redis: Arc<RwLock<Option<redis::aio::ConnectionManager>>>,
}

impl MonitoringRedisCache {
    /// Create new Redis cache
    pub fn new() -> Self {
        Self {
            redis: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize with Redis connection
    pub async fn with_redis(self, redis: redis::aio::ConnectionManager) -> Self {
        {
            let mut redis_lock = self.redis.write().await;
            *redis_lock = Some(redis);
        }
        self
    }

    /// Cache risk score
    pub async fn cache_risk_score(&self, user_id: Uuid, score: f64, ttl_seconds: u64) -> Result<(), MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("risk_score:{}", user_id);
            
            let risk_data = CachedRiskScore {
                user_id,
                score,
                cached_at: Utc::now(),
                expires_at: Utc::now() + Duration::seconds(ttl_seconds as i64),
            };
            
            let serialized = serde_json::to_string(&risk_data)
                .map_err(|e| MonitoringError::Serialization(e))?;
            
            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl_seconds)
                .arg(serialized)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            debug!("Cached risk score {} for user {} with TTL {}s", score, user_id, ttl_seconds);
            Ok(())
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Get cached risk score
    pub async fn get_risk_score(&self, user_id: Uuid) -> Result<Option<CachedRiskScore>, MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("risk_score:{}", user_id);
            
            let result: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            if let Some(serialized) = result {
                let risk_data: CachedRiskScore = serde_json::from_str(&serialized)
                    .map_err(|e| MonitoringError::Serialization(e))?;
                
                // Check if expired
                if risk_data.expires_at > Utc::now() {
                    debug!("Retrieved cached risk score {} for user {}", risk_data.score, user_id);
                    Ok(Some(risk_data))
                } else {
                    debug!("Cached risk score for user {} has expired", user_id);
                    Ok(None)
                }
            } else {
                debug!("No cached risk score found for user {}", user_id);
                Ok(None)
            }
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Cache transaction status
    pub async fn cache_transaction_status(&self, transaction_id: Uuid, status: String, ttl_seconds: u64) -> Result<(), MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("transaction_status:{}", transaction_id);
            
            let status_data = CachedTransactionStatus {
                transaction_id,
                status,
                cached_at: Utc::now(),
                expires_at: Utc::now() + Duration::seconds(ttl_seconds as i64),
            };
            
            let serialized = serde_json::to_string(&status_data)
                .map_err(|e| MonitoringError::Serialization(e))?;
            
            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl_seconds)
                .arg(serialized)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            debug!("Cached transaction status {} for transaction {} with TTL {}s", status_data.status, transaction_id, ttl_seconds);
            Ok(())
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Get cached transaction status
    pub async fn get_transaction_status(&self, transaction_id: Uuid) -> Result<Option<CachedTransactionStatus>, MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("transaction_status:{}", transaction_id);
            
            let result: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            if let Some(serialized) = result {
                let status_data: CachedTransactionStatus = serde_json::from_str(&serialized)
                    .map_err(|e| MonitoringError::Serialization(e))?;
                
                if status_data.expires_at > Utc::now() {
                    debug!("Retrieved cached transaction status {} for transaction {}", status_data.status, transaction_id);
                    Ok(Some(status_data))
                } else {
                    debug!("Cached transaction status for transaction {} has expired", transaction_id);
                    Ok(None)
                }
            } else {
                debug!("No cached transaction status found for transaction {}", transaction_id);
                Ok(None)
            }
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Cache alert history
    pub async fn cache_alert_history(&self, user_id: Uuid, alert_id: Uuid, alert_type: String, ttl_seconds: u64) -> Result<(), MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("alert_history:{}", user_id);
            
            let alert_data = CachedAlertHistory {
                alert_id,
                user_id,
                alert_type,
                cached_at: Utc::now(),
            };
            
            let serialized = serde_json::to_string(&alert_data)
                .map_err(|e| MonitoringError::Serialization(e))?;
            
            // Use LPUSH to add to list and LTRIM to keep only recent alerts
            redis::cmd("LPUSH")
                .arg(&key)
                .arg(serialized)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            // Keep only last 100 alerts
            redis::cmd("LTRIM")
                .arg(&key)
                .arg(0)
                .arg(99)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            // Set expiry on the list
            redis::cmd("EXPIRE")
                .arg(&key)
                .arg(ttl_seconds)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            debug!("Cached alert history for user {} with alert {}", user_id, alert_id);
            Ok(())
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Get alert history
    pub async fn get_alert_history(&self, user_id: Uuid, limit: usize) -> Result<Vec<CachedAlertHistory>, MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            let key = format!("alert_history:{}", user_id);
            
            let result: Vec<String> = redis::cmd("LRANGE")
                .arg(&key)
                .arg(0)
                .arg(limit as i64 - 1)
                .query_async(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            let mut alerts = Vec::new();
            for serialized in result {
                let alert_data: CachedAlertHistory = serde_json::from_str(&serialized)
                    .map_err(|e| MonitoringError::Serialization(e))?;
                alerts.push(alert_data);
            }
            
            debug!("Retrieved {} alert history items for user {}", alerts.len(), user_id);
            Ok(alerts)
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Clear cache for user
    pub async fn clear_user_cache(&self, user_id: Uuid) -> Result<(), MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            
            let keys = vec![
                format!("risk_score:{}", user_id),
                format!("alert_history:{}", user_id),
            ];
            
            for key in keys {
                redis::cmd("DEL")
                    .arg(&key)
                    .query_async::<()>(&mut conn)
                    .await
                    .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            }
            
            info!("Cleared cache for user {}", user_id);
            Ok(())
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats, MonitoringError> {
        let redis_lock = self.redis.read().await;
        if let Some(redis) = redis_lock.as_ref() {
            let mut conn = redis.clone();
            
            let info: String = redis::cmd("INFO")
                .arg("memory")
                .query_async(&mut conn)
                .await
                .map_err(|e| MonitoringError::Redis(e.to_string()))?;
            
            // Parse memory usage from INFO output
            let used_memory = info.lines()
                .find(|line| line.starts_with("used_memory:"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            
            let stats = CacheStats {
                used_memory_bytes: used_memory,
                timestamp: Utc::now(),
            };
            
            debug!("Retrieved cache stats: {} bytes used", used_memory);
            Ok(stats)
        } else {
            Err(MonitoringError::Redis("Redis connection not initialized".to_string()))
        }
    }
}

/// Cached risk score data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRiskScore {
    pub user_id: Uuid,
    pub score: f64,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Cached transaction status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransactionStatus {
    pub transaction_id: Uuid,
    pub status: String,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Cached alert history data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAlertHistory {
    pub alert_id: Uuid,
    pub user_id: Uuid,
    pub alert_type: String,
    pub cached_at: DateTime<Utc>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub used_memory_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

impl Default for MonitoringRedisCache {
    fn default() -> Self {
        Self::new()
    }
}