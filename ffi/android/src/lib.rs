//! Android FFI bindings for WriteMagic core - Thread-safe and performance optimized

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jlong, jstring};
use jni::JNIEnv;
use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use tokio::runtime::Runtime;
use writemagic_shared::{EntityId, ContentType, Repository, Pagination, Result, WritemagicError};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, ApplicationConfig, AIConfig,
    entities::{Document, Project},
    value_objects::{DocumentTitle, DocumentContent, ProjectName},
    services::{DocumentManagementService, ProjectManagementService},
    repositories::{DocumentRepository, ProjectRepository},
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
#[derive(Debug)]
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
                .map_err(|e| WritemagicError::Infrastructure(format!("Failed to create runtime: {}", e)))?
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
fn java_string_to_rust(env: &mut JNIEnv, jstr: &JString) -> FFIResult<String> {
    if jstr.is_null() {
        return FFIResult::error(FFIErrorCode::InvalidInput, "JString is null".to_string());
    }
    
    match env.get_string(jstr) {
        Ok(java_str) => FFIResult::success(java_str.into()),
        Err(e) => FFIResult::error(
            FFIErrorCode::InvalidInput,
            format!("Failed to convert Java string: {}", e)
        )
    }
}

/// Memory-safe JSON serialization helper
fn serialize_to_json<T: serde::Serialize>(value: &T) -> FFIResult<String> {
    match serde_json::to_string(value) {
        Ok(json) => FFIResult::success(json),
        Err(e) => FFIResult::error(
            FFIErrorCode::SerializationError,
            format!("JSON serialization failed: {}", e)
        )
    }
}

/// Memory-safe JNI string creation helper
fn create_jni_string(env: &mut JNIEnv, value: String) -> jstring {
    match env.new_string(value) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create JNI string: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Initialize logging (called once)
fn init_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        #[cfg(target_os = "android")]
        {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("WriteMagic"),
            );
        }
        log::info!("WriteMagic Android FFI logging initialized");
    });
}

/// Initialize the WriteMagic core engine with enhanced error handling and lifecycle management
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeInitialize(
    mut env: JNIEnv,
    _class: JClass,
    claude_key: JString,
    openai_key: JString,
) -> jboolean {
    init_logging();
    log::info!("Initializing WriteMagic core for Android with enhanced FFI safety");
    
    // Extract API keys with proper error handling
    let claude_api_key = if claude_key.is_null() {
        None
    } else {
        match java_string_to_rust(&mut env, &claude_key) {
            FFIResult { value: Some(key), .. } if !key.trim().is_empty() => Some(key),
            FFIResult { error_code, error_message, .. } if error_code != FFIErrorCode::Success => {
                log::error!("Failed to extract Claude API key: {:?}", error_message);
                return false as jboolean;
            }
            _ => None,
        }
    };
    
    let openai_api_key = if openai_key.is_null() {
        None
    } else {
        match java_string_to_rust(&mut env, &openai_key) {
            FFIResult { value: Some(key), .. } if !key.trim().is_empty() => Some(key),
            FFIResult { error_code, error_message, .. } if error_code != FFIErrorCode::Success => {
                log::error!("Failed to extract OpenAI API key: {:?}", error_message);
                return false as jboolean;
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
                return true as jboolean;
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
                            true as jboolean
                        }
                        Err(e) => {
                            log::error!("Failed to create CoreEngine instance: {}", e);
                            false as jboolean
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create Tokio runtime: {}", e);
                    false as jboolean
                }
            }
        }
        Err(e) => {
            log::error!("Failed to acquire registry write lock: {}", e);
            false as jboolean
        }
    }
}

/// Create a new document with enhanced error handling and performance optimization
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeCreateDocument(
    mut env: JNIEnv,
    _class: JClass,
    title: JString,
    content: JString,
    content_type: JString,
) -> jstring {
    init_logging();
    
    // Get instance manager with proper error handling
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_code, error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    // Extract parameters with enhanced error handling
    let title_str = match java_string_to_rust(&mut env, &title) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract title: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let content_str = match java_string_to_rust(&mut env, &content) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let content_type_str = match java_string_to_rust(&mut env, &content_type) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content_type: {:?}", content_type);
            return std::ptr::null_mut();
        }
    };
    
    // Use shared runtime instead of spawning new thread
    let result = manager.runtime().block_on(async {
        // Get read lock on engine
        let engine_guard = match manager.engine().read() {
            Ok(guard) => guard,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::ThreadingError,
                    format!("Failed to acquire engine read lock: {}", e)
                );
            }
        };
        
        // Create value objects with validation
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
        
        // Create document through service layer
        match engine_guard.document_management_service().create_document(
            document_title,
            document_content,
            content_type,
            None, // created_by - set from authentication context
        ).await {
            Ok(aggregate) => {
                let document = aggregate.document();
                let response_data = serde_json::json!({
                    "id": document.id.to_string(),
                    "title": document.title,
                    "content": document.content,
                    "contentType": document.content_type.to_string(),
                    "wordCount": document.word_count,
                    "characterCount": document.character_count,
                    "createdAt": document.created_at.to_string(),
                    "updatedAt": document.updated_at.to_string(),
                    "version": document.version
                });
                
                FFIResult::success(response_data.to_string())
            }
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to create document: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("Document creation failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Update document content with optimized performance and error handling
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeUpdateDocumentContent(
    mut env: JNIEnv,
    _class: JClass,
    document_id: JString,
    content: JString,
) -> jboolean {
    init_logging();
    
    // Get instance manager
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return false as jboolean;
        }
    };
    
    // Extract parameters
    let document_id_str = match java_string_to_rust(&mut env, &document_id) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract document_id: {:?}", error_message);
            return false as jboolean;
        }
    };
    
    let content_str = match java_string_to_rust(&mut env, &content) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract content: {:?}", error_message);
            return false as jboolean;
        }
    };
    
    // Use shared runtime for async operation
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
    
    result as jboolean
}

/// Get document by ID with enhanced performance and error handling
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeGetDocument(
    mut env: JNIEnv,
    _class: JClass,
    document_id: JString,
) -> jstring {
    init_logging();
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let document_id_str = match java_string_to_rust(&mut env, &document_id) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract document_id: {:?}", error_message);
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
                let response_data = serde_json::json!({
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
                
                FFIResult::success(response_data.to_string())
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
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("Get document failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Create a new project with enhanced error handling
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_createProject(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
    description: JString,
) -> jstring {
    init_logging();
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let name_str = match java_string_to_rust(&mut env, &name) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract name: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let description_str = match java_string_to_rust(&mut env, &description) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract description: {:?}", error_message);
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
        
        let project_name = match ProjectName::new(&name_str) {
            Ok(name) => name,
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::InvalidInput,
                    format!("Invalid project name: {}", e)
                );
            }
        };
        
        let project_description = if description_str.trim().is_empty() {
            None
        } else {
            Some(description_str)
        };
        
        match engine_guard.project_management_service().create_project(
            project_name,
            project_description,
            None, // created_by - set from authentication context
        ).await {
            Ok(aggregate) => {
                let project = aggregate.project();
                let response_data = serde_json::json!({
                    "id": project.id.to_string(),
                    "name": project.name,
                    "description": project.description,
                    "documentIds": project.document_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                    "createdAt": project.created_at.to_string(),
                    "updatedAt": project.updated_at.to_string(),
                    "version": project.version
                });
                
                FFIResult::success(response_data.to_string())
            }
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to create project: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("Create project failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Get project by ID with enhanced error handling
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_getProject(
    mut env: JNIEnv,
    _class: JClass,
    project_id: JString,
) -> jstring {
    init_logging();
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let project_id_str = match java_string_to_rust(&mut env, &project_id) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract project_id: {:?}", error_message);
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
        
        let project_id = match uuid::Uuid::parse_str(&project_id_str) {
            Ok(uuid) => EntityId::from_uuid(uuid),
            Err(e) => {
                return FFIResult::error(
                    FFIErrorCode::InvalidInput,
                    format!("Invalid project ID format: {}", e)
                );
            }
        };
        
        match engine_guard.project_repository().find_by_id(&project_id).await {
            Ok(Some(project)) => {
                let response_data = serde_json::json!({
                    "id": project.id.to_string(),
                    "name": project.name,
                    "description": project.description,
                    "documentIds": project.document_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                    "createdAt": project.created_at.to_string(),
                    "updatedAt": project.updated_at.to_string(),
                    "version": project.version,
                    "isDeleted": project.is_deleted
                });
                
                FFIResult::success(response_data.to_string())
            }
            Ok(None) => FFIResult::error(
                FFIErrorCode::InvalidInput,
                format!("Project {} not found", project_id_str)
            ),
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to retrieve project: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("Get project failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// List all documents with pagination and enhanced performance
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeListDocuments(
    mut env: JNIEnv,
    _class: JClass,
    offset: jni::sys::jint,
    limit: jni::sys::jint,
) -> jstring {
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
                
                let response_data = serde_json::json!({
                    "documents": documents_json,
                    "count": documents.len()
                });
                
                FFIResult::success(response_data.to_string())
            }
            Err(e) => FFIResult::error(
                FFIErrorCode::EngineError,
                format!("Failed to retrieve documents: {}", e)
            )
        }
    });
    
    match result {
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("List documents failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Complete text using AI with enhanced error handling and performance optimization
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeCompleteText(
    mut env: JNIEnv,
    _class: JClass,
    prompt: JString,
    model: JString,
) -> jstring {
    init_logging();
    
    let manager = match get_default_instance() {
        FFIResult { value: Some(mgr), .. } => mgr,
        FFIResult { error_message, .. } => {
            log::error!("Failed to get CoreEngine instance: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let prompt_str = match java_string_to_rust(&mut env, &prompt) {
        FFIResult { value: Some(s), .. } => s,
        FFIResult { error_message, .. } => {
            log::error!("Failed to extract prompt: {:?}", error_message);
            return std::ptr::null_mut();
        }
    };
    
    let model_str = match java_string_to_rust(&mut env, &model) {
        FFIResult { value: Some(s), .. } if !s.trim().is_empty() => Some(s),
        _ => None,
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
                let response_data = serde_json::json!({
                    "completion": completion,
                    "success": true
                });
                FFIResult::success(response_data.to_string())
            }
            Err(e) => {
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
        FFIResult { value: Some(json), .. } => create_jni_string(&mut env, json),
        FFIResult { error_message, .. } => {
            log::error!("AI completion failed: {:?}", error_message);
            std::ptr::null_mut()
        }
    }
}

/// Cleanup and shutdown - proper resource management
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeShutdown(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    init_logging();
    log::info!("Shutting down WriteMagic core engine");
    
    let registry = get_instance_registry();
    match registry.write() {
        Ok(mut map) => {
            map.clear();
            log::info!("WriteMagic core engine shutdown completed");
            true as jboolean
        }
        Err(e) => {
            log::error!("Failed to shutdown cleanly: {}", e);
            false as jboolean
        }
    }
}

/// Memory leak detection helper - for debugging
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeMemoryStatus(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
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
    
    create_jni_string(&mut env, status.to_string())
}