use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Subscribe to document updates
    SubscribeDocument {
        document_id: String,
    },
    /// Unsubscribe from document updates
    UnsubscribeDocument {
        document_id: String,
    },
    /// Real-time document edit
    DocumentEdit {
        document_id: String,
        operation: EditOperation,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Cursor position update
    CursorUpdate {
        document_id: String,
        position: CursorPosition,
    },
    /// Ping to keep connection alive
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Confirmation of subscription
    SubscriptionConfirmed {
        document_id: String,
        user_count: usize,
    },
    /// Document event from another user
    DocumentEvent {
        event: DocumentEvent,
    },
    /// User joined document
    UserJoined {
        document_id: String,
        user_id: String,
        username: String,
    },
    /// User left document
    UserLeft {
        document_id: String,
        user_id: String,
    },
    /// Cursor update from another user
    CursorUpdate {
        document_id: String,
        user_id: String,
        username: String,
        position: CursorPosition,
    },
    /// Error message
    Error {
        message: String,
        code: Option<String>,
    },
    /// Pong response to ping
    Pong {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Connection established
    Connected {
        connection_id: String,
        user_id: String,
    },
}

/// Document events that can be broadcast to subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEvent {
    pub document_id: String,
    pub user_id: String,
    pub username: String,
    pub operation: EditOperation,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

/// Text editing operations for real-time collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum EditOperation {
    /// Insert text at position
    Insert {
        position: u32,
        text: String,
    },
    /// Delete text range
    Delete {
        start: u32,
        end: u32,
    },
    /// Replace text range
    Replace {
        start: u32,
        end: u32,
        text: String,
    },
    /// Set entire document content
    SetContent {
        content: String,
    },
}

/// Cursor position in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Character offset in the document
    pub offset: u32,
    /// Selection start (if different from offset)
    pub selection_start: Option<u32>,
    /// Selection end (if different from offset)
    pub selection_end: Option<u32>,
    /// Line number (for display purposes)
    pub line: Option<u32>,
    /// Column number (for display purposes)
    pub column: Option<u32>,
}

impl EditOperation {
    /// Apply this operation to text content
    pub fn apply(&self, content: &str) -> Result<String, String> {
        match self {
            EditOperation::Insert { position, text } => {
                let pos = *position as usize;
                if pos > content.len() {
                    return Err("Insert position out of bounds".to_string());
                }
                let mut result = String::with_capacity(content.len() + text.len());
                result.push_str(&content[..pos]);
                result.push_str(text);
                result.push_str(&content[pos..]);
                Ok(result)
            }
            EditOperation::Delete { start, end } => {
                let start = *start as usize;
                let end = *end as usize;
                if start > content.len() || end > content.len() || start > end {
                    return Err("Delete range out of bounds".to_string());
                }
                let mut result = String::with_capacity(content.len());
                result.push_str(&content[..start]);
                result.push_str(&content[end..]);
                Ok(result)
            }
            EditOperation::Replace { start, end, text } => {
                let start = *start as usize;
                let end = *end as usize;
                if start > content.len() || end > content.len() || start > end {
                    return Err("Replace range out of bounds".to_string());
                }
                let mut result = String::with_capacity(content.len() + text.len());
                result.push_str(&content[..start]);
                result.push_str(text);
                result.push_str(&content[end..]);
                Ok(result)
            }
            EditOperation::SetContent { content } => Ok(content.clone()),
        }
    }

    /// Check if this operation conflicts with another operation
    pub fn conflicts_with(&self, other: &EditOperation) -> bool {
        match (self, other) {
            (EditOperation::Insert { position: p1, .. }, EditOperation::Insert { position: p2, .. }) => {
                p1 == p2 // Same position insertion might conflict
            }
            (EditOperation::Delete { start: s1, end: e1 }, EditOperation::Delete { start: s2, end: e2 }) => {
                // Check if ranges overlap
                !(e1 <= s2 || e2 <= s1)
            }
            (EditOperation::Insert { position, .. }, EditOperation::Delete { start, end }) |
            (EditOperation::Delete { start, end }, EditOperation::Insert { position, .. }) => {
                position >= start && position <= end
            }
            // For simplicity, consider all other combinations as conflicting
            _ => true,
        }
    }
}

impl CursorPosition {
    /// Create a simple cursor position at an offset
    pub fn at_offset(offset: u32) -> Self {
        Self {
            offset,
            selection_start: None,
            selection_end: None,
            line: None,
            column: None,
        }
    }

    /// Create a cursor with selection
    pub fn with_selection(offset: u32, selection_start: u32, selection_end: u32) -> Self {
        Self {
            offset,
            selection_start: Some(selection_start),
            selection_end: Some(selection_end),
            line: None,
            column: None,
        }
    }

    /// Check if this cursor has a selection
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() || self.selection_end.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operation() {
        let content = "Hello world";
        let operation = EditOperation::Insert {
            position: 6,
            text: "beautiful ".to_string(),
        };
        
        let result = operation.apply(content).unwrap();
        assert_eq!(result, "Hello beautiful world");
    }

    #[test]
    fn test_delete_operation() {
        let content = "Hello beautiful world";
        let operation = EditOperation::Delete {
            start: 6,
            end: 16,
        };
        
        let result = operation.apply(content).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_replace_operation() {
        let content = "Hello world";
        let operation = EditOperation::Replace {
            start: 6,
            end: 11,
            text: "universe".to_string(),
        };
        
        let result = operation.apply(content).unwrap();
        assert_eq!(result, "Hello universe");
    }

    #[test]
    fn test_operation_conflicts() {
        let insert1 = EditOperation::Insert {
            position: 5,
            text: "A".to_string(),
        };
        let insert2 = EditOperation::Insert {
            position: 5,
            text: "B".to_string(),
        };
        
        assert!(insert1.conflicts_with(&insert2));
        
        let delete = EditOperation::Delete {
            start: 3,
            end: 7,
        };
        
        assert!(insert1.conflicts_with(&delete));
    }

    #[test]
    fn test_cursor_position() {
        let cursor = CursorPosition::at_offset(42);
        assert_eq!(cursor.offset, 42);
        assert!(!cursor.has_selection());
        
        let cursor_with_selection = CursorPosition::with_selection(42, 40, 50);
        assert!(cursor_with_selection.has_selection());
    }
}