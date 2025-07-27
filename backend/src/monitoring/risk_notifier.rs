// src/monitoring/risk_notifier.rs - Risk alert notifications
use super::{MonitoringError, RiskAlertData, RiskLevel, RiskAlertType};
use crate::websocket::{WebSocketRegistry, WebSocketBroadcaster, RealTimeEvent, RiskAlertEvent};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error, debug};

/// Risk notification service
#[derive(Clone)]
pub struct RiskNotifier {
    websocket_registry: Arc<WebSocketRegistry>,
    broadcaster: WebSocketBroadcaster,
}

impl RiskNotifier {
    /// Create new risk notifier
    pub fn new(websocket_registry: Arc<WebSocketRegistry>) -> Self {
        let broadcaster = WebSocketBroadcaster::new(websocket_registry.clone());
        Self {
            websocket_registry,
            broadcaster,
        }
    }

    /// Send risk alert notification
    pub async fn send_risk_alert(&self, alert: RiskAlertData) -> Result<(), MonitoringError> {
        debug!("Processing risk alert: {} for user {:?}", alert.alert_id, alert.user_id);
        
        // Create risk alert event
        let event = RiskAlertEvent {
            alert_id: alert.alert_id,
            user_id: alert.user_id,
            transaction_id: alert.transaction_id,
            risk_level: convert_risk_level(alert.risk_level.clone()),
            risk_score: alert.risk_score,
            alert_type: convert_risk_alert_type(alert.alert_type.clone()),
            message: alert.message.clone(),
            timestamp: Utc::now(),
            requires_action: alert.requires_action,
        };

        // Store risk level for later logging
        let risk_level = event.risk_level.clone();
        
        // Determine notification strategy based on risk level and user
        let result = match (&alert.user_id, &risk_level) {
            // High/Critical alerts for specific users
            (Some(user_id), crate::websocket::RiskLevel::High | crate::websocket::RiskLevel::Critical) => {
                // Send to user and also to admin/operator connections
                let user_result = self.broadcaster.broadcast_to_user(*user_id, RealTimeEvent::RiskAlert(event.clone())).await;
                let admin_result = self.broadcast_to_admins(event.clone()).await;
                
                match (user_result, admin_result) {
                    (Ok(user_sent), Ok(admin_sent)) => {
                        info!("Critical risk alert sent to {} user connections and {} admin connections", user_sent, admin_sent);
                        Ok(user_sent + admin_sent)
                    }
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            }
            
            // User-specific alerts
            (Some(user_id), _) => {
                self.broadcaster.broadcast_to_user(*user_id, RealTimeEvent::RiskAlert(event)).await
            }
            
            // System-wide alerts (no specific user)
            (None, crate::websocket::RiskLevel::Critical) => {
                // Critical system alerts go to all connections
                self.broadcaster.broadcast_to_all(RealTimeEvent::RiskAlert(event)).await
            }
            
            (None, _) => {
                // Other system alerts go to admin/operator connections only
                self.broadcast_to_admins(event).await
            }
        };

        match result {
            Ok(sent_count) => {
                info!("Risk alert {} sent to {} connections", alert.alert_id, sent_count);
                
                // Log high-priority alerts
                if matches!(risk_level, crate::websocket::RiskLevel::High | crate::websocket::RiskLevel::Critical) {
                    warn!("High priority risk alert: {} - {}", alert.alert_id, alert.message);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to send risk alert {}: {}", alert.alert_id, e);
                Err(MonitoringError::WebSocket(e))
            }
        }
    }

    /// Send risk threshold exceeded notification
    pub async fn send_threshold_exceeded(&self, user_id: Uuid, threshold: f64, current_score: f64, transaction_id: Option<Uuid>) -> Result<(), MonitoringError> {
        let alert = RiskAlertData {
            alert_id: Uuid::new_v4(),
            user_id: Some(user_id),
            transaction_id,
            risk_level: if current_score >= 0.8 { RiskLevel::Critical } else { RiskLevel::High },
            risk_score: current_score,
            alert_type: RiskAlertType::ThresholdExceeded,
            message: format!("Risk threshold exceeded: {:.2} (threshold: {:.2})", current_score, threshold),
            requires_action: current_score >= 0.8,
            metadata: Some(serde_json::json!({
                "threshold": threshold,
                "current_score": current_score,
                "difference": current_score - threshold
            })),
        };

        self.send_risk_alert(alert).await
    }

    /// Send blacklist match notification
    pub async fn send_blacklist_match(&self, user_id: Option<Uuid>, address: String, chain: String, transaction_id: Option<Uuid>) -> Result<(), MonitoringError> {
        let alert = RiskAlertData {
            alert_id: Uuid::new_v4(),
            user_id,
            transaction_id,
            risk_level: RiskLevel::Critical,
            risk_score: 1.0,
            alert_type: RiskAlertType::BlacklistMatch,
            message: format!("Blacklisted address detected: {} on {}", address, chain),
            requires_action: true,
            metadata: Some(serde_json::json!({
                "address": address,
                "chain": chain,
                "detection_time": Utc::now()
            })),
        };

        self.send_risk_alert(alert).await
    }

    /// Send suspicious activity notification
    pub async fn send_suspicious_activity(&self, user_id: Uuid, activity_type: String, details: String, risk_score: f64) -> Result<(), MonitoringError> {
        let alert = RiskAlertData {
            alert_id: Uuid::new_v4(),
            user_id: Some(user_id),
            transaction_id: None,
            risk_level: if risk_score >= 0.7 { RiskLevel::High } else { RiskLevel::Medium },
            risk_score,
            alert_type: RiskAlertType::SuspiciousActivity,
            message: format!("Suspicious activity detected: {}", activity_type),
            requires_action: risk_score >= 0.7,
            metadata: Some(serde_json::json!({
                "activity_type": activity_type,
                "details": details,
                "risk_score": risk_score
            })),
        };

        self.send_risk_alert(alert).await
    }

    /// Send compliance violation notification
    pub async fn send_compliance_violation(&self, user_id: Option<Uuid>, violation_type: String, details: String, transaction_id: Option<Uuid>) -> Result<(), MonitoringError> {
        let alert = RiskAlertData {
            alert_id: Uuid::new_v4(),
            user_id,
            transaction_id,
            risk_level: RiskLevel::Critical,
            risk_score: 1.0,
            alert_type: RiskAlertType::ComplianceViolation,
            message: format!("Compliance violation: {}", violation_type),
            requires_action: true,
            metadata: Some(serde_json::json!({
                "violation_type": violation_type,
                "details": details,
                "severity": "critical"
            })),
        };

        self.send_risk_alert(alert).await
    }

    /// Broadcast to admin/operator connections
    async fn broadcast_to_admins(&self, event: RiskAlertEvent) -> Result<usize, String> {
        // For now, broadcast to all connections
        // In a real implementation, this would filter by admin/operator role
        // TODO: Implement role-based connection filtering
        self.broadcaster.broadcast_to_all(RealTimeEvent::RiskAlert(event)).await
    }

    /// Get risk notifier statistics
    pub async fn get_stats(&self) -> RiskNotifierStats {
        let total_connections = self.websocket_registry.connection_count().await;
        let active_connections = self.websocket_registry.get_all_connections().await.len();
        
        RiskNotifierStats {
            total_connections,
            active_connections,
            timestamp: Utc::now(),
        }
    }
}

/// Risk notifier statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RiskNotifierStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Helper conversion functions
fn convert_risk_level(level: RiskLevel) -> crate::websocket::RiskLevel {
    match level {
        RiskLevel::Low => crate::websocket::RiskLevel::Low,
        RiskLevel::Medium => crate::websocket::RiskLevel::Medium,
        RiskLevel::High => crate::websocket::RiskLevel::High,
        RiskLevel::Critical => crate::websocket::RiskLevel::Critical,
    }
}

fn convert_risk_alert_type(alert_type: RiskAlertType) -> crate::websocket::RiskAlertType {
    match alert_type {
        RiskAlertType::HighRiskTransaction => crate::websocket::RiskAlertType::HighRiskTransaction,
        RiskAlertType::SuspiciousActivity => crate::websocket::RiskAlertType::SuspiciousActivity,
        RiskAlertType::BlacklistMatch => crate::websocket::RiskAlertType::BlacklistMatch,
        RiskAlertType::ThresholdExceeded => crate::websocket::RiskAlertType::ThresholdExceeded,
        RiskAlertType::AnomalyDetected => crate::websocket::RiskAlertType::AnomalyDetected,
        RiskAlertType::ComplianceViolation => crate::websocket::RiskAlertType::ComplianceViolation,
    }
}