//! iOS FFI bindings for WriteMagic core

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex, Once};
use writemagic_shared::{EntityId, ContentType, Repository, Pagination};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, ApplicationConfig, AIConfig,
    entities::{Document, Project},
    value_objects::{DocumentTitle, DocumentContent, ProjectName},
    services::{DocumentManagementService, ProjectManagementService},
    repositories::{DocumentRepository, ProjectRepository},
};

static INIT: Once = Once::new();
static mut CORE_ENGINE: Option<Arc<Mutex<CoreEngine>>> = None;

/// Get or initialize the core engine with configuration
async fn get_or_create_core_engine(claude_key: Option<String>, openai_key: Option<String>) -> Result<Arc<Mutex<CoreEngine>>, String> {
    unsafe {
        if CORE_ENGINE.is_none() {
            log::info!("Creating new CoreEngine instance for iOS");
            
            let engine = ApplicationConfigBuilder::new()
                .with_sqlite()  // Use persistent SQLite for iOS
                .with_claude_key(claude_key.unwrap_or_else(|| "".to_string()))
                .with_openai_key(openai_key.unwrap_or_else(|| "".to_string()))
                .with_log_level("info".to_string())
                .with_content_filtering(true)
                .build()
                .await
                .map_err(|e| format!("Failed to create CoreEngine: {}", e))?;
                
            CORE_ENGINE = Some(Arc::new(Mutex::new(engine)));
        }
        Ok(CORE_ENGINE.as_ref().unwrap().clone())
    }
}

/// Get existing core engine (if initialized)
fn get_core_engine() -> Result<Arc<Mutex<CoreEngine>>, String> {
    unsafe {
        CORE_ENGINE.as_ref()
            .map(Arc::clone)
            .ok_or_else(|| "CoreEngine not initialized - call initialize first".to_string())
    }
}

/// Initialize logging for iOS
fn init_logging() {
    INIT.call_once(|| {
        #[cfg(target_os = "ios")]
        {
            // Initialize iOS-specific logging
            log::info!("WriteMagic iOS FFI initialized");
        }
    });
}

/// Execute async function with runtime
fn execute_async<F, R>(f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}

/// Initialize the WriteMagic core engine with AI configuration
/// use_sqlite: 1 to use SQLite, 0 to use in-memory storage
/// claude_key: Claude API key (can be NULL)
/// openai_key: OpenAI API key (can be NULL)
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_initialize_with_ai(
    use_sqlite: c_int,
    claude_key: *const c_char,
    openai_key: *const c_char,
) -> c_int {
    init_logging();
    log::info!("Initializing WriteMagic core for iOS with storage type: {}", 
        if use_sqlite == 1 { "SQLite" } else { "In-Memory" });

    // Extract API keys
    let claude_api_key = if claude_key.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(claude_key).to_str() {
                Ok(s) => if s.trim().is_empty() { None } else { Some(s.to_string()) },
                Err(_) => None,
            }
        }
    };

    let openai_api_key = if openai_key.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(openai_key).to_str() {
                Ok(s) => if s.trim().is_empty() { None } else { Some(s.to_string()) },
                Err(_) => None,
            }
        }
    };

    // Initialize the core engine with async runtime
    let result = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            get_or_create_core_engine(claude_api_key, openai_api_key).await
        })
    }).join();
    
    match result {
        Ok(Ok(_)) => {
            log::info!("WriteMagic core engine initialized successfully");
            1
        }
        Ok(Err(e)) => {
            log::error!("Failed to initialize WriteMagic core engine: {}", e);
            0
        }
        Err(_) => {
            log::error!("Thread panic during initialization");
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

/// Create a new document
/// Returns document ID as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_create_document(
    title: *const c_char,
    content: *const c_char,
    content_type: *const c_char,
) -> *mut c_char {
    init_logging();
    
    if title.is_null() || content.is_null() || content_type.is_null() {
        return std::ptr::null_mut();
    }
    
    let title = unsafe {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    let content = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    let content_type_str = unsafe {
        match CStr::from_ptr(content_type).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    log::info!("Creating document: {} ({})", title, content_type_str);
    
    let result = execute_async(async {
        match get_core_engine() {
            Ok(engine) => {
                let engine_guard = match engine.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        log::error!("Failed to lock core engine: {}", e);
                        return None;
                    }
                };
                
                let document_title = match DocumentTitle::new(title) {
                    Ok(title) => title,
                    Err(e) => {
                        log::error!("Invalid document title: {}", e);
                        return None;
                    }
                };
                
                let document_content = match DocumentContent::new(content) {
                    Ok(content) => content,
                    Err(e) => {
                        log::error!("Invalid document content: {}", e);
                        return None;
                    }
                };
                
                let content_type = match content_type_str {
                    "markdown" => ContentType::Markdown,
                    "plain_text" => ContentType::PlainText,
                    "html" => ContentType::Html,
                    _ => ContentType::PlainText,
                };
                
                match engine_guard.runtime().block_on(async {
                    engine_guard.document_service().create_document(
                        document_title,
                        document_content,
                        content_type,
                        None, // created_by - would be set from authentication context
                    ).await
                }) {
                    Ok(aggregate) => {
                        let document = aggregate.document();
                        log::info!("Document created successfully: {}", document.id);
                        Some(document.id.to_string())
                    }
                    Err(e) => {
                        log::error!("Failed to create document: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get core engine: {}", e);
                None
            }
        }
    });
    
    match result {
        Some(doc_id) => {
            match CString::new(doc_id) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Update document content
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_update_document_content(
    document_id: *const c_char,
    content: *const c_char,
) -> c_int {
    init_logging();
    
    if document_id.is_null() || content.is_null() {
        return 0;
    }
    
    let document_id_str = unsafe {
        match CStr::from_ptr(document_id).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };
    
    let content = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };
    
    log::info!("Updating document {} with new content", document_id_str);
    
    let result = execute_async(async {
        match get_core_engine() {
            Ok(engine) => {
                let engine_guard = match engine.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        log::error!("Failed to lock core engine: {}", e);
                        return false;
                    }
                };
                
                // Parse document ID
                let document_id = match uuid::Uuid::parse_str(document_id_str) {
                    Ok(uuid) => EntityId::from_uuid(uuid),
                    Err(e) => {
                        log::error!("Invalid document ID format: {}", e);
                        return false;
                    }
                };
                
                let document_content = match DocumentContent::new(content) {
                    Ok(content) => content,
                    Err(e) => {
                        log::error!("Invalid document content: {}", e);
                        return false;
                    }
                };
                
                match engine_guard.runtime().block_on(async {
                    engine_guard.document_service().update_document_content(
                        document_id,
                        document_content,
                        None, // text selection
                        None, // updated_by - would be set from authentication context
                    ).await
                }) {
                    Ok(_) => {
                        log::info!("Successfully updated document {}", document_id_str);
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to update document content: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get core engine: {}", e);
                false
            }
        }
    });
    
    if result { 1 } else { 0 }
}

/// Get document by ID
/// Returns document JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_get_document(document_id: *const c_char) -> *mut c_char {
    init_logging();
    
    if document_id.is_null() {
        return std::ptr::null_mut();
    }
    
    let document_id_str = unsafe {
        match CStr::from_ptr(document_id).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    log::info!("Getting document {}", document_id_str);
    
    let result = execute_async(async {
        match get_core_engine() {
            Ok(engine) => {
                let engine_guard = match engine.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        log::error!("Failed to lock core engine: {}", e);
                        return None;
                    }
                };
                
                // Parse document ID
                let document_id = match uuid::Uuid::parse_str(document_id_str) {
                    Ok(uuid) => EntityId::from_uuid(uuid),
                    Err(e) => {
                        log::error!("Invalid document ID format: {}", e);
                        return None;
                    }
                };
                
                match engine_guard.runtime().block_on(async {
                    engine_guard.document_repository().find_by_id(&document_id).await
                }) {
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
                        
                        Some(response.to_string())
                    }
                    Ok(None) => {
                        log::warn!("Document {} not found", document_id_str);
                        None
                    }
                    Err(e) => {
                        log::error!("Failed to retrieve document: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get core engine: {}", e);
                None
            }
        }
    });
    
    match result {
        Some(json_str) => {
            match CString::new(json_str) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Complete text using AI with provider fallback
/// Returns completion JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_complete_text(
    prompt: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    init_logging();
    
    if prompt.is_null() {
        return std::ptr::null_mut();
    }
    
    let prompt = unsafe {
        match CStr::from_ptr(prompt).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    let model = if model.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(model).to_str() {
                Ok(s) => if s.trim().is_empty() { None } else { Some(s.to_string()) },
                Err(_) => None,
            }
        }
    };
    
    log::info!("Completing text with model {:?} and prompt: {}", model, prompt);
    
    let result = execute_async(async {
        match get_core_engine() {
            Ok(engine) => {
                let engine_guard = match engine.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        log::error!("Failed to lock core engine: {}", e);
                        return None;
                    }
                };
                
                match engine_guard.runtime().block_on(async {
                    engine_guard.complete_text(prompt.to_string(), model).await
                }) {
                    Ok(completion) => {
                        let response = serde_json::json!({
                            "completion": completion,
                            "success": true
                        });
                        Some(response.to_string())
                    }
                    Err(e) => {
                        log::error!("AI completion failed: {}", e);
                        let error_response = serde_json::json!({
                            "error": e.to_string(),
                            "success": false
                        });
                        Some(error_response.to_string())
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get core engine: {}", e);
                None
            }
        }
    });
    
    match result {
        Some(json_str) => {
            match CString::new(json_str) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => {
            // Return error response as fallback
            let fallback_error = serde_json::json!({
                "error": "CoreEngine not available",
                "success": false
            });
            match CString::new(fallback_error.to_string()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
    }
}

/// Free a C string allocated by this library
#[no_mangle]
pub extern "C" fn writemagic_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Get the library version
#[no_mangle]
pub extern "C" fn writemagic_get_version() -> *const c_char {
    static VERSION: &str = "0.1.0\0";
    VERSION.as_ptr() as *const c_char
}