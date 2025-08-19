//! Android FFI bindings for WriteMagic core

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jlong, jstring};
use jni::JNIEnv;
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
            log::info!("Creating new CoreEngine instance for Android");
            
            let engine = ApplicationConfigBuilder::new()
                .with_sqlite()  // Use persistent SQLite for mobile
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

/// Initialize logging for Android
fn init_logging() {
    INIT.call_once(|| {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("WriteMagic"),
        );
    });
}

/// Initialize the WriteMagic core engine with optional AI keys
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeInitialize(
    mut env: JNIEnv,
    _class: JClass,
    claude_key: JString,
    openai_key: JString,
) -> jboolean {
    init_logging();
    log::info!("Initializing WriteMagic core for Android with AI integration");
    
    // Extract API keys (can be empty strings)
    let claude_api_key = if claude_key.is_null() {
        None
    } else {
        match env.get_string(&claude_key) {
            Ok(s) => {
                let key: String = s.into();
                if key.trim().is_empty() { None } else { Some(key) }
            }
            Err(e) => {
                log::error!("Failed to get claude_key string: {}", e);
                None
            }
        }
    };
    
    let openai_api_key = if openai_key.is_null() {
        None
    } else {
        match env.get_string(&openai_key) {
            Ok(s) => {
                let key: String = s.into();
                if key.trim().is_empty() { None } else { Some(key) }
            }
            Err(e) => {
                log::error!("Failed to get openai_key string: {}", e);
                None
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
            true as jboolean
        }
        Ok(Err(e)) => {
            log::error!("Failed to initialize WriteMagic core engine: {}", e);
            false as jboolean
        }
        Err(_) => {
            log::error!("Thread panic during initialization");
            false as jboolean
        }
    }
}

/// Create a new document
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeCreateDocument(
    mut env: JNIEnv,
    _class: JClass,
    title: JString,
    content: JString,
    content_type: JString,
) -> jstring {
    init_logging();
    
    let title: String = match env.get_string(&title) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get title string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    let content: String = match env.get_string(&content) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get content string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    let content_type_str: String = match env.get_string(&content_type) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get content_type string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Create document using the domain service
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        let document_title = match DocumentTitle::new(&title) {
            Ok(title) => title,
            Err(e) => {
                log::error!("Invalid document title: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        let document_content = match DocumentContent::new(&content) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Invalid document content: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        let content_type = match content_type_str.as_str() {
            "markdown" => ContentType::Markdown,
            "plain_text" => ContentType::PlainText,
            "html" => ContentType::Html,
            _ => ContentType::PlainText,
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.document_management_service().create_document(
                document_title,
                document_content,
                content_type,
                None, // created_by - would be set from authentication context
            ).await
        })
    };
    
    match result {
        Ok(aggregate) => {
            let document = aggregate.document();
            let response = serde_json::json!({
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
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create document: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Update document content
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeUpdateDocumentContent(
    mut env: JNIEnv,
    _class: JClass,
    document_id: JString,
    content: JString,
) -> jboolean {
    init_logging();
    
    let document_id_str: String = match env.get_string(&document_id) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get document_id string: {}", e);
            return false as jboolean;
        }
    };
    
    let content: String = match env.get_string(&content) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get content string: {}", e);
            return false as jboolean;
        }
    };
    
    // Parse document ID
    let document_id = match uuid::Uuid::parse_str(&document_id_str) {
        Ok(uuid) => EntityId::from_uuid(uuid),
        Err(e) => {
            log::error!("Invalid document ID format: {}", e);
            return false as jboolean;
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return false as jboolean;
        }
    };
    
    // Update document using the domain service
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return false as jboolean;
            }
        };
        
        let document_content = match DocumentContent::new(&content) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Invalid document content: {}", e);
                return false as jboolean;
            }
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.document_management_service().update_document_content(
                document_id,
                document_content,
                None, // text selection
                None, // updated_by - would be set from authentication context
            ).await
        })
    };
    
    match result {
        Ok(_) => {
            log::info!("Successfully updated document {}", document_id_str);
            true as jboolean
        }
        Err(e) => {
            log::error!("Failed to update document content: {}", e);
            false as jboolean
        }
    }
}

/// Get document by ID
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeGetDocument(
    mut env: JNIEnv,
    _class: JClass,
    document_id: JString,
) -> jstring {
    init_logging();
    
    let document_id_str: String = match env.get_string(&document_id) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get document_id string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Parse document ID
    let document_id = match uuid::Uuid::parse_str(&document_id_str) {
        Ok(uuid) => EntityId::from_uuid(uuid),
        Err(e) => {
            log::error!("Invalid document ID format: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Retrieve document using the domain repository
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.document_repository().find_by_id(&document_id).await
        })
    };
    
    match result {
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
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Ok(None) => {
            log::warn!("Document {} not found", document_id_str);
            std::ptr::null_mut()
        }
        Err(e) => {
            log::error!("Failed to retrieve document: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Create a new project
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_createProject(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
    description: JString,
) -> jstring {
    init_logging();
    
    let name: String = match env.get_string(&name) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get name string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    let description: String = match env.get_string(&description) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get description string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Create project using the domain service
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        let project_name = match ProjectName::new(&name) {
            Ok(name) => name,
            Err(e) => {
                log::error!("Invalid project name: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        let project_description = if description.trim().is_empty() {
            None
        } else {
            Some(description)
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.project_management_service().create_project(
                project_name,
                project_description,
                None, // created_by - would be set from authentication context
            ).await
        })
    };
    
    match result {
        Ok(aggregate) => {
            let project = aggregate.project();
            let response = serde_json::json!({
                "id": project.id.to_string(),
                "name": project.name,
                "description": project.description,
                "documentIds": project.document_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "createdAt": project.created_at.to_string(),
                "updatedAt": project.updated_at.to_string(),
                "version": project.version
            });
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create project: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get project by ID
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_getProject(
    mut env: JNIEnv,
    _class: JClass,
    project_id: JString,
) -> jstring {
    init_logging();
    
    let project_id_str: String = match env.get_string(&project_id) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get project_id string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Parse project ID
    let project_id = match uuid::Uuid::parse_str(&project_id_str) {
        Ok(uuid) => EntityId::from_uuid(uuid),
        Err(e) => {
            log::error!("Invalid project ID format: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Retrieve project using the domain repository
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.project_repository().find_by_id(&project_id).await
        })
    };
    
    match result {
        Ok(Some(project)) => {
            let response = serde_json::json!({
                "id": project.id.to_string(),
                "name": project.name,
                "description": project.description,
                "documentIds": project.document_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "createdAt": project.created_at.to_string(),
                "updatedAt": project.updated_at.to_string(),
                "version": project.version,
                "isDeleted": project.is_deleted
            });
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Ok(None) => {
            log::warn!("Project {} not found", project_id_str);
            std::ptr::null_mut()
        }
        Err(e) => {
            log::error!("Failed to retrieve project: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// List all documents with pagination
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeListDocuments(
    mut env: JNIEnv,
    _class: JClass,
    offset: jni::sys::jint,
    limit: jni::sys::jint,
) -> jstring {
    init_logging();
    
    // Create pagination
    let pagination = match Pagination::new(offset as u32, limit as u32) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Invalid pagination parameters: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Retrieve documents using the domain repository
    let result = {
        let engine_guard = match engine.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock core engine: {}", e);
                return std::ptr::null_mut();
            }
        };
        
        engine_guard.runtime().block_on(async {
            engine_guard.document_repository().find_all(pagination).await
        })
    };
    
    match result {
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
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to retrieve documents: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Complete text using AI with provider fallback
#[no_mangle]
pub extern "system" fn Java_com_writemagic_core_WriteMagicCore_nativeCompleteText(
    mut env: JNIEnv,
    _class: JClass,
    prompt: JString,
    model: JString,
) -> jstring {
    init_logging();
    
    let prompt: String = match env.get_string(&prompt) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get prompt string: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    let model: Option<String> = match env.get_string(&model) {
        Ok(s) => {
            let model_str: String = s.into();
            if model_str.trim().is_empty() { None } else { Some(model_str) }
        }
        Err(e) => {
            log::error!("Failed to get model string: {}", e);
            None
        }
    };
    
    log::info!("Completing text with model {:?} and prompt: {}", model, prompt);
    
    // Get the core engine
    let engine = match get_core_engine() {
        Ok(engine) => engine,
        Err(e) => {
            log::error!("Failed to get core engine: {}", e);
            return std::ptr::null_mut();
        }
    };
    
    // Complete text using the AI orchestration service
    let result = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine_guard = match engine.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("Failed to lock core engine: {}", e);
                    return Err(format!("Failed to lock core engine: {}", e));
                }
            };
            
            match engine_guard.complete_text(prompt, model).await {
                Ok(completion) => Ok(completion),
                Err(e) => Err(format!("AI completion failed: {}", e)),
            }
        })
    }).join();
    
    match result {
        Ok(Ok(completion)) => {
            let response = serde_json::json!({
                "completion": completion,
                "success": true
            });
            
            match env.new_string(response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    log::error!("Failed to create return string: {}", e);
                    std::ptr::null_mut()
                }
            }
        }
        Ok(Err(e)) => {
            log::error!("AI completion failed: {}", e);
            let error_response = serde_json::json!({
                "error": e,
                "success": false
            });
            match env.new_string(error_response.to_string()) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null_mut()
            }
        }
        Err(_) => {
            log::error!("Thread panic during AI completion");
            std::ptr::null_mut()
        }
    }
}