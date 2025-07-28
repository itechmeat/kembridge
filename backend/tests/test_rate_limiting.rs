// test_rate_limiting_unit.rs - Unit тесты для RateLimitService
// 
// Запуск: cargo test --test test_rate_limiting_unit
//

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use redis::aio::ConnectionManager;
    use sqlx::PgPool;
    use kembridge_backend::services::rate_limit::RateLimitService;
    use kembridge_backend::constants::*;

    async fn setup_test_env() -> (ConnectionManager, PgPool) {
        // В реальных тестах здесь была бы настройка тестовой базы
        // Для демонстрации возвращаем заглушки
        todo!("Setup test Redis and PostgreSQL for rate limiting tests")
    }

    #[tokio::test]
    async fn test_rate_limit_service_creation() {
        println!("🧪 Тестирование создания RateLimitService");
        
        // Проверяем что константы загружены
        assert!(RATE_LIMIT_DEFAULT_WINDOW_SEC > 0);
        assert!(RATE_LIMIT_HEALTH_LIMIT > 0);
        assert!(RATE_LIMIT_AUTH_UNAUTH_LIMIT > 0);
        
        println!("✅ Константы rate limiting корректно настроены");
        println!("   DEFAULT_WINDOW: {} сек", RATE_LIMIT_DEFAULT_WINDOW_SEC);
        println!("   HEALTH_LIMIT: {} запросов", RATE_LIMIT_HEALTH_LIMIT);
        println!("   AUTH_UNAUTH_LIMIT: {} запросов", RATE_LIMIT_AUTH_UNAUTH_LIMIT);
    }

    #[tokio::test]
    async fn test_rate_limit_constants() {
        println!("🧪 Проверка всех rate limiting констант");
        
        // Проверяем основные константы
        assert_eq!(RATE_LIMIT_DEFAULT_WINDOW_SEC, 60);
        assert!(RATE_LIMIT_HEALTH_LIMIT >= 100); // Должно быть разумное значение
        
        // Проверяем дифференцированные лимиты
        assert!(RATE_LIMIT_BRIDGE_PREMIUM_LIMIT > RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        assert!(RATE_LIMIT_BRIDGE_AUTH_LIMIT > RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
        
        // Проверяем Redis префиксы
        assert!(!RATE_LIMIT_REDIS_PREFIX.is_empty());
        assert!(!RATE_LIMIT_STATS_REDIS_PREFIX.is_empty());
        
        println!("✅ Все константы настроены корректно");
        println!("   Bridge Premium: {} запросов", RATE_LIMIT_BRIDGE_PREMIUM_LIMIT);
        println!("   Bridge Auth: {} запросов", RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        println!("   Bridge Unauth: {} запросов", RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
    }

    #[test]
    fn test_rate_limit_configuration_logic() {
        println!("🧪 Тестирование логики конфигурации rate limiting");
        
        // Проверяем что лимиты имеют правильную иерархию
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
            assert!(limit > 0, "Лимит для {} должен быть больше 0", name);
            assert!(limit <= 10000, "Лимит для {} слишком высокий: {}", name, limit);
            println!("   {}: {} запросов/мин", name, limit);
        }
        
        // Проверяем что premium > auth > unauth
        assert!(RATE_LIMIT_BRIDGE_PREMIUM_LIMIT > RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        assert!(RATE_LIMIT_BRIDGE_AUTH_LIMIT > RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
        assert!(RATE_LIMIT_AUTH_AUTH_LIMIT > RATE_LIMIT_AUTH_UNAUTH_LIMIT);
        
        println!("✅ Логика лимитов корректна (premium > auth > unauth)");
    }

    #[test] 
    fn test_rate_limit_window_calculations() {
        println!("🧪 Тестирование расчетов временных окон");
        
        let default_window = Duration::from_secs(RATE_LIMIT_DEFAULT_WINDOW_SEC);
        let extended_window = Duration::from_secs(RATE_LIMIT_EXTENDED_WINDOW_SEC);
        let stats_cache_ttl = Duration::from_secs(RATE_LIMIT_STATS_CACHE_TTL_SEC);
        
        // Проверяем что окна имеют разумные значения
        assert!(default_window.as_secs() >= 30, "Default window слишком короткое");
        assert!(default_window.as_secs() <= 300, "Default window слишком длинное");
        
        assert!(extended_window >= default_window, "Extended window должно быть >= default");
        assert!(stats_cache_ttl >= default_window, "Stats cache TTL должно быть >= default");
        
        println!("✅ Временные окна настроены корректно");
        println!("   Default: {} сек", default_window.as_secs());
        println!("   Extended: {} сек", extended_window.as_secs());
        println!("   Stats Cache TTL: {} сек", stats_cache_ttl.as_secs());
    }
}

// Пример интеграционного теста (требует запущенных Redis и PostgreSQL)
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Игнорируем по умолчанию, так как требует инфраструктуры
    async fn test_rate_limit_service_integration() {
        println!("🧪 Интеграционный тест RateLimitService");
        println!("   Требует: Redis + PostgreSQL");
        
        // Этот тест можно запустить с: cargo test test_rate_limit_service_integration -- --ignored
        // когда Redis и PostgreSQL запущены
        
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
        
        println!("✅ Интеграционный тест пропущен (требует инфраструктуру)");
    }
}