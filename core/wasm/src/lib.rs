//! WASM bindings for WriteMagic core functionality
//! 
//! This crate provides WebAssembly bindings for the WriteMagic core engine,
//! enabling the Rust core to be used in web applications through JavaScript interop.

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys::Promise;
use web_sys::console;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

// Import core WriteMagic types and services
use writemagic_shared::{
    WritemagicError, Result, EntityId, ContentType, DatabaseConfig,
};
use writemagic_writing::{
    CoreEngine, ApplicationConfig, AIConfig, LoggingConfig, SecurityConfig,
    Document, DocumentManagementService, ProjectManagementService, ContentAnalysisService,
    DocumentTitle, DocumentContent, ProjectName, TextSelection,
};
use writemagic_ai::{
    AIOrchestrationService, CompletionRequest, CompletionResponse, Message, MessageRole,
};
use writemagic_project::{
    Workspace, Pane, WorkspaceLayout,
};

// Import async utilities
use futures::Future;
use tokio::sync::Mutex;

// Set up panic hook for better debugging in WASM
#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Initialize logging for WASM
#[wasm_bindgen]
pub fn init_logging() {
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logging");
}

/// Error type for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmError {
    message: String,
    code: String,
}

#[wasm_bindgen]
impl WasmError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }
}

impl From<WritemagicError> for WasmError {
    fn from(error: WritemagicError) -> Self {
        let (message, code) = match &error {
            WritemagicError::ValidationError(msg) => (msg.clone(), "VALIDATION_ERROR".to_string()),
            WritemagicError::RepositoryError(msg) => (msg.clone(), "REPOSITORY_ERROR".to_string()),
            WritemagicError::AIProviderError(msg) => (msg.clone(), "AI_PROVIDER_ERROR".to_string()),
            WritemagicError::ConfigurationError(msg) => (msg.clone(), "CONFIGURATION_ERROR".to_string()),
            WritemagicError::Internal(msg) => (msg.clone(), "INTERNAL_ERROR".to_string()),
            _ => (error.to_string(), "UNKNOWN_ERROR".to_string()),
        };
        
        WasmError { message, code }
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

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn project_id(&self) -> Option<String> {
        self.project_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn word_count(&self) -> u32 {
        self.word_count
    }

    #[wasm_bindgen(getter)]
    pub fn character_count(&self) -> u32 {
        self.character_count
    }

    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> String {
        self.created_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn updated_at(&self) -> String {
        self.updated_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn created_by(&self) -> Option<String> {
        self.created_by.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

impl From<Document> for WasmDocument {
    fn from(doc: Document) -> Self {
        WasmDocument {
            id: doc.id.to_string(),
            title: doc.title,
            content: doc.content,
            project_id: None, // Documents don't directly store project_id
            content_type: format!("{:?}", doc.content_type),
            word_count: doc.word_count,
            character_count: doc.character_count,
            created_at: doc.created_at.to_string(),
            updated_at: doc.updated_at.to_string(),
            created_by: doc.created_by.map(|id| id.to_string()),
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
    owner_id: Option<String>,
    document_ids: Vec<String>,
    created_at: String,
    updated_at: String,
    is_deleted: bool,
}

#[wasm_bindgen]
impl WasmProject {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn owner_id(&self) -> Option<String> {
        self.owner_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn document_ids(&self) -> Vec<JsValue> {
        self.document_ids.iter().map(|id| JsValue::from_str(id)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> String {
        self.created_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn updated_at(&self) -> String {
        self.updated_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

impl From<writemagic_writing::Project> for WasmProject {
    fn from(project: writemagic_writing::Project) -> Self {
        WasmProject {
            id: project.id.to_string(),
            name: project.name,
            description: project.description,
            owner_id: project.created_by.map(|id| id.to_string()),
            document_ids: project.document_ids.into_iter().map(|id| id.to_string()).collect(),
            created_at: project.created_at.to_string(),
            updated_at: project.updated_at.to_string(),
            is_deleted: project.is_deleted,
        }
    }
}

/// AI completion request for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCompletionRequest {
    prompt: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    context: Option<String>,
}

#[wasm_bindgen]
impl WasmCompletionRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(prompt: String, model: String) -> WasmCompletionRequest {
        WasmCompletionRequest {
            prompt,
            model,
            max_tokens: None,
            temperature: None,
            context: None,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.max_tokens = Some(max_tokens);
    }

    #[wasm_bindgen(setter)]
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = Some(temperature);
    }

    #[wasm_bindgen(setter)]
    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }
}

/// AI completion response for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCompletionResponse {
    content: String,
    model: String,
    tokens_used: u32,
    finish_reason: String,
}

#[wasm_bindgen]
impl WasmCompletionResponse {
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn model(&self) -> String {
        self.model.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn tokens_used(&self) -> u32 {
        self.tokens_used
    }

    #[wasm_bindgen(getter)]
    pub fn finish_reason(&self) -> String {
        self.finish_reason.clone()
    }
}

impl TryFrom<WasmCompletionRequest> for CompletionRequest {
    type Error = WasmError;

    fn try_from(request: WasmCompletionRequest) -> std::result::Result<Self, Self::Error> {
        let messages = vec![Message::user(request.prompt)];
        
        let mut completion_request = CompletionRequest::new(messages, request.model);
        
        if let Some(max_tokens) = request.max_tokens {
            completion_request = completion_request.with_max_tokens(max_tokens);
        }
        
        if let Some(temperature) = request.temperature {
            completion_request = completion_request.with_temperature(temperature);
        }

        Ok(completion_request)
    }
}

impl From<CompletionResponse> for WasmCompletionResponse {
    fn from(response: CompletionResponse) -> Self {
        let content = response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();
            
        WasmCompletionResponse {
            content,
            model: response.model,
            tokens_used: response.usage.total_tokens,
            finish_reason: response.choices
                .first()
                .map(|choice| choice.finish_reason.clone().unwrap_or_else(|| "stop".to_string()))
                .unwrap_or_else(|| "stop".to_string()),
        }
    }
}

/// Configuration for the WASM engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEngineConfig {
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub default_model: Option<String>,
    pub log_level: String,
    pub enable_content_filtering: bool,
    pub database_type: String,
}

impl Default for WasmEngineConfig {
    fn default() -> Self {
        Self {
            claude_api_key: None,
            openai_api_key: None,
            default_model: Some("claude-3-haiku-20240307".to_string()),
            log_level: "info".to_string(),
            enable_content_filtering: true,
            database_type: "indexeddb".to_string(),
        }
    }
}

/// Main WriteMagic WASM engine
#[wasm_bindgen]
pub struct WriteMagicEngine {
    core_engine: Option<Arc<CoreEngine>>,
    runtime: Option<Arc<tokio::runtime::Runtime>>,
}

#[wasm_bindgen]
impl WriteMagicEngine {
    /// Create a new WriteMagic engine instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> WriteMagicEngine {
        console::log_1(&"Initializing WriteMagic WASM engine".into());
        WriteMagicEngine {
            core_engine: None,
            runtime: None,
        }
    }

    /// Initialize the engine with configuration
    #[wasm_bindgen]
    pub async fn initialize(&mut self, config: JsValue) -> std::result::Result<(), JsValue> {
        let config_str = config.as_string().unwrap_or_else(|| "{}".to_string());
        console::log_2(&"Initializing with config:".into(), &config_str.into());

        // Parse configuration
        let wasm_config: WasmEngineConfig = serde_json::from_str(&config_str)
            .unwrap_or_else(|_| WasmEngineConfig::default());

        // Create runtime
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .map_err(|e| JsValue::from_str(&format!("Failed to create runtime: {}", e)))?
        );
        
        // Build core engine configuration for WASM with IndexedDB
        let mut app_config = writemagic_writing::ApplicationConfig::default(); // Uses IndexedDB by default in WASM
        app_config.logging.level = wasm_config.log_level.clone();

        // Configure AI if keys are provided
        if let Some(claude_key) = wasm_config.claude_api_key {
            app_config.ai.claude_api_key = Some(claude_key);
        }
        
        if let Some(openai_key) = wasm_config.openai_api_key {
            app_config.ai.openai_api_key = Some(openai_key);
        }

        if let Some(model) = wasm_config.default_model {
            app_config.ai.default_model = model;
        }

        app_config.ai.enable_content_filtering = wasm_config.enable_content_filtering;

        // Initialize core engine with IndexedDB
        let core_engine = runtime.block_on(async {
            writemagic_writing::CoreEngine::new_with_indexeddb(app_config).await
        }).map_err(|e| JsValue::from(WasmError::from(e)))?;

        self.core_engine = Some(Arc::new(core_engine));
        self.runtime = Some(runtime);

        console::log_1(&"WriteMagic engine initialized successfully".into());
        Ok(())
    }

    /// Create a new document
    #[wasm_bindgen]
    pub async fn create_document(
        &self,
        title: String,
        content: String,
        content_type: Option<String>,
        created_by: Option<String>,
    ) -> std::result::Result<WasmDocument, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let document_service = core_engine.document_management_service();
            
            let doc_title = DocumentTitle::new(title)
                .map_err(WritemagicError::ValidationError)?;
            let doc_content = DocumentContent::new(content);
            let content_type = match content_type.as_deref() {
                Some("plain") => ContentType::Plain,
                Some("markdown") => ContentType::Markdown,
                Some("html") => ContentType::Html,
                _ => ContentType::Markdown, // Default
            };
            let created_by_id = created_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid created_by ID: {}", e)))?;

            let aggregate = document_service
                .create_document(doc_title, doc_content, content_type, created_by_id)
                .await?;

            Ok(WasmDocument::from(aggregate.document().clone()))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Update document content
    #[wasm_bindgen]
    pub async fn update_document(
        &self,
        document_id: String,
        content: String,
        updated_by: Option<String>,
    ) -> std::result::Result<WasmDocument, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let document_service = core_engine.document_management_service();
            
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;
            let doc_content = DocumentContent::new(content);
            let updated_by_id = updated_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid updated_by ID: {}", e)))?;

            let aggregate = document_service
                .update_document_content(doc_id, doc_content, None, updated_by_id)
                .await?;

            Ok(WasmDocument::from(aggregate.document().clone()))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Get document by ID
    #[wasm_bindgen]
    pub async fn get_document(
        &self,
        document_id: String,
    ) -> std::result::Result<WasmDocument, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let document_repo = core_engine.document_repository();
            
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;

            let document = document_repo
                .find_by_id(&doc_id)
                .await?
                .ok_or_else(|| WritemagicError::repository("Document not found"))?;

            Ok(WasmDocument::from(document))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Create a new project
    #[wasm_bindgen]
    pub async fn create_project(
        &self,
        name: String,
        description: Option<String>,
        created_by: Option<String>,
    ) -> std::result::Result<WasmProject, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let project_service = core_engine.project_management_service();
            
            let project_name = ProjectName::new(name)
                .map_err(WritemagicError::ValidationError)?;
            let created_by_id = created_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid created_by ID: {}", e)))?;

            let aggregate = project_service
                .create_project(project_name, description, created_by_id)
                .await?;

            Ok(WasmProject::from(aggregate.project().clone()))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Get project by ID
    #[wasm_bindgen]
    pub async fn get_project(
        &self,
        project_id: String,
    ) -> std::result::Result<WasmProject, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let project_repo = core_engine.project_repository();
            
            let proj_id = EntityId::from_string(&project_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid project ID: {}", e)))?;

            let project = project_repo
                .find_by_id(&proj_id)
                .await?
                .ok_or_else(|| WritemagicError::repository("Project not found"))?;

            Ok(WasmProject::from(project))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Complete text using AI
    #[wasm_bindgen]
    pub async fn complete_text(
        &self,
        request: WasmCompletionRequest,
    ) -> std::result::Result<WasmCompletionResponse, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            // Convert WASM request to core request
            let completion_request = request.try_into()
                .map_err(|e: WasmError| WritemagicError::ValidationError(e.message))?;

            // Use the core engine's AI completion
            let ai_service = core_engine.ai_orchestration_service()
                .ok_or_else(|| WritemagicError::configuration("AI service not configured"))?;

            let response = ai_service.complete_with_fallback(completion_request).await?;
            
            Ok(WasmCompletionResponse::from(response))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Check if engine is initialized
    #[wasm_bindgen]
    pub fn is_initialized(&self) -> bool {
        self.core_engine.is_some() && self.runtime.is_some()
    }

    /// Get AI provider health status
    #[wasm_bindgen]
    pub async fn get_ai_provider_health(&self) -> std::result::Result<JsValue, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let health = core_engine.check_ai_provider_health().await?;
            serde_wasm_bindgen::to_value(&health)
                .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e)))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Add document to project
    #[wasm_bindgen]
    pub async fn add_document_to_project(
        &self,
        project_id: String,
        document_id: String,
        updated_by: Option<String>,
    ) -> std::result::Result<WasmProject, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let project_service = core_engine.project_management_service();
            
            let proj_id = EntityId::from_string(&project_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid project ID: {}", e)))?;
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;
            let updated_by_id = updated_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid updated_by ID: {}", e)))?;

            let aggregate = project_service
                .add_document_to_project(proj_id, doc_id, updated_by_id)
                .await?;

            Ok(WasmProject::from(aggregate.project().clone()))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// List documents in a project
    #[wasm_bindgen]
    pub async fn list_project_documents(
        &self,
        project_id: String,
    ) -> std::result::Result<Vec<JsValue>, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let project_repo = core_engine.project_repository();
            let document_repo = core_engine.document_repository();
            
            let proj_id = EntityId::from_string(&project_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid project ID: {}", e)))?;

            let project = project_repo
                .find_by_id(&proj_id)
                .await?
                .ok_or_else(|| WritemagicError::repository("Project not found"))?;

            let mut documents = Vec::new();
            for doc_id in project.document_ids {
                if let Some(document) = document_repo.find_by_id(&doc_id).await? {
                    let wasm_doc = WasmDocument::from(document);
                    documents.push(serde_wasm_bindgen::to_value(&wasm_doc)
                        .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e)))?);
                }
            }

            Ok(documents)
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }
}

impl Default for WriteMagicEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to log messages from WASM
#[wasm_bindgen]
pub fn log(message: &str) {
    console::log_1(&message.into());
}

/// Utility function to log errors from WASM
#[wasm_bindgen]
pub fn log_error(message: &str) {
    console::error_1(&message.into());
}

// Export types for TypeScript definitions
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface WasmDocumentData {
    id: string;
    title: string;
    content: string;
    project_id?: string;
    content_type: string;
    word_count: number;
    character_count: number;
    created_at: string;
    updated_at: string;
    created_by?: string;
    is_deleted: boolean;
}

export interface WasmProjectData {
    id: string;
    name: string;
    description?: string;
    owner_id?: string;
    document_ids: string[];
    created_at: string;
    updated_at: string;
    is_deleted: boolean;
}

export interface WasmCompletionRequestData {
    prompt: string;
    model: string;
    max_tokens?: number;
    temperature?: number;
    context?: string;
}

export interface WasmCompletionResponseData {
    content: string;
    model: string;
    tokens_used: number;
    finish_reason: string;
}

export interface WasmEngineConfig {
    claude_api_key?: string;
    openai_api_key?: string;
    default_model?: string;
    log_level?: string;
    enable_content_filtering?: boolean;
    database_type?: string;
}

export interface AIProviderHealth {
    [provider: string]: boolean;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_engine_creation() {
        let engine = WriteMagicEngine::new();
        assert!(!engine.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_error_conversion() {
        let error = WritemagicError::ValidationError("Test error".to_string());
        let wasm_error: WasmError = error.into();
        assert_eq!(wasm_error.message(), "Test error");
        assert_eq!(wasm_error.code(), "VALIDATION_ERROR");
    }

    #[wasm_bindgen_test]
    fn test_wasm_config_default() {
        let config = WasmEngineConfig::default();
        assert_eq!(config.log_level, "info");
        assert!(config.enable_content_filtering);
        assert_eq!(config.database_type, "indexeddb");
    }
}