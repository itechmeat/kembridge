// src/services/rate_limit.rs - Rate Limiting Service with Statistics and Monitoring
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use redis::AsyncCommands;
use sqlx::{PgPool, Row};
use tracing::{warn, error, debug};
use utoipa::ToSchema;

use crate::constants::*;
use crate::middleware::error_handler::ApiError;

/// Rate Limiting Service for centralized management and statistics
#[derive(Clone)]
pub struct RateLimitService {
    redis: redis::aio::ConnectionManager,
    db_pool: PgPool,
}

impl RateLimitService {
    /// Create new RateLimitService
    pub fn new(redis: redis::aio::ConnectionManager, db_pool: PgPool) -> Self {
        Self { redis, db_pool }
    }

    /// Check rate limit using Redis sliding window and collect statistics
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: Duration,
        endpoint_class: &str,
        user_id: Option<&str>,
        ip_address: &str,
    ) -> Result<RateLimitResult, ApiError> {
        let mut conn = self.redis.clone();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now - window.as_secs();
        let request_id = format!("{}:{}", now, Uuid::new_v4());
        
        // Lua script for atomic sliding window rate limiting with statistics
        let script = r#"
            local key = KEYS[1]
            local stats_key = KEYS[2]
            local blocked_key = KEYS[3]
            local window_start = tonumber(ARGV[1])
            local now = tonumber(ARGV[2])
            local limit = tonumber(ARGV[3])
            local window_seconds = tonumber(ARGV[4])
            local request_id = ARGV[5]
            
            -- Remove expired entries
            redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)
            
            -- Count current requests
            local current_count = redis.call('ZCARD', key)
            
            -- Update statistics - total requests
            redis.call('HINCRBY', stats_key, 'total_requests', 1)
            redis.call('EXPIRE', stats_key, 3600) -- 1 hour TTL
            
            -- Check rate limit
            if current_count >= limit then
                -- Update blocked statistics
                redis.call('HINCRBY', stats_key, 'blocked_requests', 1)
                redis.call('HINCRBY', blocked_key, 'blocked_count', 1)
                redis.call('EXPIRE', blocked_key, window_seconds)
                
                return {0, 0, now + window_seconds, current_count}
            end
            
            -- Add current request
            redis.call('ZADD', key, now, request_id)
            
            -- Set TTL
            redis.call('EXPIRE', key, window_seconds)
            
            local remaining = limit - current_count - 1
            return {1, remaining, now + window_seconds, current_count + 1}
        "#;
        
        let stats_key = format!("{}:{}:stats", RATE_LIMIT_STATS_REDIS_PREFIX, endpoint_class);
        let blocked_key = format!("{}:{}:blocked", RATE_LIMIT_BLOCKED_REDIS_PREFIX, key);
        
        // Execute script
        let result: Vec<u64> = redis::cmd("EVAL")
            .arg(script)
            .arg(3) // Number of keys
            .arg(key)
            .arg(&stats_key)
            .arg(&blocked_key)
            .arg(window_start)
            .arg(now)
            .arg(limit)
            .arg(window.as_secs())
            .arg(&request_id)
            .query_async(&mut conn)
            .await
            .map_err(|e| ApiError::Internal(format!("Redis rate limit script failed: {}", e)))?;

        let allowed = result[0] == 1;
        let remaining = result[1] as u32;
        let reset_time = result[2];
        let current_requests = result[3] as u32;

        // Store detailed statistics in background (non-blocking)
        if !allowed {
            self.record_rate_limit_violation(endpoint_class, user_id, ip_address, limit, current_requests).await;
        }

        debug!(
            key = %key,
            allowed = allowed,
            limit = limit,
            remaining = remaining,
            current_requests = current_requests,
            endpoint_class = endpoint_class,
            "Rate limit check completed"
        );

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_time,
            current_requests,
            limit,
            window_seconds: window.as_secs(),
        })
    }

    /// Get rate limiting statistics for an endpoint class
    pub async fn get_endpoint_stats(&self, endpoint_class: &str) -> Result<RateLimitStats, ApiError> {
        let mut conn = self.redis.clone();
        let stats_key = format!("{}:{}:stats", RATE_LIMIT_STATS_REDIS_PREFIX, endpoint_class);
        
        let stats: Vec<String> = conn
            .hmget(&stats_key, &["total_requests", "blocked_requests"])
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to get rate limit stats: {}", e)))?;

        let total_requests = stats.get(0)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let blocked_requests = stats.get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let violation_rate = if total_requests > 0 {
            (blocked_requests as f64) / (total_requests as f64)
        } else {
            0.0
        };

        // Get top violators from database
        let top_violators = self.get_top_violators(endpoint_class).await?;

        Ok(RateLimitStats {
            endpoint_class: endpoint_class.to_string(),
            total_requests,
            blocked_requests,
            violation_rate,
            top_violators,
            timestamp: Utc::now(),
        })
    }

    /// Get comprehensive rate limiting statistics
    pub async fn get_comprehensive_stats(&self) -> Result<Vec<RateLimitStats>, ApiError> {
        let endpoint_classes = vec![
            "health", "auth", "bridge", "quantum", "user", "admin", "docs", "websocket", "general"
        ];

        let mut all_stats = Vec::new();
        for endpoint_class in endpoint_classes {
            match self.get_endpoint_stats(endpoint_class).await {
                Ok(stats) => all_stats.push(stats),
                Err(e) => {
                    warn!(
                        endpoint_class = endpoint_class,
                        error = ?e,
                        "Failed to get stats for endpoint class"
                    );
                }
            }
        }

        Ok(all_stats)
    }

    /// Get top rate limit violators
    pub async fn get_top_violators(&self, endpoint_class: &str) -> Result<Vec<ViolatorInfo>, ApiError> {
        let query = r#"
            SELECT ip_address, user_id, violation_count, last_violation
            FROM rate_limit_violations 
            WHERE endpoint_class = $1 
                AND created_at > NOW() - INTERVAL '24 hours'
            ORDER BY violation_count DESC 
            LIMIT $2
        "#;

        let rows = sqlx::query(query)
            .bind(endpoint_class)
            .bind(RATE_LIMIT_TOP_VIOLATORS_COUNT as i32)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to get top violators: {}", e)))?;

        let mut violators = Vec::new();
        for row in rows {
            violators.push(ViolatorInfo {
                ip_address: row.get("ip_address"),
                user_id: row.get("user_id"),
                violation_count: row.get::<i64, _>("violation_count") as u64,
                last_violation: row.get("last_violation"),
            });
        }

        Ok(violators)
    }

    /// Record a rate limit violation in the database
    async fn record_rate_limit_violation(
        &self,
        endpoint_class: &str,
        user_id: Option<&str>,
        ip_address: &str,
        limit: u32,
        current_requests: u32,
    ) {
        let query = r#"
            INSERT INTO rate_limit_violations (
                endpoint_class, user_id, ip_address, limit_exceeded, current_requests, created_at, last_violation, violation_count
            ) VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), 1)
            ON CONFLICT (endpoint_class, COALESCE(user_id, ''), ip_address)
            DO UPDATE SET 
                violation_count = rate_limit_violations.violation_count + 1,
                last_violation = NOW(),
                current_requests = $5
        "#;

        if let Err(e) = sqlx::query(query)
            .bind(endpoint_class)
            .bind(user_id)
            .bind(ip_address)
            .bind(limit as i32)
            .bind(current_requests as i32)
            .execute(&self.db_pool)
            .await
        {
            error!(
                endpoint_class = endpoint_class,
                user_id = user_id,
                ip_address = ip_address,
                error = ?e,
                "Failed to record rate limit violation"
            );
        }
    }

    /// Check if an IP or user should be alerted based on violation patterns
    pub async fn check_alert_conditions(&self, endpoint_class: &str) -> Result<Vec<AlertCondition>, ApiError> {
        let query = r#"
            SELECT ip_address, user_id, violation_count, 
                COUNT(*) OVER() as total_requests,
                violation_count::float / COUNT(*) OVER() as violation_rate
            FROM rate_limit_violations 
            WHERE endpoint_class = $1 
                AND created_at > NOW() - INTERVAL '5 minutes'
                AND violation_count >= $2
            HAVING COUNT(*) >= $3
        "#;

        let rows = sqlx::query(query)
            .bind(endpoint_class)
            .bind(RATE_LIMIT_ALERT_MIN_REQUESTS_THRESHOLD as i64)
            .bind(RATE_LIMIT_ALERT_MIN_REQUESTS_THRESHOLD as i64)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to check alert conditions: {}", e)))?;

        let mut alerts = Vec::new();
        for row in rows {
            let violation_rate: f64 = row.get("violation_rate");
            if violation_rate >= RATE_LIMIT_ALERT_VIOLATION_RATE_THRESHOLD {
                alerts.push(AlertCondition {
                    endpoint_class: endpoint_class.to_string(),
                    ip_address: row.get("ip_address"),
                    user_id: row.get("user_id"),
                    violation_count: row.get::<i64, _>("violation_count") as u64,
                    violation_rate,
                    severity: if violation_rate > 0.95 { AlertSeverity::Critical } else { AlertSeverity::High },
                });
            }
        }

        Ok(alerts)
    }
}

/// Result of rate limit check
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_time: u64,
    pub current_requests: u32,
    pub limit: u32,
    pub window_seconds: u64,
}

/// Rate limiting statistics for an endpoint class
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RateLimitStats {
    pub endpoint_class: String,
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub violation_rate: f64,
    pub top_violators: Vec<ViolatorInfo>,
    pub timestamp: DateTime<Utc>,
}

/// Information about a rate limit violator
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ViolatorInfo {
    pub ip_address: String,
    pub user_id: Option<String>,
    pub violation_count: u64,
    pub last_violation: DateTime<Utc>,
}

/// Alert condition for rate limiting violations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AlertCondition {
    pub endpoint_class: String,
    pub ip_address: String,
    pub user_id: Option<String>,
    pub violation_count: u64,
    pub violation_rate: f64,
    pub severity: AlertSeverity,
}

/// Alert severity levels
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum AlertSeverity {
    High,
    Critical,
}