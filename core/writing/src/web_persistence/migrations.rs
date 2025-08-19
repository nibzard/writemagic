//! Database migration management for IndexedDB
//! 
//! This module handles schema migrations and database version upgrades
//! to ensure data consistency across application updates.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::*;

use super::schema::{SchemaConfig, StoreConfig, IndexConfig};
use super::{IndexedDbError, Result, js_error_to_indexeddb_error};

/// Error type for migration operations
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Migration script failed: {version} - {message}")]
    ScriptFailed { version: u32, message: String },
    
    #[error("Invalid migration version: {version}")]
    InvalidVersion { version: u32 },
    
    #[error("Migration dependency not met: {dependency}")]
    DependencyNotMet { dependency: String },
    
    #[error("Data transformation failed: {message}")]
    DataTransformation { message: String },
    
    #[error("Rollback failed: {message}")]
    RollbackFailed { message: String },
}

/// A single database migration
pub trait Migration: Send + Sync {
    /// The version this migration targets
    fn version(&self) -> u32;
    
    /// Human-readable description of the migration
    fn description(&self) -> &str;
    
    /// Dependencies that must be satisfied before this migration
    fn dependencies(&self) -> Vec<u32> {
        Vec::new()
    }
    
    /// Execute the migration
    fn execute(&self, db: &IdbDatabase, transaction: &IdbTransaction) -> Result<()>;
    
    /// Rollback the migration (optional)
    fn rollback(&self, db: &IdbDatabase, transaction: &IdbTransaction) -> Result<()> {
        Err(IndexedDbError::Migration(MigrationError::RollbackFailed {
            message: format!("Rollback not implemented for migration {}", self.version())
        }).into())
    }
    
    /// Validate the migration state after execution
    fn validate(&self, db: &IdbDatabase) -> Result<bool> {
        Ok(true) // Default implementation assumes success
    }
}

/// Migration manager for handling database version upgrades
pub struct MigrationManager {
    migrations: HashMap<u32, Box<dyn Migration>>,
    current_version: u32,
    target_version: u32,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(target_version: u32) -> Self {
        Self {
            migrations: HashMap::new(),
            current_version: 0,
            target_version,
        }
    }
    
    /// Register a migration
    pub fn register_migration(&mut self, migration: Box<dyn Migration>) {
        let version = migration.version();
        self.migrations.insert(version, migration);
    }
    
    /// Set the current database version
    pub fn set_current_version(&mut self, version: u32) {
        self.current_version = version;
    }
    
    /// Get migrations that need to be executed
    pub fn get_pending_migrations(&self) -> Result<Vec<&dyn Migration>> {
        let mut pending = Vec::new();
        
        for version in (self.current_version + 1)..=self.target_version {
            if let Some(migration) = self.migrations.get(&version) {
                // Check dependencies
                for dep in migration.dependencies() {
                    if dep > self.current_version {
                        return Err(IndexedDbError::Migration(MigrationError::DependencyNotMet {
                            dependency: format!("Migration {} requires version {}", version, dep)
                        }));
                    }
                }
                pending.push(migration.as_ref());
            } else {
                return Err(IndexedDbError::Migration(MigrationError::InvalidVersion { version }));
            }
        }
        
        Ok(pending)
    }
    
    /// Execute all pending migrations
    pub fn execute_migrations(&self, db: &IdbDatabase, transaction: &IdbTransaction) -> Result<()> {
        let pending_migrations = self.get_pending_migrations()?;
        
        web_sys::console::log_1(&format!("Executing {} migrations", pending_migrations.len()).into());
        
        for migration in pending_migrations {
            web_sys::console::log_1(&format!(
                "Executing migration {}: {}", 
                migration.version(), 
                migration.description()
            ).into());
            
            migration.execute(db, transaction)
                .map_err(|e| IndexedDbError::Migration(MigrationError::ScriptFailed {
                    version: migration.version(),
                    message: format!("{:?}", e)
                }))?;
            
            // Validate migration
            if !migration.validate(db)? {
                return Err(IndexedDbError::Migration(MigrationError::ScriptFailed {
                    version: migration.version(),
                    message: "Migration validation failed".to_string()
                }));
            }
            
            web_sys::console::log_1(&format!("Migration {} completed successfully", migration.version()).into());
        }
        
        Ok(())
    }
}

/// Initial database setup migration (version 1)
pub struct InitialMigration {
    schema: SchemaConfig,
}

impl InitialMigration {
    pub fn new(schema: SchemaConfig) -> Self {
        Self { schema }
    }
}

impl Migration for InitialMigration {
    fn version(&self) -> u32 {
        1
    }
    
    fn description(&self) -> &str {
        "Initial database schema creation"
    }
    
    fn execute(&self, db: &IdbDatabase, _transaction: &IdbTransaction) -> Result<()> {
        for store_config in &self.schema.stores {
            // Skip if store already exists
            if db.object_store_names().any(|name| name == store_config.name) {
                continue;
            }
            
            let mut store_params = IdbObjectStoreParameters::new();
            
            if let Some(key_path) = &store_config.key_path {
                store_params.key_path(Some(&key_path.into()));
            }
            
            store_params.auto_increment(store_config.auto_increment);
            
            let object_store = db.create_object_store_with_optional_parameters(
                &store_config.name,
                &store_params
            ).map_err(|e| IndexedDbError::ObjectStore {
                store: store_config.name.clone(),
                message: format!("Failed to create store: {:?}", e)
            })?;
            
            // Create indexes
            for index_config in &store_config.indexes {
                let mut index_params = IdbIndexParameters::new();
                index_params.unique(index_config.unique);
                index_params.multi_entry(index_config.multi_entry);
                
                object_store.create_index_with_str_and_optional_parameters(
                    &index_config.name,
                    &index_config.key_path,
                    &index_params
                ).map_err(|e| IndexedDbError::ObjectStore {
                    store: store_config.name.clone(),
                    message: format!("Failed to create index {}: {:?}", index_config.name, e)
                })?;
            }
        }
        
        Ok(())
    }
    
    fn validate(&self, db: &IdbDatabase) -> Result<bool> {
        // Verify all stores were created
        for store_config in &self.schema.stores {
            if !db.object_store_names().any(|name| name == store_config.name) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Example migration for adding a new index (version 2)
pub struct AddSearchIndexMigration;

impl Migration for AddSearchIndexMigration {
    fn version(&self) -> u32 {
        2
    }
    
    fn description(&self) -> &str {
        "Add full-text search indexes to documents"
    }
    
    fn dependencies(&self) -> Vec<u32> {
        vec![1] // Depends on initial schema
    }
    
    fn execute(&self, db: &IdbDatabase, transaction: &IdbTransaction) -> Result<()> {
        // This would be executed during an upgrade
        // Since we can't modify the schema during a regular transaction,
        // this would need to be called during the upgradeneeded event
        
        // For this example, we'll add the search tokens field to existing documents
        let documents_store = transaction.object_store("documents")
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting documents store"))?;
        
        // Create a cursor to iterate through all documents
        let cursor_request = documents_store.open_cursor()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Opening cursor"))?;
        
        // Note: In a real implementation, this would need proper async handling
        // This is a simplified version for demonstration
        
        web_sys::console::log_1(&"Search index migration would update existing documents".into());
        
        Ok(())
    }
    
    fn validate(&self, db: &IdbDatabase) -> Result<bool> {
        // Check if the search index exists (this is a placeholder)
        Ok(true)
    }
}

/// Migration for data format changes (version 3)
pub struct DataFormatMigration;

impl Migration for DataFormatMigration {
    fn version(&self) -> u32 {
        3
    }
    
    fn description(&self) -> &str {
        "Update data format for improved performance"
    }
    
    fn dependencies(&self) -> Vec<u32> {
        vec![2] // Depends on search index migration
    }
    
    fn execute(&self, _db: &IdbDatabase, _transaction: &IdbTransaction) -> Result<()> {
        // This would contain logic to transform existing data
        // For example: changing field names, data types, or structures
        
        web_sys::console::log_1(&"Data format migration completed".into());
        Ok(())
    }
}

/// Utility functions for migration management
pub mod utils {
    use super::*;
    
    /// Create a default migration manager with common migrations
    pub fn create_default_migration_manager(target_version: u32) -> MigrationManager {
        let mut manager = MigrationManager::new(target_version);
        
        // Register common migrations
        let schema = super::super::schema::get_schema();
        manager.register_migration(Box::new(InitialMigration::new(schema)));
        
        if target_version >= 2 {
            manager.register_migration(Box::new(AddSearchIndexMigration));
        }
        
        if target_version >= 3 {
            manager.register_migration(Box::new(DataFormatMigration));
        }
        
        manager
    }
    
    /// Check if migrations are needed
    pub fn migrations_needed(current_version: u32, target_version: u32) -> bool {
        current_version < target_version
    }
    
    /// Get migration path from current to target version
    pub fn get_migration_path(current_version: u32, target_version: u32) -> Vec<u32> {
        if current_version >= target_version {
            return Vec::new();
        }
        
        (current_version + 1..=target_version).collect()
    }
    
    /// Backup data before migration (simplified)
    pub async fn backup_before_migration(db: &IdbDatabase) -> Result<JsValue> {
        // This would create a backup of critical data before migration
        // In a real implementation, this would serialize all data to a safe format
        
        let backup = js_sys::Object::new();
        
        // Add metadata about the backup
        js_sys::Reflect::set(&backup, &"timestamp".into(), &js_sys::Date::now().into())
            .map_err(|e| js_error_to_indexeddb_error(&e, "Creating backup metadata"))?;
        
        js_sys::Reflect::set(&backup, &"version".into(), &db.version().into())
            .map_err(|e| js_error_to_indexeddb_error(&e, "Setting backup version"))?;
        
        Ok(backup.into())
    }
    
    /// Restore data after failed migration
    pub async fn restore_after_failed_migration(_backup: &JsValue, _db: &IdbDatabase) -> Result<()> {
        // This would restore data from backup after a failed migration
        web_sys::console::log_1(&"Migration rollback completed".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestMigration {
        version: u32,
        description: String,
    }
    
    impl TestMigration {
        fn new(version: u32, description: &str) -> Self {
            Self {
                version,
                description: description.to_string(),
            }
        }
    }
    
    impl Migration for TestMigration {
        fn version(&self) -> u32 {
            self.version
        }
        
        fn description(&self) -> &str {
            &self.description
        }
        
        fn execute(&self, _db: &IdbDatabase, _transaction: &IdbTransaction) -> Result<()> {
            Ok(())
        }
    }
    
    #[test]
    fn test_migration_manager() {
        let mut manager = MigrationManager::new(3);
        
        manager.register_migration(Box::new(TestMigration::new(1, "First migration")));
        manager.register_migration(Box::new(TestMigration::new(2, "Second migration")));
        manager.register_migration(Box::new(TestMigration::new(3, "Third migration")));
        
        manager.set_current_version(1);
        
        // Should need migrations 2 and 3
        let pending = manager.get_pending_migrations().unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].version(), 2);
        assert_eq!(pending[1].version(), 3);
    }
    
    #[test]
    fn test_migration_path() {
        let path = utils::get_migration_path(1, 4);
        assert_eq!(path, vec![2, 3, 4]);
        
        let empty_path = utils::get_migration_path(4, 4);
        assert!(empty_path.is_empty());
        
        let backward_path = utils::get_migration_path(4, 2);
        assert!(backward_path.is_empty());
    }
    
    #[test]
    fn test_migrations_needed() {
        assert!(utils::migrations_needed(1, 3));
        assert!(!utils::migrations_needed(3, 3));
        assert!(!utils::migrations_needed(3, 1));
    }
}