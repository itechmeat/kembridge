// src/monitoring/mod.rs - Real-time monitoring module
pub mod event_processor;
pub mod redis_cache;
pub mod risk_notifier;

pub use event_processor::*;
pub use redis_cache::*;
pub use risk_notifier::*;

use std::sync::Arc;
use crate::websocket::WebSocketRegistry;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Monitoring service for real-time events
#[derive(Clone)]
pub struct MonitoringService {
    websocket_registry: Arc<WebSocketRegistry>,
    redis_cache: Arc<RwLock<Option<redis::aio::ConnectionManager>>>,
    event_processor: Arc<RealTimeEventProcessor>,
    risk_notifier: Arc<RiskNotifier>,
}

impl MonitoringService {
    /// Create new monitoring service
    pub fn new(websocket_registry: Arc<WebSocketRegistry>) -> Self {
        let event_processor = Arc::new(RealTimeEventProcessor::new(websocket_registry.clone()));
        let risk_notifier = Arc::new(RiskNotifier::new(websocket_registry.clone()));
        
        Self {
            websocket_registry,
            redis_cache: Arc::new(RwLock::new(None)),
            event_processor,
            risk_notifier,
        }
    }

    /// Initialize with Redis connection
    pub async fn with_redis(self, redis: redis::aio::ConnectionManager) -> Self {
        {
            let mut redis_cache = self.redis_cache.write().await;
            *redis_cache = Some(redis);
        }
        self
    }

    /// Get WebSocket registry
    pub fn websocket_registry(&self) -> Arc<WebSocketRegistry> {
        self.websocket_registry.clone()
    }

    /// Get event processor
    pub fn event_processor(&self) -> Arc<RealTimeEventProcessor> {
        self.event_processor.clone()
    }

    /// Get risk notifier
    pub fn risk_notifier(&self) -> Arc<RiskNotifier> {
        self.risk_notifier.clone()
    }

    /// Send risk alert notification
    pub async fn send_risk_alert(&self, alert: RiskAlertData) -> Result<(), MonitoringError> {
        self.risk_notifier.send_risk_alert(alert).await
    }

    /// Send transaction status update
    pub async fn send_transaction_update(&self, update: TransactionUpdateData) -> Result<(), MonitoringError> {
        self.event_processor.send_transaction_update(update).await
    }

    /// Send system notification
    pub async fn send_system_notification(&self, notification: SystemNotificationData) -> Result<(), MonitoringError> {
        self.event_processor.send_system_notification(notification).await
    }

    /// Get monitoring statistics
    pub async fn get_stats(&self) -> MonitoringStats {
        MonitoringStats {
            total_connections: self.websocket_registry.connection_count().await,
            active_connections: self.websocket_registry.get_all_connections().await.len(),
            timestamp: Utc::now(),
        }
    }
}

/// Risk alert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlertData {
    pub alert_id: Uuid,
    pub user_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub message: String,
    pub alert_type: RiskAlertType,
    pub requires_action: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Transaction update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdateData {
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub status: TransactionStatus,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: String, // String to avoid BigDecimal in this layer
    pub token_symbol: String,
    pub confirmation_blocks: Option<u32>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// System notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotificationData {
    pub notification_id: Uuid,
    pub user_id: Option<Uuid>,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub action_required: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk alert type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskAlertType {
    HighRiskTransaction,
    SuspiciousActivity,
    BlacklistMatch,
    ThresholdExceeded,
    AnomalyDetected,
    ComplianceViolation,
}

/// Transaction status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Processing,
    Completed,
    Failed,
    Cancelled,
    RequiresReview,
}

/// Notification level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub timestamp: DateTime<Utc>,
}

/// Monitoring error types
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Processing error: {0}")]
    Processing(String),
}