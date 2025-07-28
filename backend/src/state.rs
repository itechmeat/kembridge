// src/state.rs - Application State with Dependency Injection (Phase 5.2.2)
use std::sync::Arc;
use redis::aio::ConnectionManager;

use crate::config::AppConfig;
use crate::services::{
    AuthService, UserService, BridgeService, QuantumService, AiClient,
    RiskIntegrationService, ManualReviewService, BridgeIntegrationService,
    RateLimitService,
};
use crate::websocket::WebSocketRegistry;
use crate::monitoring::MonitoringService;
use crate::price_oracle::PriceOracleService;
use crate::oneinch::OneinchService;
use crate::dynamic_pricing::DynamicPricingService;
use kembridge_database::TransactionService;

/// Application state with dependency injection for all services
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: ConnectionManager,
    pub redis_pool: deadpool_redis::Pool,
    pub config: AppConfig,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub bridge_service: Option<Arc<BridgeService>>,
    pub quantum_service: Arc<QuantumService>,
    pub ai_client: Arc<AiClient>,
    pub risk_integration_service: Arc<RiskIntegrationService>,
    pub manual_review_service: Arc<ManualReviewService>,
    pub transaction_service: Arc<TransactionService>,
    pub websocket_registry: Arc<WebSocketRegistry>,
    pub monitoring_service: Arc<MonitoringService>,
    pub price_oracle_service: Arc<PriceOracleService>,
    pub oneinch_service: Arc<OneinchService>,
    pub dynamic_pricing_service: Arc<DynamicPricingService>,
    pub bridge_integration_service: Arc<BridgeIntegrationService>,
    pub rate_limit_service: Arc<RateLimitService>,
    pub metrics: Arc<metrics_exporter_prometheus::PrometheusHandle>,
}

impl AppState {
    /// Create new application state with all initialized services
    pub async fn new(
        db: sqlx::PgPool,
        redis: ConnectionManager,
        redis_pool: deadpool_redis::Pool,
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
        // TODO (MOCK WARNING): Temporary fallback while fixing Ethereum adapter configuration
        let bridge_service = match BridgeService::new(
            db.clone(),
            quantum_service.clone(),
            &config
        ).await {
            Ok(service) => {
                tracing::info!("Successfully initialized BridgeService");
                Some(Arc::new(service.with_risk_integration(risk_integration_service.clone())))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize BridgeService: {}. Continuing without bridge service for dynamic pricing testing.", e);
                None
            }
        };

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

        // Initialize price oracle service (Phase 6.1)
        let price_oracle_service = Arc::new(
            PriceOracleService::new(redis.clone(), Arc::new(config.clone())).await?
        );

        // Initialize 1inch service with Fusion+ (Phase 6.2)
        let oneinch_service = Arc::new(
            OneinchService::new(
                config.oneinch_api_key.clone().unwrap_or_else(|| "test_key".to_string()),
                config.ethereum_chain_id.unwrap_or(11155111), // Default to Sepolia testnet
            )
            .with_price_oracle(price_oracle_service.clone())
            .with_fusion_plus(config.oneinch_api_key.clone()) // Enable Fusion+ cross-chain functionality
        );

        // Initialize dynamic pricing service (Phase 6.3)
        let dynamic_pricing_service = Arc::new(
            DynamicPricingService::new(
                price_oracle_service.clone(),
                oneinch_service.clone(),
            )
        );

        // Initialize bridge integration service (Phase 6.2.1)
        let bridge_integration_service = Arc::new(
            BridgeIntegrationService::new(
                oneinch_service.clone(),
                bridge_service.clone(),
            )
        );

        // Initialize rate limiting service (Phase 7 - H7)
        let rate_limit_service = Arc::new(
            RateLimitService::new(redis.clone(), db.clone())
        );

        tracing::info!(
            "AppState initialized with {} services including risk integration, manual review, transaction service, WebSocket registry, monitoring service, price oracle service, 1inch Fusion+ service, dynamic pricing service, bridge integration service, and rate limiting service",
            16 // auth, user, bridge, quantum, ai, risk, manual_review, transaction, websocket, monitoring, price_oracle, oneinch, dynamic_pricing, bridge_integration, rate_limit, metrics
        );

        Ok(Self {
            db,
            redis,
            redis_pool,
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
            price_oracle_service,
            oneinch_service,
            dynamic_pricing_service,
            bridge_integration_service,
            rate_limit_service,
            metrics,
        })
    }
}