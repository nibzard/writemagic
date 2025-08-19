//! Cross-domain services and coordination

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::{EntityId, Result, WritemagicError, DomainEvent, EventBus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service registry for managing cross-domain services
pub struct CrossDomainServiceRegistry {
    writing_service: Option<Arc<dyn WritingDomainService>>,
    ai_service: Option<Arc<dyn AIDomainService>>,
    project_service: Option<Arc<dyn ProjectDomainService>>,
    version_control_service: Option<Arc<dyn VersionControlDomainService>>,
    agent_service: Option<Arc<dyn AgentDomainService>>,
    event_bus: Arc<dyn EventBus>,
}

impl CrossDomainServiceRegistry {
    /// Create a new service registry
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            writing_service: None,
            ai_service: None,
            project_service: None,
            version_control_service: None,
            agent_service: None,
            event_bus,
        }
    }
    
    /// Register writing domain service
    pub fn register_writing_service(&mut self, service: Arc<dyn WritingDomainService>) {
        self.writing_service = Some(service);
    }
    
    /// Register AI domain service
    pub fn register_ai_service(&mut self, service: Arc<dyn AIDomainService>) {
        self.ai_service = Some(service);
    }
    
    /// Register project domain service
    pub fn register_project_service(&mut self, service: Arc<dyn ProjectDomainService>) {
        self.project_service = Some(service);
    }
    
    /// Register version control domain service
    pub fn register_version_control_service(&mut self, service: Arc<dyn VersionControlDomainService>) {
        self.version_control_service = Some(service);
    }
    
    /// Register agent domain service
    pub fn register_agent_service(&mut self, service: Arc<dyn AgentDomainService>) {
        self.agent_service = Some(service);
    }
    
    /// Get writing service
    pub fn writing_service(&self) -> Result<&Arc<dyn WritingDomainService>> {
        self.writing_service.as_ref()
            .ok_or_else(|| WritemagicError::not_found("Writing service not registered"))
    }
    
    /// Get AI service
    pub fn ai_service(&self) -> Result<&Arc<dyn AIDomainService>> {
        self.ai_service.as_ref()
            .ok_or_else(|| WritemagicError::not_found("AI service not registered"))
    }
    
    /// Get project service
    pub fn project_service(&self) -> Result<&Arc<dyn ProjectDomainService>> {
        self.project_service.as_ref()
            .ok_or_else(|| WritemagicError::not_found("Project service not registered"))
    }
    
    /// Get version control service
    pub fn version_control_service(&self) -> Result<&Arc<dyn VersionControlDomainService>> {
        self.version_control_service.as_ref()
            .ok_or_else(|| WritemagicError::not_found("Version control service not registered"))
    }
    
    /// Get agent service
    pub fn agent_service(&self) -> Result<&Arc<dyn AgentDomainService>> {
        self.agent_service.as_ref()
            .ok_or_else(|| WritemagicError::not_found("Agent service not registered"))
    }
    
    /// Get event bus
    pub fn event_bus(&self) -> &Arc<dyn EventBus> {
        &self.event_bus
    }
}

/// Writing domain service interface
#[async_trait]
pub trait WritingDomainService: Send + Sync {
    /// Create a new document
    async fn create_document(&self, request: CreateDocumentRequest) -> Result<DocumentInfo>;
    
    /// Get document by ID
    async fn get_document(&self, document_id: &EntityId) -> Result<Option<DocumentInfo>>;
    
    /// Update document content
    async fn update_document(&self, request: UpdateDocumentRequest) -> Result<()>;
    
    /// Delete document
    async fn delete_document(&self, document_id: &EntityId) -> Result<()>;
    
    /// Search documents
    async fn search_documents(&self, query: &str, limit: Option<u32>) -> Result<Vec<DocumentInfo>>;
    
    /// Get document statistics
    async fn get_document_stats(&self, document_id: &EntityId) -> Result<DocumentStats>;
}

/// AI domain service interface
#[async_trait]
pub trait AIDomainService: Send + Sync {
    /// Generate content with AI
    async fn generate_content(&self, request: AIGenerationRequest) -> Result<AIGenerationResponse>;
    
    /// Analyze document with AI
    async fn analyze_document(&self, document_id: &EntityId, analysis_type: AnalysisType) -> Result<DocumentAnalysis>;
    
    /// Get AI suggestions for writing
    async fn get_writing_suggestions(&self, request: WritingSuggestionsRequest) -> Result<Vec<WritingSuggestion>>;
    
    /// Process document with AI workflow
    async fn process_document_workflow(&self, request: AIWorkflowRequest) -> Result<AIWorkflowResult>;
}

/// Project domain service interface
#[async_trait]
pub trait ProjectDomainService: Send + Sync {
    /// Create a new project
    async fn create_project(&self, request: CreateProjectRequest) -> Result<ProjectInfo>;
    
    /// Get project by ID
    async fn get_project(&self, project_id: &EntityId) -> Result<Option<ProjectInfo>>;
    
    /// Add document to project
    async fn add_document_to_project(&self, project_id: &EntityId, document_id: &EntityId) -> Result<()>;
    
    /// Remove document from project
    async fn remove_document_from_project(&self, project_id: &EntityId, document_id: &EntityId) -> Result<()>;
    
    /// Get project documents
    async fn get_project_documents(&self, project_id: &EntityId) -> Result<Vec<DocumentInfo>>;
    
    /// Update project settings
    async fn update_project(&self, request: UpdateProjectRequest) -> Result<()>;
}

/// Version control domain service interface
#[async_trait]
pub trait VersionControlDomainService: Send + Sync {
    /// Create a commit
    async fn create_commit(&self, request: CreateCommitRequest) -> Result<CommitInfo>;
    
    /// Get commit history
    async fn get_commit_history(&self, document_id: &EntityId, limit: Option<u32>) -> Result<Vec<CommitInfo>>;
    
    /// Create a branch
    async fn create_branch(&self, request: CreateBranchRequest) -> Result<BranchInfo>;
    
    /// Merge branches
    async fn merge_branches(&self, request: MergeBranchesRequest) -> Result<MergeResult>;
    
    /// Get document diff
    async fn get_document_diff(&self, from_commit: &EntityId, to_commit: &EntityId) -> Result<DocumentDiff>;
}

/// Agent domain service interface
#[async_trait]
pub trait AgentDomainService: Send + Sync {
    /// Create and start an agent
    async fn create_agent(&self, request: CreateAgentRequest) -> Result<AgentInfo>;
    
    /// Trigger agent execution
    async fn trigger_agent(&self, agent_id: &EntityId, context: AgentContext) -> Result<ExecutionResult>;
    
    /// Get agent status
    async fn get_agent_status(&self, agent_id: &EntityId) -> Result<AgentStatus>;
    
    /// List active agents
    async fn list_active_agents(&self) -> Result<Vec<AgentInfo>>;
    
    /// Get agent execution history
    async fn get_agent_executions(&self, agent_id: &EntityId, limit: Option<u32>) -> Result<Vec<ExecutionResult>>;
}

/// Cross-domain coordination service
pub struct CrossDomainCoordinator {
    registry: Arc<CrossDomainServiceRegistry>,
}

impl CrossDomainCoordinator {
    /// Create a new coordinator
    pub fn new(registry: Arc<CrossDomainServiceRegistry>) -> Self {
        Self { registry }
    }
    
    /// Create document with project association
    pub async fn create_document_in_project(
        &self,
        project_id: &EntityId,
        request: CreateDocumentRequest,
    ) -> Result<DocumentInfo> {
        let writing_service = self.registry.writing_service()?;
        let project_service = self.registry.project_service()?;
        
        // Create document
        let document = writing_service.create_document(request).await?;
        
        // Add to project
        project_service.add_document_to_project(project_id, &document.id).await?;
        
        Ok(document)
    }
    
    /// Generate AI content and save as document
    pub async fn generate_and_save_document(
        &self,
        generation_request: AIGenerationRequest,
        project_id: Option<&EntityId>,
    ) -> Result<DocumentInfo> {
        let ai_service = self.registry.ai_service()?;
        let writing_service = self.registry.writing_service()?;
        
        // Generate content
        let generation_result = ai_service.generate_content(generation_request).await?;
        
        // Create document
        let document_request = CreateDocumentRequest {
            title: generation_result.title.unwrap_or_else(|| "AI Generated Document".to_string()),
            content: generation_result.content,
            project_id: project_id.copied(),
            metadata: generation_result.metadata,
        };
        
        let document = writing_service.create_document(document_request).await?;
        
        // Add to project if specified
        if let Some(project_id) = project_id {
            let project_service = self.registry.project_service()?;
            project_service.add_document_to_project(project_id, &document.id).await?;
        }
        
        Ok(document)
    }
    
    /// Create commit with AI analysis
    pub async fn create_analyzed_commit(
        &self,
        document_id: &EntityId,
        commit_message: String,
    ) -> Result<CommitInfo> {
        let ai_service = self.registry.ai_service()?;
        let version_control_service = self.registry.version_control_service()?;
        
        // Analyze document before commit
        let analysis = ai_service.analyze_document(document_id, AnalysisType::ContentQuality).await?;
        
        // Create commit with analysis metadata
        let mut metadata = HashMap::new();
        metadata.insert("ai_quality_score".to_string(), analysis.score.to_string());
        metadata.insert("ai_analysis_summary".to_string(), analysis.summary);
        
        let commit_request = CreateCommitRequest {
            document_id: *document_id,
            message: commit_message,
            metadata,
        };
        
        version_control_service.create_commit(commit_request).await
    }
}

// Data structures for cross-domain operations

/// Document information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: EntityId,
    pub title: String,
    pub content: String,
    pub project_id: Option<EntityId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Document statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    pub word_count: u32,
    pub character_count: u32,
    pub paragraph_count: u32,
    pub reading_time_minutes: u32,
    pub last_modified: DateTime<Utc>,
}

/// Create document request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub project_id: Option<EntityId>,
    pub metadata: HashMap<String, String>,
}

/// Update document request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub document_id: EntityId,
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// AI generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGenerationRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub context: Option<String>,
    pub style: Option<String>,
}

/// AI generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGenerationResponse {
    pub content: String,
    pub title: Option<String>,
    pub metadata: HashMap<String, String>,
    pub tokens_used: u32,
    pub processing_time_ms: u64,
}

/// Analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    ContentQuality,
    Grammar,
    Style,
    Readability,
    Sentiment,
}

/// Document analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAnalysis {
    pub analysis_type: AnalysisType,
    pub score: f32,
    pub summary: String,
    pub details: HashMap<String, String>,
    pub suggestions: Vec<String>,
}

/// Writing suggestions request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSuggestionsRequest {
    pub document_id: EntityId,
    pub suggestion_types: Vec<String>,
    pub context: Option<String>,
}

/// Writing suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub original_text: Option<String>,
    pub suggested_text: Option<String>,
    pub confidence: f32,
}

/// AI workflow request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowRequest {
    pub document_id: EntityId,
    pub workflow_type: String,
    pub parameters: HashMap<String, String>,
}

/// AI workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowResult {
    pub workflow_type: String,
    pub success: bool,
    pub outputs: HashMap<String, String>,
    pub duration_ms: u64,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: EntityId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub document_count: u32,
}

/// Create project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub template: Option<String>,
}

/// Update project request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub project_id: EntityId,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: EntityId,
    pub document_id: EntityId,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Create commit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommitRequest {
    pub document_id: EntityId,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub id: EntityId,
    pub name: String,
    pub document_id: EntityId,
    pub created_at: DateTime<Utc>,
    pub head_commit_id: EntityId,
}

/// Create branch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub document_id: EntityId,
    pub from_commit_id: Option<EntityId>,
}

/// Merge branches request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeBranchesRequest {
    pub source_branch_id: EntityId,
    pub target_branch_id: EntityId,
    pub merge_message: String,
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub merge_commit_id: Option<EntityId>,
}

/// Document diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDiff {
    pub from_commit_id: EntityId,
    pub to_commit_id: EntityId,
    pub changes: Vec<DiffChange>,
}

/// Diff change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub change_type: DiffChangeType,
    pub line_number: u32,
    pub content: String,
}

/// Diff change type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffChangeType {
    Added,
    Removed,
    Modified,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: EntityId,
    pub name: String,
    pub description: Option<String>,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Paused,
    Disabled,
    Running,
    Error,
}

/// Create agent request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub workflow_definition: String,
    pub description: Option<String>,
}

/// Agent context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub trigger_type: String,
    pub variables: HashMap<String, String>,
    pub document_id: Option<EntityId>,
    pub project_id: Option<EntityId>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: EntityId,
    pub agent_id: EntityId,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outputs: HashMap<String, String>,
    pub error_message: Option<String>,
}
