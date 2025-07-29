use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub redis_url: String,
    pub log_level: String,
    pub environment: String,
}

impl ServiceConfig {
    pub fn from_env(service_name: &str, default_port: u16) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .set_default("service_name", service_name)?
            .set_default("port", default_port)?
            .set_default("host", "0.0.0.0")?
            .set_default("log_level", "info")?
            .set_default("environment", "development")?
            .add_source(Environment::default().separator("_"))
            .build()?;

        let mut service_config: ServiceConfig = config.try_deserialize()?;
        
        // Override with environment variables if they exist
        if let Ok(port) = env::var("PORT") {
            service_config.port = port.parse().unwrap_or(default_port);
        }
        
        Ok(service_config)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub gateway_url: String,
    pub auth_service_url: String,
    pub crypto_service_url: String,
    pub blockchain_service_url: String,
    pub oneinch_service_url: String,
    pub request_timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:4000".to_string(),
            auth_service_url: "http://localhost:4004".to_string(),
            crypto_service_url: "http://localhost:4003".to_string(),
            blockchain_service_url: "http://localhost:4002".to_string(),
            oneinch_service_url: "http://localhost:4001".to_string(),
            request_timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name("gateway.toml").required(false))
            .add_source(Environment::default().separator("_"))
            .build()?;

        let mut gateway_config: GatewayConfig = config.try_deserialize().unwrap_or_default();
        
        // Environment variable overrides
        if let Ok(url) = env::var("AUTH_SERVICE_URL") {
            gateway_config.auth_service_url = url;
        }
        if let Ok(url) = env::var("CRYPTO_SERVICE_URL") {
            gateway_config.crypto_service_url = url;
        }
        if let Ok(url) = env::var("BLOCKCHAIN_SERVICE_URL") {
            gateway_config.blockchain_service_url = url;
        }
        if let Ok(url) = env::var("ONEINCH_SERVICE_URL") {
            gateway_config.oneinch_service_url = url;
        }
        
        Ok(gateway_config)
    }
}