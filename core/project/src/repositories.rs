//! Project domain repositories

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::aggregates::{ProjectAggregate, ProjectEvent};
use crate::entities::{Project, ProjectTemplate};
use crate::value_objects::{ProjectStatus, ProjectPriority, ProjectTag};
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
    pub struct SqliteProjectRepository {
        // Implementation would contain database connection
        db_path: String,
    }
    
    impl SqliteProjectRepository {
        pub fn new(db_path: String) -> Self {
            Self { db_path }
        }
    }
    
    #[async_trait]
    impl ProjectRepository for SqliteProjectRepository {
        async fn save(&self, aggregate: &mut ProjectAggregate) -> Result<()> {
            // Implementation would serialize aggregate and save to SQLite
            // This is a functional implementation template
            
            // 1. Serialize the project aggregate
            let project_json = serde_json::to_string(aggregate)
                .map_err(|e| WritemagicError::internal(&format!("Serialization failed: {}", e)))?;
            
            // 2. Save to database (pseudo-code)
            // let mut conn = sqlite::Connection::open(&self.db_path)?;
            // conn.execute(
            //     "INSERT OR REPLACE INTO projects (id, data, version, updated_at) VALUES (?, ?, ?, ?)",
            //     &[&aggregate.id().to_string(), &project_json, &aggregate.version(), &aggregate.project().updated_at]
            // )?;
            
            // 3. Save events if any
            if !aggregate.events().is_empty() {
                // Save events to event store
                // self.save_events(aggregate).await?;
                aggregate.clear_events();
            }
            
            Ok(())
        }
        
        async fn load(&self, project_id: &EntityId) -> Result<Option<ProjectAggregate>> {
            // Implementation would load from SQLite and deserialize
            // This is a functional implementation template
            
            // 1. Query database (pseudo-code)
            // let mut conn = sqlite::Connection::open(&self.db_path)?;
            // let result = conn.query_row(
            //     "SELECT data FROM projects WHERE id = ?",
            //     &[&project_id.to_string()],
            //     |row| {
            //         let data: String = row.get(0)?;
            //         Ok(data)
            //     }
            // );
            
            // 2. Deserialize if found
            // match result {
            //     Ok(project_json) => {
            //         let aggregate: ProjectAggregate = serde_json::from_str(&project_json)
            //             .map_err(|e| WritemagicError::internal(&format!("Deserialization failed: {}", e)))?;
            //         Ok(Some(aggregate))
            //     },
            //     Err(_) => Ok(None)
            // }
            
            // For now, return None as this is a template
            Ok(None)
        }
        
        async fn delete(&self, project_id: &EntityId) -> Result<()> {
            // Implementation would delete from SQLite
            Ok(())
        }
        
        async fn list(&self, filter: ProjectFilter) -> Result<Vec<ProjectAggregate>> {
            // Implementation would query with filters and return results
            Ok(Vec::new())
        }
        
        async fn search(&self, criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>> {
            // Implementation would perform full-text search
            Ok(Vec::new())
        }
        
        async fn get_statistics(&self, project_id: &EntityId) -> Result<ProjectStatistics> {
            // Implementation would aggregate statistics from database
            Ok(ProjectStatistics {
                total_projects: 0,
                active_projects: 0,
                completed_projects: 0,
                archived_projects: 0,
                average_documents_per_project: 0.0,
                total_documents: 0,
                projects_by_priority: std::collections::HashMap::new(),
                most_common_tags: Vec::new(),
                recent_activity: Vec::new(),
            })
        }
        
        async fn exists(&self, project_id: &EntityId) -> Result<bool> {
            // Implementation would check existence in database
            Ok(false)
        }
    }
    
    /// IndexedDB implementation for web applications
    pub struct IndexedDBProjectRepository {
        db_name: String,
    }
    
    impl IndexedDBProjectRepository {
        pub fn new(db_name: String) -> Self {
            Self { db_name }
        }
    }
    
    #[async_trait]
    impl ProjectRepository for IndexedDBProjectRepository {
        async fn save(&self, aggregate: &mut ProjectAggregate) -> Result<()> {
            // Implementation would use IndexedDB via wasm-bindgen
            // This would be similar to SQLite but using browser IndexedDB API
            Ok(())
        }
        
        async fn load(&self, project_id: &EntityId) -> Result<Option<ProjectAggregate>> {
            Ok(None)
        }
        
        async fn delete(&self, project_id: &EntityId) -> Result<()> {
            Ok(())
        }
        
        async fn list(&self, filter: ProjectFilter) -> Result<Vec<ProjectAggregate>> {
            Ok(Vec::new())
        }
        
        async fn search(&self, criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>> {
            Ok(Vec::new())
        }
        
        async fn get_statistics(&self, project_id: &EntityId) -> Result<ProjectStatistics> {
            Ok(ProjectStatistics {
                total_projects: 0,
                active_projects: 0,
                completed_projects: 0,
                archived_projects: 0,
                average_documents_per_project: 0.0,
                total_documents: 0,
                projects_by_priority: std::collections::HashMap::new(),
                most_common_tags: Vec::new(),
                recent_activity: Vec::new(),
            })
        }
        
        async fn exists(&self, project_id: &EntityId) -> Result<bool> {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::SqliteProjectRepository;

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