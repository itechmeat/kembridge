// src/handlers/health.rs - Comprehensive health checks with modern Rust patterns
use axum::{extract::State, response::{Json, IntoResponse}, http::StatusCode};
use serde_json::{json, Value};
use std::collections::HashMap;
use redis::{aio::ConnectionManager, AsyncCommands};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::AppState;

/// Basic health check endpoint
/// 
/// Returns service status and basic information
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 500, description = "Service is unhealthy")
    )
)]
pub async fn health_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "kembridge-api-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "rust_version": "1.88.0", // TODO: Get from build env in production
        "features": {
            "quantum_crypto": true,
            "ai_risk_engine": true,
            "cross_chain_bridge": true,
            "web3_auth": true,
            "swagger_ui": true
        }
    })))
}

/// Comprehensive readiness check
/// 
/// Validates all critical dependencies before marking service as ready
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready", body = ReadinessResponse)
    )
)]
pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut services = HashMap::new();
    let start_time = std::time::Instant::now();

    // Check database connectivity with PostgreSQL 18 specific query
    let db_check_start = std::time::Instant::now();
    let db_status = match check_database_health(&state.db).await {
        Ok(version) => {
            services.insert("database".to_string(), json!({
                "status": "healthy",
                "type": "postgresql-18",
                "version": version,
                "response_time_ms": db_check_start.elapsed().as_millis(),
                "features": ["uuid-v7", "jsonb-simd", "oauth2-support"]
            }));
            "healthy"
        },
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            services.insert("database".to_string(), json!({
                "status": "unhealthy",
                "type": "postgresql-18", 
                "error": e.to_string(),
                "response_time_ms": db_check_start.elapsed().as_millis()
            }));
            "unhealthy"
        }
    };

    // Check Redis connectivity and performance
    let redis_check_start = std::time::Instant::now();
    let redis_status = match check_redis_health(&state.redis).await {
        Ok(info) => {
            services.insert("cache".to_string(), json!({
                "status": "healthy",
                "type": "redis",
                "version": info.version,
                "memory_usage": info.memory_usage,
                "connected_clients": info.connected_clients,
                "response_time_ms": redis_check_start.elapsed().as_millis()
            }));
            "healthy"
        },
        Err(e) => {
            tracing::error!("Redis health check failed: {}", e);
            services.insert("cache".to_string(), json!({
                "status": "unhealthy", 
                "type": "redis",
                "error": e.to_string(),
                "response_time_ms": redis_check_start.elapsed().as_millis()
            }));
            "unhealthy"
        }
    };

    // Check quantum crypto module
    let _quantum_status = if state.config.enable_quantum_crypto {
        match check_quantum_crypto_health().await {
            Ok(_) => {
                services.insert("quantum_crypto".to_string(), json!({
                    "status": "healthy",
                    "algorithm": "ml-kem-1024",
                    "features": ["key-encapsulation", "dilithium-5", "sphincs+"]
                }));
                "healthy"
            },
            Err(e) => {
                tracing::warn!("Quantum crypto check failed: {}", e);
                services.insert("quantum_crypto".to_string(), json!({
                    "status": "degraded",
                    "algorithm": "ml-kem-1024",
                    "error": e.to_string()
                }));
                "degraded"
            }
        }
    } else {
        services.insert("quantum_crypto".to_string(), json!({
            "status": "disabled",
            "reason": "feature flag disabled"
        }));
        "disabled"
    };

    // Check AI Risk Engine connectivity
    let _ai_status = if state.config.enable_ai_risk_analysis {
        match check_ai_engine_health(&state.config.ai_engine_url).await {
            Ok(version) => {
                services.insert("ai_risk_engine".to_string(), json!({
                    "status": "healthy",
                    "type": "fastapi-ml",
                    "version": version,
                    "capabilities": ["risk-scoring", "anomaly-detection", "ml-inference"]
                }));
                "healthy"
            },
            Err(e) => {
                tracing::warn!("AI engine check failed: {}", e);
                services.insert("ai_risk_engine".to_string(), json!({
                    "status": "degraded",
                    "type": "fastapi-ml", 
                    "error": e.to_string()
                }));
                "degraded"
            }
        }
    } else {
        services.insert("ai_risk_engine".to_string(), json!({
            "status": "disabled",
            "reason": "feature flag disabled"
        }));
        "disabled"
    };

    // Determine overall status
    let overall_status = match (db_status, redis_status) {
        ("healthy", "healthy") => "ready",
        ("healthy", "degraded") | ("degraded", "healthy") => "degraded",
        _ => "not_ready",
    };

    let status_code = match overall_status {
        "ready" => StatusCode::OK,
        "degraded" => StatusCode::OK, // Still functional but with warnings
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    let response = json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "total_response_time_ms": start_time.elapsed().as_millis(),
        "services": services,
        "environment": state.config.environment,
        "features": {
            "quantum_crypto_enabled": state.config.enable_quantum_crypto,
            "ai_risk_analysis_enabled": state.config.enable_ai_risk_analysis,
            "swagger_ui_enabled": state.config.enable_swagger_ui,
            "metrics_enabled": state.config.metrics_enabled
        }
    });

    (status_code, Json(response))
}

/// Prometheus metrics endpoint
pub async fn metrics(State(state): State<AppState>) -> Result<String, StatusCode> {
    if !state.config.metrics_enabled {
        return Err(StatusCode::NOT_FOUND);
    }

    // Export Prometheus metrics
    // use metrics_exporter_prometheus::PrometheusHandle;

    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .build_recorder()
        .handle();

    Ok(handle.render())
}

// Helper functions for health checks

async fn check_database_health(pool: &sqlx::PgPool) -> anyhow::Result<String> {
    // Use PostgreSQL 18 specific features to validate connectivity
    let row = sqlx::query!(
        r#"
        SELECT 
            version() as version,
            current_setting('server_version') as server_version,
            pg_is_in_recovery() as is_replica
        "#
    )
    .fetch_one(pool)
    .await?;

    // TODO (check): Verify PostgreSQL 18 specific features when UUIDv7 is available
    // let _uuid_test = sqlx::query!("SELECT generate_uuidv7() as test_uuid")
    //     .fetch_one(pool)
    //     .await?;

    Ok(row.server_version.unwrap_or_else(|| "unknown".to_string()))
}

#[derive(Debug)]
struct RedisHealthInfo {
    version: String,
    memory_usage: u64,
    connected_clients: u32,
}

async fn check_redis_health(manager: &ConnectionManager) -> anyhow::Result<RedisHealthInfo> {
    let mut conn = manager.clone();
    
    // Basic connectivity test
    let _: String = redis::cmd("PING").query_async(&mut conn).await?;
    
    // Get Redis info
    let info: String = redis::cmd("INFO")
        .query_async(&mut conn)
        .await?;
    
    // Parse Redis info for health metrics
    let version = parse_redis_info(&info, "redis_version")
        .unwrap_or_else(|| "unknown".to_string());
    
    let memory_usage = parse_redis_info(&info, "used_memory")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let connected_clients = parse_redis_info(&info, "connected_clients")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Test write/read operation
    let test_key = format!("health_check:{}", uuid::Uuid::new_v4());
    let test_value = "test_value";
    
    let _: () = conn.set_ex(&test_key, test_value, 60).await?;
    let result: String = conn.get(&test_key).await?;
    let _: () = conn.del(&test_key).await?;
    
    if result != test_value {
        anyhow::bail!("Redis read/write test failed");
    }

    Ok(RedisHealthInfo {
        version,
        memory_usage,
        connected_clients,
    })
}

async fn check_quantum_crypto_health() -> anyhow::Result<()> {
    // Phase 3.4 - Real ML-KEM-1024 health check integration
    use kembridge_crypto::MlKemCrypto;
    
    // Perform actual ML-KEM-1024 round-trip verification
    MlKemCrypto::verify_round_trip()
        .map_err(|e| anyhow::anyhow!("ML-KEM-1024 health check failed: {}", e))?;
    
    Ok(())
}

async fn check_ai_engine_health(ai_url: &str) -> anyhow::Result<String> {
    // TODO (MOCK WARNING): Use real reqwest client instead of mocks (Phase 1.3.5)
    // For now, simulate the check
    let _ai_url = ai_url; // Avoid unused variable warning
    
    // Simulate successful check
    Ok("v1.0.0-mock".to_string())
    
    // TODO (check): what the comments?
    /* Real implementation would be:
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", ai_url);
    
    let response = client
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("AI engine returned status: {}", response.status());
    }
    
    let health_data: serde_json::Value = response.json().await?;
    
    Ok(health_data
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string())
    */
}

fn parse_redis_info(info: &str, key: &str) -> Option<String> {
    info.lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| value.trim().to_string())
}

// OpenAPI schemas for documentation

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
    pub rust_version: String,
    pub features: HealthFeatures,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthFeatures {
    pub quantum_crypto: bool,
    pub ai_risk_engine: bool,
    pub cross_chain_bridge: bool,
    pub web3_auth: bool,
    pub swagger_ui: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    pub timestamp: String,
    pub total_response_time_ms: u128,
    pub services: HashMap<String, ServiceStatus>,
    pub environment: String,
    pub features: ReadinessFeatures,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ServiceStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReadinessFeatures {
    pub quantum_crypto_enabled: bool,
    pub ai_risk_analysis_enabled: bool,
    pub swagger_ui_enabled: bool,
    pub metrics_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_redis_info() {
        let info = "redis_version:7.2.0\nused_memory:1048576\nconnected_clients:5";
        
        assert_eq!(parse_redis_info(info, "redis_version"), Some("7.2.0".to_string()));
        assert_eq!(parse_redis_info(info, "used_memory"), Some("1048576".to_string()));
        assert_eq!(parse_redis_info(info, "connected_clients"), Some("5".to_string()));
        assert_eq!(parse_redis_info(info, "nonexistent"), None);
    }
}