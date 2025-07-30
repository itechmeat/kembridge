/**
 * WebSocket Module for Gateway Service
 * Provides real-time communication capabilities for KEMBridge
 */

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// WebSocket message types matching frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSMessage {
    #[serde(rename = "transaction_update")]
    TransactionUpdate { data: TransactionUpdate },
    #[serde(rename = "risk_alert")]
    RiskAlert { data: RiskAlert },
    #[serde(rename = "price_update")]
    PriceUpdate { data: PriceUpdate },
    #[serde(rename = "system_notification")]
    SystemNotification { data: SystemNotification },
    #[serde(rename = "bridge_operation")]
    BridgeOperation { data: BridgeOperationEvent },
    #[serde(rename = "quantum_key_event")]
    QuantumKeyEvent { data: QuantumKeyEvent },
    #[serde(rename = "user_profile_update")]
    UserProfileUpdate { data: UserProfileUpdate },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdate {
    pub transaction_id: String,
    pub status: String, // "pending" | "confirmed" | "completed" | "failed" | "expired"
    pub from_transaction_hash: Option<String>,
    pub to_transaction_hash: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub user_id: String,
    pub transaction_id: String,
    pub risk_score: f64,
    pub risk_level: String, // "low" | "medium" | "high"
    pub flags: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub from_token: String,
    pub to_token: String,
    pub price: f64,
    pub change_24h: Option<f64>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotification {
    pub id: String,
    pub level: String, // "info" | "warning" | "error"
    pub title: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOperationEvent {
    pub operation_id: String,
    pub user_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub status: String,
    pub progress_percentage: u8,
    pub estimated_completion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyEvent {
    pub operation_id: String,
    pub key_type: String, // "ml_kem" | "dilithium" | "sphincs"
    pub event_type: String, // "generated" | "rotated" | "expired"
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdate {
    pub user_id: String,
    pub field: String,
    pub value: serde_json::Value,
    pub updated_at: String,
}

// Connection manager
pub type ConnectionManager = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<WSMessage>>>>;

#[derive(Debug, Deserialize)]
pub struct WSQuery {
    token: Option<String>,
}

// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WSQuery>,
    State((_, connections)): State<(Arc<crate::circuit_breaker::CircuitBreaker>, ConnectionManager)>,
) -> Response {
    info!("🔌 WebSocket connection request received");
    
    // Basic auth validation (simplified for demo)
    let client_id = match params.token {
        Some(token) if !token.is_empty() => {
            info!("🔑 WebSocket: Authenticated connection with token");
            format!("user_{}", uuid::Uuid::new_v4())
        }
        _ => {
            info!("🔓 WebSocket: Anonymous connection");
            format!("anon_{}", uuid::Uuid::new_v4())
        }
    };
    
    ws.on_upgrade(move |socket| handle_socket(socket, client_id, connections))
}

// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, client_id: String, connections: ConnectionManager) {
    info!("✅ WebSocket: Client {} connected", client_id);
    
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WSMessage>();
    
    // Register connection
    {
        let mut conns = connections.lock().unwrap();
        conns.insert(client_id.clone(), tx);
    }
    
    // Spawn task to send messages to client
    let client_id_send = client_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        error!("❌ WebSocket: Failed to send message to {}", client_id_send);
                        break;
                    }
                }
                Err(e) => {
                    error!("❌ WebSocket: Failed to serialize message: {}", e);
                }
            }
        }
    });
    
    // Handle incoming messages from client
    let client_id_recv = client_id.clone();
    let connections_recv = connections.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("📨 WebSocket: Received from {}: {}", client_id_recv, text);
                    
                    // Handle ping/pong
                    if text == "ping" {
                        let pong_msg = WSMessage::Pong;
                        if let Some(tx) = connections_recv.lock().unwrap().get(&client_id_recv) {
                            let _ = tx.send(pong_msg);
                        }
                    }
                    
                    // Handle subscription requests
                    if let Ok(sub_request) = serde_json::from_str::<SubscriptionRequest>(&text) {
                        handle_subscription(&client_id_recv, sub_request, &connections_recv).await;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("🔌 WebSocket: Client {} disconnected", client_id_recv);
                    break;
                }
                Err(e) => {
                    warn!("⚠️ WebSocket: Error receiving from {}: {}", client_id_recv, e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
    
    // Cleanup connection
    {
        let mut conns = connections.lock().unwrap();
        conns.remove(&client_id);
    }
    
    info!("🔌 WebSocket: Client {} connection closed", client_id);
}

#[derive(Debug, Deserialize)]
struct SubscriptionRequest {
    action: String, // "subscribe" | "unsubscribe"
    event_type: String,
    filters: Option<HashMap<String, String>>,
}

async fn handle_subscription(
    client_id: &str,
    request: SubscriptionRequest,
    connections: &ConnectionManager,
) {
    info!("🔔 WebSocket: {} subscription request for {} from {}", 
          request.action, request.event_type, client_id);
    
    // For demo purposes, we'll send a mock response
    let response = match request.event_type.as_str() {
        "transaction_update" => Some(WSMessage::TransactionUpdate {
            data: TransactionUpdate {
                transaction_id: "demo_tx_123".to_string(),
                status: "pending".to_string(),
                from_transaction_hash: None,
                to_transaction_hash: None,
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        }),
        "price_update" => Some(WSMessage::PriceUpdate {
            data: PriceUpdate {
                from_token: "ETH".to_string(),
                to_token: "NEAR".to_string(),
                price: 2500.0,
                change_24h: Some(5.2),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        }),
        _ => None,
    };
    
    if let Some(msg) = response {
        if let Some(tx) = connections.lock().unwrap().get(client_id) {
            let _ = tx.send(msg);
        }
    }
}

// Start background task for periodic updates (demo)
pub async fn start_background_broadcaster(connections: ConnectionManager) {
    tokio::spawn(async move {
        let mut counter = 0;
        
        loop {
            sleep(Duration::from_secs(30)).await;
            counter += 1;
            
            let system_notification = WSMessage::SystemNotification {
                data: SystemNotification {
                    id: format!("sys_{}", counter),
                    level: "info".to_string(),
                    title: "System Status".to_string(),
                    message: "KEMBridge is operating normally".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            };
            
            // Broadcast to all connected clients
            let connections_guard = connections.lock().unwrap();
            for (client_id, tx) in connections_guard.iter() {
                if tx.send(system_notification.clone()).is_err() {
                    warn!("❌ WebSocket: Failed to broadcast to {}", client_id);
                }
            }
            drop(connections_guard);
            
            info!("📢 WebSocket: Broadcasted system notification to all clients");
        }
    });
}