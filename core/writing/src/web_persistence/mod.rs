//! Web persistence layer using IndexedDB for WriteMagic core
//! 
//! This module provides IndexedDB-based repository implementations for web browsers,
//! enabling offline functionality and data persistence in Progressive Web Apps.

pub mod indexeddb_manager;
pub mod indexeddb_repositories;
pub mod schema;
pub mod serialization;
pub mod migrations;

pub use indexeddb_manager::{IndexedDbManager, IndexedDbConfig, DatabaseInfo};
pub use indexeddb_repositories::{IndexedDbDocumentRepository, IndexedDbProjectRepository};
pub use schema::{WRITEMAGIC_DB_NAME, WRITEMAGIC_DB_VERSION, ObjectStore, Index};
pub use serialization::{IndexedDbDocument, IndexedDbProject, SerializationError};
pub use migrations::{MigrationManager, Migration, MigrationError};

/// Web-specific error types for IndexedDB operations
#[derive(Debug, thiserror::Error)]
pub enum IndexedDbError {
    #[error("Database connection failed: {message}")]
    Connection { message: String },
    
    #[error("Transaction failed: {message}")]
    Transaction { message: String },
    
    #[error("Object store operation failed: {store} - {message}")]
    ObjectStore { store: String, message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    
    #[error("Migration error: {0}")]
    Migration(#[from] MigrationError),
    
    #[error("JavaScript error: {message}")]
    JavaScript { message: String },
    
    #[error("Unsupported browser feature: {feature}")]
    UnsupportedFeature { feature: String },
    
    #[error("Data integrity error: {message}")]
    DataIntegrity { message: String },
}

impl From<IndexedDbError> for writemagic_shared::WritemagicError {
    fn from(error: IndexedDbError) -> Self {
        match error {
            IndexedDbError::Connection { message } => 
                writemagic_shared::WritemagicError::database(&message),
            IndexedDbError::Transaction { message } => 
                writemagic_shared::WritemagicError::database(&message),
            IndexedDbError::ObjectStore { store: _, message } => 
                writemagic_shared::WritemagicError::database(&message),
            IndexedDbError::Serialization(e) => 
                writemagic_shared::WritemagicError::internal(&format!("Serialization error: {}", e)),
            IndexedDbError::Migration(e) => 
                writemagic_shared::WritemagicError::internal(&format!("Migration error: {}", e)),
            IndexedDbError::JavaScript { message } => 
                writemagic_shared::WritemagicError::internal(&message),
            IndexedDbError::UnsupportedFeature { feature } => 
                writemagic_shared::WritemagicError::configuration(&format!("Unsupported feature: {}", feature)),
            IndexedDbError::DataIntegrity { message } => 
                writemagic_shared::WritemagicError::internal(&message),
        }
    }
}

/// Result type for IndexedDB operations
pub type Result<T> = std::result::Result<T, IndexedDbError>;

/// Utility function to convert JavaScript errors to IndexedDbError
pub fn js_error_to_indexeddb_error(js_value: &wasm_bindgen::JsValue, context: &str) -> IndexedDbError {
    let message = js_value
        .as_string()
        .unwrap_or_else(|| format!("Unknown JavaScript error: {:?}", js_value));
    
    IndexedDbError::JavaScript { 
        message: format!("{}: {}", context, message)
    }
}

/// Utility function for safe IndexedDB feature detection
pub fn check_indexeddb_support() -> Result<()> {
    use wasm_bindgen::JsCast;
    
    let window = web_sys::window()
        .ok_or_else(|| IndexedDbError::UnsupportedFeature { 
            feature: "Window object not available".to_string() 
        })?;
    
    let indexed_db = window.indexed_db()
        .map_err(|e| js_error_to_indexeddb_error(&e, "IndexedDB access"))?
        .ok_or_else(|| IndexedDbError::UnsupportedFeature {
            feature: "IndexedDB not supported".to_string()
        })?;
    
    // Check for basic IndexedDB methods
    if js_sys::Reflect::has(&indexed_db, &"open".into()).unwrap_or(false) {
        Ok(())
    } else {
        Err(IndexedDbError::UnsupportedFeature {
            feature: "IndexedDB.open method not available".to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let js_error = IndexedDbError::Connection { 
            message: "Test connection error".to_string() 
        };
        let writemagic_error: writemagic_shared::WritemagicError = js_error.into();
        assert!(matches!(writemagic_error, writemagic_shared::WritemagicError::DatabaseError(_)));
    }
}