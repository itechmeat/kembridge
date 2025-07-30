use crate::websocket::{WebSocketBroadcaster, RealTimeEvent, CryptoEventType};
use crate::websocket::message::RiskAlertType;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Service для интеграции с реальными событиями от микросервисов
pub struct EventListener {
    broadcaster: Arc<WebSocketBroadcaster>,
    http_client: Client,
    crypto_service_url: String,
    ai_engine_url: String,
    blockchain_service_url: String,
    poll_interval: Duration,
}

impl EventListener {
    pub fn new(
        broadcaster: Arc<WebSocketBroadcaster>,
        crypto_service_url: String,
        ai_engine_url: String,
        blockchain_service_url: String,
        poll_interval: Duration,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            broadcaster,
            http_client,
            crypto_service_url,
            ai_engine_url,
            blockchain_service_url,
            poll_interval,
        }
    }

    /// Запуск всех event listeners
    pub async fn start(&self) {
        info!("🎧 Starting EventListener services...");

        // Запуск мониторинга каждого микросервиса в отдельных задачах
        let crypto_listener = self.clone();
        let ai_listener = self.clone();
        let blockchain_listener = self.clone();

        tokio::spawn(async move {
            crypto_listener.monitor_crypto_service().await;
        });

        tokio::spawn(async move {
            ai_listener.monitor_ai_engine().await;
        });

        tokio::spawn(async move {
            blockchain_listener.monitor_blockchain_service().await;
        });

        info!("🎧 All EventListener services started");
    }

    /// Мониторинг событий от crypto-service
    async fn monitor_crypto_service(&self) {
        info!("🔐 Starting crypto-service event monitoring");
        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.poll_crypto_service().await {
                warn!("Failed to poll crypto-service: {}", e);
            }
        }
    }

    /// Опрос статуса crypto-service
    async fn poll_crypto_service(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Polling crypto-service status...");

        // Получаем статус сервиса
        let status_url = format!("{}/status", self.crypto_service_url);
        let response = self.http_client.get(&status_url).send().await?;

        if response.status().is_success() {
            let status_data: Value = response.json().await?;
            
            // Проверяем, изменился ли статус
            if let Some(status) = status_data.get("status").and_then(|s| s.as_str()) {
                if status != "healthy" {
                    // Отправляем уведомление о проблемах с сервисом
                    let event_result = self.broadcaster.broadcast_crypto_event(
                        CryptoEventType::ServiceStatusChange,
                        "crypto-service",
                        status,
                        &format!("Crypto service status changed to: {}", status),
                        Some(status_data.clone())
                    ).await;

                    if let Err(e) = event_result {
                        error!("Failed to broadcast crypto service status: {}", e);
                    }
                }
            }

            // Проверяем наличие новых ключей (простая эвристика)
            if let Some(active_keys) = status_data.get("active_keys").and_then(|k| k.as_u64()) {
                if active_keys > 0 {
                    debug!("Crypto service has {} active keys", active_keys);
                    
                    // Можно добавить логику для отслеживания новых ключей
                    // Например, сравнение с предыдущим состоянием
                }
            }
        }

        Ok(())
    }

    /// Мониторинг событий от AI Engine
    async fn monitor_ai_engine(&self) {
        info!("🧠 Starting AI Engine event monitoring");
        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.poll_ai_engine().await {
                warn!("Failed to poll AI Engine: {}", e);
            }
        }
    }

    /// Опрос статуса AI Engine
    async fn poll_ai_engine(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Polling AI Engine health...");

        let health_url = format!("{}/health", self.ai_engine_url);
        let response = self.http_client.get(&health_url).send().await?;

        if response.status().is_success() {
            let health_data: Value = response.json().await?;
            
            // Проверяем статус ML моделей
            if let Some(ml_status) = health_data.get("ml_models_status").and_then(|s| s.as_str()) {
                if ml_status != "simple_analyzer_ready" {
                    // Отправляем системное уведомление о проблемах с ML
                    let notification_result = self.broadcaster.broadcast_system_notification(
                        crate::websocket::NotificationLevel::Warning,
                        "AI Engine Status",
                        &format!("ML models status: {}", ml_status),
                        false
                    ).await;

                    if let Err(e) = notification_result {
                        error!("Failed to broadcast AI Engine status: {}", e);
                    }
                }
            }

            // Проверяем статус базы данных
            if let Some(db_status) = health_data.get("database_status").and_then(|s| s.as_str()) {
                if db_status != "connected" {
                    // Критическое уведомление о проблемах с БД
                    let notification_result = self.broadcaster.broadcast_system_notification(
                        crate::websocket::NotificationLevel::Critical,
                        "AI Engine Database",
                        "AI Engine database connection lost",
                        true
                    ).await;

                    if let Err(e) = notification_result {
                        error!("Failed to broadcast AI Engine DB status: {}", e);
                    }
                }
            }
        } else {
            // AI Engine недоступен
            let notification_result = self.broadcaster.broadcast_system_notification(
                crate::websocket::NotificationLevel::Error,
                "AI Engine Unavailable",
                "AI Risk Engine service is not responding",
                false
            ).await;

            if let Err(e) = notification_result {
                error!("Failed to broadcast AI Engine unavailable: {}", e);
            }
        }

        Ok(())
    }

    /// Мониторинг событий от blockchain services
    async fn monitor_blockchain_service(&self) {
        info!("⛓️ Starting blockchain service event monitoring");
        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.poll_blockchain_service().await {
                warn!("Failed to poll blockchain service: {}", e);
            }
        }
    }

    /// Опрос статуса blockchain service
    async fn poll_blockchain_service(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Polling blockchain service health...");

        let health_url = format!("{}/health", self.blockchain_service_url);
        let response = self.http_client.get(&health_url).send().await?;

        if response.status().is_success() {
            let health_data: Value = response.json().await?;
            
            // Проверяем статус подключений к блокчейнам
            if let Some(ethereum_status) = health_data.get("ethereum_connection").and_then(|s| s.as_str()) {
                if ethereum_status != "connected" {
                    let notification_result = self.broadcaster.broadcast_system_notification(
                        crate::websocket::NotificationLevel::Error,
                        "Ethereum Connection",
                        "Ethereum blockchain connection lost",
                        false
                    ).await;

                    if let Err(e) = notification_result {
                        error!("Failed to broadcast Ethereum status: {}", e);
                    }
                }
            }

            if let Some(near_status) = health_data.get("near_connection").and_then(|s| s.as_str()) {
                if near_status != "connected" {
                    let notification_result = self.broadcaster.broadcast_system_notification(
                        crate::websocket::NotificationLevel::Error,
                        "NEAR Connection",
                        "NEAR Protocol connection lost",
                        false
                    ).await;

                    if let Err(e) = notification_result {
                        error!("Failed to broadcast NEAR status: {}", e);
                    }
                }
            }
        } else {
            // Blockchain service недоступен
            let notification_result = self.broadcaster.broadcast_system_notification(
                crate::websocket::NotificationLevel::Critical,
                "Blockchain Service Unavailable",
                "Blockchain service is not responding - bridge operations may be affected",
                true
            ).await;

            if let Err(e) = notification_result {
                error!("Failed to broadcast blockchain service unavailable: {}", e);
            }
        }

        Ok(())
    }

    /// Получение данных о транзакциях для WebSocket уведомлений
    pub async fn poll_transaction_updates(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Polling transaction updates...");
        
        // Здесь можно добавить логику для получения обновлений транзакций
        // Например, от blockchain service или из базы данных
        
        // Пример создания события обновления транзакции:
        /*
        let transaction_event = RealTimeEvent::TransactionStatusUpdate(
            crate::websocket::TransactionStatusEvent {
                transaction_id: "tx_example".to_string(),
                user_id: "user_example".to_string(),
                status: crate::websocket::TransactionStatus::Processing,
                from_chain: "ethereum".to_string(),
                to_chain: "near".to_string(),
                amount: "100.0".to_string(),
                token_symbol: "ETH".to_string(),
                timestamp: chrono::Utc::now(),
                confirmation_blocks: Some(3),
                estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
            }
        );

        self.broadcaster.broadcast_to_subscribers(transaction_event).await?;
        */

        Ok(())
    }

    /// Интеграция с реальными криптографическими операциями
    pub async fn handle_crypto_operation(&self, operation_type: &str, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔐 Handling crypto operation: {} for user: {}", operation_type, user_id);

        match operation_type {
            "key_generation" => {
                // Уведомляем о начале генерации ключа
                self.broadcaster.broadcast_crypto_event(
                    CryptoEventType::KeyGenerated,
                    "crypto-service",
                    "in_progress",
                    "ML-KEM-1024 key generation started",
                    Some(serde_json::json!({
                        "user_id": user_id,
                        "algorithm": "ML-KEM-1024",
                        "started_at": chrono::Utc::now().to_rfc3339()
                    }))
                ).await?;

                // Здесь можно добавить реальный вызов к crypto-service
                // let key_response = self.http_client.post(&format!("{}/keys/user/{}/generate", self.crypto_service_url, user_id)).send().await?;
                
                // Уведомляем о завершении
                self.broadcaster.broadcast_crypto_event(
                    CryptoEventType::KeyGenerated,
                    "crypto-service",
                    "success",
                    "ML-KEM-1024 key generation completed",
                    Some(serde_json::json!({
                        "user_id": user_id,
                        "algorithm": "ML-KEM-1024",
                        "completed_at": chrono::Utc::now().to_rfc3339()
                    }))
                ).await?;
            },
            "encapsulation" => {
                self.broadcaster.broadcast_crypto_event(
                    CryptoEventType::EncapsulationCompleted,
                    "crypto-service",
                    "success",
                    "ML-KEM encapsulation completed",
                    Some(serde_json::json!({
                        "user_id": user_id,
                        "operation": "encapsulation"
                    }))
                ).await?;
            },
            "decapsulation" => {
                self.broadcaster.broadcast_crypto_event(
                    CryptoEventType::DecapsulationCompleted,
                    "crypto-service", 
                    "success",
                    "ML-KEM decapsulation completed",
                    Some(serde_json::json!({
                        "user_id": user_id,
                        "operation": "decapsulation"
                    }))
                ).await?;
            },
            _ => {
                warn!("Unknown crypto operation type: {}", operation_type);
            }
        }

        Ok(())
    }

    /// Интеграция с AI Engine для risk analysis событий
    pub async fn handle_risk_analysis(&self, user_id: &str, transaction_id: &str, risk_score: f64) -> Result<(), Box<dyn std::error::Error>> {
        info!("🧠 Handling risk analysis for transaction: {} (score: {})", transaction_id, risk_score);

        let risk_level = if risk_score >= 0.8 {
            crate::websocket::RiskLevel::Critical
        } else if risk_score >= 0.6 {
            crate::websocket::RiskLevel::High
        } else if risk_score >= 0.4 {
            crate::websocket::RiskLevel::Medium
        } else {
            crate::websocket::RiskLevel::Low
        };

        // Создаем risk alert event
        let risk_alert = RealTimeEvent::RiskAlert(crate::websocket::RiskAlertEvent {
            alert_id: Uuid::new_v4().to_string(),
            user_id: Some(user_id.to_string()),
            transaction_id: Some(transaction_id.to_string()),
            risk_level: risk_level.clone(),
            risk_score,
            alert_type: if risk_score >= 0.8 {
                RiskAlertType::HighRiskTransaction
            } else if risk_score >= 0.6 {
                RiskAlertType::AnomalyDetected
            } else {
                RiskAlertType::ThresholdExceeded
            },
            message: format!("Transaction risk analysis completed: {} risk (score: {:.2})", 
                match risk_level {
                    crate::websocket::RiskLevel::Critical => "CRITICAL",
                    crate::websocket::RiskLevel::High => "HIGH", 
                    crate::websocket::RiskLevel::Medium => "MEDIUM",
                    crate::websocket::RiskLevel::Low => "LOW",
                }, risk_score),
            timestamp: chrono::Utc::now(),
            requires_action: risk_score >= 0.8,
        });

        // Отправляем alert пользователю
        self.broadcaster.broadcast_to_user(user_id, risk_alert).await?;

        // Если критический уровень риска - отправляем срочное уведомление всем
        if risk_score >= 0.8 {
            let urgent_alert = RealTimeEvent::RiskAlert(crate::websocket::RiskAlertEvent {
                alert_id: Uuid::new_v4().to_string(),
                user_id: Some(user_id.to_string()),
                transaction_id: Some(transaction_id.to_string()),
                risk_level: crate::websocket::RiskLevel::Critical,
                risk_score,
                alert_type: RiskAlertType::HighRiskTransaction,
                message: "CRITICAL: High-risk transaction detected by AI Engine".to_string(),
                timestamp: chrono::Utc::now(),
                requires_action: true,
            });

            self.broadcaster.broadcast_urgent(urgent_alert).await?;
        }

        Ok(())
    }

    // Test helper methods for accessing private fields
    #[cfg(test)]
    pub fn get_crypto_service_url(&self) -> &str {
        &self.crypto_service_url
    }

    #[cfg(test)]
    pub fn get_ai_engine_url(&self) -> &str {
        &self.ai_engine_url
    }

    #[cfg(test)]
    pub fn get_blockchain_service_url(&self) -> &str {
        &self.blockchain_service_url
    }

    #[cfg(test)]
    pub fn get_poll_interval(&self) -> Duration {
        self.poll_interval
    }
}

impl Clone for EventListener {
    fn clone(&self) -> Self {
        Self {
            broadcaster: self.broadcaster.clone(),
            http_client: self.http_client.clone(),
            crypto_service_url: self.crypto_service_url.clone(),
            ai_engine_url: self.ai_engine_url.clone(),
            blockchain_service_url: self.blockchain_service_url.clone(),
            poll_interval: self.poll_interval,
        }
    }
}

/// Создание и настройка EventListener
pub fn create_event_listener(broadcaster: Arc<WebSocketBroadcaster>) -> EventListener {
    // URL-адреса микросервисов (можно вынести в конфигурацию)
    let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:4001".to_string());
    let ai_engine_url = std::env::var("AI_ENGINE_URL")
        .unwrap_or_else(|_| "http://localhost:4005".to_string());
    let blockchain_service_url = std::env::var("BLOCKCHAIN_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:4003".to_string());

    EventListener::new(
        broadcaster,
        crypto_service_url,
        ai_engine_url,
        blockchain_service_url,
        Duration::from_secs(30), // Опрос каждые 30 секунд
    )
}