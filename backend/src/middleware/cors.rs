// src/middleware/cors.rs - Production-ready CORS setup
use axum::http::{HeaderValue, Method, HeaderName};
use tower_http::cors::{CorsLayer, AllowOrigin};
use crate::config::AppConfig;

/// Create production CORS layer with environment-specific configuration
pub fn create_production_cors(config: &AppConfig) -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-quantum-signature"),
            HeaderName::from_static("x-wallet-address"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-user-agent"),
            HeaderName::from_static("x-forwarded-for"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("accept-language"),
            HeaderName::from_static("cache-control"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-response-time"),
            HeaderName::from_static("x-quantum-protected"),
            HeaderName::from_static("x-rate-limit-remaining"),
            HeaderName::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600)) // 1 hour preflight cache
}

/// Create development CORS layer (permissive for development)
pub fn create_development_cors() -> CorsLayer {
    CorsLayer::permissive()
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-quantum-signature"),
            HeaderName::from_static("x-wallet-address"),
            HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-response-time"),
            HeaderName::from_static("x-quantum-protected"),
        ])
}

/// Create CORS layer based on environment
pub fn create_cors_layer(config: &AppConfig) -> CorsLayer {
    match config.is_development() {
        true => {
            tracing::debug!("Using permissive CORS for development");
            create_development_cors()
        },
        false => {
            tracing::info!("Using production CORS with allowed origins: {:?}", config.cors_origins);
            create_production_cors(config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, Environment};

    #[test]
    fn test_production_cors_creation() {
        let mut config = AppConfig::default();
        config.environment = Environment::Production;
        config.cors_origins = vec![
            "https://kembridge.io".to_string(),
            "https://app.kembridge.io".to_string(),
        ];

        let cors_layer = create_cors_layer(&config);
        // Test would verify CORS layer configuration
        // Note: tower-http::cors doesn't expose internal config for testing
        // In practice, you'd test this with actual HTTP requests
    }

    #[test]
    fn test_development_cors_creation() {
        let mut config = AppConfig::default();
        config.environment = Environment::Development;

        let cors_layer = create_cors_layer(&config);
        // Development CORS should be permissive
    }
}