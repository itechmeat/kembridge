use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub ethereum_rpc_url: String,
    pub near_rpc_url: String,
    pub quantum_key_storage_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?,
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            ethereum_rpc_url: std::env::var("ETHEREUM_RPC_URL")
                .map_err(|_| anyhow::anyhow!("ETHEREUM_RPC_URL must be set"))?,
            near_rpc_url: std::env::var("NEAR_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.testnet.near.org".to_string()),
            quantum_key_storage_path: std::env::var("QUANTUM_KEY_STORAGE_PATH")
                .unwrap_or_else(|_| "./keys".to_string()),
        };

        Ok(config)
    }
}