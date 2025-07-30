// WebSocket connection management - Enhanced from old backend
use super::message::{EventType, RealTimeEvent, WebSocketMessage};
use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    id: Uuid,

    /// User ID (if authenticated)
    user_id: Arc<RwLock<Option<String>>>,

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

    /// Client metadata
    client_ip: Option<String>,
    user_agent: Option<String>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: Arc::new(RwLock::new(None)),
            connected_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            sender: Arc::new(RwLock::new(None)),
            is_active: Arc::new(RwLock::new(true)),
            client_ip: None,
            user_agent: None,
        }
    }

    /// Create with client metadata
    pub fn with_metadata(client_ip: Option<String>, user_agent: Option<String>) -> Self {
        let mut conn = Self::new();
        conn.client_ip = client_ip;
        conn.user_agent = user_agent;
        conn
    }

    /// Get connection ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get user ID
    pub async fn user_id(&self) -> Option<String> {
        self.user_id.read().await.clone()
    }

    /// Set user ID (authentication)
    pub async fn set_user_id(&self, user_id: String) {
        let mut uid = self.user_id.write().await;
        *uid = Some(user_id);
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
        let was_new = subscriptions.insert(event_type.clone());
        if was_new {
            info!("Connection {} subscribed to {:?}", self.id, event_type);
        }
        was_new
    }

    /// Remove subscription
    pub async fn unsubscribe(&self, event_type: &EventType) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        let was_present = subscriptions.remove(event_type);
        if was_present {
            info!("Connection {} unsubscribed from {:?}", self.id, event_type);
        }
        was_present
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
            sender
                .send(message)
                .map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("No sender available".to_string())
        }
    }

    /// Send raw text message (for legacy compatibility)
    pub async fn send_raw_message(&self, text: &str) -> Result<(), String> {
        // Create a simple text message for legacy compatibility
        let message = WebSocketMessage::Error {
            message: text.to_string(),
            code: None,
        };
        self.send_message(message).await
    }

    /// Send event if subscribed
    pub async fn send_event(&self, event: RealTimeEvent) -> Result<(), String> {
        let event_type = event.event_type();
        if self.is_subscribed(&event_type).await {
            // Check if this is a user-specific event
            if let Some(event_user_id) = event.user_id() {
                if let Some(conn_user_id) = self.user_id().await {
                    if event_user_id != conn_user_id {
                        // Skip: event is for different user
                        return Ok(());
                    }
                } else {
                    // Skip: user-specific event but connection is not authenticated
                    return Ok(());
                }
            }

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

        info!("Connection {} marked as closed", self.id);
    }

    /// Check if connection is idle (no activity for specified duration)
    pub async fn is_idle(&self, idle_duration: chrono::Duration) -> bool {
        let last_activity = self.last_activity().await;
        let now = Utc::now();
        (now - last_activity) > idle_duration
    }

    /// Get connection info for debugging
    pub async fn get_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            id: self.id,
            user_id: self.user_id().await,
            connected_at: self.connected_at,
            last_activity: self.last_activity().await,
            subscriptions: self.get_subscriptions().await,
            is_active: self.is_active().await,
            client_ip: self.client_ip.clone(),
            user_agent: self.user_agent.clone(),
        }
    }
}

/// Connection information for debugging
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub subscriptions: HashSet<EventType>,
    pub is_active: bool,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// WebSocket connection handler
pub struct WebSocketHandler {
    connection: Arc<WebSocketConnection>,
    registry: Arc<super::WebSocketRegistry>,
}

impl WebSocketHandler {
    pub fn new(
        connection: Arc<WebSocketConnection>,
        registry: Arc<super::WebSocketRegistry>,
    ) -> Self {
        Self {
            connection,
            registry,
        }
    }

    /// Handle WebSocket connection
    pub async fn handle_connection(self, socket: WebSocket) {
        info!(
            "🔌 New WebSocket connection established: {}",
            self.connection.id()
        );

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
                match message.to_json() {
                    Ok(json) => {
                        if let Err(e) = sender.send(Message::Text(json.into())).await {
                            error!("❌ Failed to send WebSocket message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to serialize WebSocket message: {}", e);
                        break;
                    }
                }
            }
            debug!(
                "📤 Outgoing task completed for connection {}",
                connection_outgoing.id()
            );
        });

        // Handle incoming messages
        let connection_incoming = self.connection.clone();
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        if let Err(e) = Self::handle_message(&connection_incoming, msg).await {
                            error!("❌ Error handling WebSocket message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ WebSocket message error: {}", e);
                        break;
                    }
                }
            }
            debug!(
                "📥 Incoming task completed for connection {}",
                connection_incoming.id()
            );
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

        info!("🔌 WebSocket connection closed: {}", self.connection.id());
    }

    /// Create test event for subscription confirmation
    fn create_test_event(event_type: &super::message::EventType) -> Option<RealTimeEvent> {
        use super::message::*;
        use chrono::Utc;
        
        match event_type {
            EventType::TransactionStatus => {
                Some(RealTimeEvent::TransactionStatusUpdate(TransactionStatusEvent {
                    transaction_id: "test_tx_123".to_string(),
                    user_id: "test_user".to_string(),
                    status: TransactionStatus::Pending,
                    from_chain: "ethereum".to_string(),
                    to_chain: "near".to_string(),
                    amount: "100.0".to_string(),
                    token_symbol: "ETH".to_string(),
                    timestamp: Utc::now(),
                    confirmation_blocks: Some(1),
                    estimated_completion: Some(Utc::now() + chrono::Duration::minutes(5)),
                }))
            }
            EventType::PriceUpdates => {
                Some(RealTimeEvent::PriceUpdate(PriceUpdateEvent {
                    token_symbol: "ETH".to_string(),
                    price_usd: "2500.00".to_string(),
                    price_change_24h: 2.5,
                    volume_24h: "1000000.00".to_string(),
                    timestamp: Utc::now(),
                    source: "test_oracle".to_string(),
                }))
            }
            EventType::SystemNotifications => {
                Some(RealTimeEvent::SystemNotification(SystemNotificationEvent {
                    notification_id: "test_notif_123".to_string(),
                    user_id: None,
                    level: NotificationLevel::Info,
                    title: "Test Notification".to_string(),
                    message: "This is a test notification for ping_test subscription".to_string(),
                    timestamp: Utc::now(),
                    action_required: false,
                    expires_at: None,
                }))
            }
            _ => None,
        }
    }

    /// Parse legacy message format (action-based)
    fn parse_legacy_message(text: &str) -> Result<WebSocketMessage, String> {
        use serde_json::Value;
        
        let json: Value = serde_json::from_str(text)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
            match action {
                "ping" => Ok(WebSocketMessage::Ping),
                "subscribe" => {
                    if let Some(event_type_str) = json.get("event_type").and_then(|v| v.as_str()) {
                        let event_type = match event_type_str {
                            "transaction_update" => super::message::EventType::TransactionStatus,
                            "price_update" => super::message::EventType::PriceUpdates,
                            "ping_test" => super::message::EventType::SystemNotifications,
                            _ => super::message::EventType::SystemNotifications,
                        };
                        Ok(WebSocketMessage::Subscribe { event_type })
                    } else {
                        Err("Missing event_type for subscribe action".to_string())
                    }
                }
                _ => Err(format!("Unsupported legacy action: {}", action))
            }
        } else {
            Err("Missing action field in legacy message".to_string())
        }
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        connection: &Arc<WebSocketConnection>,
        message: Message,
    ) -> Result<(), String> {
        connection.update_activity().await;

        match message {
            Message::Text(text) => {
                debug!(
                    "📨 Received text message from {}: {}",
                    connection.id(),
                    text
                );

                // Try to parse as WebSocket message (new format)
                match WebSocketMessage::from_json(&text) {
                    Ok(ws_message) => Self::handle_websocket_message(connection, ws_message).await,
                    Err(_) => {
                        // Try to parse as legacy format (action-based)
                        match Self::parse_legacy_message(&text) {
                            Ok(ws_message) => Self::handle_websocket_message(connection, ws_message).await,
                            Err(e) => {
                                warn!(
                                    "⚠️ Invalid WebSocket message format from {}: {} - Error: {}",
                                    connection.id(),
                                    text,
                                    e
                                );
                                // For legacy compatibility, send a simple response
                                if text.contains("ping") {
                                    connection.send_raw_message("pong").await?;
                                } else {
                                    connection
                                        .send_message(WebSocketMessage::error(
                                            "Invalid message format",
                                            Some(400),
                                        ))
                                        .await?;
                                }
                                Ok(())
                            }
                        }
                    }
                }
            }
            Message::Binary(_) => {
                warn!(
                    "⚠️ Binary messages not supported from connection {}",
                    connection.id()
                );
                connection
                    .send_message(WebSocketMessage::error(
                        "Binary messages not supported",
                        Some(400),
                    ))
                    .await?;
                Ok(())
            }
            Message::Ping(_data) => {
                debug!("🏓 Received ping from connection {}", connection.id());
                // Axum automatically handles ping/pong, but we can respond manually if needed
                Ok(())
            }
            Message::Pong(_) => {
                debug!("🏓 Received pong from connection {}", connection.id());
                Ok(())
            }
            Message::Close(close_frame) => {
                if let Some(frame) = close_frame {
                    info!(
                        "🔌 Received close message from connection {}: {} - {}",
                        connection.id(),
                        frame.code,
                        frame.reason
                    );
                } else {
                    info!(
                        "🔌 Received close message from connection {}",
                        connection.id()
                    );
                }
                Ok(())
            }
        }
    }



    /// Handle parsed WebSocket message
    async fn handle_websocket_message(
        connection: &Arc<WebSocketConnection>,
        message: WebSocketMessage,
    ) -> Result<(), String> {
        match message {
            WebSocketMessage::Auth { token } => {
                debug!(
                    "🔐 Authentication request for connection {}",
                    connection.id()
                );

                // TODO: Implement proper JWT validation
                // For now, we'll do basic validation
                if !token.is_empty() && token.len() > 10 {
                    // Extract user ID from token (simplified)
                    let user_id = format!("user_{}", uuid::Uuid::new_v4());
                    connection.set_user_id(user_id.clone()).await;

                    connection
                        .send_message(WebSocketMessage::auth_success(user_id))
                        .await?;
                    info!(
                        "✅ Authentication successful for connection {}",
                        connection.id()
                    );
                } else {
                    connection
                        .send_message(WebSocketMessage::auth_failed("Invalid token"))
                        .await?;
                    warn!(
                        "❌ Authentication failed for connection {}",
                        connection.id()
                    );
                }
                Ok(())
            }
            WebSocketMessage::Subscribe { event_type } => {
                if connection.subscribe(event_type.clone()).await {
                    connection
                        .send_message(WebSocketMessage::subscribed(event_type.clone()))
                        .await?;
                    debug!("✅ Connection {} subscribed successfully", connection.id());
                    
                    // Send a test message for the subscription to confirm it's working
                    tokio::spawn({
                        let connection = connection.clone();
                        let event_type = event_type.clone();
                        async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            let test_event = Self::create_test_event(&event_type);
                            if let Some(event) = test_event {
                                let _ = connection.send_event(event).await;
                            }
                        }
                    });
                }
                Ok(())
            }
            WebSocketMessage::Unsubscribe { event_type } => {
                connection.unsubscribe(&event_type).await;
                connection
                    .send_message(WebSocketMessage::unsubscribed(event_type.clone()))
                    .await?;
                debug!(
                    "➖ Connection {} unsubscribed from {:?}",
                    connection.id(),
                    event_type
                );
                Ok(())
            }
            WebSocketMessage::Ping => {
                connection.send_message(WebSocketMessage::Pong).await?;
                debug!("🏓 Ping-pong with connection {}", connection.id());
                Ok(())
            }
            WebSocketMessage::Pong => {
                // Heartbeat response received
                debug!("💓 Heartbeat received from connection {}", connection.id());
                Ok(())
            }
            _ => {
                warn!(
                    "⚠️ Unsupported message type received from connection {}",
                    connection.id()
                );
                connection
                    .send_message(WebSocketMessage::error(
                        "Unsupported message type",
                        Some(400),
                    ))
                    .await?;
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
