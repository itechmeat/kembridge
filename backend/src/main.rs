// src/main.rs - Modern Rust 1.88+ setup with Axum 0.8.4
use axum::{
    routing::get,
    Router,
    Json,
};
use std::net::SocketAddr;
// use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use redis::aio::ConnectionManager;
use utoipa::OpenApi;

pub mod config;
pub mod routes;
pub mod middleware;
pub mod handlers;
pub mod extractors;
pub mod models;
pub mod services;
pub mod utils;
pub mod state;
pub mod websocket;
pub mod monitoring;
pub mod price_oracle;
pub mod oneinch;
pub mod constants;
pub mod dynamic_pricing;

// Services are used via full paths in AppState

use config::AppConfig;
use constants::*;
// use middleware::error_handler::handle_error;

// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "KEMBridge API Gateway",
        version = "0.1.0",
        description = "Quantum-secure cross-chain bridge API with ML-KEM-1024 post-quantum cryptography",
        contact(
            name = "KEMBridge Team",
            email = "dev@kembridge.io"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        handlers::health::health_check,
        handlers::health::readiness_check,
        handlers::quantum::generate_keypair,
        handlers::quantum::encapsulate,
        handlers::quantum::decapsulate,
        handlers::quantum::get_user_keys,
        handlers::quantum::export_public_key,
        handlers::quantum::rotate_key,
        handlers::quantum::check_rotation_needed,
        handlers::quantum::admin_check_rotation,
        handlers::quantum::hybrid_rotate_key,
        handlers::risk::get_user_risk_profile,
        handlers::risk::get_risk_thresholds,
        handlers::risk::update_risk_thresholds,
        handlers::risk::get_risk_engine_health,
        handlers::risk::test_risk_analysis,
        handlers::manual_review::add_to_review_queue,
        handlers::manual_review::get_review_queue,
        handlers::manual_review::assign_review,
        handlers::manual_review::make_review_decision,
        handlers::manual_review::get_review_details,
        handlers::manual_review::escalate_review,
        handlers::manual_review::check_escalations,
        // 1inch Fusion+ endpoints
        handlers::oneinch::get_quote,
        handlers::oneinch::get_enhanced_quote,
        handlers::oneinch::execute_swap,
        handlers::oneinch::execute_signed_swap,
        handlers::oneinch::get_order_status,
        handlers::oneinch::get_supported_tokens,
        handlers::oneinch::get_intelligent_routing,
        handlers::oneinch::health_check,
        handlers::oneinch::comprehensive_health_check,
        handlers::oneinch::validate_api_key,
        handlers::oneinch::get_liquidity_info,
        // Bridge integration endpoints
        handlers::bridge_oneinch::execute_optimized_bridge_swap,
        handlers::bridge_oneinch::get_bridge_swap_status,
        handlers::bridge_oneinch::calculate_bridge_swap_savings,
        handlers::bridge_oneinch::get_supported_bridge_chains,
        // Fusion+ cross-chain endpoints
        handlers::fusion_plus::get_cross_chain_quote,
        handlers::fusion_plus::build_cross_chain_order,
        handlers::fusion_plus::submit_cross_chain_order,
        handlers::fusion_plus::get_active_cross_chain_orders,
        handlers::fusion_plus::get_cross_chain_order_by_hash,
        handlers::fusion_plus::get_escrow_factory,
        // Rate limiting monitoring endpoints
        handlers::rate_limiting::get_rate_limit_dashboard,
        handlers::rate_limiting::get_endpoint_rate_limits,
        handlers::rate_limiting::get_top_violators,
        handlers::rate_limiting::get_real_time_metrics,
        handlers::rate_limiting::get_active_alerts,
    ),
    components(
        schemas(
            handlers::health::HealthResponse,
            handlers::health::HealthFeatures,
            handlers::health::ReadinessResponse,
            handlers::health::ServiceStatus,
            handlers::health::ReadinessFeatures,
            models::quantum::CreateQuantumKeyRequest,
            models::quantum::QuantumKeyResponse,
            models::quantum::QuantumKeysListResponse,
            models::quantum::EncapsulateRequest,
            models::quantum::EncapsulateResponse,
            models::quantum::DecapsulateRequest,
            models::quantum::DecapsulateResponse,
            models::quantum::RotateKeyRequest,
            models::quantum::RotateKeyResponse,
            models::quantum::CheckRotationRequest,
            models::quantum::CheckRotationResponse,
            models::quantum::QuantumKeyRotationInfo,
            models::quantum::HybridRotateKeyRequest,
            models::quantum::HybridRotateKeyResponse,
            models::quantum::HybridRotationConfig,
            models::quantum::HybridEncryptionDetails,
            models::quantum::HybridKeySizes,
            handlers::risk::GetUserRiskProfileQuery,
            handlers::risk::RiskProfileResponse,
            handlers::risk::UpdateRiskThresholdsRequest,
            handlers::risk::RiskThresholdsResponse,
            handlers::risk::RiskEngineHealthResponse,
            models::review::ReviewStatus,
            models::review::ReviewPriority,
            models::review::ReviewQueueEntry,
            models::review::CreateReviewRequest,
            models::review::UpdateReviewRequest,
            models::review::ReviewQueueResponse,
            models::review::TransactionSummary,
            models::review::UserRiskSummary,
            models::review::ReviewDecision,
            models::review::ReviewQueueQuery,
            models::review::ReviewQueueListResponse,
            models::review::PaginationInfo,
            models::review::ReviewQueueStats,
            models::review::ReviewNotification,
            models::review::NotificationType,
            // 1inch Fusion+ schemas
            handlers::oneinch::QuoteRequest,
            handlers::oneinch::EnhancedQuoteRequest,
            handlers::oneinch::SwapExecutionRequest,
            handlers::oneinch::SignedSwapExecutionRequest,
            handlers::oneinch::IntelligentRoutingRequest,
            handlers::oneinch::OptimizationWeights,
            handlers::oneinch::QuoteResponse,
            handlers::oneinch::EnhancedQuoteResponse,
            handlers::oneinch::QuoteWithRating,
            handlers::oneinch::OracleComparisonData,
            handlers::oneinch::SwapResponse,
            handlers::oneinch::OrderStatusResponse,
            handlers::oneinch::TokenInfo,
            handlers::oneinch::ProtocolInfo,
            handlers::oneinch::FillInfo,
            handlers::oneinch::SupportedTokensResponse,
            handlers::oneinch::OneinchHealthResponse,
            handlers::oneinch::OneinchIntegrationHealthResponse,
            handlers::oneinch::ApiKeyValidationResponse,
            handlers::oneinch::LiquidityInfoResponse,
            handlers::oneinch::IntelligentRoutingResponse,
            handlers::oneinch::RouteInfo,
            handlers::oneinch::RouteScores,
            handlers::oneinch::SavingsInfo,
            // Bridge integration schemas
            handlers::bridge_oneinch::OptimizedBridgeSwapRequest,
            handlers::bridge_oneinch::OptimizedBridgeSwapResponse,
            handlers::bridge_oneinch::ChainOptimizationResponse,
            handlers::bridge_oneinch::OptimizationSummaryResponse,
            handlers::bridge_oneinch::BridgeSwapStatusResponse,
            // Fusion+ cross-chain schemas
            handlers::fusion_plus::CrossChainQuoteRequest,
            handlers::fusion_plus::CrossChainQuoteResponse,
            handlers::fusion_plus::TimeLocksInfo,
            handlers::fusion_plus::TokenPairPrices,
            handlers::fusion_plus::BuildOrderRequest,
            handlers::fusion_plus::BuildOrderResponse,
            handlers::fusion_plus::OrderInfo,
            handlers::fusion_plus::SubmitOrderRequest,
            handlers::fusion_plus::ActiveOrdersQuery,
            handlers::fusion_plus::ActiveOrdersResponse,
            handlers::fusion_plus::OrderSummary,
            // Rate limiting monitoring schemas
            handlers::rate_limiting::RateLimitStatsQuery,
            handlers::rate_limiting::RateLimitDashboard,
            handlers::rate_limiting::RateLimitOverview,
            handlers::rate_limiting::RealTimeMetrics,
            services::RateLimitStats,
            services::ViolatorInfo,
            services::AlertCondition,
            services::AlertSeverity,
        )
    ),
    tags(
        (name = "Health", description = "Health and status monitoring endpoints"),
        (name = "Authentication", description = "Web3 wallet authentication endpoints"),
        (name = "Bridge", description = "Cross-chain bridge operations"),
        (name = "Quantum", description = "Post-quantum cryptography operations"),
        (name = "User", description = "User management endpoints"),
        (name = "Risk Analysis", description = "AI-powered risk analysis and monitoring"),
        (name = "Manual Review", description = "Manual review queue management for suspicious transactions"),
        (name = "Admin", description = "Administrative endpoints"),
        (name = "1inch Swap", description = "1inch Fusion+ integration for optimal swap routing"),
        (name = "Fusion+", description = "1inch Fusion+ cross-chain atomic swaps"),
        (name = "Bridge Integration", description = "Cross-chain bridge with 1inch optimization"),
        (name = "Rate Limiting", description = "Rate limiting monitoring and statistics")
    ),
    servers(
        (url = "http://localhost:4000", description = "Development server"),
        (url = "https://api.kembridge.io", description = "Production server")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize comprehensive tracing
    init_tracing()?;

    // Load configuration from environment
    let config = AppConfig::from_env()?;

    // Initialize database connections
    let db_pool = kembridge_database::create_pool(&config.database_url).await?;
    kembridge_database::run_migrations(&db_pool).await?;

    // Initialize Redis connection manager
    let redis_manager = ConnectionManager::new(
        redis::Client::open(config.redis_url.as_str())?
    ).await?;

    // Initialize Redis connection pool for rate limiting
    let redis_pool = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    // Create application state with dependency injection
    let app_state = AppState::new(db_pool, redis_manager, redis_pool, config.clone()).await?;

    // Build application with comprehensive middleware stack
    let app = create_application(app_state).await?;

    // Start server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("🚀 KEMBridge API Gateway starting on {}", addr);
    tracing::info!("📋 Health check available at http://{}:{}/health", config.host, config.port);
    tracing::info!("📖 API documentation at http://{}:{}/docs", config.host, config.port);

    // Graceful shutdown handling
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn create_application(state: AppState) -> anyhow::Result<Router> {
    let app = Router::new()
        // Health & Status endpoints
        .route(API_ROUTE_HEALTH, get(handlers::health::health_check))
        .route(API_ROUTE_READY, get(handlers::health::readiness_check))
        .route(API_ROUTE_METRICS, get(handlers::health::metrics))

        // WebSocket routes (before authentication middleware)
        .merge(routes::websocket::websocket_routes())

        // API v1 routes
        .nest("/api/v1", create_v1_routes())
        
        // JWT Authentication middleware for all API routes
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware
        ))

        // OpenAPI documentation (enabled conditionally)
        .merge(create_docs_routes(&state.config))

        // Global middleware stack
        .layer(middleware::cors::create_cors_layer(&state.config))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT_BYTES))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())

        // Attach application state
        .with_state(state);

    Ok(app)
}

fn create_v1_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes
        .nest("/auth", routes::auth::create_routes())

        // Bridge operation routes
        .nest("/bridge", routes::bridge::create_routes())

        // Quantum cryptography routes
        .nest("/crypto", routes::quantum::create_routes())

        // User management routes
        .nest("/user", routes::user::create_routes())

        // Risk analysis routes
        .nest("/risk", routes::risk::create_routes())

        // Admin routes (protected)
        .nest("/admin", routes::admin::create_routes())

        // Monitoring dashboard routes (protected)
        .nest("/monitoring", routes::monitoring::monitoring_routes())
        
        // Rate limiting monitoring routes (admin only)
        .nest("/monitoring/rate-limits", routes::rate_limiting::create_rate_limiting_routes())
        
        // Price Oracle routes (protected)
        .nest("/price", routes::price_oracle::price_oracle_routes())
        
        // 1inch Fusion+ routes (protected)
        .nest("/swap", routes::oneinch::create_oneinch_routes())
        
        // 1inch Fusion+ cross-chain routes (protected)
        .nest("/fusion-plus", routes::fusion_plus::create_fusion_plus_routes())
        
        // Bridge-1inch integration routes (protected)
        .nest("/bridge-oneinch", routes::bridge_oneinch::create_bridge_oneinch_routes())
}

fn create_docs_routes(config: &AppConfig) -> Router<AppState> {
    if config.enable_swagger_ui {
        Router::new()
            .route("/api-docs/openapi.json", get(|| async { 
                Json(ApiDoc::openapi()) 
            }))
            // Full Swagger UI implementation
            .route("/docs", get(swagger_ui_handler))
            .route("/docs/", get(swagger_ui_handler))
            .route("/docs/swagger-ui-bundle.js", get(swagger_ui_bundle_js))
            .route("/docs/swagger-ui-standalone-preset.js", get(swagger_ui_standalone_preset_js))
            .route("/docs/swagger-ui.css", get(swagger_ui_css))
    } else {
        Router::new()
    }
}

async fn swagger_ui_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>KEMBridge API Documentation</title>
    <link rel="stylesheet" type="text/css" href="/docs/swagger-ui.css" />
    <style>
        html {
            box-sizing: border-box;
            overflow: -moz-scrollbars-vertical;
            overflow-y: scroll;
        }
        *, *:before, *:after {
            box-sizing: inherit;
        }
        body {
            margin:0;
            background: #fafafa;
        }
        .swagger-ui .topbar { display: none; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="/docs/swagger-ui-bundle.js" charset="UTF-8"></script>
    <script src="/docs/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: '/api-docs/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                validatorUrl: null,
                tryItOutEnabled: true,
                supportedSubmitMethods: ['get', 'post', 'put', 'delete', 'patch'],
                onComplete: function() {
                    console.log('KEMBridge Swagger UI loaded successfully');
                }
            });
        };
    </script>
</body>
</html>
    "#)
}

async fn swagger_ui_bundle_js() -> impl axum::response::IntoResponse {
    // Using CDN instead of embedded files for simplicity
    axum::response::Redirect::temporary(&format!("{}{}/swagger-ui-bundle.js", SWAGGER_UI_CDN_BASE, SWAGGER_UI_VERSION))
}

async fn swagger_ui_standalone_preset_js() -> impl axum::response::IntoResponse {
    // Using CDN instead of embedded files for simplicity
    axum::response::Redirect::temporary(&format!("{}{}/swagger-ui-standalone-preset.js", SWAGGER_UI_CDN_BASE, SWAGGER_UI_VERSION))
}

async fn swagger_ui_css() -> impl axum::response::IntoResponse {
    // Using CDN instead of embedded files for simplicity
    axum::response::Redirect::temporary(&format!("{}{}/swagger-ui.css", SWAGGER_UI_CDN_BASE, SWAGGER_UI_VERSION))
}


async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("🛑 Ctrl+C received, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("🛑 SIGTERM received, shutting down gracefully...");
        },
    }
}

fn init_tracing() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| DEFAULT_TRACING_FILTER.into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    Ok(())
}

// Re-export AppState from state module
pub use state::AppState;

