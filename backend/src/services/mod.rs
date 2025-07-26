// src/services/mod.rs - Service layer modules
#![allow(dead_code)]

use crate::config::AppConfig;
use anyhow::Result;
use redis::aio::ConnectionManager;

/// Authentication service - Web3 wallet authentication with JWT
pub use kembridge_auth::AuthService;

/// User management service - implemented in Phase 2.3
pub mod user;
pub use user::UserService;

/// Quantum cryptography service - implemented in Phase 3.2
pub mod quantum;
pub use quantum::{QuantumService, QuantumServiceError};

#[cfg(test)]
mod user_service_tests;

/// Bridge service - will be implemented in Phase 4.3
/// TODO: Phase 4.3 - Replace with real cross-chain bridge orchestration
pub struct BridgeService;

impl BridgeService {
    pub async fn new(
        _db: sqlx::PgPool,
        _quantum_service: std::sync::Arc<QuantumService>,
        _config: &AppConfig,
    ) -> Result<Self> {
        Ok(Self)
    }
}


/// AI risk engine client - will be implemented in Phase 5.1
/// TODO: Phase 5.1 - Replace with real FastAPI ML risk analysis client
pub struct AiClient;

impl AiClient {
    pub fn new(_ai_engine_url: &str) -> Result<Self> {
        Ok(Self)
    }
}