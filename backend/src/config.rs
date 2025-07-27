// src/config.rs - Production-ready configuration management
use serde::{Deserialize, Serialize};
use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Server configuration
    pub port: u16,
    pub host: String,
    pub environment: Environment,

    // Database configuration
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,

    // Redis configuration
    pub redis_url: String,
    pub redis_pool_size: u32,

    // Security configuration
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub cors_origins: Vec<String>,

    // External services
    pub ai_engine_url: String,
    pub ai_engine_api_key: Option<String>,
    pub ai_engine_timeout_ms: u64,
    pub ai_engine_max_retries: u32,
    pub ethereum_rpc_url: String,
    pub near_rpc_url: String,

    // Feature flags
    pub enable_quantum_crypto: bool,
    pub enable_ai_risk_analysis: bool,
    pub enable_swagger_ui: bool,

    // Risk Analysis Configuration
    pub risk_low_threshold: f64,
    pub risk_medium_threshold: f64,
    pub risk_high_threshold: f64,
    pub risk_auto_block_threshold: f64,
    pub risk_manual_review_threshold: f64,
    pub risk_admin_bypass_enabled: bool,

    // Monitoring & Observability
    pub metrics_enabled: bool,
    pub tracing_level: String,
    pub prometheus_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            host: "0.0.0.0".to_string(),
            environment: Environment::Development,

            database_url: "postgresql://kembridge:kembridge@localhost:5432/kembridge".to_string(),
            database_max_connections: 10,
            database_min_connections: 1,

            redis_url: "redis://localhost:6379".to_string(),
            redis_pool_size: 10,

            jwt_secret: "dev-secret-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            cors_origins: vec![
                "http://localhost:4001".to_string(),
                "http://localhost:4000".to_string(),
            ],

            ai_engine_url: "http://localhost:4003".to_string(),
            ai_engine_api_key: None,
            ai_engine_timeout_ms: 5000,
            ai_engine_max_retries: 3,
            ethereum_rpc_url: "https://sepolia.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            near_rpc_url: "https://rpc.testnet.near.org".to_string(),

            enable_quantum_crypto: true,
            enable_ai_risk_analysis: true,
            enable_swagger_ui: true,

            // Risk thresholds with secure defaults
            risk_low_threshold: 0.3,
            risk_medium_threshold: 0.6,
            risk_high_threshold: 0.8,
            risk_auto_block_threshold: 0.9,
            risk_manual_review_threshold: 0.7,
            risk_admin_bypass_enabled: false, // Secure default

            metrics_enabled: true,
            tracing_level: "debug".to_string(),
            prometheus_endpoint: "/metrics".to_string(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables with fallback to defaults
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if it exists

        let mut config = Self::default();

        // Server configuration
        if let Ok(port) = env::var("PORT") {
            config.port = port.parse()
                .context("Invalid PORT value")?;
        }

        if let Ok(host) = env::var("HOST") {
            config.host = host;
        }

        if let Ok(env) = env::var("ENVIRONMENT") {
            config.environment = match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "testing" | "test" => Environment::Testing,
                "staging" => Environment::Staging,
                "production" | "prod" => Environment::Production,
                _ => Environment::Development,
            };
        }

        // Database configuration
        if let Ok(database_url) = env::var("DATABASE_URL") {
            config.database_url = database_url;
        }

        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            config.database_max_connections = max_conn.parse()
                .context("Invalid DATABASE_MAX_CONNECTIONS value")?;
        }

        // Redis configuration
        if let Ok(redis_url) = env::var("REDIS_URL") {
            config.redis_url = redis_url;
        }

        // Security configuration
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        } else if matches!(config.environment, Environment::Production) {
            anyhow::bail!("JWT_SECRET must be set in production environment");
        }

        if let Ok(jwt_exp) = env::var("JWT_EXPIRATION_HOURS") {
            config.jwt_expiration_hours = jwt_exp.parse()
                .context("Invalid JWT_EXPIRATION_HOURS value")?;
        }

        if let Ok(origins) = env::var("CORS_ORIGINS") {
            config.cors_origins = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // External services
        if let Ok(ai_url) = env::var("AI_ENGINE_URL") {
            config.ai_engine_url = ai_url;
        }

        config.ai_engine_api_key = env::var("AI_ENGINE_API_KEY").ok();

        if let Ok(timeout) = env::var("AI_ENGINE_TIMEOUT_MS") {
            config.ai_engine_timeout_ms = timeout.parse()
                .context("Invalid AI_ENGINE_TIMEOUT_MS value")?;
        }

        if let Ok(retries) = env::var("AI_ENGINE_MAX_RETRIES") {
            config.ai_engine_max_retries = retries.parse()
                .context("Invalid AI_ENGINE_MAX_RETRIES value")?;
        }

        if let Ok(eth_rpc) = env::var("ETHEREUM_RPC_URL") {
            config.ethereum_rpc_url = eth_rpc;
        }

        if let Ok(near_rpc) = env::var("NEAR_RPC_URL") {
            config.near_rpc_url = near_rpc;
        }

        // Feature flags
        if let Ok(quantum) = env::var("ENABLE_QUANTUM_CRYPTO") {
            config.enable_quantum_crypto = quantum.parse().unwrap_or(true);
        }

        if let Ok(ai_risk) = env::var("ENABLE_AI_RISK_ANALYSIS") {
            config.enable_ai_risk_analysis = ai_risk.parse().unwrap_or(true);
        }

        if let Ok(swagger) = env::var("ENABLE_SWAGGER_UI") {
            config.enable_swagger_ui = swagger.parse().unwrap_or(true);
        }

        // Risk Analysis Configuration
        if let Ok(low_threshold) = env::var("RISK_LOW_THRESHOLD") {
            config.risk_low_threshold = low_threshold.parse()
                .context("Invalid RISK_LOW_THRESHOLD value")?;
        }

        if let Ok(medium_threshold) = env::var("RISK_MEDIUM_THRESHOLD") {
            config.risk_medium_threshold = medium_threshold.parse()
                .context("Invalid RISK_MEDIUM_THRESHOLD value")?;
        }

        if let Ok(high_threshold) = env::var("RISK_HIGH_THRESHOLD") {
            config.risk_high_threshold = high_threshold.parse()
                .context("Invalid RISK_HIGH_THRESHOLD value")?;
        }

        if let Ok(auto_block_threshold) = env::var("RISK_AUTO_BLOCK_THRESHOLD") {
            config.risk_auto_block_threshold = auto_block_threshold.parse()
                .context("Invalid RISK_AUTO_BLOCK_THRESHOLD value")?;
        }

        if let Ok(manual_review_threshold) = env::var("RISK_MANUAL_REVIEW_THRESHOLD") {
            config.risk_manual_review_threshold = manual_review_threshold.parse()
                .context("Invalid RISK_MANUAL_REVIEW_THRESHOLD value")?;
        }

        if let Ok(admin_bypass) = env::var("RISK_ADMIN_BYPASS_ENABLED") {
            config.risk_admin_bypass_enabled = admin_bypass.parse().unwrap_or(false);
        }

        // Monitoring
        if let Ok(metrics) = env::var("METRICS_ENABLED") {
            config.metrics_enabled = metrics.parse().unwrap_or(true);
        }

        if let Ok(tracing) = env::var("TRACING_LEVEL") {
            config.tracing_level = tracing;
        }

        Ok(config)
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    /// Get database configuration for SQLx
    pub fn database_config(&self) -> DatabaseConfig {
        DatabaseConfig {
            url: self.database_url.clone(),
            max_connections: self.database_max_connections,
            min_connections: self.database_min_connections,
        }
    }

    /// Get Redis configuration
    pub fn redis_config(&self) -> RedisConfig {
        RedisConfig {
            url: self.redis_url.clone(),
            pool_size: self.redis_pool_size,
        }
    }

    /// Get risk thresholds from configuration
    pub fn risk_thresholds(&self) -> crate::models::risk::RiskThresholds {
        crate::models::risk::RiskThresholds {
            low_threshold: self.risk_low_threshold,
            medium_threshold: self.risk_medium_threshold,
            high_threshold: self.risk_high_threshold,
            auto_block_threshold: self.risk_auto_block_threshold,
            manual_review_threshold: self.risk_manual_review_threshold,
        }
    }

    /// Get Ethereum configuration for blockchain adapter (Phase 5.2.7)
    pub fn ethereum_config(&self) -> kembridge_blockchain::ethereum::EthereumConfig {
        kembridge_blockchain::ethereum::EthereumConfig {
            rpc_url: self.ethereum_rpc_url.clone(),
            chain_id: 11155111, // Sepolia testnet
            gas_price_multiplier: 1.2,
            confirmation_blocks: 2,
            max_retry_attempts: 3,
        }
    }

    /// Get NEAR configuration for blockchain adapter (Phase 5.2.7)
    pub fn near_config(&self) -> kembridge_blockchain::near::NearConfig {
        kembridge_blockchain::near::NearConfig {
            network_id: "testnet".to_string(),
            rpc_url: self.near_rpc_url.clone(),
            helper_url: Some("https://helper.testnet.near.org".to_string()),
            explorer_url: Some("https://explorer.testnet.near.org".to_string()),
            wallet_url: Some("https://wallet.testnet.near.org".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

// Type alias removed - use AppConfig directly for clarity

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.port, 4000);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_quantum_crypto);
        assert!(config.enable_ai_risk_analysis);
    }

    #[test]
    fn test_from_env() {
        env::set_var("PORT", "8080");
        env::set_var("ENVIRONMENT", "production");
        env::set_var("JWT_SECRET", "test-secret");
        
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.port, 8080);
        assert!(config.is_production());
        assert_eq!(config.jwt_secret, "test-secret");

        // Cleanup
        env::remove_var("PORT");
        env::remove_var("ENVIRONMENT");
        env::remove_var("JWT_SECRET");
    }
}