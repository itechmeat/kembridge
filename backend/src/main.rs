use axum::{
    http::{header, Method, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

use kembridge_auth as auth;
use kembridge_database as database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kembridge_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    // Setup database connection pool
    let db_pool = database::create_pool(&config.database_url).await?;
    
    // Run migrations
    database::run_migrations(&db_pool).await?;

    // Setup Redis connection
    let redis_client = redis::Client::open(config.redis_url.as_str())?;

    // Create application state
    let app_state = AppState {
        db: db_pool,
        redis: redis_client,
        config: config.clone(),
    };

    // Build our application with routes
    let app = create_router(app_state).await?;

    // Run our application
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("KEMBridge backend listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn create_router(state: AppState) -> anyhow::Result<Router> {
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Auth routes (will be added in next phases)
        .nest("/api/v1/auth", auth::routes())
        
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3001".parse::<header::HeaderValue>()?)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(app)
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "kembridge-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "database": "ready",
            "redis": "ready",
            "auth": "ready"
        }
    })))
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub config: config::Config,
}