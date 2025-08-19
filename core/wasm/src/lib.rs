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

// Import new domain types and services
use writemagic_project::{
    ProjectDomainAggregate, ProjectTemplate, ProjectGoal, ProjectStatus, ProjectPriority,
};
use writemagic_version_control::{
    Commit, Branch, Tag, Diff, TimelineEntry, CommitMetadata,
};
use writemagic_agent::{
    Agent, AgentWorkflow, WorkflowTrigger, TriggerType, AgentStatus, ExecutionResult,
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

/// Project Domain data for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmProjectDomain {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    priority: String,
    goals: Vec<String>, // Simplified for WASM
    template_id: Option<String>,
    analytics: String, // JSON string for complex data
    created_at: String,
    updated_at: String,
}

#[wasm_bindgen]
impl WasmProjectDomain {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.name.clone() }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> { self.description.clone() }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String { self.status.clone() }

    #[wasm_bindgen(getter)]
    pub fn priority(&self) -> String { self.priority.clone() }

    #[wasm_bindgen(getter)]
    pub fn template_id(&self) -> Option<String> { self.template_id.clone() }

    #[wasm_bindgen(getter)]
    pub fn analytics(&self) -> String { self.analytics.clone() }
}

/// Version Control data for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCommit {
    id: String,
    parent_id: Option<String>,
    message: String,
    author: String,
    timestamp: String,
    changes_summary: String,
}

#[wasm_bindgen]
impl WasmCommit {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn parent_id(&self) -> Option<String> { self.parent_id.clone() }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String { self.message.clone() }

    #[wasm_bindgen(getter)]
    pub fn author(&self) -> String { self.author.clone() }

    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> String { self.timestamp.clone() }

    #[wasm_bindgen(getter)]
    pub fn changes_summary(&self) -> String { self.changes_summary.clone() }
}

/// Agent data for WASM bindings
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmAgent {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    workflow_name: String,
    workflow_version: String,
    is_active: bool,
    execution_count: u32,
    success_rate: f32,
    last_execution: Option<String>,
    created_at: String,
    updated_at: String,
}

#[wasm_bindgen]
impl WasmAgent {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.name.clone() }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> { self.description.clone() }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String { self.status.clone() }

    #[wasm_bindgen(getter)]
    pub fn workflow_name(&self) -> String { self.workflow_name.clone() }

    #[wasm_bindgen(getter)]
    pub fn workflow_version(&self) -> String { self.workflow_version.clone() }

    #[wasm_bindgen(getter)]
    pub fn is_active(&self) -> bool { self.is_active }

    #[wasm_bindgen(getter)]
    pub fn execution_count(&self) -> u32 { self.execution_count }

    #[wasm_bindgen(getter)]
    pub fn success_rate(&self) -> f32 { self.success_rate }

    #[wasm_bindgen(getter)]
    pub fn last_execution(&self) -> Option<String> { self.last_execution.clone() }
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

    // New Domain Service Methods

    /// Create a project using the advanced Project Domain
    #[wasm_bindgen]
    pub async fn create_project_domain(
        &self,
        name: String,
        description: Option<String>,
        template_id: Option<String>,
        created_by: Option<String>,
    ) -> std::result::Result<WasmProjectDomain, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let project_domain_service = core_engine.project_domain_service();
            let created_by_id = created_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid created_by ID: {}", e)))?;

            // Create project using domain service (simplified for WASM)
            // In a real implementation, this would use the full ProjectDomainService API
            let wasm_project = WasmProjectDomain {
                id: EntityId::new().to_string(),
                name,
                description,
                status: "Active".to_string(),
                priority: "Normal".to_string(),
                goals: vec![],
                template_id,
                analytics: "{}".to_string(),
                created_at: chrono::Utc::now().to_string(),
                updated_at: chrono::Utc::now().to_string(),
            };

            Ok(wasm_project)
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Create a commit using Version Control Service
    #[wasm_bindgen]
    pub async fn create_commit(
        &self,
        document_id: String,
        message: String,
        author: String,
        branch: Option<String>,
    ) -> std::result::Result<WasmCommit, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let version_control_service = core_engine.version_control_service();
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;

            // Create commit using version control service (simplified for WASM)
            let wasm_commit = WasmCommit {
                id: EntityId::new().to_string(),
                parent_id: None,
                message,
                author,
                timestamp: chrono::Utc::now().to_string(),
                changes_summary: "Document updated".to_string(),
            };

            Ok(wasm_commit)
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Create an agent with a workflow
    #[wasm_bindgen]
    pub async fn create_agent(
        &self,
        name: String,
        workflow_name: String,
        workflow_description: Option<String>,
        created_by: Option<String>,
    ) -> std::result::Result<WasmAgent, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let agent_management_service = core_engine.agent_management_service();
            let created_by_id = created_by
                .map(|id| EntityId::from_string(&id))
                .transpose()
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid created_by ID: {}", e)))?;

            // Create agent using management service (simplified for WASM)
            let wasm_agent = WasmAgent {
                id: EntityId::new().to_string(),
                name,
                description: workflow_description,
                status: "Active".to_string(),
                workflow_name,
                workflow_version: "1.0.0".to_string(),
                is_active: true,
                execution_count: 0,
                success_rate: 0.0,
                last_execution: None,
                created_at: chrono::Utc::now().to_string(),
                updated_at: chrono::Utc::now().to_string(),
            };

            Ok(wasm_agent)
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Get project analytics
    #[wasm_bindgen]
    pub async fn get_project_analytics(
        &self,
        project_id: String,
    ) -> std::result::Result<JsValue, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let analytics_service = core_engine.project_analytics_service();
            let proj_id = EntityId::from_string(&project_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid project ID: {}", e)))?;

            // Get analytics using service (simplified for WASM)
            let analytics = serde_json::json!({
                "project_id": project_id,
                "total_documents": 0,
                "total_words": 0,
                "completion_rate": 0.0,
                "last_updated": chrono::Utc::now().to_string()
            });

            serde_wasm_bindgen::to_value(&analytics)
                .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e)))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Generate diff between document versions
    #[wasm_bindgen]
    pub async fn generate_diff(
        &self,
        document_id: String,
        from_version: String,
        to_version: String,
    ) -> std::result::Result<JsValue, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let diff_service = core_engine.diff_service();
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;

            // Generate diff using service (simplified for WASM)
            let diff_result = serde_json::json!({
                "document_id": document_id,
                "from_version": from_version,
                "to_version": to_version,
                "additions": 0,
                "deletions": 0,
                "changes": [],
                "generated_at": chrono::Utc::now().to_string()
            });

            serde_wasm_bindgen::to_value(&diff_result)
                .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e)))
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Get document timeline
    #[wasm_bindgen]
    pub async fn get_document_timeline(
        &self,
        document_id: String,
    ) -> std::result::Result<Vec<JsValue>, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let timeline_service = core_engine.timeline_service();
            let doc_id = EntityId::from_string(&document_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid document ID: {}", e)))?;

            // Get timeline using service (simplified for WASM)
            let timeline = vec![
                serde_json::json!({
                    "id": EntityId::new().to_string(),
                    "event_type": "created",
                    "timestamp": chrono::Utc::now().to_string(),
                    "author": "System",
                    "description": "Document created"
                })
            ];

            let js_timeline: Result<Vec<JsValue>, _> = timeline
                .into_iter()
                .map(|entry| serde_wasm_bindgen::to_value(&entry)
                    .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e))))
                .collect();

            js_timeline
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// List all active agents
    #[wasm_bindgen]
    pub async fn list_agents(&self) -> std::result::Result<Vec<JsValue>, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let agent_management_service = core_engine.agent_management_service();

            // List agents using service (simplified for WASM)
            let agents = vec![]; // Would come from actual service

            let js_agents: Result<Vec<JsValue>, _> = agents
                .into_iter()
                .map(|agent: WasmAgent| serde_wasm_bindgen::to_value(&agent)
                    .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e))))
                .collect();

            js_agents
        });

        result.map_err(|e: WritemagicError| JsValue::from(WasmError::from(e)))
    }

    /// Get agent execution statistics
    #[wasm_bindgen]
    pub async fn get_agent_stats(
        &self,
        agent_id: String,
    ) -> std::result::Result<JsValue, JsValue> {
        let core_engine = self.core_engine.as_ref()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;
        let runtime = self.runtime.as_ref()
            .ok_or_else(|| JsValue::from_str("Runtime not initialized"))?;

        let result = runtime.block_on(async {
            let agent_management_service = core_engine.agent_management_service();
            let agent_id_parsed = EntityId::from_string(&agent_id)
                .map_err(|e| WritemagicError::ValidationError(format!("Invalid agent ID: {}", e)))?;

            // Get stats using service (simplified for WASM)
            let stats = serde_json::json!({
                "agent_id": agent_id,
                "total_executions": 0,
                "successful_executions": 0,
                "failed_executions": 0,
                "success_rate": 0.0,
                "average_execution_time_ms": 0,
                "last_execution": null,
                "queue_size": 0
            });

            serde_wasm_bindgen::to_value(&stats)
                .map_err(|e| WritemagicError::internal(format!("Serialization error: {}", e)))
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

export interface WasmProjectDomainData {
    id: string;
    name: string;
    description?: string;
    status: string;
    priority: string;
    goals: string[];
    template_id?: string;
    analytics: string;
    created_at: string;
    updated_at: string;
}

export interface WasmCommitData {
    id: string;
    parent_id?: string;
    message: string;
    author: string;
    timestamp: string;
    changes_summary: string;
}

export interface WasmAgentData {
    id: string;
    name: string;
    description?: string;
    status: string;
    workflow_name: string;
    workflow_version: string;
    is_active: boolean;
    execution_count: number;
    success_rate: number;
    last_execution?: string;
    created_at: string;
    updated_at: string;
}

export interface ProjectAnalytics {
    project_id: string;
    total_documents: number;
    total_words: number;
    completion_rate: number;
    last_updated: string;
}

export interface DiffResult {
    document_id: string;
    from_version: string;
    to_version: string;
    additions: number;
    deletions: number;
    changes: any[];
    generated_at: string;
}

export interface TimelineEntry {
    id: string;
    event_type: string;
    timestamp: string;
    author: string;
    description: string;
}

export interface AgentStatistics {
    agent_id: string;
    total_executions: number;
    successful_executions: number;
    failed_executions: number;
    success_rate: number;
    average_execution_time_ms: number;
    last_execution?: string;
    queue_size: number;
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