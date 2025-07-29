// Simple service clients for gateway
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct ServiceClients {
    pub http_client: Client,
    pub oneinch_url: String,
    pub blockchain_url: String,
    pub crypto_url: String,
    pub auth_url: String,
}

impl ServiceClients {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            oneinch_url: std::env::var("ONEINCH_SERVICE_URL")
                .unwrap_or_else(|_| "http://oneinch-service:4001".to_string()),
            blockchain_url: std::env::var("BLOCKCHAIN_SERVICE_URL")
                .unwrap_or_else(|_| "http://blockchain-service:4002".to_string()),
            crypto_url: std::env::var("CRYPTO_SERVICE_URL")
                .unwrap_or_else(|_| "http://crypto-service:4003".to_string()),
            auth_url: std::env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:4004".to_string()),
        }
    }
}