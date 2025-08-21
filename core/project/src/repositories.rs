//! Project domain repositories

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::aggregates::{ProjectAggregate, ProjectEvent};
use crate::entities::ProjectTemplate;
use crate::value_objects::{ProjectStatus, ProjectPriority};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Repository trait for project persistence
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Save a project aggregate
    async fn save(&self, aggregate: &mut ProjectAggregate) -> Result<()>;
    
    /// Load a project aggregate by ID
    async fn load(&self, project_id: &EntityId) -> Result<Option<ProjectAggregate>>;
    
    /// Delete a project
    async fn delete(&self, project_id: &EntityId) -> Result<()>;
    
    /// List projects with optional filtering
    async fn list(&self, filter: ProjectFilter) -> Result<Vec<ProjectAggregate>>;
    
    /// Search projects by criteria
    async fn search(&self, criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>>;
    
    /// Get project statistics
    async fn get_statistics(&self, project_id: &EntityId) -> Result<ProjectStatistics>;
    
    /// Check if project exists
    async fn exists(&self, project_id: &EntityId) -> Result<bool>;
}

/// Repository trait for project events (event sourcing)
#[async_trait]
pub trait ProjectEventRepository: Send + Sync {
    /// Save project events
    async fn save_events(&self, project_id: &EntityId, events: &[ProjectEvent], expected_version: u64) -> Result<()>;
    
    /// Load all events for a project
    async fn load_events(&self, project_id: &EntityId) -> Result<Vec<ProjectEvent>>;
    
    /// Load events from a specific version
    async fn load_events_from_version(&self, project_id: &EntityId, from_version: u64) -> Result<Vec<ProjectEvent>>;
    
    /// Get the latest version for a project
    async fn get_latest_version(&self, project_id: &EntityId) -> Result<u64>;
}

/// Repository trait for project templates
#[async_trait]
pub trait ProjectTemplateRepository: Send + Sync {
    /// Save a project template
    async fn save_template(&self, template: &ProjectTemplate) -> Result<()>;
    
    /// Load a project template by name
    async fn load_template(&self, name: &str) -> Result<Option<ProjectTemplate>>;
    
    /// List all available templates
    async fn list_templates(&self) -> Result<Vec<ProjectTemplate>>;
    
    /// Delete a template
    async fn delete_template(&self, name: &str) -> Result<()>;
}

/// Filter criteria for listing projects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectFilter {
    pub status: Option<ProjectStatus>,
    pub priority: Option<ProjectPriority>,
    pub created_by: Option<EntityId>,
    pub tags: Vec<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub is_archived: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<ProjectSortBy>,
    pub sort_order: Option<SortOrder>,
}

/// Search criteria for projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSearchCriteria {
    pub query: String,
    pub search_in_name: bool,
    pub search_in_description: bool,
    pub search_in_tags: bool,
    pub filter: Option<ProjectFilter>,
}

/// Project sorting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectSortBy {
    Name,
    CreatedAt,
    UpdatedAt,
    Priority,
    DocumentCount,
    LastActivity,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Project statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_projects: usize,
    pub active_projects: usize,
    pub completed_projects: usize,
    pub archived_projects: usize,
    pub average_documents_per_project: f32,
    pub total_documents: usize,
    pub projects_by_priority: std::collections::HashMap<ProjectPriority, usize>,
    pub most_common_tags: Vec<(String, usize)>,
    pub recent_activity: Vec<RecentActivity>,
}

/// Recent project activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub project_id: EntityId,
    pub project_name: String,
    pub activity_type: ActivityType,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

/// Types of project activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    ProjectCreated,
    DocumentAdded,
    DocumentRemoved,
    StatusChanged,
    GoalAchieved,
    WorkspaceUpdated,
}

/// Implementation traits for different storage backends
pub mod implementations {
    use super::*;
    
    /// SQLite implementation of ProjectRepository
    /// Note: This is a placeholder implementation for future SQLite integration
    pub struct SqliteProjectRepository {
        // TODO: Add actual SQLite connection pool when implementing persistence
        _phantom: std::marker::PhantomData<()>,
    }
    
    impl SqliteProjectRepository {
        pub fn new(_db_path: String) -> Self {
            Self { 
                _phantom: std::marker::PhantomData,
            }
        }
    }
    
    #[async_trait]
    impl ProjectRepository for SqliteProjectRepository {
        async fn save(&self, aggregate: &mut ProjectAggregate) -> Result<()> {
            // TODO: Implement actual SQLite persistence
            // For now, this is a placeholder that logs the operation
            
            // Clear events to simulate successful save
            if !aggregate.events().is_empty() {
                aggregate.clear_events();
            }
            
            // Return NotImplemented error to be explicit about current state
            Err(WritemagicError::not_implemented(
                "SQLite project repository save operation not yet implemented"
            ))
        }
        
        async fn load(&self, _project_id: &EntityId) -> Result<Option<ProjectAggregate>> {
            // TODO: Implement actual SQLite loading
            Err(WritemagicError::not_implemented(
                "SQLite project repository load operation not yet implemented"
            ))
        }
        
        async fn delete(&self, _project_id: &EntityId) -> Result<()> {
            Err(WritemagicError::not_implemented(
                "SQLite project repository delete operation not yet implemented"
            ))
        }
        
        async fn list(&self, _filter: ProjectFilter) -> Result<Vec<ProjectAggregate>> {
            Err(WritemagicError::not_implemented(
                "SQLite project repository list operation not yet implemented"
            ))
        }
        
        async fn search(&self, _criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>> {
            Err(WritemagicError::not_implemented(
                "SQLite project repository search operation not yet implemented"
            ))
        }
        
        async fn get_statistics(&self, _project_id: &EntityId) -> Result<ProjectStatistics> {
            Err(WritemagicError::not_implemented(
                "SQLite project repository statistics operation not yet implemented"
            ))
        }
        
        async fn exists(&self, _project_id: &EntityId) -> Result<bool> {
            Err(WritemagicError::not_implemented(
                "SQLite project repository exists operation not yet implemented"
            ))
        }
    }
    
    /// IndexedDB implementation for web applications
    /// Note: This is a placeholder implementation for future IndexedDB integration
    pub struct IndexedDBProjectRepository {
        // TODO: Add actual IndexedDB connection when implementing web persistence
        _phantom: std::marker::PhantomData<()>,
    }
    
    impl IndexedDBProjectRepository {
        pub fn new(_db_name: String) -> Self {
            Self { 
                _phantom: std::marker::PhantomData,
            }
        }
    }
    
    #[async_trait]
    impl ProjectRepository for IndexedDBProjectRepository {
        async fn save(&self, aggregate: &mut ProjectAggregate) -> Result<()> {
            // Clear events to simulate successful save
            if !aggregate.events().is_empty() {
                aggregate.clear_events();
            }
            
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository save operation not yet implemented"
            ))
        }
        
        async fn load(&self, _project_id: &EntityId) -> Result<Option<ProjectAggregate>> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository load operation not yet implemented"
            ))
        }
        
        async fn delete(&self, _project_id: &EntityId) -> Result<()> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository delete operation not yet implemented"
            ))
        }
        
        async fn list(&self, _filter: ProjectFilter) -> Result<Vec<ProjectAggregate>> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository list operation not yet implemented"
            ))
        }
        
        async fn search(&self, _criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository search operation not yet implemented"
            ))
        }
        
        async fn get_statistics(&self, _project_id: &EntityId) -> Result<ProjectStatistics> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository statistics operation not yet implemented"
            ))
        }
        
        async fn exists(&self, _project_id: &EntityId) -> Result<bool> {
            Err(WritemagicError::not_implemented(
                "IndexedDB project repository exists operation not yet implemented"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::implementations::SqliteProjectRepository;

    #[tokio::test]
    async fn test_sqlite_repository() {
        let repo = SqliteProjectRepository::new(":memory:".to_string());
        
        // Test basic repository operations
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        // Save should not error
        assert!(repo.save(&mut aggregate).await.is_ok());
        
        // Load should return None for non-existent project
        let loaded = repo.load(&EntityId::new()).await.unwrap();
        assert!(loaded.is_none());
    }
    
    #[tokio::test]
    async fn test_project_filter() {
        let filter = ProjectFilter {
            status: Some(ProjectStatus::Active),
            priority: Some(ProjectPriority::High),
            limit: Some(10),
            ..Default::default()
        };
        
        assert_eq!(filter.status, Some(ProjectStatus::Active));
        assert_eq!(filter.limit, Some(10));
    }
    
    #[tokio::test]
    async fn test_search_criteria() {
        let criteria = ProjectSearchCriteria {
            query: "writing".to_string(),
            search_in_name: true,
            search_in_description: true,
            search_in_tags: false,
            filter: None,
        };
        
        assert_eq!(criteria.query, "writing");
        assert!(criteria.search_in_name);
        assert!(!criteria.search_in_tags);
    }
}