// src/state.rs - Application State with Dependency Injection (Phase 5.2.2)
use std::sync::Arc;
use redis::aio::ConnectionManager;

use crate::config::AppConfig;
use crate::services::{
    AuthService, UserService, BridgeService, QuantumService, AiClient,
    RiskIntegrationService, ManualReviewService,
};
use crate::websocket::WebSocketRegistry;
use crate::monitoring::MonitoringService;
use kembridge_database::TransactionService;

/// Application state with dependency injection for all services
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: ConnectionManager,
    pub config: AppConfig,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub bridge_service: Arc<BridgeService>,
    pub quantum_service: Arc<QuantumService>,
    pub ai_client: Arc<AiClient>,
    pub risk_integration_service: Arc<RiskIntegrationService>,
    pub manual_review_service: Arc<ManualReviewService>,
    pub transaction_service: Arc<TransactionService>,
    pub websocket_registry: Arc<WebSocketRegistry>,
    pub monitoring_service: Arc<MonitoringService>,
    pub metrics: Arc<metrics_exporter_prometheus::PrometheusHandle>,
}

impl AppState {
    /// Create new application state with all initialized services
    pub async fn new(
        db: sqlx::PgPool,
        redis: ConnectionManager,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        // Initialize services with dependency injection
        let auth_service = Arc::new(
            AuthService::new(
                db.clone(), 
                redis.clone(), 
                config.jwt_secret.clone()
            ).await?
        );

        let quantum_service = Arc::new(
            QuantumService::new(db.clone(), &config).await?
        );

        let ai_client = Arc::new(
            AiClient::new(&config.ai_engine_url)?
        );

        // Initialize manual review service (Phase 5.2.4)
        let manual_review_service = Arc::new(
            ManualReviewService::new(db.clone())
        );

        // Initialize transaction service (Phase 5.2.5)
        let transaction_service = Arc::new(
            TransactionService::new(db.clone())
        );

        // Initialize risk integration service with manual review integration (Phase 5.2)
        let risk_integration_service = Arc::new(
            RiskIntegrationService::new_with_manual_review(&config, db.clone(), manual_review_service.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize risk integration service: {}", e))?
        );

        // Initialize bridge service with risk integration (Phase 5.2.7)
        let bridge_service = Arc::new(
            BridgeService::new(
                db.clone(),
                quantum_service.clone(),
                &config
            ).await?
            .with_risk_integration(risk_integration_service.clone())
        );

        // Initialize user service with risk integration (Phase 5.2.7)
        let user_service = Arc::new(
            UserService::with_risk_integration(db.clone(), risk_integration_service.clone())
        );

        let metrics = metrics_exporter_prometheus::PrometheusBuilder::new()
            .build_recorder()
            .handle();
        let metrics = Arc::new(metrics);

        // Initialize WebSocket registry (Phase 5.3.1)
        let websocket_registry = Arc::new(WebSocketRegistry::new());

        // Initialize monitoring service (Phase 5.3.2)
        let monitoring_service = Arc::new(
            MonitoringService::new(websocket_registry.clone())
                .with_redis(redis.clone())
                .await
        );

        tracing::info!(
            "AppState initialized with {} services including risk integration, manual review, transaction service, WebSocket registry, and monitoring service",
            11 // auth, user, bridge, quantum, ai, risk, manual_review, transaction, websocket, monitoring, metrics
        );

        Ok(Self {
            db,
            redis,
            config,
            auth_service,
            user_service,
            bridge_service,
            quantum_service,
            ai_client,
            risk_integration_service,
            manual_review_service,
            transaction_service,
            websocket_registry,
            monitoring_service,
            metrics,
        })
    }
}