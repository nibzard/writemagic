//! IndexedDB schema definitions for WriteMagic
//! 
//! This module defines the database schema, object stores, and indexes
//! used by the IndexedDB persistence layer.

use serde::{Deserialize, Serialize};

/// WriteMagic database name in IndexedDB
pub const WRITEMAGIC_DB_NAME: &str = "WritemagicDB";

/// Current database version
pub const WRITEMAGIC_DB_VERSION: u32 = 1;

/// Object store names
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectStore {
    Documents,
    Projects,
    ProjectDocuments,
    Settings,
    Metadata,
}

impl ObjectStore {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectStore::Documents => "documents",
            ObjectStore::Projects => "projects",
            ObjectStore::ProjectDocuments => "project_documents",
            ObjectStore::Settings => "settings",
            ObjectStore::Metadata => "metadata",
        }
    }
    
    pub fn all() -> Vec<ObjectStore> {
        vec![
            ObjectStore::Documents,
            ObjectStore::Projects,
            ObjectStore::ProjectDocuments,
            ObjectStore::Settings,
            ObjectStore::Metadata,
        ]
    }
}

/// Index definitions for object stores
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
    pub multi_entry: bool,
}

impl Index {
    pub fn new(name: &str, key_path: &str, unique: bool) -> Self {
        Self {
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique,
            multi_entry: false,
        }
    }
    
    pub fn with_multi_entry(mut self, multi_entry: bool) -> Self {
        self.multi_entry = multi_entry;
        self
    }
}

/// Document store indexes
pub fn document_indexes() -> Vec<Index> {
    vec![
        Index::new("title", "title", false),
        Index::new("content_type", "content_type", false),
        Index::new("created_by", "created_by", false),
        Index::new("updated_by", "updated_by", false),
        Index::new("created_at", "created_at", false),
        Index::new("updated_at", "updated_at", false),
        Index::new("is_deleted", "is_deleted", false),
        Index::new("word_count", "word_count", false),
        Index::new("character_count", "character_count", false),
        Index::new("content_hash", "content_hash", true), // Unique content hash
    ]
}

/// Project store indexes
pub fn project_indexes() -> Vec<Index> {
    vec![
        Index::new("name", "name", false),
        Index::new("created_by", "created_by", false),
        Index::new("updated_by", "updated_by", false),
        Index::new("created_at", "created_at", false),
        Index::new("updated_at", "updated_at", false),
        Index::new("is_deleted", "is_deleted", false),
    ]
}

/// Project-Document relationship store indexes
pub fn project_document_indexes() -> Vec<Index> {
    vec![
        Index::new("project_id", "project_id", false),
        Index::new("document_id", "document_id", false),
        Index::new("composite", "composite_key", true), // Unique project-document pairs
    ]
}

/// Database schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub version: u32,
    pub stores: Vec<StoreConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub name: String,
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub indexes: Vec<IndexConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
    pub multi_entry: bool,
}

impl From<Index> for IndexConfig {
    fn from(index: Index) -> Self {
        IndexConfig {
            name: index.name,
            key_path: index.key_path,
            unique: index.unique,
            multi_entry: index.multi_entry,
        }
    }
}

impl From<IndexConfig> for Index {
    fn from(config: IndexConfig) -> Self {
        Index {
            name: config.name,
            key_path: config.key_path,
            unique: config.unique,
            multi_entry: config.multi_entry,
        }
    }
}

/// Get the complete database schema for the current version
pub fn get_schema() -> SchemaConfig {
    SchemaConfig {
        version: WRITEMAGIC_DB_VERSION,
        stores: vec![
            StoreConfig {
                name: ObjectStore::Documents.as_str().to_string(),
                key_path: Some("id".to_string()),
                auto_increment: false,
                indexes: document_indexes().into_iter().map(IndexConfig::from).collect(),
            },
            StoreConfig {
                name: ObjectStore::Projects.as_str().to_string(),
                key_path: Some("id".to_string()),
                auto_increment: false,
                indexes: project_indexes().into_iter().map(IndexConfig::from).collect(),
            },
            StoreConfig {
                name: ObjectStore::ProjectDocuments.as_str().to_string(),
                key_path: Some("composite_key".to_string()),
                auto_increment: false,
                indexes: project_document_indexes().into_iter().map(IndexConfig::from).collect(),
            },
            StoreConfig {
                name: ObjectStore::Settings.as_str().to_string(),
                key_path: Some("key".to_string()),
                auto_increment: false,
                indexes: vec![],
            },
            StoreConfig {
                name: ObjectStore::Metadata.as_str().to_string(),
                key_path: Some("key".to_string()),
                auto_increment: false,
                indexes: vec![],
            },
        ],
    }
}

/// Search configuration for full-text search simulation
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub min_word_length: usize,
    pub stop_words: Vec<String>,
    pub case_sensitive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            min_word_length: 3,
            stop_words: vec![
                "the".to_string(), "a".to_string(), "an".to_string(),
                "and".to_string(), "or".to_string(), "but".to_string(),
                "in".to_string(), "on".to_string(), "at".to_string(),
                "to".to_string(), "for".to_string(), "of".to_string(),
                "with".to_string(), "by".to_string(), "is".to_string(),
                "are".to_string(), "was".to_string(), "were".to_string(),
                "be".to_string(), "been".to_string(), "being".to_string(),
                "have".to_string(), "has".to_string(), "had".to_string(),
                "do".to_string(), "does".to_string(), "did".to_string(),
                "will".to_string(), "would".to_string(), "could".to_string(),
                "should".to_string(), "may".to_string(), "might".to_string(),
                "must".to_string(), "can".to_string(), "shall".to_string(),
            ],
            case_sensitive: false,
        }
    }
}

impl SearchConfig {
    /// Tokenize text for search indexing
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let text = if self.case_sensitive { text.to_string() } else { text.to_lowercase() };
        
        text.split_whitespace()
            .map(|word| {
                // Remove punctuation
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| {
                !word.is_empty() 
                && word.len() >= self.min_word_length
                && !self.stop_words.contains(word)
            })
            .collect()
    }
    
    /// Prepare search query
    pub fn prepare_query(&self, query: &str) -> Vec<String> {
        self.tokenize(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_store_names() {
        assert_eq!(ObjectStore::Documents.as_str(), "documents");
        assert_eq!(ObjectStore::Projects.as_str(), "projects");
        assert_eq!(ObjectStore::ProjectDocuments.as_str(), "project_documents");
    }
    
    #[test]
    fn test_schema_generation() {
        let schema = get_schema();
        assert_eq!(schema.version, WRITEMAGIC_DB_VERSION);
        assert!(schema.stores.len() > 0);
        
        // Check that documents store exists
        let docs_store = schema.stores.iter()
            .find(|s| s.name == ObjectStore::Documents.as_str())
            .expect("Documents store should exist");
        
        assert_eq!(docs_store.key_path.as_deref(), Some("id"));
        assert!(!docs_store.auto_increment);
        assert!(!docs_store.indexes.is_empty());
    }
    
    #[test]
    fn test_search_config_tokenization() {
        let config = SearchConfig::default();
        
        let tokens = config.tokenize("The quick brown fox jumps over the lazy dog!");
        assert!(!tokens.contains(&"the".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(!tokens.contains(&"over".to_string())); // Too short
        assert!(tokens.contains(&"jumps".to_string()));
        assert!(tokens.contains(&"lazy".to_string()));
    }
    
    #[test]
    fn test_index_creation() {
        let index = Index::new("test_index", "test_field", true)
            .with_multi_entry(true);
        
        assert_eq!(index.name, "test_index");
        assert_eq!(index.key_path, "test_field");
        assert!(index.unique);
        assert!(index.multi_entry);
    }
}