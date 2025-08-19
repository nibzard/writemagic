//! iOS FFI bindings for WriteMagic core

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex, Once};
use writemagic_shared::{EntityId, Pagination, ContentType};
use writemagic_writing::{CoreEngine, ApplicationConfigBuilder};
use writemagic_writing::entities::{Document, Project};

static INIT: Once = Once::new();
static mut CORE_ENGINE: Option<Arc<Mutex<Option<CoreEngine>>>> = None;

/// Initialize logging for iOS
fn init_logging() {
    INIT.call_once(|| {
        unsafe {
            CORE_ENGINE = Some(Arc::new(Mutex::new(None)));
        }
        
        #[cfg(target_os = "ios")]
        {
            // Initialize iOS-specific logging
            log::info!("WriteMagic iOS FFI initialized");
        }
    });
}

/// Get the core engine instance
fn get_core_engine() -> Option<Arc<Mutex<Option<CoreEngine>>>> {
    unsafe {
        CORE_ENGINE.as_ref().map(Arc::clone)
    }
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

    let engine_result = execute_async(async {
        let mut builder = ApplicationConfigBuilder::new();
        
        if use_sqlite == 1 {
            builder = builder.with_sqlite();
        } else {
            builder = builder.with_sqlite_in_memory();
        }
        
        if let Some(claude_key) = claude_api_key {
            builder = builder.with_claude_key(claude_key);
        }
        
        if let Some(openai_key) = openai_api_key {
            builder = builder.with_openai_key(openai_key);
        }
        
        builder
            .with_log_level("info".to_string())
            .with_content_filtering(true)
            .build()
            .await
    });

    match engine_result {
        Ok(engine) => {
            if let Some(core_ref) = get_core_engine() {
                if let Ok(mut guard) = core_ref.lock() {
                    *guard = Some(engine);
                    log::info!("WriteMagic core engine initialized successfully with AI");
                    return 1;
                }
            }
            log::error!("Failed to store core engine instance");
            0
        }
        Err(e) => {
            log::error!("Failed to initialize core engine: {}", e);
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
        if let Some(core_ref) = get_core_engine() {
            if let Ok(guard) = core_ref.lock() {
                if let Some(ref engine) = *guard {
                    let content_type = ContentType::from_string(content_type_str)
                        .unwrap_or(ContentType::Markdown);
                    
                    let document = Document::new(
                        title.to_string(),
                        content.to_string(),
                        content_type,
                        None, // No user context in FFI for now
                    );
                    
                    let repo = engine.document_repository();
                    match repo.save(&document).await {
                        Ok(saved_doc) => {
                            log::info!("Document created successfully: {}", saved_doc.id);
                            return Some(saved_doc.id.to_string());
                        }
                        Err(e) => {
                            log::error!("Failed to save document: {}", e);
                            return None;
                        }
                    }
                }
            }
        }
        None
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
        if let Some(core_ref) = get_core_engine() {
            if let Ok(guard) = core_ref.lock() {
                if let Some(ref engine) = *guard {
                    let entity_id = match EntityId::from_string(document_id_str) {
                        Ok(id) => id,
                        Err(e) => {
                            log::error!("Invalid document ID: {}", e);
                            return false;
                        }
                    };
                    
                    let repo = engine.document_repository();
                    
                    // Load the document
                    match repo.find_by_id(&entity_id).await {
                        Ok(Some(mut document)) => {
                            // Update content
                            document.update_content(content.to_string(), None);
                            
                            // Save updated document
                            match repo.save(&document).await {
                                Ok(_) => {
                                    log::info!("Document updated successfully: {}", entity_id);
                                    return true;
                                }
                                Err(e) => {
                                    log::error!("Failed to save updated document: {}", e);
                                    return false;
                                }
                            }
                        }
                        Ok(None) => {
                            log::error!("Document not found: {}", entity_id);
                            return false;
                        }
                        Err(e) => {
                            log::error!("Failed to load document: {}", e);
                            return false;
                        }
                    }
                }
            }
        }
        false
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
        if let Some(core_ref) = get_core_engine() {
            if let Ok(guard) = core_ref.lock() {
                if let Some(ref engine) = *guard {
                    let entity_id = match EntityId::from_string(document_id_str) {
                        Ok(id) => id,
                        Err(e) => {
                            log::error!("Invalid document ID: {}", e);
                            return None;
                        }
                    };
                    
                    let repo = engine.document_repository();
                    
                    match repo.find_by_id(&entity_id).await {
                        Ok(Some(document)) => {
                            let document_json = serde_json::json!({
                                "id": document.id.to_string(),
                                "title": document.title,
                                "content": document.content,
                                "contentType": document.content_type.to_string(),
                                "contentHash": document.content_hash.to_string(),
                                "filePath": document.file_path.as_ref().map(|p| p.to_string()),
                                "wordCount": document.word_count,
                                "characterCount": document.character_count,
                                "createdAt": document.created_at.to_string(),
                                "updatedAt": document.updated_at.to_string(),
                                "createdBy": document.created_by.as_ref().map(|id| id.to_string()),
                                "updatedBy": document.updated_by.as_ref().map(|id| id.to_string()),
                                "version": document.version,
                                "isDeleted": document.is_deleted,
                                "deletedAt": document.deleted_at.as_ref().map(|t| t.to_string())
                            });
                            
                            return Some(document_json.to_string());
                        }
                        Ok(None) => {
                            log::error!("Document not found: {}", entity_id);
                            return None;
                        }
                        Err(e) => {
                            log::error!("Failed to load document: {}", e);
                            return None;
                        }
                    }
                }
            }
        }
        None
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
        if let Some(core_ref) = get_core_engine() {
            if let Ok(guard) = core_ref.lock() {
                if let Some(ref engine) = *guard {
                    match engine.complete_text(prompt.to_string(), model).await {
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
                } else {
                    log::error!("CoreEngine not initialized");
                    None
                }
            } else {
                log::error!("Failed to lock core engine");
                None
            }
        } else {
            log::error!("CoreEngine reference not available");
            None
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