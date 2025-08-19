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

/// In-memory document repository implementation
#[derive(Debug, Clone)]
pub struct InMemoryDocumentRepository {
    base: writemagic_shared::InMemoryRepository<Document>,
}

impl InMemoryDocumentRepository {
    pub fn new() -> Self {
        Self {
            base: writemagic_shared::InMemoryRepository::new(),
        }
    }
}

impl Default for InMemoryDocumentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Repository<Document, EntityId> for InMemoryDocumentRepository {
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Document>> {
        self.base.find_by_id(id).await
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<Document>> {
        self.base.find_all(pagination).await
    }

    async fn save(&self, entity: &Document) -> Result<Document> {
        self.base.save(entity).await
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        self.base.delete(id).await
    }

    async fn exists(&self, id: &EntityId) -> Result<bool> {
        self.base.exists(id).await
    }

    async fn count(&self) -> Result<u64> {
        self.base.count().await
    }
}

#[async_trait]
impl DocumentRepository for InMemoryDocumentRepository {
    async fn find_by_project_id(&self, _project_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>> {
        // For in-memory implementation, return all for now
        // In a real implementation, this would filter by project_id
        self.find_all(pagination).await
    }

    async fn find_by_content_type(&self, content_type: &writemagic_shared::ContentType, pagination: Pagination) -> Result<Vec<Document>> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .filter(|doc| &doc.content_type == content_type)
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn search_by_title(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let query_lower = query.to_lowercase();
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .filter(|doc| doc.title.to_lowercase().contains(&query_lower))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn search_by_content(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let query_lower = query.to_lowercase();
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .filter(|doc| doc.content.to_lowercase().contains(&query_lower))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .filter(|doc| doc.created_by.as_ref() == Some(user_id))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Document>> {
        let mut all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        all_docs.sort_by(|a, b| b.updated_at.0.cmp(&a.updated_at.0));
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn find_deleted(&self, pagination: Pagination) -> Result<Vec<Document>> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let filtered: Vec<Document> = all_docs
            .into_iter()
            .filter(|doc| doc.is_deleted)
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn get_statistics(&self) -> Result<DocumentStatistics> {
        let all_docs = self.find_all(Pagination::new(0, 10000)?).await?;
        let total_documents = all_docs.len() as u64;
        let total_word_count: u64 = all_docs.iter().map(|doc| doc.word_count as u64).sum();
        let total_character_count: u64 = all_docs.iter().map(|doc| doc.character_count as u64).sum();
        let deleted_documents = all_docs.iter().filter(|doc| doc.is_deleted).count() as u64;

        let mut documents_by_type = std::collections::HashMap::new();
        for doc in &all_docs {
            let type_str = doc.content_type.to_string();
            *documents_by_type.entry(type_str).or_insert(0) += 1;
        }

        let average_word_count = if total_documents > 0 {
            total_word_count as f64 / total_documents as f64
        } else {
            0.0
        };

        let average_character_count = if total_documents > 0 {
            total_character_count as f64 / total_documents as f64
        } else {
            0.0
        };

        Ok(DocumentStatistics {
            total_documents,
            total_word_count,
            total_character_count,
            documents_by_type,
            average_word_count,
            average_character_count,
            deleted_documents,
        })
    }
}

/// In-memory project repository implementation
#[derive(Debug, Clone)]
pub struct InMemoryProjectRepository {
    base: writemagic_shared::InMemoryRepository<Project>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            base: writemagic_shared::InMemoryRepository::new(),
        }
    }
}

impl Default for InMemoryProjectRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Repository<Project, EntityId> for InMemoryProjectRepository {
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Project>> {
        self.base.find_by_id(id).await
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<Project>> {
        self.base.find_all(pagination).await
    }

    async fn save(&self, entity: &Project) -> Result<Project> {
        self.base.save(entity).await
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        self.base.delete(id).await
    }

    async fn exists(&self, id: &EntityId) -> Result<bool> {
        self.base.exists(id).await
    }

    async fn count(&self) -> Result<u64> {
        self.base.count().await
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>> {
        let all_projects = self.find_all(Pagination::new(0, 10000)?).await?;
        let filtered: Vec<Project> = all_projects
            .into_iter()
            .filter(|project| project.created_by.as_ref() == Some(user_id))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn search_by_name(&self, query: &str, pagination: Pagination) -> Result<Vec<Project>> {
        let all_projects = self.find_all(Pagination::new(0, 10000)?).await?;
        let query_lower = query.to_lowercase();
        let filtered: Vec<Project> = all_projects
            .into_iter()
            .filter(|project| project.name.to_lowercase().contains(&query_lower))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn find_containing_document(&self, document_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>> {
        let all_projects = self.find_all(Pagination::new(0, 10000)?).await?;
        let filtered: Vec<Project> = all_projects
            .into_iter()
            .filter(|project| project.document_ids.contains(document_id))
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Project>> {
        let mut all_projects = self.find_all(Pagination::new(0, 10000)?).await?;
        all_projects.sort_by(|a, b| b.updated_at.0.cmp(&a.updated_at.0));
        let filtered: Vec<Project> = all_projects
            .into_iter()
            .skip(pagination.offset as usize)
            .take(pagination.limit as usize)
            .collect();
        Ok(filtered)
    }

    async fn get_statistics(&self) -> Result<ProjectStatistics> {
        let all_projects = self.find_all(Pagination::new(0, 10000)?).await?;
        let total_projects = all_projects.len() as u64;
        let total_documents_in_projects: u64 = all_projects
            .iter()
            .map(|project| project.document_ids.len() as u64)
            .sum();

        let average_documents_per_project = if total_projects > 0 {
            total_documents_in_projects as f64 / total_projects as f64
        } else {
            0.0
        };

        let largest_project_size = all_projects
            .iter()
            .map(|project| project.document_ids.len() as u64)
            .max()
            .unwrap_or(0);

        let smallest_project_size = all_projects
            .iter()
            .map(|project| project.document_ids.len() as u64)
            .min()
            .unwrap_or(0);

        Ok(ProjectStatistics {
            total_projects,
            total_documents_in_projects,
            average_documents_per_project,
            largest_project_size,
            smallest_project_size,
        })
    }
}