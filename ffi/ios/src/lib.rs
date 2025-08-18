//! iOS FFI bindings for WriteMagic core

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Once;

static INIT: Once = Once::new();

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

/// Initialize the WriteMagic core engine
/// Returns 1 for success, 0 for failure
#[no_mangle]
pub extern "C" fn writemagic_initialize() -> c_int {
    init_logging();
    log::info!("Initializing WriteMagic core for iOS");
    
    // Initialize the core engine here
    // This would set up the database, repositories, services, etc.
    
    1 // Success
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
    
    let content_type = unsafe {
        match CStr::from_ptr(content_type).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    log::info!("Creating document: {} ({})", title, content_type);
    
    // Create document using the core domain
    let document_id = writemagic_shared::EntityId::new();
    
    // Return the document ID as a C string
    match CString::new(document_id.to_string()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
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
    
    let document_id = unsafe {
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
    
    log::info!("Updating document {} with new content", document_id);
    
    // Update document using the core domain
    // This would involve loading the document, updating it, and saving
    
    1 // Success
}

/// Get document by ID
/// Returns document JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_get_document(document_id: *const c_char) -> *mut c_char {
    init_logging();
    
    if document_id.is_null() {
        return std::ptr::null_mut();
    }
    
    let document_id = unsafe {
        match CStr::from_ptr(document_id).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    log::info!("Getting document {}", document_id);
    
    // Retrieve document using the core domain
    // This would involve querying the repository
    
    // For now, return a mock JSON response
    let mock_document = serde_json::json!({
        "id": document_id,
        "title": "Sample Document",
        "content": "Sample content",
        "contentType": "markdown",
        "wordCount": 2,
        "characterCount": 14
    });
    
    match CString::new(mock_document.to_string()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Complete text using AI
/// Returns completion JSON as C string (must be freed by caller)
#[no_mangle]
pub extern "C" fn writemagic_complete_text(
    prompt: *const c_char,
    model: *const c_char,
) -> *mut c_char {
    init_logging();
    
    if prompt.is_null() || model.is_null() {
        return std::ptr::null_mut();
    }
    
    let prompt = unsafe {
        match CStr::from_ptr(prompt).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    let model = unsafe {
        match CStr::from_ptr(model).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    log::info!("Completing text with model {} and prompt: {}", model, prompt);
    
    // Use AI service to complete text
    // This would involve the AI orchestration service
    
    // For now, return a mock completion
    let mock_completion = serde_json::json!({
        "completion": "This is a sample completion for the prompt.",
        "usage": {
            "promptTokens": 10,
            "completionTokens": 12,
            "totalTokens": 22
        }
    });
    
    match CString::new(mock_completion.to_string()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
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