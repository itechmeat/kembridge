//! Unit tests for WebSocket message handling

use super::message::{WebSocketMessage, EventType};
use super::connection::WebSocketConnection;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_auth_success_message_creation() {
    let user_id = "test-user-123".to_string();
    let message = WebSocketMessage::auth_success(user_id.clone());
    
    match &message {
        WebSocketMessage::AuthSuccess { user_id: msg_user_id } => {
            assert_eq!(msg_user_id, &user_id);
        }
        _ => panic!("Expected AuthSuccess message"),
    }
    
    // Test JSON serialization
    let json = message.to_json().expect("Should serialize to JSON");
    assert!(json.contains("AuthSuccess"));
    assert!(json.contains("test-user-123"));
    
    println!("✅ AuthSuccess message created correctly: {}", json);
}

#[tokio::test]
async fn test_auth_failed_message_creation() {
    let error_msg = "Invalid token signature";
    let message = WebSocketMessage::auth_failed(error_msg);
    
    match &message {
        WebSocketMessage::AuthFailed { error } => {
            assert_eq!(error, error_msg);
        }
        _ => panic!("Expected AuthFailed message"),
    }
    
    // Test JSON serialization
    let json = message.to_json().expect("Should serialize to JSON");
    assert!(json.contains("AuthFailed"));
    assert!(json.contains("Invalid token signature"));
    
    println!("✅ AuthFailed message created correctly: {}", json);
}

#[tokio::test]
async fn test_auth_failed_with_different_errors() {
    let test_cases = vec![
        "Token has expired",
        "Invalid token signature", 
        "Invalid user ID in token",
        "Invalid token"
    ];
    
    for error_msg in test_cases {
        let message = WebSocketMessage::auth_failed(error_msg);
        let json = message.to_json().expect("Should serialize to JSON");
        
        assert!(json.contains("AuthFailed"));
        assert!(json.contains(error_msg));
        
        println!("✅ AuthFailed message for '{}': {}", error_msg, json);
    }
}

#[tokio::test]
async fn test_websocket_connection_send_auth_messages() {
    let connection = Arc::new(WebSocketConnection::new());
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Set up the connection with a sender
    connection.set_sender(tx).await;
    
    // Test sending AuthSuccess
    let user_id = "test-user-456".to_string();
    let result = connection.send_message(WebSocketMessage::auth_success(user_id.clone())).await;
    assert!(result.is_ok(), "Should send AuthSuccess message successfully");
    
    // Verify the message was sent
    let received_message = rx.recv().await.expect("Should receive message");
    match received_message {
        WebSocketMessage::AuthSuccess { user_id: msg_user_id } => {
            assert_eq!(msg_user_id, user_id);
        }
        _ => panic!("Expected AuthSuccess message"),
    }
    
    // Test sending AuthFailed
    let error_msg = "Test authentication failure";
    let result = connection.send_message(WebSocketMessage::auth_failed(error_msg)).await;
    assert!(result.is_ok(), "Should send AuthFailed message successfully");
    
    // Verify the message was sent
    let received_message = rx.recv().await.expect("Should receive message");
    match received_message {
        WebSocketMessage::AuthFailed { error } => {
            assert_eq!(error, error_msg);
        }
        _ => panic!("Expected AuthFailed message"),
    }
    
    println!("✅ WebSocket connection can send auth messages correctly");
}

#[tokio::test]
async fn test_message_json_roundtrip() {
    // Test AuthSuccess roundtrip
    let auth_success = WebSocketMessage::auth_success("user123".to_string());
    let json = auth_success.to_json().expect("Should serialize");
    let parsed = WebSocketMessage::from_json(&json).expect("Should deserialize");
    
    match parsed {
        WebSocketMessage::AuthSuccess { user_id } => {
            assert_eq!(user_id, "user123");
        }
        _ => panic!("Expected AuthSuccess after roundtrip"),
    }
    
    // Test AuthFailed roundtrip
    let auth_failed = WebSocketMessage::auth_failed("Test error");
    let json = auth_failed.to_json().expect("Should serialize");
    let parsed = WebSocketMessage::from_json(&json).expect("Should deserialize");
    
    match parsed {
        WebSocketMessage::AuthFailed { error } => {
            assert_eq!(error, "Test error");
        }
        _ => panic!("Expected AuthFailed after roundtrip"),
    }
    
    println!("✅ JSON serialization/deserialization works correctly");
}

#[test]
fn test_auth_message_format_compatibility() {
    // Test that our messages match the expected format for the frontend
    let auth_success = WebSocketMessage::auth_success("user123".to_string());
    let json = auth_success.to_json().expect("Should serialize");
    
    // Parse as generic JSON to check structure
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should parse as JSON");
    
    assert_eq!(parsed["type"], "AuthSuccess");
    assert_eq!(parsed["user_id"], "user123");
    
    let auth_failed = WebSocketMessage::auth_failed("Invalid token");
    let json = auth_failed.to_json().expect("Should serialize");
    
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should parse as JSON");
    
    assert_eq!(parsed["type"], "AuthFailed");
    assert_eq!(parsed["error"], "Invalid token");
    
    println!("✅ Message format is compatible with frontend expectations");
}