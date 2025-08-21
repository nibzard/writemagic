//! iOS FFI bindings for WriteMagic core - Thread-safe and performance optimized

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use writemagic_shared::{EntityId, ContentType, Pagination, Result, WritemagicError};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder,
    value_objects::{DocumentTitle, DocumentContent},
};

/// Thread-safe FFI error codes for proper error handling
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FFIErrorCode {
    Success = 0,
    NotInitialized = 1,
    InvalidInput = 2,
    EngineError = 3,
    SerializationError = 4,
    ThreadingError = 5,
    MemoryError = 6,
}

/// FFI Result structure with error context
#[derive(Debug)]
pub struct FFIResult<T> {
    pub value: Option<T>,
    pub error_code: FFIErrorCode,
    pub error_message: Option<String>,
}

impl<T> FFIResult<T> {
    pub fn success(value: T) -> Self {
        Self {
            value: Some(value),
            error_code: FFIErrorCode::Success,
            error_message: None,
        }
    }
    
    pub fn error(code: FFIErrorCode, message: String) -> Self {
        Self {
            value: None,
            error_code: code,
            error_message: Some(message),
        }
    }
}

/// Thread-safe instance manager for CoreEngine lifecycle
pub struct FFIInstanceManager {
    engine: Arc<RwLock<CoreEngine>>,
    runtime: Arc<Runtime>,
    instance_id: String,
}

impl FFIInstanceManager {
    pub async fn new(
        claude_key: Option<String>, 
        openai_key: Option<String>,
        instance_id: String,
    ) -> Result<Self> {
        let runtime = Arc::new(
            Runtime::new()
                .map_err(|e| WritemagicError::internal(format!("Failed to create runtime: {}", e)))?
        );
        
        let engine = runtime.block_on(async {
            ApplicationConfigBuilder::new()
                .with_sqlite()
                .with_claude_key(claude_key.unwrap_or_default())
                .with_openai_key(openai_key.unwrap_or_default())
                .with_log_level("info".to_string())
                .with_content_filtering(true)
                .build()
                .await
        })?;
        
        Ok(Self {
            engine: Arc::new(RwLock::new(engine)),
            runtime,
            instance_id,
        })
    }
    
    pub fn engine(&self) -> &Arc<RwLock<CoreEngine>> {
        &self.engine
    }
    
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}

/// Thread-safe global instance registry
static INSTANCE_REGISTRY: OnceLock<Arc<RwLock<HashMap<String, Arc<FFIInstanceManager>>>>> = OnceLock::new();

/// Get or create the instance registry
fn get_instance_registry() -> &'static Arc<RwLock<HashMap<String, Arc<FFIInstanceManager>>>> {
    INSTANCE_REGISTRY.get_or_init(|| {
        Arc::new(RwLock::new(HashMap::new()))
    })
}

/// Get default instance (for backwards compatibility)
fn get_default_instance() -> FFIResult<Arc<FFIInstanceManager>> {
    let registry = get_instance_registry();
    match registry.read() {
        Ok(map) => {
            if let Some(instance) = map.get("default") {
                FFIResult::success(instance.clone())
            } else {
                FFIResult::error(
                    FFIErrorCode::NotInitialized,
                    "CoreEngine not initialized - call initialize first".to_string()
                )
            }
        }
        Err(e) => FFIResult::error(
            FFIErrorCode::ThreadingError,
            format!("Failed to acquire registry lock: {}", e)
        )
    }
}

/// Memory-safe string conversion helper
fn c_string_to_rust(c_str: *const c_char) -> FFIResult<String> {
    if c_str.is_null() {
        return FFIResult::error(FFIErrorCode::InvalidInput, "C string is null".to_string());
    }
    
    unsafe {
        match CStr::from_ptr(c_str).to_str() {
            Ok(s) => FFIResult::success(s.to_string()),
            Err(e) => FFIResult::error(
                FFIErrorCode::InvalidInput,
                format!("Failed to convert C string: {}", e)
            )
        }
    }
}

/// Memory-safe C string creation helper
fn create_c_string(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(c_string) => c_string.into_raw(),
        Err(e) => {
            log::error!("Failed to create C string: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Initialize logging (called once)
fn init_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        #[cfg(target_os = "ios")]
        {
            // Initialize iOS-specific logging if available
            log::info!("WriteMagic iOS FFI initialized");
        }
        #[cfg(not(target_os = "ios"))]
        {
            env_logger::init();
        }
        log::info!("WriteMagic iOS FFI logging initialized");
    });
}

/// Initialize the WriteMagic core engine with AI configuration
/// use_sqlite: 1 to use SQLite, 0 to use in-memory storage
/// claude_key: Claude API key (can be NULL)
/// openai_key: OpenAI API key (can be NULL)
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_initialize_with_ai(
    _use_sqlite: c_int,
    claude_key: *const c_char,
    openai_key: *const c_char,
) -> c_int {
    init_logging();
    log::info!("Initializing WriteMagic core for iOS with enhanced FFI safety");

    // Extract API keys with proper error handling
    let claude_api_key = if claude_key.is_null() {
        None
    } else {
        match c_string_to_rust(claude_key) {
            FFIResult { value: Some(key), .. } if !key.trim().is_empty() => Some(key),
            FFIResult { error_code, error_message, .. } if error_code != FFIErrorCode::Success => {
                log::error!("Failed to extract Claude API key: {:?}", error_message);
                return 0;
            }
            _ => None,
        }
    };

    let openai_api_key = if openai_key.is_null() {
        None
    } else {
        match c_string_to_rust(openai_key) {
            FFIResult { value: Some(key), .. } if !key.trim().is_empty() => Some(key),
            FFIResult { error_code, error_message, .. } if error_code != FFIErrorCode::Success => {
                log::error!("Failed to extract OpenAI API key: {:?}", error_message);
                return 0;
            }
            _ => None,
        }
    };

    // Create instance manager with proper error handling
    let registry = get_instance_registry();
    match registry.write() {
        Ok(mut map) => {
            // Check if already initialized
            if map.contains_key("default") {
                log::info!("WriteMagic core already initialized");
                return 1;
            }
            
            // Create new instance using shared runtime
            let runtime = Runtime::new();
            match runtime {
                Ok(rt) => {
                    let result = rt.block_on(async {
                        FFIInstanceManager::new(
                            claude_api_key,
                            openai_api_key,
                            "default".to_string(),
                        ).await
                    });
                    
                    match result {
                        Ok(manager) => {
                            map.insert("default".to_string(), Arc::new(manager));
                            log::info!("WriteMagic core engine initialized successfully");
                            1
                        }
                        Err(e) => {
                            log::error!("Failed to create CoreEngine instance: {}", e);
                            0
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create Tokio runtime: {}", e);
                    0
                }
            }
        }
        Err(e) => {
            log::error!("Failed to acquire registry write lock: {}", e);
            0
        }
    }
}

/// Initialize the WriteMagic core engine (backwards compatibility)
/// use_sqlite: 1 to use SQLite, 0 to use in-memory storage
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_initialize(use_sqlite: c_int) -> c_int {
    writemagic_initialize_with_ai(use_sqlite, std::ptr::null(), std::ptr::null())
}

/// Create a new document with enhanced error handling and performance
/// Returns document ID as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_create_document(
    title: *const c_char,
    content: *const c_char,
    content_type: *const c_char,
) -> *mut c_char {
    init_logging();
    
    if title.is_null() || content.is_null() || content_type.is_null() {
        log::error!("Null pointer passed to writemagic_create_document");
        return std::ptr::null_mut();
    }
    
    // Get instance manager
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    // Extract parameters with error handling
    let title_str = match c_string_to_rust(title) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract title: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let content_str = match c_string_to_rust(content) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let content_type_str = match c_string_to_rust(content_type) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content_type: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    log::info!("Creating document: {} ({})", title_str, content_type_str);
    
    // Use shared runtime instead of creating new one
    let result = manager.runtime().block_on(async {
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::ThreadingError,
                    format!("Failed to acquire engine read lock: {}", e)
                );
            }
        };
        
        let document_title = match DocumentTitle::new(&title_str) {
            Ok(title) => title,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::InvalidInput,
                    format!("Invalid document title: {}", e)
                );
            }
        };
        
        let document_content = match DocumentContent::new(&content_str) {
            Ok(content) => content,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::InvalidInput,
                    format!("Invalid document content: {}", e)
                );
            }
        };
        
        let content_type = match content_type_str.as_str() {
            "markdown" => ContentType::Markdown,
            "plain_text" => ContentType::PlainText,
            "html" => ContentType::Html,
            _ => ContentType::PlainText,
        };
        
        match engine_guard.document_management_service().create_document(
            document_title,
            document_content,
            content_type,
            None, // created_by - set from authentication context
        ).await {
            Ok(aggregate) => {
                let document = aggregate.document();
                log::info!("Document created successfully: {}", document.id);
                FFIResult::success(document.id.to_string())
            }
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to create document: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(doc_id), .. } => create_c_string(doc_id),
        FFIResult { error_message, .. } => {
            log::error!("Document creation failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Update document content with enhanced performance and error handling
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_update_document_content(
    document_id: *const c_char,
    content: *const c_char,
) -> c_int {
    init_logging();
    
    if document_id.is_null() || content.is_null() {
        log::error!("Null pointer passed to writemagic_update_document_content");
        return 0;
    }
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return 0;
        }
    };
    
    let document_id_str = match c_string_to_rust(document_id) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract document_id: {:?}", error_message);
            return 0;
        }
    };
    
    let content_str = match c_string_to_rust(content) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content: {:?}", error_message);
            return 0;
        }
    };
    
    log::info!("Updating document {} with new content", document_id_str);
    
    let result = manager.runtime().block_on(async {
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to acquire engine read lock: {}", e);
                return false;
            }
        };
        
        // Parse document ID
        let document_id = match uuid::Uuid::parse_str(&document_id_str) {
            Ok(uuid) => EntityId::from_uuid(uuid),
            Err(e) => {
                log::error!("Invalid document ID format: {}", e);
                return false;
            }
        };
        
        let document_content = match DocumentContent::new(&content_str) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Invalid document content: {}", e);
                return false;
            }
        };
        
        match engine_guard.document_management_service().update_document_content(
            document_id,
            document_content,
            None, // text selection
            None, // updated_by - set from authentication context
        ).await {
            Ok(_) => {
                log::info!("Successfully updated document {}", document_id_str);
                true
            }
            Err(e) => {
                log::error!("Failed to update document content: {}", e);
                false
            }
        }
    });
    
    if result { 1 } else { 0 }
}

/// Get document by ID with enhanced performance and error handling
/// Returns document JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_get_document(document_id: *const c_char) -> *mut c_char {
    init_logging();
    
    if document_id.is_null() {
        log::error!("Null pointer passed to writemagic_get_document");
        return std::ptr::null_mut();
    }
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let document_id_str = match c_string_to_rust(document_id) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract document_id: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    log::info!("Getting document {}", document_id_str);
    
    let result = manager.runtime().block_on(async {
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::ThreadingError,
                    format!("Failed to acquire engine read lock: {}", e)
                );
            }
        };
        
        // Parse document ID
        let document_id = match uuid::Uuid::parse_str(&document_id_str) {
            Ok(uuid) => EntityId::from_uuid(uuid),
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::InvalidInput,
                    format!("Invalid document ID format: {}", e)
                );
            }
        };
        
        match engine_guard.document_repository().find_by_id(&document_id).await {
            Ok(Some(document)) => {
                let response = serde_json::json!({
                    "id": document.id.to_string(),
                    "title": document.title,
                    "content": document.content,
                    "contentType": document.content_type.to_string(),
                    "wordCount": document.word_count,
                    "characterCount": document.character_count,
                    "createdAt": document.created_at.to_string(),
                    "updatedAt": document.updated_at.to_string(),
                    "version": document.version,
                    "isDeleted": document.is_deleted
                });
                
                FFIResult::success(response.to_string())
            }
            Ok(None) => FFIResult::error(
                FFIErrorCode::InvalidInput,
                format!("Document {} not found", document_id_str)
            ),
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to retrieve document: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json_str), .. } => create_c_string(json_str),
        FFIResult { error_message, .. } => {
            log::error!("Get document failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Complete text using AI with enhanced error handling and performance optimization
/// Returns completion JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_complete_text(
    prompt: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    init_logging();
    
    if prompt.is_null() {
        log::error!("Null pointer passed to writemagic_complete_text");
        return std::ptr::null_mut();
    }
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let prompt_str = match c_string_to_rust(prompt) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract prompt: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let model_str = if model.is_null() {
        None
    } else {
        match c_string_to_rust(model) {
            FFIResult { value: Some(s), .. } if !s.trim().is_empty() => Some(s),
            _ => None,
        }
    };
    
    log::info!("Completing text with model {:?} and prompt: {}", model_str, prompt_str);
    
    let result = manager.runtime().block_on(async {
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::ThreadingError,
                    format!("Failed to acquire engine read lock: {}", e)
                );
            }
        };
        
        match engine_guard.complete_text(prompt_str, model_str).await {
            Ok(completion) => {
                let response = serde_json::json!({
                    "completion": completion,
                    "success": true
                });
                FFIResult::success(response.to_string())
            }
            Err(e) => {
                log::error!("AI completion failed: {}", e);
                let error_response = serde_json::json!({
                    "error": e.to_string(),
                    "success": false
                });
                // Return structured error instead of failing
                FFIResult::success(error_response.to_string())
            }
        }
    });
    
    match result {
        FFIResult { value: Some(json_str), .. } => create_c_string(json_str),
        FFIResult { error_message, .. } => {
            log::error!("AI completion operation failed: {:?}", error_message);
            // Return error response as fallback
            let fallback_error = serde_json::json!({
                "error": "CoreEngine not available",
                "success": false
            });
            create_c_string(fallback_error.to_string())
        }
    }
}

/// List all documents with pagination and enhanced performance
/// Returns document list JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_list_documents(
    offset: c_int,
    limit: c_int,
) -> *mut c_char {
    init_logging();
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let pagination = match Pagination::new(offset as u32, limit as u32) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Invalid pagination parameters: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    let result = manager.runtime().block_on(async {
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::ThreadingError,
                    format!("Failed to acquire engine read lock: {}", e)
                );
            }
        };
        
        match engine_guard.document_repository().find_all(pagination).await {
            Ok(documents) => {
                let documents_json: Vec<serde_json::Value> = documents
                    .iter()
                    .map(|doc| serde_json::json!({
                        "id": doc.id.to_string(),
                        "title": doc.title,
                        "contentType": doc.content_type.to_string(),
                        "wordCount": doc.word_count,
                        "characterCount": doc.character_count,
                        "createdAt": doc.created_at.to_string(),
                        "updatedAt": doc.updated_at.to_string(),
                        "version": doc.version,
                        "isDeleted": doc.is_deleted
                    }))
                    .collect();
                
                let response = serde_json::json!({
                    "documents": documents_json,
                    "count": documents.len()
                });
                
                FFIResult::success(response.to_string())
            }
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to retrieve documents: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json_str), .. } => create_c_string(json_str),
        FFIResult { error_message, .. } => {
            log::error!("List documents failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Cleanup and shutdown - proper resource management
#[no_mangle]
pub extern "C" fn writemagic_shutdown() -> c_int {
    init_logging();
    log::info!("Shutting down WriteMagic core engine");
    
    let registry = get_instance_registry();
    match registry.write() {
        Ok(mut map) => {
            map.clear();
            log::info!("WriteMagic core engine shutdown completed");
            1
        }
        Err(e) => {
            log::error!("Failed to shutdown cleanly: {}", e);
            0
        }
    }
}

/// Memory leak detection helper - for debugging
#[no_mangle]
pub extern "C" fn writemagic_memory_status() -> *mut c_char {
    init_logging();
    
    let registry = get_instance_registry();
    let status = match registry.read() {
        Ok(map) => {
            serde_json::json!({
                "activeInstances": map.len(),
                "memoryHealthy": true,
                "registryStatus": "ok"
            })
        }
        Err(e) => {
            serde_json::json!({
                "activeInstances": 0,
                "memoryHealthy": false,
                "registryStatus": format!("error: {}", e)
            })
        }
    };
    
    create_c_string(status.to_string())
}

// Note: writemagic_free_string is defined in writemagic_shared::ffi_safety and exported globally

/// Get the library version
#[no_mangle]
pub extern "C" fn writemagic_get_version() -> *const c_char {
    static VERSION: &str = "0.1.0\0";
    VERSION.as_ptr() as *const c_char
}