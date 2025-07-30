// WebSocket message types - Enhanced from old backend
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Authentication request
    Auth { token: String },

    /// Authentication successful
    AuthSuccess { user_id: String },

    /// Authentication failed
    AuthFailed { error: String },

    /// Subscribe to events
    Subscribe { event_type: EventType },

    /// Unsubscribe from events
    Unsubscribe { event_type: EventType },

    /// Subscription confirmation
    Subscribed { event_type: EventType },

    /// Unsubscription confirmation
    Unsubscribed { event_type: EventType },

    /// Real-time event notification
    Event { event: RealTimeEvent },

    /// Heartbeat ping
    Ping,

    /// Heartbeat pong
    Pong,

    /// Error message
    Error { message: String, code: Option<u16> },

    /// Connection closed
    Close { reason: String },
}

/// Event types for subscription
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Transaction status updates
    TransactionStatus,

    /// Risk analysis alerts
    RiskAlerts,

    /// Price updates
    PriceUpdates,

    /// System notifications
    SystemNotifications,

    /// Bridge operation updates
    BridgeOperations,

    /// Quantum key events
    QuantumKeys,

    /// User profile updates
    UserProfile,

    /// Crypto service events (NEW)
    CryptoService,
}

/// Real-time event data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum RealTimeEvent {
    TransactionStatusUpdate(TransactionStatusEvent),
    RiskAlert(RiskAlertEvent),
    PriceUpdate(PriceUpdateEvent),
    SystemNotification(SystemNotificationEvent),
    BridgeOperation(BridgeOperationEvent),
    QuantumKeyEvent(QuantumKeyEvent),
    UserProfileUpdate(UserProfileEvent),
    CryptoServiceEvent(CryptoServiceEvent), // NEW
}

/// Transaction status update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusEvent {
    pub transaction_id: String,
    pub user_id: String,
    pub status: TransactionStatus,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: String,
    pub token_symbol: String,
    pub timestamp: DateTime<Utc>,
    pub confirmation_blocks: Option<u32>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Risk alert event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlertEvent {
    pub alert_id: String,
    pub user_id: Option<String>,
    pub transaction_id: Option<String>,
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub alert_type: RiskAlertType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub requires_action: bool,
}

/// Price update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateEvent {
    pub token_symbol: String,
    pub price_usd: String,
    pub price_change_24h: f64,
    pub volume_24h: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// System notification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotificationEvent {
    pub notification_id: String,
    pub user_id: Option<String>,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub action_required: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Bridge operation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOperationEvent {
    pub operation_id: String,
    pub user_id: String,
    pub operation_type: BridgeOperationType,
    pub status: BridgeOperationStatus,
    pub progress: f32,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub timestamp: DateTime<Utc>,
}

/// Quantum key event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyEvent {
    pub key_id: String,
    pub user_id: String,
    pub event_type: QuantumKeyEventType,
    pub algorithm: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// User profile update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileEvent {
    pub user_id: String,
    pub field_updated: String,
    pub risk_profile_updated: bool,
    pub timestamp: DateTime<Utc>,
}

/// NEW: Crypto service event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoServiceEvent {
    pub event_id: String,
    pub event_type: CryptoEventType,
    pub service_name: String,
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// NEW: Crypto event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptoEventType {
    ServiceStatusChange,
    KeyGenerated,
    KeyRotated,
    EncapsulationCompleted,
    DecapsulationCompleted,
    SystemHealthCheck,
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

/// Notification level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Bridge operation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeOperationType {
    EthToNear,
    NearToEth,
    TokenSwap,
    CrossChainTransfer,
}

/// Bridge operation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeOperationStatus {
    Initiated,
    Validating,
    Locking,
    Bridging,
    Minting,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

/// Quantum key event type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantumKeyEventType {
    Generated,
    Rotated,
    Expired,
    Revoked,
    BackupCreated,
}

impl WebSocketMessage {
    /// Create an authentication success message
    pub fn auth_success(user_id: String) -> Self {
        Self::AuthSuccess { user_id }
    }

    /// Create an authentication failed message
    pub fn auth_failed(error: impl Into<String>) -> Self {
        Self::AuthFailed {
            error: error.into(),
        }
    }

    /// Create a subscription confirmation message
    pub fn subscribed(event_type: EventType) -> Self {
        Self::Subscribed { event_type }
    }

    /// Create an unsubscription confirmation message
    pub fn unsubscribed(event_type: EventType) -> Self {
        Self::Unsubscribed { event_type }
    }

    /// Create an event notification message
    pub fn event(event: RealTimeEvent) -> Self {
        Self::Event { event }
    }

    /// Create an error message
    pub fn error(message: impl Into<String>, code: Option<u16>) -> Self {
        Self::Error {
            message: message.into(),
            code,
        }
    }

    /// Create a close message
    pub fn close(reason: impl Into<String>) -> Self {
        Self::Close {
            reason: reason.into(),
        }
    }

    /// Convert message to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse message from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl RealTimeEvent {
    /// Get the user ID associated with this event, if any
    pub fn user_id(&self) -> Option<String> {
        match self {
            RealTimeEvent::TransactionStatusUpdate(event) => Some(event.user_id.clone()),
            RealTimeEvent::RiskAlert(event) => event.user_id.clone(),
            RealTimeEvent::PriceUpdate(_) => None, // Global event
            RealTimeEvent::SystemNotification(event) => event.user_id.clone(),
            RealTimeEvent::BridgeOperation(event) => Some(event.user_id.clone()),
            RealTimeEvent::QuantumKeyEvent(event) => Some(event.user_id.clone()),
            RealTimeEvent::UserProfileUpdate(event) => Some(event.user_id.clone()),
            RealTimeEvent::CryptoServiceEvent(_) => None, // Global service event
        }
    }

    /// Get the event type for subscription filtering
    pub fn event_type(&self) -> EventType {
        match self {
            RealTimeEvent::TransactionStatusUpdate(_) => EventType::TransactionStatus,
            RealTimeEvent::RiskAlert(_) => EventType::RiskAlerts,
            RealTimeEvent::PriceUpdate(_) => EventType::PriceUpdates,
            RealTimeEvent::SystemNotification(_) => EventType::SystemNotifications,
            RealTimeEvent::BridgeOperation(_) => EventType::BridgeOperations,
            RealTimeEvent::QuantumKeyEvent(_) => EventType::QuantumKeys,
            RealTimeEvent::UserProfileUpdate(_) => EventType::UserProfile,
            RealTimeEvent::CryptoServiceEvent(_) => EventType::CryptoService,
        }
    }

    /// Check if this event requires immediate attention
    pub fn is_urgent(&self) -> bool {
        match self {
            RealTimeEvent::RiskAlert(event) => {
                matches!(event.risk_level, RiskLevel::High | RiskLevel::Critical)
            }
            RealTimeEvent::SystemNotification(event) => matches!(
                event.level,
                NotificationLevel::Error | NotificationLevel::Critical
            ),
            RealTimeEvent::BridgeOperation(event) => matches!(
                event.status,
                BridgeOperationStatus::Failed | BridgeOperationStatus::TimedOut
            ),
            RealTimeEvent::CryptoServiceEvent(event) => {
                event.status == "error" || event.status == "critical"
            }
            _ => false,
        }
    }
}

/// Helper to create crypto service events
impl CryptoServiceEvent {
    pub fn service_status_change(service: &str, status: &str, message: &str) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: CryptoEventType::ServiceStatusChange,
            service_name: service.to_string(),
            status: status.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    pub fn key_generated(key_id: &str, algorithm: &str) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: CryptoEventType::KeyGenerated,
            service_name: "crypto-service".to_string(),
            status: "success".to_string(),
            message: format!("New {} key generated", algorithm),
            timestamp: Utc::now(),
            metadata: Some(serde_json::json!({
                "key_id": key_id,
                "algorithm": algorithm
            })),
        }
    }
}
