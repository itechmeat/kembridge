// src/websocket/auth.rs - WebSocket authentication
use super::connection::WebSocketConnection;
use super::message::WebSocketMessage;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};

/// WebSocket authentication service
pub struct WebSocketAuth;

impl WebSocketAuth {
    /// Authenticate WebSocket connection using JWT token
    pub async fn authenticate(
        connection: &Arc<WebSocketConnection>,
        token: String,
    ) -> Result<Uuid, String> {
        use kembridge_auth::JwtManager;
        
        if token.is_empty() {
            return Err("Empty token".to_string());
        }
        
        // Get JWT secret from environment (same as HTTP auth)
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "hackathon-super-secret-key-change-in-production".to_string());
        
        // Create JWT manager and validate token
        let jwt_manager = JwtManager::new(jwt_secret)
            .map_err(|e| format!("JWT manager initialization failed: {}", e))?;
        
        // Validate JWT token and extract user ID
        let claims = jwt_manager.verify_token(&token)
            .await
            .map_err(|e| format!("JWT validation failed: {}", e))?;
        
        let user_id = claims.user_id;
        
        info!("WebSocket authentication successful for connection {}, user: {}", 
            connection.id(), user_id);
        
        Ok(user_id)
    }
    
    /// Handle authentication message
    pub async fn handle_auth_message(
        connection: &Arc<WebSocketConnection>,
        token: String,
    ) -> Result<(), String> {
        match Self::authenticate(connection, token).await {
            Ok(user_id) => {
                // Update connection with user ID
                // Note: This is a simplified approach
                // In a real implementation, we'd need to handle this differently
                // since we can't mutate the connection directly
                
                connection.send_message(WebSocketMessage::auth_success(user_id)).await?;
                info!("WebSocket authentication successful for user: {}", user_id);
                Ok(())
            }
            Err(error) => {
                warn!("WebSocket authentication failed: {}", error);
                connection.send_message(WebSocketMessage::auth_failed(error.clone())).await?;
                Err(error)
            }
        }
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user_id: Uuid,
    pub permissions: Vec<String>,
}

/// WebSocket permissions
#[derive(Debug, Clone)]
pub enum WebSocketPermission {
    ReadTransactions,
    ReadRiskAlerts,
    ReadPriceUpdates,
    ReadSystemNotifications,
    ReadBridgeOperations,
    ReadQuantumKeys,
    ReadUserProfile,
    AdminAccess,
}

impl WebSocketPermission {
    /// Convert permission to string
    pub fn as_str(&self) -> &'static str {
        match self {
            WebSocketPermission::ReadTransactions => "read_transactions",
            WebSocketPermission::ReadRiskAlerts => "read_risk_alerts",
            WebSocketPermission::ReadPriceUpdates => "read_price_updates",
            WebSocketPermission::ReadSystemNotifications => "read_system_notifications",
            WebSocketPermission::ReadBridgeOperations => "read_bridge_operations",
            WebSocketPermission::ReadQuantumKeys => "read_quantum_keys",
            WebSocketPermission::ReadUserProfile => "read_user_profile",
            WebSocketPermission::AdminAccess => "admin_access",
        }
    }
}