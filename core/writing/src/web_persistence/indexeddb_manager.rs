//! IndexedDB connection and transaction management
//! 
//! This module provides low-level IndexedDB operations including database
//! initialization, transaction management, and connection handling.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;
use futures::Future;
use std::pin::Pin;
use js_sys::{Array, Object, Reflect};

use super::schema::{SchemaConfig, ObjectStore, get_schema, WRITEMAGIC_DB_NAME, WRITEMAGIC_DB_VERSION};
use super::{IndexedDbError, Result, js_error_to_indexeddb_error};

/// Configuration for IndexedDB manager
#[derive(Debug, Clone)]
pub struct IndexedDbConfig {
    pub database_name: String,
    pub version: u32,
    pub timeout_ms: u32,
    pub enable_logging: bool,
    pub auto_migrate: bool,
}

impl Default for IndexedDbConfig {
    fn default() -> Self {
        Self {
            database_name: WRITEMAGIC_DB_NAME.to_string(),
            version: WRITEMAGIC_DB_VERSION,
            timeout_ms: 30000, // 30 seconds
            enable_logging: true,
            auto_migrate: true,
        }
    }
}

/// Information about the current database state
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub name: String,
    pub version: u32,
    pub size_estimate: Option<u64>,
    pub object_stores: Vec<String>,
}

/// IndexedDB manager for handling database connections and operations
pub struct IndexedDbManager {
    config: IndexedDbConfig,
    db: Option<IdbDatabase>,
}

impl IndexedDbManager {
    /// Create a new IndexedDB manager
    pub fn new(config: IndexedDbConfig) -> Self {
        Self {
            config,
            db: None,
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(IndexedDbConfig::default())
    }
    
    /// Initialize the database connection
    pub async fn initialize(&mut self) -> Result<()> {
        if self.config.enable_logging {
            web_sys::console::log_1(&format!("Initializing IndexedDB: {}", self.config.database_name).into());
        }
        
        // Check IndexedDB support
        super::check_indexeddb_support()?;
        
        // Open database connection
        let db = self.open_database().await?;
        self.db = Some(db);
        
        if self.config.enable_logging {
            web_sys::console::log_1(&"IndexedDB initialized successfully".into());
        }
        
        Ok(())
    }
    
    /// Open the IndexedDB database
    async fn open_database(&self) -> Result<IdbDatabase> {
        let window = web_sys::window()
            .ok_or_else(|| IndexedDbError::UnsupportedFeature { 
                feature: "Window object".to_string() 
            })?;
        
        let indexed_db = window.indexed_db()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting IndexedDB"))?
            .ok_or_else(|| IndexedDbError::UnsupportedFeature {
                feature: "IndexedDB".to_string()
            })?;
        
        // Create database open request
        let request = indexed_db.open_with_u32(&self.config.database_name, self.config.version)
            .map_err(|e| js_error_to_indexeddb_error(&e, "Opening database"))?;
        
        // Set up upgrade handler
        let schema = get_schema();
        let upgrade_callback = Closure::wrap(Box::new(move |event: Event| {
            if let Ok(target) = event.target().unwrap().dyn_into::<IdbOpenDbRequest>() {
                if let Ok(db) = target.result().unwrap().dyn_into::<IdbDatabase>() {
                    if let Err(e) = Self::handle_upgrade(&db, &schema) {
                        web_sys::console::error_2(&"Database upgrade failed:".into(), &format!("{:?}", e).into());
                    }
                }
            }
        }) as Box<dyn FnMut(Event)>);
        
        request.set_onupgradeneeded(Some(upgrade_callback.as_ref().unchecked_ref()));
        upgrade_callback.forget(); // Keep callback alive
        
        // Wait for database to open
        let result = JsFuture::from(request)
            .await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Database open"))?;
        
        let db = result.dyn_into::<IdbDatabase>()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Converting to IdbDatabase"))?;
        
        Ok(db)
    }
    
    /// Handle database schema upgrade
    fn handle_upgrade(db: &IdbDatabase, schema: &SchemaConfig) -> Result<()> {
        web_sys::console::log_1(&format!("Upgrading database to version {}", schema.version).into());
        
        for store_config in &schema.stores {
            // Create object store if it doesn't exist
            if !db.object_store_names().any(|name| name == store_config.name) {
                let mut store_params = IdbObjectStoreParameters::new();
                
                if let Some(key_path) = &store_config.key_path {
                    store_params.key_path(Some(&key_path.into()));
                }
                
                store_params.auto_increment(store_config.auto_increment);
                
                let object_store = db.create_object_store_with_optional_parameters(
                    &store_config.name,
                    &store_params
                ).map_err(|e| js_error_to_indexeddb_error(&e, &format!("Creating object store {}", store_config.name)))?;
                
                // Create indexes
                for index_config in &store_config.indexes {
                    if !object_store.index_names().any(|name| name == index_config.name) {
                        let mut index_params = IdbIndexParameters::new();
                        index_params.unique(index_config.unique);
                        index_params.multi_entry(index_config.multi_entry);
                        
                        object_store.create_index_with_str_and_optional_parameters(
                            &index_config.name,
                            &index_config.key_path,
                            &index_params
                        ).map_err(|e| js_error_to_indexeddb_error(&e, &format!("Creating index {}", index_config.name)))?;
                    }
                }
                
                web_sys::console::log_1(&format!("Created object store: {}", store_config.name).into());
            }
        }
        
        Ok(())
    }
    
    /// Get database information
    pub async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let db = self.get_database()?;
        
        // Get storage estimate if available
        let size_estimate = self.get_storage_estimate().await.ok();
        
        let object_stores = db.object_store_names()
            .iter()
            .map(|name| name.as_string().unwrap_or_default())
            .collect();
        
        Ok(DatabaseInfo {
            name: db.name(),
            version: db.version(),
            size_estimate,
            object_stores,
        })
    }
    
    /// Get storage estimate from Storage API
    async fn get_storage_estimate(&self) -> Result<u64> {
        let window = web_sys::window()
            .ok_or_else(|| IndexedDbError::UnsupportedFeature {
                feature: "Window object".to_string()
            })?;
        
        let navigator = window.navigator();
        
        // Check if Storage API is available
        if let Ok(storage) = Reflect::get(&navigator, &"storage".into()) {
            if !storage.is_undefined() && !storage.is_null() {
                // Call navigator.storage.estimate()
                if let Ok(estimate_method) = Reflect::get(&storage, &"estimate".into()) {
                    if estimate_method.is_function() {
                        if let Ok(promise) = Reflect::apply(&estimate_method, &storage, &Array::new()) {
                            if let Ok(result) = JsFuture::from(promise.dyn_into().unwrap()).await {
                                if let Ok(usage) = Reflect::get(&result, &"usage".into()) {
                                    if let Some(usage_num) = usage.as_f64() {
                                        return Ok(usage_num as u64);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(IndexedDbError::UnsupportedFeature {
            feature: "Storage API".to_string()
        })
    }
    
    /// Begin a transaction with specified stores and mode
    pub fn transaction(&self, stores: &[ObjectStore], mode: IdbTransactionMode) -> Result<IdbTransaction> {
        let db = self.get_database()?;
        
        let store_names = Array::new();
        for store in stores {
            store_names.push(&store.as_str().into());
        }
        
        let transaction = db.transaction_with_str_sequence_and_mode(&store_names, mode)
            .map_err(|e| js_error_to_indexeddb_error(&e, "Creating transaction"))?;
        
        Ok(transaction)
    }
    
    /// Begin a read-only transaction
    pub fn read_transaction(&self, stores: &[ObjectStore]) -> Result<IdbTransaction> {
        self.transaction(stores, IdbTransactionMode::Readonly)
    }
    
    /// Begin a read-write transaction
    pub fn write_transaction(&self, stores: &[ObjectStore]) -> Result<IdbTransaction> {
        self.transaction(stores, IdbTransactionMode::Readwrite)
    }
    
    /// Get an object store from a transaction
    pub fn object_store(&self, transaction: &IdbTransaction, store: ObjectStore) -> Result<IdbObjectStore> {
        transaction.object_store(store.as_str())
            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Getting object store {}", store.as_str())))
    }
    
    /// Execute a transaction and wait for completion
    pub async fn execute_transaction(&self, transaction: IdbTransaction) -> Result<()> {
        let complete_promise = JsFuture::from(transaction);
        complete_promise.await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Transaction execution"))?;
        
        Ok(())
    }
    
    /// Get the current database connection
    fn get_database(&self) -> Result<&IdbDatabase> {
        self.db.as_ref()
            .ok_or_else(|| IndexedDbError::Connection {
                message: "Database not initialized".to_string()
            })
    }
    
    /// Close the database connection
    pub fn close(&mut self) {
        if let Some(db) = &self.db {
            db.close();
            if self.config.enable_logging {
                web_sys::console::log_1(&"IndexedDB connection closed".into());
            }
        }
        self.db = None;
    }
    
    /// Check if database is connected
    pub fn is_connected(&self) -> bool {
        self.db.is_some()
    }
    
    /// Clear all data from specified object stores
    pub async fn clear_stores(&self, stores: &[ObjectStore]) -> Result<()> {
        let transaction = self.write_transaction(stores)?;
        
        for store in stores {
            let object_store = self.object_store(&transaction, store.clone())?;
            let clear_request = object_store.clear()
                .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Clearing store {}", store.as_str())))?;
            
            JsFuture::from(clear_request).await
                .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Clear operation for {}", store.as_str())))?;
        }
        
        self.execute_transaction(transaction).await?;
        
        if self.config.enable_logging {
            web_sys::console::log_1(&format!("Cleared {} object stores", stores.len()).into());
        }
        
        Ok(())
    }
    
    /// Delete the entire database
    pub async fn delete_database(&self) -> Result<()> {
        let window = web_sys::window()
            .ok_or_else(|| IndexedDbError::UnsupportedFeature {
                feature: "Window object".to_string()
            })?;
        
        let indexed_db = window.indexed_db()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting IndexedDB"))?
            .ok_or_else(|| IndexedDbError::UnsupportedFeature {
                feature: "IndexedDB".to_string()
            })?;
        
        let delete_request = indexed_db.delete_database(&self.config.database_name)
            .map_err(|e| js_error_to_indexeddb_error(&e, "Deleting database"))?;
        
        JsFuture::from(delete_request).await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Database deletion"))?;
        
        if self.config.enable_logging {
            web_sys::console::log_1(&format!("Deleted database: {}", self.config.database_name).into());
        }
        
        Ok(())
    }
    
    /// Perform a backup of all data to a JavaScript object
    pub async fn backup_data(&self) -> Result<JsValue> {
        let backup = Object::new();
        let all_stores = ObjectStore::all();
        let transaction = self.read_transaction(&all_stores)?;
        
        for store in &all_stores {
            let object_store = self.object_store(&transaction, store.clone())?;
            let get_all_request = object_store.get_all()
                .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Backing up store {}", store.as_str())))?;
            
            let store_data = JsFuture::from(get_all_request).await
                .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Backup operation for {}", store.as_str())))?;
            
            Reflect::set(&backup, &store.as_str().into(), &store_data)
                .map_err(|e| js_error_to_indexeddb_error(&e, "Setting backup data"))?;
        }
        
        Ok(backup.into())
    }
    
    /// Restore data from a backup object
    pub async fn restore_data(&self, backup_data: &JsValue) -> Result<()> {
        let all_stores = ObjectStore::all();
        
        // Clear existing data first
        self.clear_stores(&all_stores).await?;
        
        let transaction = self.write_transaction(&all_stores)?;
        
        for store in &all_stores {
            if let Ok(store_data) = Reflect::get(backup_data, &store.as_str().into()) {
                if store_data.is_array() {
                    let array = Array::from(&store_data);
                    let object_store = self.object_store(&transaction, store.clone())?;
                    
                    for i in 0..array.length() {
                        let item = array.get(i);
                        let add_request = object_store.add(&item)
                            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Restoring item to {}", store.as_str())))?;
                        
                        JsFuture::from(add_request).await
                            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Restore operation for {}", store.as_str())))?;
                    }
                }
            }
        }
        
        self.execute_transaction(transaction).await?;
        
        if self.config.enable_logging {
            web_sys::console::log_1(&"Data restored from backup".into());
        }
        
        Ok(())
    }
}

impl Drop for IndexedDbManager {
    fn drop(&mut self) {
        self.close();
    }
}

/// Utility functions for IndexedDB operations
pub mod utils {
    use super::*;
    
    /// Convert IdbCursorDirection to string
    pub fn cursor_direction_to_string(direction: IdbCursorDirection) -> &'static str {
        match direction {
            IdbCursorDirection::Next => "next",
            IdbCursorDirection::Nextunique => "nextunique",
            IdbCursorDirection::Prev => "prev",
            IdbCursorDirection::Prevunique => "prevunique",
        }
    }
    
    /// Create an IdbKeyRange for queries
    pub fn create_key_range_bound(lower: &JsValue, upper: &JsValue, lower_open: bool, upper_open: bool) -> Result<IdbKeyRange> {
        IdbKeyRange::bound_with_lower_open_and_upper_open(lower, upper, lower_open, upper_open)
            .map_err(|e| js_error_to_indexeddb_error(&e, "Creating key range"))
    }
    
    /// Create a prefix key range for string searches
    pub fn create_prefix_range(prefix: &str) -> Result<IdbKeyRange> {
        let lower_bound = JsValue::from_str(prefix);
        let upper_bound = JsValue::from_str(&format!("{}\u{10FFFF}", prefix)); // Unicode max char
        
        create_key_range_bound(&lower_bound, &upper_bound, false, true)
    }
    
    /// Convert JavaScript timestamp to Rust timestamp string
    pub fn js_timestamp_to_string(js_timestamp: f64) -> String {
        use chrono::{DateTime, Utc, TimeZone};
        let datetime = Utc.timestamp_millis(js_timestamp as i64);
        datetime.to_rfc3339()
    }
    
    /// Convert Rust timestamp string to JavaScript timestamp
    pub fn timestamp_string_to_js(timestamp: &str) -> Result<f64> {
        use chrono::{DateTime, Utc};
        
        DateTime::parse_from_rfc3339(timestamp)
            .map(|dt| dt.with_timezone(&Utc).timestamp_millis() as f64)
            .map_err(|e| IndexedDbError::DataIntegrity {
                message: format!("Invalid timestamp format: {}", e)
            })
    }
}