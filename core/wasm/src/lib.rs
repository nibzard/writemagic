//! WASM bindings for WriteMagic core functionality
//! 
//! This crate provides WebAssembly bindings for the WriteMagic core engine,
//! enabling the Rust core to be used in web applications through JavaScript interop.

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;
use js_sys::Promise;
use web_sys::console;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::cell::RefCell;

// Import core WriteMagic types and services
use writemagic_writing::ProjectName;
use writemagic_shared::{
    WritemagicError, EntityId,
};

use writemagic_writing::{
    CoreEngine, ApplicationConfig,
    Document, 
    DocumentTitle, DocumentContent,
};

// Note: AI, version-control, and agent domains not available in WASM build
// due to native compilation requirements and networking dependencies


// Set up panic hook for better debugging in WASM
#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Initialize WASM module
#[wasm_bindgen]
pub fn initialize() {
    console::log_1(&"WriteMagic WASM initialized".into());
}

/// Error type for WASM bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmError {
    message: String,
    code: String,
}

impl WasmError {
    /// Get the error message
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the error code
    pub fn code(&self) -> String {
        self.code.clone()
    }
}

impl From<WritemagicError> for WasmError {
    fn from(error: WritemagicError) -> Self {
        let (message, code) = match &error {
            WritemagicError::Validation { message } => (message.clone(), "VALIDATION_ERROR".to_string()),
            WritemagicError::Repository { message } => (message.clone(), "REPOSITORY_ERROR".to_string()),
            WritemagicError::AiProvider { message } => (message.clone(), "AI_PROVIDER_ERROR".to_string()),
            WritemagicError::Configuration { message } => (message.clone(), "CONFIGURATION_ERROR".to_string()),
            WritemagicError::Internal { message, .. } => (message.clone(), "INTERNAL_ERROR".to_string()),
            _ => (error.to_string(), "UNKNOWN_ERROR".to_string()),
        };
        
        WasmError { message, code }
    }
}

impl From<uuid::Error> for WasmError {
    fn from(error: uuid::Error) -> Self {
        WasmError {
            message: format!("UUID error: {}", error),
            code: "UUID_ERROR".to_string(),
        }
    }
}

impl From<WasmError> for JsValue {
    fn from(error: WasmError) -> Self {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"message".into(), &error.message.into()).unwrap();
        js_sys::Reflect::set(&obj, &"code".into(), &error.code.into()).unwrap();
        obj.into()
    }
}

/// Document data for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmDocument {
    id: String,
    title: String,
    content: String,
    project_id: Option<String>,
    content_type: String,
    word_count: u32,
    character_count: u32,
    created_at: String,
    updated_at: String,
    created_by: Option<String>,
    is_deleted: bool,
}

#[wasm_bindgen]
impl WasmDocument {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get the document title
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// Get the document content
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// Get the project ID if the document belongs to a project
    #[wasm_bindgen(getter)]
    pub fn project_id(&self) -> Option<String> {
        self.project_id.clone()
    }

    /// Get the content type
    #[wasm_bindgen(getter)]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }

    /// Get the word count
    #[wasm_bindgen(getter)]
    pub fn word_count(&self) -> u32 {
        self.word_count
    }

    /// Get the character count
    #[wasm_bindgen(getter)]
    pub fn character_count(&self) -> u32 {
        self.character_count
    }

    /// Get the creation timestamp
    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> String {
        self.created_at.clone()
    }

    /// Get the last update timestamp
    #[wasm_bindgen(getter)]
    pub fn updated_at(&self) -> String {
        self.updated_at.clone()
    }

    /// Get the creator ID if available
    #[wasm_bindgen(getter)]
    pub fn created_by(&self) -> Option<String> {
        self.created_by.clone()
    }

    /// Check if the document is deleted
    #[wasm_bindgen(getter)]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

/// Convert from Document to WasmDocument
impl From<&Document> for WasmDocument {
    fn from(doc: &Document) -> Self {
        Self {
            id: doc.id.to_string(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            project_id: None, // TODO: Add project association when needed
            content_type: format!("{:?}", doc.content_type),
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at: doc.created_at.as_datetime().to_rfc3339(),
            updated_at: doc.updated_at.as_datetime().to_rfc3339(),
            created_by: doc.created_by.as_ref().map(|id| id.to_string()),
            is_deleted: doc.is_deleted,
        }
    }
}

/// Convert from DocumentAggregate to WasmDocument  
impl From<&writemagic_writing::DocumentAggregate> for WasmDocument {
    fn from(aggregate: &writemagic_writing::DocumentAggregate) -> Self {
        let doc = aggregate.document();
        Self {
            id: doc.id.to_string(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            project_id: None, // TODO: Add project association when needed
            content_type: format!("{:?}", doc.content_type),
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at: doc.created_at.as_datetime().to_rfc3339(),
            updated_at: doc.updated_at.as_datetime().to_rfc3339(),
            created_by: doc.created_by.as_ref().map(|id| id.to_string()),
            is_deleted: doc.is_deleted,
        }
    }
}

/// Project data for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmProject {
    id: String,
    name: String,
    description: Option<String>,
    document_ids: Vec<String>,
    created_at: String,
    updated_at: String,
    created_by: Option<String>,
    is_archived: bool,
}

#[wasm_bindgen]
impl WasmProject {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get the project name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get the project description
    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    /// Get the document IDs as a JavaScript array
    #[wasm_bindgen(getter)]
    pub fn document_ids(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for id in &self.document_ids {
            array.push(&JsValue::from_str(id));
        }
        array
    }

    /// Get the project creation timestamp
    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> String {
        self.created_at.clone()
    }

    /// Get the project last update timestamp  
    #[wasm_bindgen(getter)]
    pub fn updated_at(&self) -> String {
        self.updated_at.clone()
    }

    /// Get the project creator ID if available
    #[wasm_bindgen(getter)]
    pub fn created_by(&self) -> Option<String> {
        self.created_by.clone()
    }

    /// Check if the project is archived
    #[wasm_bindgen(getter)]
    pub fn is_archived(&self) -> bool {
        self.is_archived
    }
}

// Note: AI completion structs removed - not available in WASM build due to native networking dependencies

/// Main WriteMagic engine for WASM
#[wasm_bindgen]
pub struct WriteMagicEngine {
    #[allow(dead_code)]
    inner: Rc<RefCell<Option<CoreEngine>>>,
}

#[wasm_bindgen]
impl WriteMagicEngine {
    /// Create a new WriteMagic engine
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(None)),
        }
    }

    /// Initialize the engine with configuration
    pub fn initialize(&mut self, config_json: Option<String>) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let _config = if let Some(json) = config_json {
                serde_json::from_str::<ApplicationConfig>(&json)
                    .map_err(|e| WasmError {
                        message: format!("Invalid configuration: {}", e),
                        code: "CONFIG_ERROR".to_string(),
                    })?
            } else {
                ApplicationConfig::default()
            };

            // For WASM, use in-memory storage for now
            let engine = CoreEngine::new_in_memory()
                .await
                .map_err(WasmError::from)?;
            
            *inner.borrow_mut() = Some(engine);
            
            Ok(JsValue::from("Engine initialized successfully"))
        })
    }

    /// Create a new document
    pub fn create_document(&self, title: String, content: String, project_id: Option<String>) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let doc_title = DocumentTitle::new(title).map_err(WasmError::from)?;
            let doc_content = DocumentContent::new(content).map_err(WasmError::from)?;
            let _project_entity_id = project_id.as_ref()
                .map(|id| EntityId::from_string(id))
                .transpose()
                .map_err(WasmError::from)?;

            let document = engine.document_management_service()
                .create_document(doc_title, doc_content, writemagic_shared::ContentType::Markdown, None)
                .await
                .map_err(WasmError::from)?;

            let wasm_doc = WasmDocument::from(&document);
            let serialized = serde_wasm_bindgen::to_value(&wasm_doc)
                .map_err(|e| WasmError {
                    message: format!("Serialization error: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?;

            Ok(serialized)
        })
    }

    /// Get a document by ID
    pub fn get_document(&self, id: String) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let entity_id = EntityId::from_string(&id).map_err(WasmError::from)?;
            
            let document = engine.document_repository()
                .find_by_id(&entity_id)
                .await
                .map_err(WasmError::from)?
                .ok_or_else(|| WasmError {
                    message: "Document not found".to_string(),
                    code: "DOCUMENT_NOT_FOUND".to_string(),
                })?;

            let wasm_doc = WasmDocument::from(&document);
            let serialized = serde_wasm_bindgen::to_value(&wasm_doc)
                .map_err(|e| WasmError {
                    message: format!("Serialization error: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?;

            Ok(serialized)
        })
    }

    /// Update a document
    pub fn update_document(&self, id: String, _title: Option<String>, content: Option<String>) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let entity_id = EntityId::from_string(&id).map_err(WasmError::from)?;
            
            // For now, we'll only support content updates since that's what's available in the service
            let updated_document = if let Some(new_content) = content {
                let doc_content = DocumentContent::new(new_content).map_err(WasmError::from)?;
                engine.document_management_service()
                    .update_document_content(entity_id, doc_content, None, None)
                    .await
                    .map_err(WasmError::from)?
            } else {
                // If only title update or no updates, just fetch the document
                // TODO: Add title update support when available in service
                return Ok(JsValue::from("Title updates not yet supported"));
            };

            let wasm_doc = WasmDocument::from(&updated_document);
            let serialized = serde_wasm_bindgen::to_value(&wasm_doc)
                .map_err(|e| WasmError {
                    message: format!("Serialization error: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?;

            Ok(serialized)
        })
    }

    /// Delete a document
    pub fn delete_document(&self, id: String) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let entity_id = EntityId::from_string(&id).map_err(WasmError::from)?;
            
            engine.document_management_service()
                .delete_document(entity_id, None)
                .await
                .map_err(WasmError::from)?;

            Ok(JsValue::from("Document deleted successfully"))
        })
    }

    /// List all documents
    pub fn list_documents(&self) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let pagination = writemagic_shared::Pagination::new(0, 100).map_err(WasmError::from)?; // Default pagination
            let documents = engine.document_repository()
                .find_all(pagination)
                .await
                .map_err(WasmError::from)?;

            let wasm_docs: Vec<WasmDocument> = documents.iter().map(WasmDocument::from).collect();
            let serialized = serde_wasm_bindgen::to_value(&wasm_docs)
                .map_err(|e| WasmError {
                    message: format!("Serialization error: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?;

            Ok(serialized)
        })
    }

    /// Create a new project
    pub fn create_project(&self, name: String, description: Option<String>) -> Promise {
        let inner = self.inner.clone();
        
        wasm_bindgen_futures::future_to_promise(async move {
            let engine = inner.borrow();
            let engine = engine.as_ref().ok_or_else(|| WasmError {
                message: "Engine not initialized".to_string(),
                code: "ENGINE_NOT_INITIALIZED".to_string(),
            })?;

            let project_name = ProjectName::new(name).map_err(WasmError::from)?;
            
            let project = engine.project_management_service()
                .create_project(project_name, description, None)
                .await
                .map_err(WasmError::from)?;

            let wasm_project = WasmProject {
                id: project.project().id.to_string(),
                name: project.project().name.clone(),
                description: project.project().description.clone(),
                document_ids: project.project().document_ids.iter().map(|id| id.to_string()).collect(),
                created_at: project.project().created_at.as_datetime().to_rfc3339(),
                updated_at: project.project().updated_at.as_datetime().to_rfc3339(),
                created_by: project.project().created_by.as_ref().map(|id| id.to_string()),
                is_archived: project.project().is_deleted, // Map is_deleted to is_archived for WASM
            };

            let serialized = serde_wasm_bindgen::to_value(&wasm_project)
                .map_err(|e| WasmError {
                    message: format!("Serialization error: {}", e),
                    code: "SERIALIZATION_ERROR".to_string(),
                })?;

            Ok(serialized)
        })
    }

    /// Request AI completion (Not available in WASM - requires native networking)
    pub fn ai_completion(&self, _request_json: String) -> Promise {
        wasm_bindgen_futures::future_to_promise(async move {
            Err(WasmError {
                message: "AI completion not available in WASM build. Use native or server-side AI integration.".to_string(),
                code: "FEATURE_NOT_AVAILABLE".to_string(),
            }.into())
        })
    }
}

/// Version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get available AI providers (Not available in WASM build)
#[wasm_bindgen]
pub fn get_ai_providers() -> Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        Err(WasmError {
            message: "AI providers not available in WASM build. Use server-side integration.".to_string(),
            code: "FEATURE_NOT_AVAILABLE".to_string(),
        }.into())
    })
}