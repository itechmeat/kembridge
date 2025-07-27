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

mod config;
mod routes;
mod middleware;
mod handlers;
mod extractors;
mod models;
mod services;
mod utils;
mod state;
mod websocket;

// Services are used via full paths in AppState

use config::AppConfig;
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
        (name = "Admin", description = "Administrative endpoints")
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

    // Create application state with dependency injection
    let app_state = AppState::new(db_pool, redis_manager, config.clone()).await?;

    // Build application with comprehensive middleware stack
    let app = create_application(app_state).await?;

    // Start server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("🚀 KEMBridge API Gateway starting on {}", addr);
    tracing::info!("📋 Health check available at http://localhost:{}/health", config.port);
    tracing::info!("📖 API documentation at http://localhost:{}/docs", config.port);

    // Graceful shutdown handling
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn create_application(state: AppState) -> anyhow::Result<Router> {
    let app = Router::new()
        // Health & Status endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        .route("/metrics", get(handlers::health::metrics))

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
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
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
    axum::response::Redirect::temporary("https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui-bundle.js")
}

async fn swagger_ui_standalone_preset_js() -> impl axum::response::IntoResponse {
    // Using CDN instead of embedded files for simplicity
    axum::response::Redirect::temporary("https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui-standalone-preset.js")
}

async fn swagger_ui_css() -> impl axum::response::IntoResponse {
    // Using CDN instead of embedded files for simplicity
    axum::response::Redirect::temporary("https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui.css")
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
                .unwrap_or_else(|_| "kembridge_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    Ok(())
}

// Re-export AppState from state module
pub use state::AppState;

