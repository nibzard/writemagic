use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::websocket::{ClientMessage, ServerMessage};

/// Unique identifier for a WebSocket connection
pub type ConnectionId = String;

/// WebSocket connection wrapper that handles message serialization/deserialization
pub struct WebSocketConnection {
    pub id: ConnectionId,
    pub user_id: String,
    pub username: String,
    sender: mpsc::UnboundedSender<ServerMessage>,
    subscriptions: Arc<RwLock<Vec<String>>>, // Document IDs
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new(
        websocket: WebSocket,
        user_id: String,
        username: String,
    ) -> (Self, mpsc::UnboundedReceiver<ClientMessage>) {
        let id = Uuid::new_v4().to_string();
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (server_tx, server_rx) = mpsc::unbounded_channel();

        let connection = Self {
            id: id.clone(),
            user_id: user_id.clone(),
            username,
            sender: server_tx,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        };

        // Spawn task to handle WebSocket communication
        tokio::spawn(Self::handle_websocket_messages(
            websocket,
            message_tx,
            server_rx,
            id.clone(),
            user_id,
        ));

        (connection, message_rx)
    }

    /// Send a message to the client
    pub async fn send_message(&self, message: ServerMessage) -> Result<(), String> {
        self.sender
            .send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Subscribe to document updates
    pub async fn subscribe_to_document(&self, document_id: String) {
        let mut subscriptions = self.subscriptions.write().await;
        if !subscriptions.contains(&document_id) {
            subscriptions.push(document_id);
        }
    }

    /// Unsubscribe from document updates
    pub async fn unsubscribe_from_document(&self, document_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.retain(|id| id != document_id);
    }

    /// Check if connected to a specific document
    pub async fn is_subscribed_to_document(&self, document_id: &str) -> bool {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.contains(&document_id.to_string())
    }

    /// Get all document subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.clone()
    }

    /// Handle WebSocket message communication
    async fn handle_websocket_messages(
        mut websocket: WebSocket,
        message_sender: mpsc::UnboundedSender<ClientMessage>,
        mut server_receiver: mpsc::UnboundedReceiver<ServerMessage>,
        connection_id: String,
        user_id: String,
    ) {
        // Send initial connection confirmation
        let connected_message = ServerMessage::Connected {
            connection_id: connection_id.clone(),
            user_id: user_id.clone(),
        };

        if let Ok(json) = serde_json::to_string(&connected_message) {
            let _ = websocket.send(Message::Text(json)).await;
        }

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                message = websocket.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<ClientMessage>(&text) {
                                Ok(client_message) => {
                                    if message_sender.send(client_message).is_err() {
                                        tracing::warn!("Failed to forward client message");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Invalid client message format: {}", e);
                                    let error_message = ServerMessage::Error {
                                        message: "Invalid message format".to_string(),
                                        code: Some("INVALID_FORMAT".to_string()),
                                    };
                                    if let Ok(json) = serde_json::to_string(&error_message) {
                                        let _ = websocket.send(Message::Text(json)).await;
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            tracing::info!("WebSocket connection {} closed by client", connection_id);
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = websocket.send(Message::Pong(data)).await;
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error for connection {}: {}", connection_id, e);
                            break;
                        }
                        None => {
                            tracing::info!("WebSocket connection {} ended", connection_id);
                            break;
                        }
                        _ => {
                            // Handle other message types (binary, pong, etc.)
                            continue;
                        }
                    }
                }

                // Handle outgoing server messages
                server_message = server_receiver.recv() => {
                    match server_message {
                        Some(message) => {
                            match serde_json::to_string(&message) {
                                Ok(json) => {
                                    if websocket.send(Message::Text(json)).await.is_err() {
                                        tracing::warn!("Failed to send message to client");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to serialize server message: {}", e);
                                }
                            }
                        }
                        None => {
                            tracing::info!("Server message channel closed for connection {}", connection_id);
                            break;
                        }
                    }
                }
            }
        }

        // Clean shutdown
        let _ = websocket.close().await;
        tracing::info!("WebSocket connection {} handler terminated", connection_id);
    }
}

/// Connection statistics for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionStats {
    pub connection_id: String,
    pub user_id: String,
    pub username: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub subscription_count: usize,
    pub subscriptions: Vec<String>,
}

impl WebSocketConnection {
    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        let subscriptions = self.get_subscriptions().await;
        
        ConnectionStats {
            connection_id: self.id.clone(),
            user_id: self.user_id.clone(),
            username: self.username.clone(),
            connected_at: chrono::Utc::now(), // In real implementation, store actual connect time
            subscription_count: subscriptions.len(),
            subscriptions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_subscription() {
        // This is a simplified test - in reality you'd need to mock the WebSocket
        let subscriptions = Arc::new(RwLock::new(Vec::new()));
        let document_id = "doc_123".to_string();

        // Simulate subscription
        {
            let mut subs = subscriptions.write().await;
            subs.push(document_id.clone());
        }

        // Check subscription
        {
            let subs = subscriptions.read().await;
            assert!(subs.contains(&document_id));
        }

        // Simulate unsubscription
        {
            let mut subs = subscriptions.write().await;
            subs.retain(|id| id != &document_id);
        }

        // Check unsubscription
        {
            let subs = subscriptions.read().await;
            assert!(!subs.contains(&document_id));
        }
    }
}