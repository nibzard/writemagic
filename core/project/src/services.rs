//! Project domain services

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::aggregates::{ProjectAggregate, ProjectStatistics};
use crate::entities::{ProjectTemplate};
use crate::value_objects::{ProjectStatus, ProjectPriority, ProjectGoal, ProjectTag, GoalType};
use crate::repositories::{ProjectRepository, ProjectTemplateRepository, ProjectFilter, ProjectSearchCriteria};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project management service - orchestrates project operations
pub struct ProjectManagementService {
    project_repository: Arc<dyn ProjectRepository>,
    template_repository: Arc<dyn ProjectTemplateRepository>,
}

impl ProjectManagementService {
    /// Create a new project management service
    pub fn new(
        project_repository: Arc<dyn ProjectRepository>,
        template_repository: Arc<dyn ProjectTemplateRepository>,
    ) -> Self {
        Self {
            project_repository,
            template_repository,
        }
    }
    
    /// Create a new project
    pub async fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> Result<ProjectAggregate> {
        // Validate request
        request.validate()?;
        
        // Create project aggregate
        let mut aggregate = if let Some(template_name) = request.template_name {
            // Create from template
            let template = self.template_repository
                .load_template(&template_name)
                .await?
                .ok_or_else(|| WritemagicError::not_found("Template not found"))?;
            
            ProjectAggregate::from_template(template, request.created_by)?
        } else {
            // Create new project
            ProjectAggregate::new(
                request.name,
                request.description,
                request.created_by,
            )?
        };
        
        // Set initial properties if provided
        if let Some(priority) = request.priority {
            aggregate.update_priority(priority);
        }
        
        // Add tags if provided
        for tag_str in request.tags {
            let tag = ProjectTag::new(tag_str)?;
            aggregate.add_tag(tag)?;
        }
        
        // Add goals if provided
        for goal in request.goals {
            aggregate.add_goal(goal)?;
        }
        
        // Save to repository
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
    
    /// Load a project by ID
    pub async fn get_project(&self, project_id: &EntityId) -> Result<Option<ProjectAggregate>> {
        self.project_repository.load(project_id).await
    }
    
    /// Update project properties
    pub async fn update_project(
        &self,
        project_id: &EntityId,
        request: UpdateProjectRequest,
    ) -> Result<ProjectAggregate> {
        // Load existing project
        let mut aggregate = self.project_repository
            .load(project_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Project not found"))?;
        
        // Apply updates
        if let Some(status) = request.status {
            aggregate.update_status(status)?;
        }
        
        if let Some(priority) = request.priority {
            aggregate.update_priority(priority);
        }
        
        if let Some(config) = request.workspace_config {
            aggregate.update_workspace_config(config);
        }
        
        // Handle tag operations
        for tag_str in request.add_tags {
            let tag = ProjectTag::new(tag_str)?;
            aggregate.add_tag(tag)?;
        }
        
        for tag_str in request.remove_tags {
            aggregate.remove_tag(&tag_str)?;
        }
        
        // Handle goal operations
        for goal in request.add_goals {
            aggregate.add_goal(goal)?;
        }
        
        for (goal_type, progress) in request.update_goal_progress {
            aggregate.update_goal_progress(goal_type, progress)?;
        }
        
        // Save changes
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
    
    /// Delete a project
    pub async fn delete_project(&self, project_id: &EntityId) -> Result<()> {
        // Check if project exists
        if !self.project_repository.exists(project_id).await? {
            return Err(WritemagicError::not_found("Project not found"));
        }
        
        // Delete from repository
        self.project_repository.delete(project_id).await
    }
    
    /// Archive a project
    pub async fn archive_project(&self, project_id: &EntityId) -> Result<ProjectAggregate> {
        let mut aggregate = self.project_repository
            .load(project_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Project not found"))?;
        
        aggregate.archive()?;
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
    
    /// Add a document to a project
    pub async fn add_document_to_project(
        &self,
        project_id: &EntityId,
        document_id: EntityId,
        pane_position: Option<usize>,
    ) -> Result<ProjectAggregate> {
        let mut aggregate = self.project_repository
            .load(project_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Project not found"))?;
        
        aggregate.add_document(document_id, pane_position)?;
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
    
    /// Remove a document from a project
    pub async fn remove_document_from_project(
        &self,
        project_id: &EntityId,
        document_id: &EntityId,
    ) -> Result<ProjectAggregate> {
        let mut aggregate = self.project_repository
            .load(project_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Project not found"))?;
        
        aggregate.remove_document(document_id)?;
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
    
    /// List projects with filtering
    pub async fn list_projects(&self, filter: ProjectFilter) -> Result<Vec<ProjectAggregate>> {
        self.project_repository.list(filter).await
    }
    
    /// Search projects
    pub async fn search_projects(&self, criteria: ProjectSearchCriteria) -> Result<Vec<ProjectAggregate>> {
        self.project_repository.search(criteria).await
    }
    
    /// Get project statistics
    pub async fn get_project_statistics(&self, project_id: &EntityId) -> Result<ProjectStatistics> {
        self.project_repository.get_statistics(project_id).await
    }
    
    /// Update goal progress
    pub async fn update_goal_progress(
        &self,
        project_id: &EntityId,
        goal_type: GoalType,
        new_value: u32,
    ) -> Result<ProjectAggregate> {
        let mut aggregate = self.project_repository
            .load(project_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Project not found"))?;
        
        aggregate.update_goal_progress(goal_type, new_value)?;
        self.project_repository.save(&mut aggregate).await?;
        
        Ok(aggregate)
    }
}

/// Project template service - manages project templates
pub struct ProjectTemplateService {
    template_repository: Arc<dyn ProjectTemplateRepository>,
}

impl ProjectTemplateService {
    /// Create a new template service
    pub fn new(template_repository: Arc<dyn ProjectTemplateRepository>) -> Self {
        Self { template_repository }
    }
    
    /// Create a new template
    pub async fn create_template(&self, template: ProjectTemplate) -> Result<()> {
        // Check if template already exists
        if self.template_repository.load_template(&template.name).await?.is_some() {
            return Err(WritemagicError::validation("Template already exists"));
        }
        
        self.template_repository.save_template(&template).await
    }
    
    /// Get all available templates
    pub async fn list_templates(&self) -> Result<Vec<ProjectTemplate>> {
        self.template_repository.list_templates().await
    }
    
    /// Get a specific template
    pub async fn get_template(&self, name: &str) -> Result<Option<ProjectTemplate>> {
        self.template_repository.load_template(name).await
    }
    
    /// Delete a template
    pub async fn delete_template(&self, name: &str) -> Result<()> {
        self.template_repository.delete_template(name).await
    }
}

/// Project analytics service - provides insights and statistics
pub struct ProjectAnalyticsService {
    project_repository: Arc<dyn ProjectRepository>,
}

impl ProjectAnalyticsService {
    /// Create a new analytics service
    pub fn new(project_repository: Arc<dyn ProjectRepository>) -> Self {
        Self { project_repository }
    }
    
    /// Get comprehensive project analytics
    pub async fn get_analytics(&self, filter: Option<ProjectFilter>) -> Result<ProjectAnalytics> {
        let projects = self.project_repository
            .list(filter.unwrap_or_default())
            .await?;
        
        let mut analytics = ProjectAnalytics::default();
        
        for project in &projects {
            analytics.total_projects += 1;
            
            match project.status() {
                ProjectStatus::Active => analytics.active_projects += 1,
                ProjectStatus::Completed => analytics.completed_projects += 1,
                ProjectStatus::Archived => analytics.archived_projects += 1,
                ProjectStatus::Paused => analytics.paused_projects += 1,
            }
            
            analytics.total_documents += project.project().document_count();
            
            // Count by priority
            let priority_count = analytics.projects_by_priority
                .entry(project.priority().clone())
                .or_insert(0);
            *priority_count += 1;
            
            // Count tags
            for tag in project.tags() {
                let tag_count = analytics.tag_usage
                    .entry(tag.value().to_string())
                    .or_insert(0);
                *tag_count += 1;
            }
            
            // Goal statistics
            for goal in project.goals() {
                analytics.total_goals += 1;
                if goal.is_achieved() {
                    analytics.achieved_goals += 1;
                }
            }
        }
        
        // Calculate averages
        if analytics.total_projects > 0 {
            analytics.average_documents_per_project = 
                analytics.total_documents as f32 / analytics.total_projects as f32;
        }
        
        if analytics.total_goals > 0 {
            analytics.goal_completion_rate = 
                analytics.achieved_goals as f32 / analytics.total_goals as f32;
        }
        
        // Get most common tags
        let mut tag_pairs: Vec<_> = analytics.tag_usage.iter().collect();
        tag_pairs.sort_by(|a, b| b.1.cmp(a.1));
        analytics.most_common_tags = tag_pairs
            .into_iter()
            .take(10)
            .map(|(tag, count)| (tag.clone(), *count))
            .collect();
        
        Ok(analytics)
    }
    
    /// Get productivity metrics for a date range
    pub async fn get_productivity_metrics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ProductivityMetrics> {
        let filter = ProjectFilter {
            created_after: Some(start_date),
            created_before: Some(end_date),
            ..Default::default()
        };
        
        let projects = self.project_repository.list(filter).await?;
        
        let mut metrics = ProductivityMetrics {
            period_start: start_date,
            period_end: end_date,
            projects_created: projects.len(),
            projects_completed: projects.iter()
                .filter(|p| *p.status() == ProjectStatus::Completed)
                .count(),
            goals_achieved: projects.iter()
                .map(|p| p.goals().iter().filter(|g| g.is_achieved()).count())
                .sum(),
            total_documents_created: projects.iter()
                .map(|p| p.project().document_count())
                .sum(),
            average_project_completion_time: 0.0, // Would need more data to calculate
        };
        
        Ok(metrics)
    }
}

// Request/Response DTOs

/// Request to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<EntityId>,
    pub template_name: Option<String>,
    pub priority: Option<ProjectPriority>,
    pub tags: Vec<String>,
    pub goals: Vec<ProjectGoal>,
}

impl CreateProjectRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(WritemagicError::validation("Project name cannot be empty"));
        }
        
        if self.name.len() > 200 {
            return Err(WritemagicError::validation("Project name cannot exceed 200 characters"));
        }
        
        // Validate tags
        for tag in &self.tags {
            ProjectTag::new(tag.clone())?;
        }
        
        Ok(())
    }
}

/// Request to update a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub status: Option<ProjectStatus>,
    pub priority: Option<ProjectPriority>,
    pub workspace_config: Option<crate::entities::WorkspaceConfig>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub add_goals: Vec<ProjectGoal>,
    pub update_goal_progress: Vec<(GoalType, u32)>,
}

/// Project analytics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAnalytics {
    pub total_projects: usize,
    pub active_projects: usize,
    pub completed_projects: usize,
    pub archived_projects: usize,
    pub paused_projects: usize,
    pub total_documents: usize,
    pub average_documents_per_project: f32,
    pub total_goals: usize,
    pub achieved_goals: usize,
    pub goal_completion_rate: f32,
    pub projects_by_priority: std::collections::HashMap<ProjectPriority, usize>,
    pub tag_usage: std::collections::HashMap<String, usize>,
    pub most_common_tags: Vec<(String, usize)>,
}

/// Productivity metrics for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub projects_created: usize,
    pub projects_completed: usize,
    pub goals_achieved: usize,
    pub total_documents_created: usize,
    pub average_project_completion_time: f32, // in days
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::implementations::SqliteProjectRepository;
    use std::sync::Arc;

    // Mock template repository for testing
    struct MockTemplateRepository;
    
    #[async_trait::async_trait]
    impl ProjectTemplateRepository for MockTemplateRepository {
        async fn save_template(&self, _template: &ProjectTemplate) -> Result<()> {
            Ok(())
        }
        
        async fn load_template(&self, name: &str) -> Result<Option<ProjectTemplate>> {
            if name == "writing" {
                Ok(Some(ProjectTemplate::writing_template()))
            } else {
                Ok(None)
            }
        }
        
        async fn list_templates(&self) -> Result<Vec<ProjectTemplate>> {
            Ok(vec![ProjectTemplate::writing_template()])
        }
        
        async fn delete_template(&self, _name: &str) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_project() {
        let project_repo = Arc::new(SqliteProjectRepository::new(":memory:".to_string()));
        let template_repo = Arc::new(MockTemplateRepository);
        let service = ProjectManagementService::new(project_repo, template_repo);
        
        let request = CreateProjectRequest {
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            created_by: Some(EntityId::new()),
            template_name: None,
            priority: Some(ProjectPriority::High),
            tags: vec!["writing".to_string()],
            goals: vec![ProjectGoal::new(GoalType::WordCount, 1000)],
        };
        
        let result = service.create_project(request).await;
        assert!(result.is_ok());
        
        let project = result.unwrap();
        assert_eq!(project.project().name, "Test Project");
        assert_eq!(*project.priority(), ProjectPriority::High);
        assert_eq!(project.tags().len(), 1);
        assert_eq!(project.goals().len(), 1);
    }
    
    #[tokio::test]
    async fn test_create_project_validation() {
        let project_repo = Arc::new(SqliteProjectRepository::new(":memory:".to_string()));
        let template_repo = Arc::new(MockTemplateRepository);
        let service = ProjectManagementService::new(project_repo, template_repo);
        
        let request = CreateProjectRequest {
            name: "".to_string(), // Empty name should fail
            description: None,
            created_by: None,
            template_name: None,
            priority: None,
            tags: Vec::new(),
            goals: Vec::new(),
        };
        
        let result = service.create_project(request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_create_project_from_template() {
        let project_repo = Arc::new(SqliteProjectRepository::new(":memory:".to_string()));
        let template_repo = Arc::new(MockTemplateRepository);
        let service = ProjectManagementService::new(project_repo, template_repo);
        
        let request = CreateProjectRequest {
            name: "Template Project".to_string(),
            description: None,
            created_by: None,
            template_name: Some("writing".to_string()),
            priority: None,
            tags: Vec::new(),
            goals: Vec::new(),
        };
        
        let result = service.create_project(request).await;
        assert!(result.is_ok());
        
        let project = result.unwrap();
        // Should have the template's default workspace config
        assert!(!project.project().workspace_config.panes.is_empty());
    }
    
    #[tokio::test]
    async fn test_template_service() {
        let template_repo = Arc::new(MockTemplateRepository);
        let service = ProjectTemplateService::new(template_repo);
        
        let templates = service.list_templates().await.unwrap();
        assert_eq!(templates.len(), 1);
        
        let template = service.get_template("writing").await.unwrap();
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Writing Project");
    }
}