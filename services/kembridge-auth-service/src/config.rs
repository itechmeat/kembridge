use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub port: u16,
    pub service_name: String,
}

impl ServiceConfig {
    pub fn new() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if it exists

        let port = env::var("PORT")
            .or_else(|_| env::var("AUTH_SERVICE_PORT"))
            .unwrap_or_else(|_| "4004".to_string())
            .parse::<u16>()?;

        Ok(Self {
            port,
            service_name: "kembridge-auth-service".to_string(),
        })
    }
}