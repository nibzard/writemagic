//! Serialization utilities for IndexedDB storage
//! 
//! This module handles conversion between Rust domain entities and
//! JavaScript objects suitable for IndexedDB storage.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use writemagic_shared::{EntityId, Timestamp, ContentType, ContentHash, FilePath};
use crate::entities::{Document, Project};

/// Error type for serialization operations
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("JSON serialization failed: {message}")]
    JsonSerialization { message: String },
    
    #[error("JSON deserialization failed: {message}")]
    JsonDeserialization { message: String },
    
    #[error("JavaScript conversion failed: {message}")]
    JavaScriptConversion { message: String },
    
    #[error("Invalid entity data: {field} - {message}")]
    InvalidEntityData { field: String, message: String },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

/// Document structure for IndexedDB storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub content_hash: String,
    pub file_path: Option<String>,
    pub word_count: u32,
    pub character_count: u32,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: u64,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    
    // Search index fields (pre-computed for performance)
    pub search_title: String,
    pub search_content: String,
    pub search_tokens: Vec<String>,
}

impl IndexedDbDocument {
    /// Create search tokens from document content
    pub fn create_search_tokens(title: &str, content: &str) -> Vec<String> {
        use crate::web_persistence::schema::SearchConfig;
        
        let search_config = SearchConfig::default();
        let mut tokens = Vec::new();
        
        // Tokenize title (with higher weight)
        tokens.extend(search_config.tokenize(title));
        
        // Tokenize content
        tokens.extend(search_config.tokenize(content));
        
        // Remove duplicates and sort
        tokens.sort();
        tokens.dedup();
        
        tokens
    }
    
    /// Convert to JsValue for IndexedDB storage
    pub fn to_js_value(&self) -> Result<JsValue, SerializationError> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert document to JsValue: {}", e)
            })
    }
    
    /// Create from JsValue from IndexedDB
    pub fn from_js_value(value: &JsValue) -> Result<Self, SerializationError> {
        serde_wasm_bindgen::from_value(value.clone())
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert JsValue to document: {}", e)
            })
    }
    
    /// Validate document data consistency
    pub fn validate(&self) -> Result<(), SerializationError> {
        if self.id.is_empty() {
            return Err(SerializationError::InvalidEntityData {
                field: "id".to_string(),
                message: "Document ID cannot be empty".to_string(),
            });
        }
        
        if self.title.is_empty() {
            return Err(SerializationError::InvalidEntityData {
                field: "title".to_string(),
                message: "Document title cannot be empty".to_string(),
            });
        }
        
        if self.word_count == 0 && !self.content.is_empty() {
            return Err(SerializationError::InvalidEntityData {
                field: "word_count".to_string(),
                message: "Word count should be greater than 0 for non-empty content".to_string(),
            });
        }
        
        Ok(())
    }
}

impl From<&Document> for IndexedDbDocument {
    fn from(doc: &Document) -> Self {
        let search_tokens = Self::create_search_tokens(&doc.title, &doc.content);
        
        Self {
            id: doc.id.to_string(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            content_type: doc.content_type.to_string(),
            content_hash: doc.content_hash.to_string(),
            file_path: doc.file_path.as_ref().map(|p| p.to_string()),
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at: doc.created_at.to_string(),
            updated_at: doc.updated_at.to_string(),
            created_by: doc.created_by.as_ref().map(|id| id.to_string()),
            updated_by: doc.updated_by.as_ref().map(|id| id.to_string()),
            version: doc.version,
            is_deleted: doc.is_deleted,
            deleted_at: doc.deleted_at.as_ref().map(|t| t.to_string()),
            search_title: doc.title.to_lowercase(),
            search_content: doc.content.to_lowercase(),
            search_tokens,
        }
    }
}

impl TryFrom<IndexedDbDocument> for Document {
    type Error = SerializationError;
    
    fn try_from(doc: IndexedDbDocument) -> Result<Self, Self::Error> {
        doc.validate()?;
        
        let id = EntityId::from_string(&doc.id)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "id".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let content_type = ContentType::from_string(&doc.content_type)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "content_type".to_string(),
                message: format!("Invalid content type: {}", e),
            })?;
        
        let content_hash = ContentHash::from_string(&doc.content_hash);
        
        let file_path = doc.file_path
            .map(|path| FilePath::new(&path))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "file_path".to_string(),
                message: format!("Invalid file path: {}", e),
            })?;
        
        let created_at = Timestamp::from_string(&doc.created_at)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "created_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        let updated_at = Timestamp::from_string(&doc.updated_at)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "updated_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        let created_by = doc.created_by
            .map(|id| EntityId::from_string(&id))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "created_by".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let updated_by = doc.updated_by
            .map(|id| EntityId::from_string(&id))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "updated_by".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let deleted_at = doc.deleted_at
            .map(|ts| Timestamp::from_string(&ts))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "deleted_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        Ok(Document {
            id,
            title: doc.title,
            content: doc.content,
            content_type,
            content_hash,
            file_path,
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at,
            updated_at,
            created_by,
            updated_by,
            version: doc.version,
            is_deleted: doc.is_deleted,
            deleted_at,
        })
    }
}

/// Project structure for IndexedDB storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub document_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: u64,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    
    // Search fields
    pub search_name: String,
    pub search_description: String,
}

impl IndexedDbProject {
    /// Convert to JsValue for IndexedDB storage
    pub fn to_js_value(&self) -> Result<JsValue, SerializationError> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert project to JsValue: {}", e)
            })
    }
    
    /// Create from JsValue from IndexedDB
    pub fn from_js_value(value: &JsValue) -> Result<Self, SerializationError> {
        serde_wasm_bindgen::from_value(value.clone())
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert JsValue to project: {}", e)
            })
    }
    
    /// Validate project data consistency
    pub fn validate(&self) -> Result<(), SerializationError> {
        if self.id.is_empty() {
            return Err(SerializationError::InvalidEntityData {
                field: "id".to_string(),
                message: "Project ID cannot be empty".to_string(),
            });
        }
        
        if self.name.is_empty() {
            return Err(SerializationError::InvalidEntityData {
                field: "name".to_string(),
                message: "Project name cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

impl From<&Project> for IndexedDbProject {
    fn from(proj: &Project) -> Self {
        let search_description = proj.description
            .as_ref()
            .map(|d| d.to_lowercase())
            .unwrap_or_default();
        
        Self {
            id: proj.id.to_string(),
            name: proj.name.clone(),
            description: proj.description.clone(),
            document_ids: proj.document_ids.iter().map(|id| id.to_string()).collect(),
            created_at: proj.created_at.to_string(),
            updated_at: proj.updated_at.to_string(),
            created_by: proj.created_by.as_ref().map(|id| id.to_string()),
            updated_by: proj.updated_by.as_ref().map(|id| id.to_string()),
            version: proj.version,
            is_deleted: proj.is_deleted,
            deleted_at: proj.deleted_at.as_ref().map(|t| t.to_string()),
            search_name: proj.name.to_lowercase(),
            search_description,
        }
    }
}

impl TryFrom<IndexedDbProject> for Project {
    type Error = SerializationError;
    
    fn try_from(proj: IndexedDbProject) -> Result<Self, Self::Error> {
        proj.validate()?;
        
        let id = EntityId::from_string(&proj.id)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "id".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let document_ids = proj.document_ids.into_iter()
            .map(|id| EntityId::from_string(&id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "document_ids".to_string(),
                message: format!("Invalid document ID: {}", e),
            })?;
        
        let created_at = Timestamp::from_string(&proj.created_at)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "created_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        let updated_at = Timestamp::from_string(&proj.updated_at)
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "updated_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        let created_by = proj.created_by
            .map(|id| EntityId::from_string(&id))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "created_by".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let updated_by = proj.updated_by
            .map(|id| EntityId::from_string(&id))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "updated_by".to_string(),
                message: format!("Invalid entity ID: {}", e),
            })?;
        
        let deleted_at = proj.deleted_at
            .map(|ts| Timestamp::from_string(&ts))
            .transpose()
            .map_err(|e| SerializationError::InvalidEntityData {
                field: "deleted_at".to_string(),
                message: format!("Invalid timestamp: {}", e),
            })?;
        
        Ok(Project {
            id,
            name: proj.name,
            description: proj.description,
            document_ids,
            created_at,
            updated_at,
            created_by,
            updated_by,
            version: proj.version,
            is_deleted: proj.is_deleted,
            deleted_at,
        })
    }
}

/// Project-Document relationship for IndexedDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbProjectDocument {
    pub composite_key: String, // Format: "project_id|document_id"
    pub project_id: String,
    pub document_id: String,
    pub added_at: String,
}

impl IndexedDbProjectDocument {
    pub fn new(project_id: &EntityId, document_id: &EntityId) -> Self {
        let composite_key = format!("{}|{}", project_id.to_string(), document_id.to_string());
        
        Self {
            composite_key,
            project_id: project_id.to_string(),
            document_id: document_id.to_string(),
            added_at: Timestamp::now().to_string(),
        }
    }
    
    /// Convert to JsValue for IndexedDB storage
    pub fn to_js_value(&self) -> Result<JsValue, SerializationError> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert project document to JsValue: {}", e)
            })
    }
    
    /// Create from JsValue from IndexedDB
    pub fn from_js_value(value: &JsValue) -> Result<Self, SerializationError> {
        serde_wasm_bindgen::from_value(value.clone())
            .map_err(|e| SerializationError::JavaScriptConversion {
                message: format!("Failed to convert JsValue to project document: {}", e)
            })
    }
}

/// Batch operation wrapper for efficient IndexedDB operations
#[derive(Debug, Clone)]
pub struct BatchOperation<T> {
    pub operations: Vec<BatchOperationType<T>>,
}

#[derive(Debug, Clone)]
pub enum BatchOperationType<T> {
    Insert(T),
    Update(T),
    Delete(String), // ID to delete
}

impl<T> BatchOperation<T> {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
    
    pub fn insert(mut self, item: T) -> Self {
        self.operations.push(BatchOperationType::Insert(item));
        self
    }
    
    pub fn update(mut self, item: T) -> Self {
        self.operations.push(BatchOperationType::Update(item));
        self
    }
    
    pub fn delete(mut self, id: String) -> Self {
        self.operations.push(BatchOperationType::Delete(id));
        self
    }
    
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.operations.len()
    }
}

impl<T> Default for BatchOperation<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writemagic_shared::ContentType;
    
    #[test]
    fn test_document_serialization() {
        let doc = Document {
            id: EntityId::new(),
            title: "Test Document".to_string(),
            content: "This is a test document with some content.".to_string(),
            content_type: ContentType::Markdown,
            content_hash: ContentHash::new("test content"),
            file_path: None,
            word_count: 8,
            character_count: 42,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: None,
            updated_by: None,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        };
        
        let indexed_doc = IndexedDbDocument::from(&doc);
        assert_eq!(indexed_doc.title, doc.title);
        assert_eq!(indexed_doc.content, doc.content);
        assert!(!indexed_doc.search_tokens.is_empty());
        
        let converted_doc: Document = indexed_doc.try_into().unwrap();
        assert_eq!(converted_doc.id, doc.id);
        assert_eq!(converted_doc.title, doc.title);
    }
    
    #[test]
    fn test_project_serialization() {
        let proj = Project {
            id: EntityId::new(),
            name: "Test Project".to_string(),
            description: Some("A test project description".to_string()),
            document_ids: vec![EntityId::new(), EntityId::new()],
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: None,
            updated_by: None,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        };
        
        let indexed_proj = IndexedDbProject::from(&proj);
        assert_eq!(indexed_proj.name, proj.name);
        assert_eq!(indexed_proj.document_ids.len(), proj.document_ids.len());
        
        let converted_proj: Project = indexed_proj.try_into().unwrap();
        assert_eq!(converted_proj.id, proj.id);
        assert_eq!(converted_proj.name, proj.name);
        assert_eq!(converted_proj.document_ids.len(), proj.document_ids.len());
    }
    
    #[test]
    fn test_search_token_creation() {
        let tokens = IndexedDbDocument::create_search_tokens(
            "Test Document Title",
            "This is the content of the document with various words."
        );
        
        assert!(tokens.contains(&"test".to_string()));
        assert!(tokens.contains(&"document".to_string()));
        assert!(tokens.contains(&"title".to_string()));
        assert!(tokens.contains(&"content".to_string()));
        assert!(tokens.contains(&"various".to_string()));
        assert!(tokens.contains(&"words".to_string()));
        
        // Stop words should be filtered out
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"of".to_string()));
        assert!(!tokens.contains(&"with".to_string()));
    }
    
    #[test]
    fn test_project_document_relationship() {
        let project_id = EntityId::new();
        let document_id = EntityId::new();
        
        let rel = IndexedDbProjectDocument::new(&project_id, &document_id);
        assert_eq!(rel.project_id, project_id.to_string());
        assert_eq!(rel.document_id, document_id.to_string());
        assert_eq!(rel.composite_key, format!("{}|{}", project_id.to_string(), document_id.to_string()));
    }
    
    #[test]
    fn test_batch_operation() {
        let mut batch = BatchOperation::new();
        assert!(batch.is_empty());
        
        batch = batch.insert("test1".to_string())
                    .update("test2".to_string())
                    .delete("test3".to_string());
        
        assert_eq!(batch.len(), 3);
        assert!(!batch.is_empty());
    }
}