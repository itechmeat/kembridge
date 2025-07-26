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
        )
    ),
    tags(
        (name = "Health", description = "Health and status monitoring endpoints"),
        (name = "Authentication", description = "Web3 wallet authentication endpoints"),
        (name = "Bridge", description = "Cross-chain bridge operations"),
        (name = "Quantum", description = "Post-quantum cryptography operations"),
        (name = "User", description = "User management endpoints"),
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

        // API v1 routes
        .nest("/api/v1", create_v1_routes())
        
        // JWT Authentication middleware for all API routes
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware
        ))

        // WebSocket for real-time updates
        .route("/ws", get(handlers::websocket::websocket_handler))

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

// Application state with dependency injection
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: ConnectionManager,
    pub config: AppConfig,
    pub auth_service: std::sync::Arc<services::AuthService>,
    pub user_service: std::sync::Arc<services::UserService>,
    pub bridge_service: std::sync::Arc<services::BridgeService>,
    pub quantum_service: std::sync::Arc<services::QuantumService>,
    pub ai_client: std::sync::Arc<services::AiClient>,
    pub metrics: std::sync::Arc<metrics_exporter_prometheus::PrometheusHandle>,
}

impl AppState {
    pub async fn new(
        db: sqlx::PgPool,
        redis: ConnectionManager,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        use std::sync::Arc;

        // Initialize services with dependency injection
        let auth_service = Arc::new(
            services::AuthService::new(
                db.clone(), 
                redis.clone(), 
                config.jwt_secret.clone()
            ).await?
        );

        let user_service = Arc::new(
            services::UserService::new(db.clone())
        );

        let quantum_service = Arc::new(
            services::QuantumService::new(db.clone(), &config).await?
        );

        let bridge_service = Arc::new(
            services::BridgeService::new(
                db.clone(),
                quantum_service.clone(),
                &config
            ).await?
        );

        let ai_client = Arc::new(
            services::AiClient::new(&config.ai_engine_url)?
        );

        let metrics = metrics_exporter_prometheus::PrometheusBuilder::new()
            .build_recorder()
            .handle();
        let metrics = Arc::new(metrics);

        Ok(Self {
            db,
            redis,
            config,
            auth_service,
            user_service,
            bridge_service,
            quantum_service,
            ai_client,
            metrics,
        })
    }
}