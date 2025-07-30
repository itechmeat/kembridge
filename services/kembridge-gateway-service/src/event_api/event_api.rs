use crate::event_listener::EventListener;
use crate::websocket::{CryptoEventType, RealTimeEvent, WebSocketBroadcaster};
use axum::{
    extract::{Path, State},
    Json,
};
use kembridge_common::ServiceResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// State for event API endpoints
pub type EventApiState = (Arc<WebSocketBroadcaster>, Arc<EventListener>);

#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerCryptoEventRequest {
    pub event_type: String,
    pub user_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerRiskAnalysisRequest {
    pub user_id: String,
    pub transaction_id: String,
    pub risk_score: f64,
}

#[derive(Debug, Serialize)]
pub struct EventTriggerResponse {
    pub success: bool,
    pub message: String,
    pub event_id: String,
    pub timestamp: String,
}

/// Trigger crypto service event manually (for testing)
pub async fn trigger_crypto_event(
    State((broadcaster, event_listener)): State<EventApiState>,
    Json(request): Json<TriggerCryptoEventRequest>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("🔐 Manual trigger crypto event: {}", request.event_type);

    let event_type = match request.event_type.as_str() {
        "key_generation" => {
            if let Some(user_id) = &request.user_id {
                event_listener
                    .handle_crypto_operation("key_generation", user_id)
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            } else {
                broadcaster
                    .broadcast_crypto_event(
                        CryptoEventType::KeyGenerated,
                        "crypto-service",
                        "success",
                        &request.message,
                        Some(serde_json::json!({
                            "triggered_manually": true,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })),
                    )
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            }
            "key_generation"
        }
        "encapsulation" => {
            if let Some(user_id) = &request.user_id {
                event_listener
                    .handle_crypto_operation("encapsulation", user_id)
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            } else {
                broadcaster
                    .broadcast_crypto_event(
                        CryptoEventType::EncapsulationCompleted,
                        "crypto-service",
                        "success",
                        &request.message,
                        None,
                    )
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            }
            "encapsulation"
        }
        "decapsulation" => {
            if let Some(user_id) = &request.user_id {
                event_listener
                    .handle_crypto_operation("decapsulation", user_id)
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            } else {
                broadcaster
                    .broadcast_crypto_event(
                        CryptoEventType::DecapsulationCompleted,
                        "crypto-service",
                        "success",
                        &request.message,
                        None,
                    )
                    .await
                    .map_err(|e| crate::errors::GatewayServiceError::Internal {
                        message: e.to_string(),
                    })?;
            }
            "decapsulation"
        }
        "key_rotation" => {
            broadcaster
                .broadcast_crypto_event(
                    CryptoEventType::KeyRotated,
                    "crypto-service",
                    "success",
                    &request.message,
                    Some(serde_json::json!({
                        "rotation_reason": "manual_trigger",
                        "old_key_archived": true
                    })),
                )
                .await
                .map_err(|e| crate::errors::GatewayServiceError::Internal {
                    message: e.to_string(),
                })?;
            "key_rotation"
        }
        "service_status" => {
            broadcaster
                .broadcast_crypto_event(
                    CryptoEventType::ServiceStatusChange,
                    "crypto-service",
                    "healthy",
                    &request.message,
                    None,
                )
                .await
                .map_err(|e| crate::errors::GatewayServiceError::Internal {
                    message: e.to_string(),
                })?;
            "service_status"
        }
        _ => {
            return Ok(Json(ServiceResponse::error(format!(
                "Unknown event type: {}",
                request.event_type
            ))));
        }
    };

    let response = EventTriggerResponse {
        success: true,
        message: format!("Crypto event '{}' triggered successfully", event_type),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Trigger risk analysis event manually (for testing)
pub async fn trigger_risk_analysis(
    State((_, event_listener)): State<EventApiState>,
    Json(request): Json<TriggerRiskAnalysisRequest>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!(
        "🧠 Manual trigger risk analysis for user: {} transaction: {}",
        request.user_id,
        request.transaction_id
    );

    // Validate risk score range
    if request.risk_score < 0.0 || request.risk_score > 1.0 {
        return Ok(Json(ServiceResponse::error(
            "Risk score must be between 0.0 and 1.0".to_string(),
        )));
    }

    event_listener
        .handle_risk_analysis(
            &request.user_id,
            &request.transaction_id,
            request.risk_score,
        )
        .await
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!(
            "Risk analysis triggered for score: {:.2}",
            request.risk_score
        ),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Trigger system notification manually (for testing)
pub async fn trigger_system_notification(
    State((broadcaster, _)): State<EventApiState>,
    Path(level): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("📢 Manual trigger system notification: {}", level);

    let notification_level = match level.as_str() {
        "info" => crate::websocket::NotificationLevel::Info,
        "warning" => crate::websocket::NotificationLevel::Warning,
        "error" => crate::websocket::NotificationLevel::Error,
        "critical" => crate::websocket::NotificationLevel::Critical,
        _ => {
            return Ok(Json(ServiceResponse::error(format!(
                "Invalid notification level: {}",
                level
            ))));
        }
    };

    let title = payload
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Manual Test Notification");

    let message = payload
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("This is a manual test notification");

    let urgent = payload
        .get("urgent")
        .and_then(|u| u.as_bool())
        .unwrap_or(false);

    broadcaster
        .broadcast_system_notification(notification_level, title, message, urgent)
        .await
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!("System notification '{}' sent successfully", level),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Get current WebSocket connection statistics
pub async fn get_websocket_stats(
    State((broadcaster, _)): State<EventApiState>,
) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
    tracing::info!("📊 Getting WebSocket connection stats");

    let stats = broadcaster.get_stats().await;
    let connection_details = broadcaster.get_connection_details().await;

    let response = serde_json::json!({
        "stats": {
            "total_connections": stats.total_connections,
            "active_connections": stats.active_connections,
            "authenticated_connections": stats.authenticated_connections,
            "unique_users": stats.unique_users
        },
        "connections": connection_details.into_iter().map(|detail| {
            serde_json::json!({
                "id": detail.id,
                "user_id": detail.user_id,
                "is_active": detail.is_active,
                "client_ip": detail.client_ip,
                "user_agent": detail.user_agent,
                "subscriptions": detail.subscriptions,
                "connected_at": detail.connected_at,
                "last_activity": detail.last_activity
            })
        }).collect::<Vec<_>>(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(ServiceResponse::success(response)))
}

/// Cleanup inactive WebSocket connections manually
pub async fn cleanup_connections(
    State((broadcaster, _)): State<EventApiState>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("🧹 Manual cleanup of inactive connections");

    let inactive_cleaned = broadcaster
        .cleanup_inactive_connections()
        .await
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let idle_cleaned = broadcaster
        .cleanup_idle_connections(30)
        .await // 30 minutes
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!(
            "Cleaned {} inactive and {} idle connections",
            inactive_cleaned, idle_cleaned
        ),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Send heartbeat to all connections
pub async fn send_heartbeat(
    State((broadcaster, _)): State<EventApiState>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("💓 Manual heartbeat send");

    let heartbeat_count = broadcaster.send_heartbeat().await.map_err(|e| {
        crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        }
    })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!("Heartbeat sent to {} connections", heartbeat_count),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Disconnect specific user (for testing)
pub async fn disconnect_user(
    State((broadcaster, _)): State<EventApiState>,
    Path(user_id): Path<String>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("🔌 Manual disconnect user: {}", user_id);

    let disconnected_count = broadcaster
        .disconnect_user(&user_id, "Manual disconnect via API")
        .await
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!(
            "Disconnected {} connections for user: {}",
            disconnected_count, user_id
        ),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}

/// Test WebSocket broadcasting to specific user
pub async fn test_user_broadcast(
    State((broadcaster, _)): State<EventApiState>,
    Path(user_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ServiceResponse<EventTriggerResponse>>, crate::errors::GatewayServiceError> {
    tracing::info!("📤 Test broadcast to user: {}", user_id);

    let _message = payload
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Test message from event API");

    // Create a test transaction status event
    let test_event =
        RealTimeEvent::TransactionStatusUpdate(crate::websocket::TransactionStatusEvent {
            transaction_id: format!("test_tx_{}", Uuid::new_v4()),
            user_id: user_id.clone(),
            status: crate::websocket::TransactionStatus::Processing,
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: "100.0".to_string(),
            token_symbol: "ETH".to_string(),
            timestamp: chrono::Utc::now(),
            confirmation_blocks: Some(1),
            estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        });

    broadcaster
        .broadcast_to_user(&user_id, test_event)
        .await
        .map_err(|e| crate::errors::GatewayServiceError::Internal {
            message: e.to_string(),
        })?;

    let response = EventTriggerResponse {
        success: true,
        message: format!("Test event sent to user: {}", user_id),
        event_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ServiceResponse::success(response)))
}
