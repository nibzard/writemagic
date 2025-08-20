//! WASM-specific utilities and helpers for cross-platform compatibility

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// WASM-compatible storage interface
pub trait WasmStorage {
    /// Store a value by key
    fn set_item(&self, key: &str, value: &str) -> Result<(), String>;
    
    /// Retrieve a value by key
    fn get_item(&self, key: &str) -> Result<Option<String>, String>;
    
    /// Remove a value by key
    fn remove_item(&self, key: &str) -> Result<(), String>;
    
    /// Clear all stored values
    fn clear(&self) -> Result<(), String>;
    
    /// Get all keys
    fn keys(&self) -> Result<Vec<String>, String>;
}

/// In-memory storage implementation for WASM
#[derive(Debug, Default)]
pub struct InMemoryWasmStorage {
    storage: std::sync::RwLock<HashMap<String, String>>,
}

impl InMemoryWasmStorage {
    /// Create a new in-memory storage instance
    pub fn new() -> Self {
        Self {
            storage: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl WasmStorage for InMemoryWasmStorage {
    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        match self.storage.write() {
            Ok(mut storage) => {
                storage.insert(key.to_string(), value.to_string());
                Ok(())
            }
            Err(e) => Err(format!("Failed to acquire write lock: {}", e)),
        }
    }
    
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        match self.storage.read() {
            Ok(storage) => Ok(storage.get(key).cloned()),
            Err(e) => Err(format!("Failed to acquire read lock: {}", e)),
        }
    }
    
    fn remove_item(&self, key: &str) -> Result<(), String> {
        match self.storage.write() {
            Ok(mut storage) => {
                storage.remove(key);
                Ok(())
            }
            Err(e) => Err(format!("Failed to acquire write lock: {}", e)),
        }
    }
    
    fn clear(&self) -> Result<(), String> {
        match self.storage.write() {
            Ok(mut storage) => {
                storage.clear();
                Ok(())
            }
            Err(e) => Err(format!("Failed to acquire write lock: {}", e)),
        }
    }
    
    fn keys(&self) -> Result<Vec<String>, String> {
        match self.storage.read() {
            Ok(storage) => Ok(storage.keys().cloned().collect()),
            Err(e) => Err(format!("Failed to acquire read lock: {}", e)),
        }
    }
}

/// Configuration for WASM runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Whether to enable console logging
    pub enable_console_logging: bool,
    
    /// Whether to enable panic hooks
    pub enable_panic_hooks: bool,
    
    /// Storage type to use
    pub storage_type: WasmStorageType,
    
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    
    /// Additional configuration options
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            enable_console_logging: true,
            enable_panic_hooks: true,
            storage_type: WasmStorageType::Memory,
            max_memory_bytes: None,
            options: HashMap::new(),
        }
    }
}

/// Storage types available in WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmStorageType {
    /// In-memory storage (lost on refresh)
    Memory,
    /// Local storage (persisted)
    LocalStorage,
    /// Session storage (lost on tab close)
    SessionStorage,
    /// IndexedDB (advanced persistent storage)
    IndexedDB,
}

/// WASM runtime initialization result
pub struct WasmInitResult {
    /// Whether initialization was successful
    pub success: bool,
    /// Storage implementation
    pub storage: Box<dyn WasmStorage>,
    /// Any initialization messages
    pub messages: Vec<String>,
}

impl std::fmt::Debug for WasmInitResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmInitResult")
            .field("success", &self.success)
            .field("storage", &"Box<dyn WasmStorage>")
            .field("messages", &self.messages)
            .finish()
    }
}

/// Initialize WASM runtime with given configuration
pub fn init_wasm_runtime(config: WasmConfig) -> Result<WasmInitResult, String> {
    let mut messages = Vec::new();
    
    // Initialize panic hooks if enabled
    if config.enable_panic_hooks {
        #[cfg(feature = "console_error_panic_hook")]
        {
            console_error_panic_hook::set_once();
            messages.push("Panic hooks initialized".to_string());
        }
        #[cfg(not(feature = "console_error_panic_hook"))]
        {
            messages.push("Panic hooks requested but not available".to_string());
        }
    }
    
    // Initialize logging if enabled
    if config.enable_console_logging {
        messages.push("Console logging enabled".to_string());
    }
    
    // Create storage based on configuration
    let storage: Box<dyn WasmStorage> = match config.storage_type {
        WasmStorageType::Memory => {
            messages.push("Using in-memory storage".to_string());
            Box::new(InMemoryWasmStorage::new())
        }
        _ => {
            messages.push("Advanced storage types not yet implemented, falling back to memory".to_string());
            Box::new(InMemoryWasmStorage::new())
        }
    };
    
    Ok(WasmInitResult {
        success: true,
        storage,
        messages,
    })
}

/// Utility function to convert JavaScript errors to Rust strings
pub fn js_error_to_string(error: &wasm_bindgen::JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "Unknown JavaScript error".to_string())
}

/// Utility function to create a JavaScript error from a Rust string
pub fn string_to_js_error(message: &str) -> wasm_bindgen::JsValue {
    wasm_bindgen::JsValue::from_str(message)
}

/// Performance measurement utilities for WASM
pub struct WasmPerformance {
    start_time: f64,
    label: String,
}

impl WasmPerformance {
    /// Start measuring performance with a label
    pub fn start(label: impl Into<String>) -> Self {
        let start_time = js_sys::Date::now();
        Self {
            start_time,
            label: label.into(),
        }
    }
    
    /// End measurement and log the duration
    pub fn end(self) {
        let duration = js_sys::Date::now() - self.start_time;
        web_sys::console::log_2(
            &format!("Performance [{}]:", self.label).into(),
            &format!("{:.2}ms", duration).into(),
        );
    }
    
    /// Get elapsed time without ending measurement
    pub fn elapsed(&self) -> f64 {
        js_sys::Date::now() - self.start_time
    }
}

/// Macro for easy performance measurement
#[macro_export]
macro_rules! measure_wasm_perf {
    ($label:expr, $block:block) => {
        {
            let _perf = $crate::wasm_utils::WasmPerformance::start($label);
            let result = $block;
            _perf.end();
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_storage() {
        let storage = InMemoryWasmStorage::new();
        
        // Test set and get
        assert!(storage.set_item("key1", "value1").is_ok());
        assert_eq!(storage.get_item("key1").unwrap(), Some("value1".to_string()));
        
        // Test non-existent key
        assert_eq!(storage.get_item("nonexistent").unwrap(), None);
        
        // Test removal
        assert!(storage.remove_item("key1").is_ok());
        assert_eq!(storage.get_item("key1").unwrap(), None);
        
        // Test clear
        assert!(storage.set_item("key2", "value2").is_ok());
        assert!(storage.set_item("key3", "value3").is_ok());
        assert!(storage.clear().is_ok());
        assert_eq!(storage.get_item("key2").unwrap(), None);
        assert_eq!(storage.get_item("key3").unwrap(), None);
    }
    
    #[test]
    fn test_wasm_config_default() {
        let config = WasmConfig::default();
        assert!(config.enable_console_logging);
        assert!(config.enable_panic_hooks);
        assert!(matches!(config.storage_type, WasmStorageType::Memory));
    }
}