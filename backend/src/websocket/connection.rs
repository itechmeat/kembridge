// src/websocket/connection.rs - WebSocket connection management
use super::message::{WebSocketMessage, EventType, RealTimeEvent};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    id: Uuid,
    
    /// User ID (if authenticated)
    user_id: Option<Uuid>,
    
    /// Connection timestamp
    connected_at: DateTime<Utc>,
    
    /// Last activity timestamp
    last_activity: Arc<RwLock<DateTime<Utc>>>,
    
    /// Subscribed event types
    subscriptions: Arc<RwLock<HashSet<EventType>>>,
    
    /// Message sender
    sender: Arc<RwLock<Option<mpsc::UnboundedSender<WebSocketMessage>>>>,
    
    /// Connection status
    is_active: Arc<RwLock<bool>>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            connected_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            sender: Arc::new(RwLock::new(None)),
            is_active: Arc::new(RwLock::new(true)),
        }
    }

    /// Get connection ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get user ID
    pub fn user_id(&self) -> Option<Uuid> {
        self.user_id
    }

    /// Set user ID (authentication)
    pub fn set_user_id(&mut self, user_id: Uuid) {
        self.user_id = Some(user_id);
    }

    /// Get connection timestamp
    pub fn connected_at(&self) -> DateTime<Utc> {
        self.connected_at
    }

    /// Update last activity timestamp
    pub async fn update_activity(&self) {
        let mut last_activity = self.last_activity.write().await;
        *last_activity = Utc::now();
    }

    /// Get last activity timestamp
    pub async fn last_activity(&self) -> DateTime<Utc> {
        *self.last_activity.read().await
    }

    /// Add subscription
    pub async fn subscribe(&self, event_type: EventType) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(event_type)
    }

    /// Remove subscription
    pub async fn unsubscribe(&self, event_type: &EventType) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(event_type)
    }

    /// Check if subscribed to event type
    pub async fn is_subscribed(&self, event_type: &EventType) -> bool {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.contains(event_type)
    }

    /// Get all subscriptions
    pub async fn get_subscriptions(&self) -> HashSet<EventType> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.clone()
    }

    /// Set message sender
    pub async fn set_sender(&self, sender: mpsc::UnboundedSender<WebSocketMessage>) {
        let mut sender_lock = self.sender.write().await;
        *sender_lock = Some(sender);
    }

    /// Send message to connection
    pub async fn send_message(&self, message: WebSocketMessage) -> Result<(), String> {
        let sender = self.sender.read().await;
        if let Some(sender) = sender.as_ref() {
            self.update_activity().await;
            sender.send(message).map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("No sender available".to_string())
        }
    }

    /// Send event if subscribed
    pub async fn send_event(&self, event: RealTimeEvent) -> Result<(), String> {
        if self.is_subscribed(&event.event_type()).await {
            self.send_message(WebSocketMessage::event(event)).await
        } else {
            Ok(()) // Not subscribed, skip silently
        }
    }

    /// Check if connection is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Mark connection as inactive
    pub async fn close(&self) {
        let mut is_active = self.is_active.write().await;
        *is_active = false;
        
        // Clear sender to prevent further messages
        let mut sender = self.sender.write().await;
        *sender = None;
    }

    /// Check if connection is idle (no activity for specified duration)
    pub async fn is_idle(&self, idle_duration: chrono::Duration) -> bool {
        let last_activity = self.last_activity().await;
        let now = Utc::now();
        (now - last_activity) > idle_duration
    }
}

/// WebSocket connection handler
pub struct WebSocketHandler {
    connection: Arc<WebSocketConnection>,
    registry: Arc<super::WebSocketRegistry>,
}

impl WebSocketHandler {
    pub fn new(connection: Arc<WebSocketConnection>, registry: Arc<super::WebSocketRegistry>) -> Self {
        Self { connection, registry }
    }

    /// Handle WebSocket connection
    pub async fn handle_connection(self, mut socket: WebSocket) {
        info!("New WebSocket connection established: {}", self.connection.id());
        
        // Add connection to registry
        self.registry.add_connection(self.connection.clone()).await;
        
        // Create message channel
        let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketMessage>();
        self.connection.set_sender(tx).await;
        
        // Split socket into sender and receiver
        let (mut sender, mut receiver) = socket.split();
        
        // Spawn task to handle outgoing messages
        let connection_outgoing = self.connection.clone();
        let outgoing_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Ok(json) = message.to_json() {
                    if let Err(e) = sender.send(Message::Text(json.into())).await {
                        error!("Failed to send WebSocket message: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Handle incoming messages
        let connection_incoming = self.connection.clone();
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    if let Err(e) = Self::handle_message(&connection_incoming, msg).await {
                        error!("Error handling WebSocket message: {}", e);
                    }
                } else {
                    break;
                }
            }
        });
        
        // Wait for either task to complete
        tokio::select! {
            _ = outgoing_task => {
                debug!("Outgoing task completed for connection {}", self.connection.id());
            }
            _ = incoming_task => {
                debug!("Incoming task completed for connection {}", self.connection.id());
            }
        }
        
        // Cleanup
        self.connection.close().await;
        self.registry.remove_connection(&self.connection.id()).await;
        
        info!("WebSocket connection closed: {}", self.connection.id());
    }

    /// Handle incoming WebSocket message
    async fn handle_message(connection: &Arc<WebSocketConnection>, message: Message) -> Result<(), String> {
        connection.update_activity().await;
        
        match message {
            Message::Text(text) => {
                if let Ok(ws_message) = WebSocketMessage::from_json(&text) {
                    Self::handle_websocket_message(connection, ws_message).await
                } else {
                    warn!("Invalid WebSocket message format: {}", text);
                    connection.send_message(WebSocketMessage::error("Invalid message format", Some(400))).await?;
                    Ok(())
                }
            }
            Message::Binary(_) => {
                warn!("Binary messages not supported");
                connection.send_message(WebSocketMessage::error("Binary messages not supported", Some(400))).await?;
                Ok(())
            }
            Message::Ping(_data) => {
                debug!("Received ping from connection {}", connection.id());
                // Axum automatically handles ping/pong
                Ok(())
            }
            Message::Pong(_) => {
                debug!("Received pong from connection {}", connection.id());
                Ok(())
            }
            Message::Close(_) => {
                debug!("Received close message from connection {}", connection.id());
                Ok(())
            }
        }
    }

    /// Handle parsed WebSocket message
    async fn handle_websocket_message(connection: &Arc<WebSocketConnection>, message: WebSocketMessage) -> Result<(), String> {
        match message {
            WebSocketMessage::Auth { token: _ } => {
                // Authentication will be handled by auth module
                debug!("Authentication request for connection {}", connection.id());
                // For now, we'll implement basic authentication in the next step
                Ok(())
            }
            WebSocketMessage::Subscribe { event_type } => {
                if connection.subscribe(event_type.clone()).await {
                    debug!("Connection {} subscribed to {:?}", connection.id(), event_type);
                    connection.send_message(WebSocketMessage::subscribed(event_type)).await?;
                }
                Ok(())
            }
            WebSocketMessage::Unsubscribe { event_type } => {
                if connection.unsubscribe(&event_type).await {
                    debug!("Connection {} unsubscribed from {:?}", connection.id(), event_type);
                }
                Ok(())
            }
            WebSocketMessage::Ping => {
                connection.send_message(WebSocketMessage::Pong).await?;
                Ok(())
            }
            WebSocketMessage::Pong => {
                // Heartbeat response received
                Ok(())
            }
            _ => {
                warn!("Unsupported message type received from connection {}", connection.id());
                connection.send_message(WebSocketMessage::error("Unsupported message type", Some(400))).await?;
                Ok(())
            }
        }
    }
}

impl Default for WebSocketConnection {
    fn default() -> Self {
        Self::new()
    }
}