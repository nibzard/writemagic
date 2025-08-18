//! Writing domain repositories

use async_trait::async_trait;
use writemagic_shared::{EntityId, Pagination, Repository, Result};
use crate::entities::{Document, Project};

/// Document repository interface
#[async_trait]
pub trait DocumentRepository: Repository<Document, EntityId> + Send + Sync {
    /// Find documents by project ID
    async fn find_by_project_id(&self, project_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>>;

    /// Find documents by content type
    async fn find_by_content_type(&self, content_type: &writemagic_shared::ContentType, pagination: Pagination) -> Result<Vec<Document>>;

    /// Search documents by title
    async fn search_by_title(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>>;

    /// Search documents by content
    async fn search_by_content(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>>;

    /// Find documents created by user
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>>;

    /// Find recently updated documents
    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Document>>;

    /// Find deleted documents
    async fn find_deleted(&self, pagination: Pagination) -> Result<Vec<Document>>;

    /// Get document statistics
    async fn get_statistics(&self) -> Result<DocumentStatistics>;
}

/// Project repository interface
#[async_trait]
pub trait ProjectRepository: Repository<Project, EntityId> + Send + Sync {
    /// Find projects by creator
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>>;

    /// Search projects by name
    async fn search_by_name(&self, query: &str, pagination: Pagination) -> Result<Vec<Project>>;

    /// Find projects containing document
    async fn find_containing_document(&self, document_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>>;

    /// Find recently updated projects
    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Project>>;

    /// Get project statistics
    async fn get_statistics(&self) -> Result<ProjectStatistics>;
}

/// Document repository statistics
#[derive(Debug, Clone)]
pub struct DocumentStatistics {
    pub total_documents: u64,
    pub total_word_count: u64,
    pub total_character_count: u64,
    pub documents_by_type: std::collections::HashMap<String, u64>,
    pub average_word_count: f64,
    pub average_character_count: f64,
    pub deleted_documents: u64,
}

/// Project repository statistics
#[derive(Debug, Clone)]
pub struct ProjectStatistics {
    pub total_projects: u64,
    pub total_documents_in_projects: u64,
    pub average_documents_per_project: f64,
    pub largest_project_size: u64,
    pub smallest_project_size: u64,
}

/// Document search criteria
#[derive(Debug, Clone)]
pub struct DocumentSearchCriteria {
    pub title_query: Option<String>,
    pub content_query: Option<String>,
    pub content_types: Vec<writemagic_shared::ContentType>,
    pub creator_ids: Vec<EntityId>,
    pub project_ids: Vec<EntityId>,
    pub created_after: Option<writemagic_shared::Timestamp>,
    pub created_before: Option<writemagic_shared::Timestamp>,
    pub updated_after: Option<writemagic_shared::Timestamp>,
    pub updated_before: Option<writemagic_shared::Timestamp>,
    pub min_word_count: Option<u32>,
    pub max_word_count: Option<u32>,
    pub include_deleted: bool,
}

impl Default for DocumentSearchCriteria {
    fn default() -> Self {
        Self {
            title_query: None,
            content_query: None,
            content_types: Vec::new(),
            creator_ids: Vec::new(),
            project_ids: Vec::new(),
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            min_word_count: None,
            max_word_count: None,
            include_deleted: false,
        }
    }
}

/// Advanced document repository with search capabilities
#[async_trait]
pub trait AdvancedDocumentRepository: DocumentRepository {
    /// Advanced search with multiple criteria
    async fn search(&self, criteria: DocumentSearchCriteria, pagination: Pagination) -> Result<Vec<Document>>;

    /// Count documents matching criteria
    async fn count_by_criteria(&self, criteria: DocumentSearchCriteria) -> Result<u64>;

    /// Find similar documents based on content
    async fn find_similar(&self, document_id: &EntityId, limit: u32) -> Result<Vec<Document>>;

    /// Get document word frequency analysis
    async fn get_word_frequency(&self, document_id: &EntityId) -> Result<std::collections::HashMap<String, u32>>;
}

/// Project search criteria
#[derive(Debug, Clone)]
pub struct ProjectSearchCriteria {
    pub name_query: Option<String>,
    pub description_query: Option<String>,
    pub creator_ids: Vec<EntityId>,
    pub created_after: Option<writemagic_shared::Timestamp>,
    pub created_before: Option<writemagic_shared::Timestamp>,
    pub updated_after: Option<writemagic_shared::Timestamp>,
    pub updated_before: Option<writemagic_shared::Timestamp>,
    pub min_document_count: Option<u32>,
    pub max_document_count: Option<u32>,
    pub contains_document_ids: Vec<EntityId>,
}

impl Default for ProjectSearchCriteria {
    fn default() -> Self {
        Self {
            name_query: None,
            description_query: None,
            creator_ids: Vec::new(),
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            min_document_count: None,
            max_document_count: None,
            contains_document_ids: Vec::new(),
        }
    }
}

/// Advanced project repository with search capabilities
#[async_trait]
pub trait AdvancedProjectRepository: ProjectRepository {
    /// Advanced search with multiple criteria
    async fn search(&self, criteria: ProjectSearchCriteria, pagination: Pagination) -> Result<Vec<Project>>;

    /// Count projects matching criteria
    async fn count_by_criteria(&self, criteria: ProjectSearchCriteria) -> Result<u64>;

    /// Get project collaboration statistics
    async fn get_collaboration_stats(&self, project_id: &EntityId) -> Result<CollaborationStatistics>;
}

/// Collaboration statistics for a project
#[derive(Debug, Clone)]
pub struct CollaborationStatistics {
    pub total_contributors: u32,
    pub total_edits: u32,
    pub edits_by_contributor: std::collections::HashMap<EntityId, u32>,
    pub most_active_contributor: Option<EntityId>,
    pub last_activity: Option<writemagic_shared::Timestamp>,
}