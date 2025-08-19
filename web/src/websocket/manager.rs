use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::websocket::{
    connection::{ConnectionId, ConnectionStats},
    messages::{ClientMessage, DocumentEvent, ServerMessage},
    WebSocketConnection,
};

/// Manages all WebSocket connections and message broadcasting
#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<DashMap<ConnectionId, Arc<WebSocketConnection>>>,
    document_subscribers: Arc<DashMap<String, Vec<ConnectionId>>>, // document_id -> connection_ids
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            document_subscribers: Arc::new(DashMap::new()),
        }
    }

    /// Add a new WebSocket connection
    pub async fn add_connection(
        &self,
        connection: WebSocketConnection,
        message_receiver: mpsc::UnboundedReceiver<ClientMessage>,
    ) {
        let connection_id = connection.id.clone();
        let connection = Arc::new(connection);
        
        // Store the connection
        self.connections.insert(connection_id.clone(), connection.clone());
        
        tracing::info!(
            connection_id = %connection_id,
            user_id = %connection.user_id,
            username = %connection.username,
            "WebSocket connection added"
        );

        // Handle messages from this connection
        let manager = self.clone();
        tokio::spawn(async move {
            manager.handle_connection_messages(connection, message_receiver).await;
        });
    }

    /// Remove a WebSocket connection
    pub async fn remove_connection(&self, connection_id: &ConnectionId) {
        if let Some((_, connection)) = self.connections.remove(connection_id) {
            // Clean up document subscriptions
            let subscriptions = connection.get_subscriptions().await;
            for document_id in subscriptions {
                self.remove_document_subscriber(&document_id, connection_id).await;
            }
            
            tracing::info!(
                connection_id = %connection_id,
                user_id = %connection.user_id,
                "WebSocket connection removed"
            );
        }
    }

    /// Get connection by ID
    pub fn get_connection(&self, connection_id: &ConnectionId) -> Option<Arc<WebSocketConnection>> {
        self.connections.get(connection_id).map(|entry| entry.clone())
    }

    /// Subscribe a connection to document updates
    pub async fn subscribe_to_document(&self, connection_id: &ConnectionId, document_id: String) {
        if let Some(connection) = self.get_connection(connection_id) {
            // Add to connection's subscriptions
            connection.subscribe_to_document(document_id.clone()).await;
            
            // Add to document subscribers
            self.add_document_subscriber(document_id.clone(), connection_id.clone()).await;
            
            // Send confirmation
            let subscriber_count = self.get_document_subscriber_count(&document_id).await;
            let confirmation = ServerMessage::SubscriptionConfirmed {
                document_id: document_id.clone(),
                user_count: subscriber_count,
            };
            
            let _ = connection.send_message(confirmation).await;
            
            // Notify other users about the new subscriber
            let user_joined = ServerMessage::UserJoined {
                document_id,
                user_id: connection.user_id.clone(),
                username: connection.username.clone(),
            };
            
            self.broadcast_to_document_subscribers(&document_id, user_joined, Some(connection_id)).await;
            
            tracing::debug!(
                connection_id = %connection_id,
                document_id = %document_id,
                "User subscribed to document"
            );
        }
    }

    /// Unsubscribe a connection from document updates
    pub async fn unsubscribe_from_document(&self, connection_id: &ConnectionId, document_id: &str) {
        if let Some(connection) = self.get_connection(connection_id) {
            // Remove from connection's subscriptions
            connection.unsubscribe_from_document(document_id).await;
            
            // Remove from document subscribers
            self.remove_document_subscriber(document_id, connection_id).await;
            
            // Notify other users about the departure
            let user_left = ServerMessage::UserLeft {
                document_id: document_id.to_string(),
                user_id: connection.user_id.clone(),
            };
            
            self.broadcast_to_document_subscribers(document_id, user_left, Some(connection_id)).await;
            
            tracing::debug!(
                connection_id = %connection_id,
                document_id = %document_id,
                "User unsubscribed from document"
            );
        }
    }

    /// Broadcast a document event to all subscribers
    pub async fn broadcast_document_event(&self, event: DocumentEvent) {
        let message = ServerMessage::DocumentEvent {
            event: event.clone(),
        };
        
        self.broadcast_to_document_subscribers(&event.document_id, message, None).await;
        
        tracing::debug!(
            document_id = %event.document_id,
            user_id = %event.user_id,
            operation = ?event.operation,
            "Document event broadcasted"
        );
    }

    /// Get statistics for all connections
    pub async fn get_connection_stats(&self) -> Vec<ConnectionStats> {
        let mut stats = Vec::new();
        
        for entry in self.connections.iter() {
            let connection = entry.value();
            stats.push(connection.get_stats().await);
        }
        
        stats
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get subscriber count for a document
    pub async fn get_document_subscriber_count(&self, document_id: &str) -> usize {
        self.document_subscribers
            .get(document_id)
            .map(|subscribers| subscribers.len())
            .unwrap_or(0)
    }

    /// Handle messages from a specific connection
    async fn handle_connection_messages(
        &self,
        connection: Arc<WebSocketConnection>,
        mut message_receiver: mpsc::UnboundedReceiver<ClientMessage>,
    ) {
        let connection_id = connection.id.clone();
        
        while let Some(message) = message_receiver.recv().await {
            match self.process_client_message(&connection, message).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::warn!(
                        connection_id = %connection_id,
                        error = %e,
                        "Error processing client message"
                    );
                    
                    let error_message = ServerMessage::Error {
                        message: e,
                        code: Some("PROCESSING_ERROR".to_string()),
                    };
                    
                    let _ = connection.send_message(error_message).await;
                }
            }
        }
        
        // Connection closed, clean up
        self.remove_connection(&connection_id).await;
    }

    /// Process a client message
    async fn process_client_message(
        &self,
        connection: &Arc<WebSocketConnection>,
        message: ClientMessage,
    ) -> Result<(), String> {
        match message {
            ClientMessage::SubscribeDocument { document_id } => {
                self.subscribe_to_document(&connection.id, document_id).await;
                Ok(())
            }
            ClientMessage::UnsubscribeDocument { document_id } => {
                self.unsubscribe_from_document(&connection.id, &document_id).await;
                Ok(())
            }
            ClientMessage::DocumentEdit {
                document_id,
                operation,
                timestamp,
            } => {
                // Verify user is subscribed to the document
                if !connection.is_subscribed_to_document(&document_id).await {
                    return Err("Not subscribed to document".to_string());
                }

                // Create document event
                let event = DocumentEvent {
                    document_id,
                    user_id: connection.user_id.clone(),
                    username: connection.username.clone(),
                    operation,
                    timestamp,
                    version: 1, // In a real implementation, this would be managed properly
                };

                // Broadcast to other subscribers
                self.broadcast_document_event(event).await;
                Ok(())
            }
            ClientMessage::CursorUpdate {
                document_id,
                position,
            } => {
                // Verify user is subscribed to the document
                if !connection.is_subscribed_to_document(&document_id).await {
                    return Err("Not subscribed to document".to_string());
                }

                let cursor_message = ServerMessage::CursorUpdate {
                    document_id: document_id.clone(),
                    user_id: connection.user_id.clone(),
                    username: connection.username.clone(),
                    position,
                };

                self.broadcast_to_document_subscribers(&document_id, cursor_message, Some(&connection.id)).await;
                Ok(())
            }
            ClientMessage::Ping { timestamp } => {
                let pong = ServerMessage::Pong { timestamp };
                connection.send_message(pong).await.map_err(|e| e.to_string())?;
                Ok(())
            }
        }
    }

    /// Add a subscriber to a document
    async fn add_document_subscriber(&self, document_id: String, connection_id: ConnectionId) {
        let mut subscribers = self.document_subscribers
            .entry(document_id)
            .or_insert_with(Vec::new);
        
        if !subscribers.contains(&connection_id) {
            subscribers.push(connection_id);
        }
    }

    /// Remove a subscriber from a document
    async fn remove_document_subscriber(&self, document_id: &str, connection_id: &ConnectionId) {
        if let Some(mut subscribers) = self.document_subscribers.get_mut(document_id) {
            subscribers.retain(|id| id != connection_id);
            
            // Clean up empty subscriber lists
            if subscribers.is_empty() {
                drop(subscribers); // Release the mutable reference
                self.document_subscribers.remove(document_id);
            }
        }
    }

    /// Broadcast a message to all document subscribers
    async fn broadcast_to_document_subscribers(
        &self,
        document_id: &str,
        message: ServerMessage,
        exclude_connection: Option<&ConnectionId>,
    ) {
        if let Some(subscribers) = self.document_subscribers.get(document_id) {
            let subscriber_ids: Vec<ConnectionId> = subscribers.clone();
            
            for connection_id in subscriber_ids {
                // Skip excluded connection (usually the sender)
                if let Some(exclude_id) = exclude_connection {
                    if &connection_id == exclude_id {
                        continue;
                    }
                }
                
                if let Some(connection) = self.get_connection(&connection_id) {
                    let _ = connection.send_message(message.clone()).await;
                }
            }
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global connection manager statistics
#[derive(Debug, serde::Serialize)]
pub struct ManagerStats {
    pub total_connections: usize,
    pub active_documents: usize,
    pub total_subscriptions: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConnectionManager {
    /// Get manager statistics
    pub async fn get_manager_stats(&self) -> ManagerStats {
        let total_subscriptions: usize = self.document_subscribers
            .iter()
            .map(|entry| entry.value().len())
            .sum();

        ManagerStats {
            total_connections: self.connection_count(),
            active_documents: self.document_subscribers.len(),
            total_subscriptions,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_connection_manager_basic_operations() {
        let manager = ConnectionManager::new();
        
        // Initially empty
        assert_eq!(manager.connection_count(), 0);
        
        // Manager stats should show empty state
        let stats = manager.get_manager_stats().await;
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_documents, 0);
        assert_eq!(stats.total_subscriptions, 0);
    }

    #[test]
    fn test_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.connection_count(), 0);
        
        let default_manager = ConnectionManager::default();
        assert_eq!(default_manager.connection_count(), 0);
    }
}