// src/monitoring/event_processor.rs - Real-time event processing
use super::{MonitoringError, TransactionUpdateData, SystemNotificationData};
use crate::websocket::{WebSocketRegistry, WebSocketBroadcaster, RealTimeEvent, TransactionStatusEvent, SystemNotificationEvent};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error, debug};
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Real-time event processor
#[derive(Clone)]
pub struct RealTimeEventProcessor {
    websocket_registry: Arc<WebSocketRegistry>,
    broadcaster: WebSocketBroadcaster,
}

impl RealTimeEventProcessor {
    /// Create new event processor
    pub fn new(websocket_registry: Arc<WebSocketRegistry>) -> Self {
        let broadcaster = WebSocketBroadcaster::new(websocket_registry.clone());
        Self {
            websocket_registry,
            broadcaster,
        }
    }

    /// Send transaction status update
    pub async fn send_transaction_update(&self, update: TransactionUpdateData) -> Result<(), MonitoringError> {
        debug!("Processing transaction update for transaction {}", update.transaction_id);
        
        // Convert string amount to BigDecimal
        let amount = BigDecimal::from_str(&update.amount)
            .map_err(|e| MonitoringError::Processing(format!("Invalid amount format: {}", e)))?;
        
        // Create transaction status event
        let event = TransactionStatusEvent {
            transaction_id: update.transaction_id,
            user_id: update.user_id,
            status: convert_transaction_status(update.status),
            from_chain: update.from_chain,
            to_chain: update.to_chain,
            amount,
            token_symbol: update.token_symbol,
            timestamp: Utc::now(),
            confirmation_blocks: update.confirmation_blocks,
            estimated_completion: update.estimated_completion,
        };

        // Broadcast to user's connections
        match self.broadcaster.broadcast_to_user(update.user_id, RealTimeEvent::TransactionStatusUpdate(event)).await {
            Ok(sent_count) => {
                info!("Transaction update sent to {} connections for user {}", sent_count, update.user_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send transaction update: {}", e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Send system notification
    pub async fn send_system_notification(&self, notification: SystemNotificationData) -> Result<(), MonitoringError> {
        debug!("Processing system notification: {}", notification.notification_id);
        
        // Create system notification event
        let event = SystemNotificationEvent {
            notification_id: notification.notification_id,
            user_id: notification.user_id,
            level: convert_notification_level(notification.level),
            title: notification.title,
            message: notification.message,
            timestamp: Utc::now(),
            action_required: notification.action_required,
            expires_at: notification.expires_at,
        };

        // Broadcast based on target (user-specific or global)
        let result = if let Some(user_id) = notification.user_id {
            // Send to specific user
            self.broadcaster.broadcast_to_user(user_id, RealTimeEvent::SystemNotification(event)).await
        } else {
            // Send to all connections
            self.broadcaster.broadcast_to_all(RealTimeEvent::SystemNotification(event)).await
        };

        match result {
            Ok(sent_count) => {
                info!("System notification sent to {} connections", sent_count);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send system notification: {}", e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Send bridge operation update
    pub async fn send_bridge_operation_update(&self, operation_id: Uuid, user_id: Uuid, status: String, progress: f32) -> Result<(), MonitoringError> {
        debug!("Processing bridge operation update for operation {}", operation_id);
        
        use crate::websocket::{BridgeOperationEvent, BridgeOperationType, BridgeOperationStatus};
        
        let event = BridgeOperationEvent {
            operation_id,
            user_id,
            operation_type: BridgeOperationType::CrossChainTransfer, // Default, could be parameterized
            status: convert_bridge_operation_status(status),
            progress,
            current_step: format!("Progress: {:.1}%", progress * 100.0),
            estimated_completion: None, // Could be calculated based on progress
            timestamp: Utc::now(),
        };

        match self.broadcaster.broadcast_to_user(user_id, RealTimeEvent::BridgeOperation(event)).await {
            Ok(sent_count) => {
                info!("Bridge operation update sent to {} connections for user {}", sent_count, user_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send bridge operation update: {}", e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Send quantum key event
    pub async fn send_quantum_key_event(&self, user_id: Uuid, key_id: Uuid, event_type: String, algorithm: String) -> Result<(), MonitoringError> {
        debug!("Processing quantum key event for user {}", user_id);
        
        use crate::websocket::{QuantumKeyEvent, QuantumKeyEventType};
        
        let event = QuantumKeyEvent {
            key_id,
            user_id,
            event_type: convert_quantum_key_event_type(event_type),
            algorithm,
            timestamp: Utc::now(),
            expires_at: None, // Could be calculated based on key lifecycle
        };

        match self.broadcaster.broadcast_to_user(user_id, RealTimeEvent::QuantumKeyEvent(event)).await {
            Ok(sent_count) => {
                info!("Quantum key event sent to {} connections for user {}", sent_count, user_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send quantum key event: {}", e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Send user profile update
    pub async fn send_user_profile_update(&self, user_id: Uuid, field_updated: String, risk_profile_updated: bool) -> Result<(), MonitoringError> {
        debug!("Processing user profile update for user {}", user_id);
        
        use crate::websocket::UserProfileEvent;
        
        let event = UserProfileEvent {
            user_id,
            field_updated,
            risk_profile_updated,
            timestamp: Utc::now(),
        };

        match self.broadcaster.broadcast_to_user(user_id, RealTimeEvent::UserProfileUpdate(event)).await {
            Ok(sent_count) => {
                info!("User profile update sent to {} connections for user {}", sent_count, user_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send user profile update: {}", e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Get event processor statistics
    pub async fn get_stats(&self) -> EventProcessorStats {
        let total_connections = self.websocket_registry.connection_count().await;
        let active_connections = self.websocket_registry.get_all_connections().await.len();
        
        EventProcessorStats {
            total_connections,
            active_connections,
            timestamp: Utc::now(),
        }
    }
}

/// Event processor statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventProcessorStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Helper conversion functions
fn convert_transaction_status(status: super::TransactionStatus) -> crate::websocket::TransactionStatus {
    match status {
        super::TransactionStatus::Pending => crate::websocket::TransactionStatus::Pending,
        super::TransactionStatus::Confirmed => crate::websocket::TransactionStatus::Confirmed,
        super::TransactionStatus::Processing => crate::websocket::TransactionStatus::Processing,
        super::TransactionStatus::Completed => crate::websocket::TransactionStatus::Completed,
        super::TransactionStatus::Failed => crate::websocket::TransactionStatus::Failed,
        super::TransactionStatus::Cancelled => crate::websocket::TransactionStatus::Cancelled,
        super::TransactionStatus::RequiresReview => crate::websocket::TransactionStatus::RequiresReview,
    }
}

fn convert_notification_level(level: super::NotificationLevel) -> crate::websocket::NotificationLevel {
    match level {
        super::NotificationLevel::Info => crate::websocket::NotificationLevel::Info,
        super::NotificationLevel::Warning => crate::websocket::NotificationLevel::Warning,
        super::NotificationLevel::Error => crate::websocket::NotificationLevel::Error,
        super::NotificationLevel::Critical => crate::websocket::NotificationLevel::Critical,
    }
}

fn convert_bridge_operation_status(status: String) -> crate::websocket::BridgeOperationStatus {
    match status.as_str() {
        "initiated" => crate::websocket::BridgeOperationStatus::Initiated,
        "validating" => crate::websocket::BridgeOperationStatus::Validating,
        "locking" => crate::websocket::BridgeOperationStatus::Locking,
        "bridging" => crate::websocket::BridgeOperationStatus::Bridging,
        "minting" => crate::websocket::BridgeOperationStatus::Minting,
        "completed" => crate::websocket::BridgeOperationStatus::Completed,
        "failed" => crate::websocket::BridgeOperationStatus::Failed,
        "timed_out" => crate::websocket::BridgeOperationStatus::TimedOut,
        "cancelled" => crate::websocket::BridgeOperationStatus::Cancelled,
        _ => crate::websocket::BridgeOperationStatus::Failed, // Default for unknown status
    }
}

fn convert_quantum_key_event_type(event_type: String) -> crate::websocket::QuantumKeyEventType {
    match event_type.as_str() {
        "generated" => crate::websocket::QuantumKeyEventType::Generated,
        "rotated" => crate::websocket::QuantumKeyEventType::Rotated,
        "expired" => crate::websocket::QuantumKeyEventType::Expired,
        "revoked" => crate::websocket::QuantumKeyEventType::Revoked,
        "backup_created" => crate::websocket::QuantumKeyEventType::BackupCreated,
        _ => crate::websocket::QuantumKeyEventType::Generated, // Default for unknown type
    }
}