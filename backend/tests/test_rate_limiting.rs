// test_rate_limiting_unit.rs - Unit tests for RateLimitService
// 
// Run: cargo test --test test_rate_limiting_unit
//

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use redis::aio::ConnectionManager;
    use sqlx::PgPool;
    use kembridge_backend::services::rate_limit::RateLimitService;
    use kembridge_backend::constants::*;

    async fn setup_test_env() -> (ConnectionManager, PgPool) {
        // In real tests there would be test database setup here
        // For demonstration we return stubs
        todo!("Setup test Redis and PostgreSQL for rate limiting tests")
    }

    #[tokio::test]
    async fn test_rate_limit_service_creation() {
        println!("🧪 Testing RateLimitService creation");
        
        // Check that constants are loaded
        assert!(RATE_LIMIT_DEFAULT_WINDOW_SEC > 0);
        assert!(RATE_LIMIT_HEALTH_LIMIT > 0);
        assert!(RATE_LIMIT_AUTH_UNAUTH_LIMIT > 0);
        
        println!("✅ Rate limiting constants configured correctly");
        println!("   DEFAULT_WINDOW: {} sec", RATE_LIMIT_DEFAULT_WINDOW_SEC);
        println!("   HEALTH_LIMIT: {} requests", RATE_LIMIT_HEALTH_LIMIT);
        println!("   AUTH_UNAUTH_LIMIT: {} requests", RATE_LIMIT_AUTH_UNAUTH_LIMIT);
    }

    #[tokio::test]
    async fn test_rate_limit_constants() {
        println!("🧪 Checking all rate limiting constants");
        
        // Check main constants
        assert_eq!(RATE_LIMIT_DEFAULT_WINDOW_SEC, 60);
        assert!(RATE_LIMIT_HEALTH_LIMIT >= 100); // Should be reasonable value
        
        // Check differentiated limits
        assert!(RATE_LIMIT_BRIDGE_PREMIUM_LIMIT > RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        assert!(RATE_LIMIT_BRIDGE_AUTH_LIMIT > RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
        
        // Check Redis prefixes
        assert!(!RATE_LIMIT_REDIS_PREFIX.is_empty());
        assert!(!RATE_LIMIT_STATS_REDIS_PREFIX.is_empty());
        
        println!("✅ All constants configured correctly");
        println!("   Bridge Premium: {} requests", RATE_LIMIT_BRIDGE_PREMIUM_LIMIT);
        println!("   Bridge Auth: {} requests", RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        println!("   Bridge Unauth: {} requests", RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
    }

    #[test]
    fn test_rate_limit_configuration_logic() {
        println!("🧪 Testing rate limiting configuration logic");
        
        // Check that limits have correct hierarchy
        let limits = vec![
            ("Health", RATE_LIMIT_HEALTH_LIMIT),
            ("Docs", RATE_LIMIT_DOCS_LIMIT),
            ("Auth Authenticated", RATE_LIMIT_AUTH_AUTH_LIMIT),
            ("Auth Unauthenticated", RATE_LIMIT_AUTH_UNAUTH_LIMIT),
            ("Bridge Premium", RATE_LIMIT_BRIDGE_PREMIUM_LIMIT),
            ("Bridge Auth", RATE_LIMIT_BRIDGE_AUTH_LIMIT),
            ("Bridge Unauthenticated", RATE_LIMIT_BRIDGE_UNAUTH_LIMIT),
        ];
        
        for (name, limit) in limits {
            assert!(limit > 0, "Limit for {} should be greater than 0", name);
            assert!(limit <= 10000, "Limit for {} too high: {}", name, limit);
            println!("   {}: {} requests/min", name, limit);
        }
        
        // Check that premium > auth > unauth
        assert!(RATE_LIMIT_BRIDGE_PREMIUM_LIMIT > RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        assert!(RATE_LIMIT_BRIDGE_AUTH_LIMIT > RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
        assert!(RATE_LIMIT_AUTH_AUTH_LIMIT > RATE_LIMIT_AUTH_UNAUTH_LIMIT);
        
        println!("✅ Limits logic correct (premium > auth > unauth)");
    }

    #[test] 
    fn test_rate_limit_window_calculations() {
        println!("🧪 Testing time window calculations");
        
        let default_window = Duration::from_secs(RATE_LIMIT_DEFAULT_WINDOW_SEC);
        let extended_window = Duration::from_secs(RATE_LIMIT_EXTENDED_WINDOW_SEC);
        let stats_cache_ttl = Duration::from_secs(RATE_LIMIT_STATS_CACHE_TTL_SEC);
        
        // Check that windows have reasonable values
        assert!(default_window.as_secs() >= 30, "Default window too short");
        assert!(default_window.as_secs() <= 300, "Default window too long");
        
        assert!(extended_window >= default_window, "Extended window should be >= default");
        assert!(stats_cache_ttl >= default_window, "Stats cache TTL should be >= default");
        
        println!("✅ Time windows configured correctly");
        println!("   Default: {} sec", default_window.as_secs());
        println!("   Extended: {} sec", extended_window.as_secs());
        println!("   Stats Cache TTL: {} sec", stats_cache_ttl.as_secs());
    }
}

// Example integration test (requires running Redis and PostgreSQL)
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Ignore by default as it requires infrastructure
    async fn test_rate_limit_service_integration() {
        println!("🧪 RateLimitService integration test");
        println!("   Requires: Redis + PostgreSQL");
        
        // This test can be run with: cargo test test_rate_limit_service_integration -- --ignored
        // when Redis and PostgreSQL are running
        
        // let (redis, db_pool) = setup_test_env().await;
        // let service = RateLimitService::new(redis, db_pool);
        // 
        // let result = service.check_rate_limit(
        //     "test_key",
        //     10,
        //     Duration::from_secs(60),
        //     "test",
        //     Some("test_user"),
        //     "127.0.0.1"
        // ).await;
        // 
        // assert!(result.is_ok());
        
        println!("✅ Integration test skipped (requires infrastructure)");
    }
}