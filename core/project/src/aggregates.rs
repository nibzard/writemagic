//! Project domain aggregates

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::entities::{Project, WorkspaceConfig, ProjectTemplate};
use crate::value_objects::{ProjectStatus, ProjectPriority, ProjectGoal, ProjectTag};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project aggregate root that encapsulates project business logic and invariants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAggregate {
    project: Project,
    status: ProjectStatus,
    priority: ProjectPriority,
    goals: Vec<ProjectGoal>,
    tags: Vec<ProjectTag>,
    version: u64,
    events: Vec<ProjectEvent>,
}

impl ProjectAggregate {
    /// Create a new project aggregate
    pub fn new(
        name: String,
        description: Option<String>,
        created_by: Option<EntityId>,
    ) -> Result<Self> {
        let project = Project::new(name, description, created_by);
        
        let aggregate = Self {
            project,
            status: ProjectStatus::Active,
            priority: ProjectPriority::Medium,
            goals: Vec::new(),
            tags: Vec::new(),
            version: 1,
            events: Vec::new(),
        };
        
        Ok(aggregate)
    }
    
    /// Create project from template
    pub fn from_template(
        template: ProjectTemplate,
        created_by: Option<EntityId>,
    ) -> Result<Self> {
        let mut project = Project::new(
            template.name,
            Some(template.description),
            created_by,
        );
        
        project.update_workspace_config(template.workspace_config);
        
        let tags = template.tags.into_iter()
            .filter_map(|tag| ProjectTag::new(tag).ok())
            .collect();
        
        let mut aggregate = Self {
            project,
            status: ProjectStatus::Active,
            priority: ProjectPriority::Medium,
            goals: Vec::new(),
            tags,
            version: 1,
            events: Vec::new(),
        };
        
        aggregate.add_event(ProjectEvent::ProjectCreated {
            project_id: aggregate.project.id,
            name: aggregate.project.name.clone(),
            created_by,
            timestamp: Utc::now(),
        });
        
        Ok(aggregate)
    }
    
    /// Get the project entity
    pub fn project(&self) -> &Project {
        &self.project
    }
    
    /// Get project ID
    pub fn id(&self) -> EntityId {
        self.project.id
    }
    
    /// Get project status
    pub fn status(&self) -> &ProjectStatus {
        &self.status
    }
    
    /// Get project priority
    pub fn priority(&self) -> &ProjectPriority {
        &self.priority
    }
    
    /// Get project goals
    pub fn goals(&self) -> &[ProjectGoal] {
        &self.goals
    }
    
    /// Get project tags
    pub fn tags(&self) -> &[ProjectTag] {
        &self.tags
    }
    
    /// Get aggregate version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get pending events
    pub fn events(&self) -> &[ProjectEvent] {
        &self.events
    }
    
    /// Update project status
    pub fn update_status(&mut self, new_status: ProjectStatus) -> Result<()> {
        if self.status == new_status {
            return Ok(());
        }
        
        // Business rules for status transitions
        match (&self.status, &new_status) {
            (ProjectStatus::Archived, _) => {
                return Err(WritemagicError::validation("Cannot change status of archived project"));
            },
            (ProjectStatus::Completed, ProjectStatus::Active) => {
                // Allow reactivating completed projects
            },
            _ => {
                // All other transitions are allowed
            }
        }
        
        let old_status = self.status.clone();
        self.status = new_status.clone();
        self.project.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(ProjectEvent::StatusChanged {
            project_id: self.project.id,
            old_status,
            new_status,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Update project priority
    pub fn update_priority(&mut self, new_priority: ProjectPriority) {
        if self.priority != new_priority {
            let old_priority = self.priority.clone();
            self.priority = new_priority.clone();
            self.project.updated_at = Utc::now();
            self.version += 1;
            
            self.add_event(ProjectEvent::PriorityChanged {
                project_id: self.project.id,
                old_priority,
                new_priority,
                timestamp: Utc::now(),
            });
        }
    }
    
    /// Add a document to the project
    pub fn add_document(&mut self, document_id: EntityId, pane_position: Option<usize>) -> Result<()> {
        self.project.add_document(document_id, pane_position)?;
        self.version += 1;
        
        self.add_event(ProjectEvent::DocumentAdded {
            project_id: self.project.id,
            document_id,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Remove a document from the project
    pub fn remove_document(&mut self, document_id: &EntityId) -> Result<()> {
        self.project.remove_document(document_id)?;
        self.version += 1;
        
        self.add_event(ProjectEvent::DocumentRemoved {
            project_id: self.project.id,
            document_id: *document_id,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Add a goal to the project
    pub fn add_goal(&mut self, goal: ProjectGoal) -> Result<()> {
        // Check for duplicate goal types
        if self.goals.iter().any(|g| g.goal_type == goal.goal_type) {
            return Err(WritemagicError::validation("Goal type already exists for this project"));
        }
        
        self.goals.push(goal.clone());
        self.project.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(ProjectEvent::GoalAdded {
            project_id: self.project.id,
            goal_type: goal.goal_type,
            target_value: goal.target_value,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Update goal progress
    pub fn update_goal_progress(&mut self, goal_type: crate::value_objects::GoalType, new_value: u32) -> Result<()> {
        let goal_type_clone = goal_type.clone();
        
        // Find the goal and collect necessary info
        let goal_info = {
            if let Some(goal) = self.goals.iter_mut().find(|g| g.goal_type == goal_type) {
                let old_value = goal.current_value;
                goal.update_progress(new_value);
                Some((old_value, goal.is_achieved(), goal.target_value))
            } else {
                None
            }
        };
        
        if let Some((old_value, is_achieved, target_value)) = goal_info {
            self.project.updated_at = Utc::now();
            self.version += 1;
            
            self.add_event(ProjectEvent::GoalProgressUpdated {
                project_id: self.project.id,
                goal_type: goal_type_clone.clone(),
                old_value,
                new_value,
                timestamp: Utc::now(),
            });
            
            // Check if goal was achieved
            if is_achieved && old_value < target_value {
                self.add_event(ProjectEvent::GoalAchieved {
                    project_id: self.project.id,
                    goal_type: goal_type_clone,
                    timestamp: Utc::now(),
                });
            }
            
            Ok(())
        } else {
            Err(WritemagicError::not_found("Goal not found"))
        }
    }
    
    /// Add a tag to the project
    pub fn add_tag(&mut self, tag: ProjectTag) -> Result<()> {
        if self.tags.contains(&tag) {
            return Err(WritemagicError::validation("Tag already exists"));
        }
        
        self.tags.push(tag.clone());
        self.project.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(ProjectEvent::TagAdded {
            project_id: self.project.id,
            tag: tag.value().to_string(),
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Remove a tag from the project
    pub fn remove_tag(&mut self, tag_value: &str) -> Result<()> {
        let original_len = self.tags.len();
        self.tags.retain(|t| t.value() != tag_value);
        
        if self.tags.len() == original_len {
            return Err(WritemagicError::not_found("Tag not found"));
        }
        
        self.project.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(ProjectEvent::TagRemoved {
            project_id: self.project.id,
            tag: tag_value.to_string(),
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Update workspace configuration
    pub fn update_workspace_config(&mut self, config: WorkspaceConfig) {
        self.project.update_workspace_config(config);
        self.version += 1;
        
        self.add_event(ProjectEvent::WorkspaceConfigUpdated {
            project_id: self.project.id,
            timestamp: Utc::now(),
        });
    }
    
    /// Archive the project
    pub fn archive(&mut self) -> Result<()> {
        if self.status == ProjectStatus::Archived {
            return Err(WritemagicError::validation("Project is already archived"));
        }
        
        self.project.archive();
        self.status = ProjectStatus::Archived;
        self.version += 1;
        
        self.add_event(ProjectEvent::ProjectArchived {
            project_id: self.project.id,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Get project statistics
    pub fn get_statistics(&self) -> ProjectStatistics {
        let total_goals = self.goals.len();
        let achieved_goals = self.goals.iter().filter(|g| g.is_achieved()).count();
        
        let average_goal_progress = if total_goals > 0 {
            self.goals.iter().map(|g| g.progress_percentage()).sum::<f32>() / total_goals as f32
        } else {
            0.0
        };
        
        ProjectStatistics {
            document_count: self.project.document_count(),
            total_goals,
            achieved_goals,
            average_goal_progress,
            last_activity: self.project.metadata.last_activity,
            tag_count: self.tags.len(),
            status: self.status.clone(),
            priority: self.priority.clone(),
        }
    }
    
    /// Clear events (called after persisting)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
    
    /// Add an event to the aggregate
    fn add_event(&mut self, event: ProjectEvent) {
        self.events.push(event);
    }
}

/// Project events for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectEvent {
    ProjectCreated {
        project_id: EntityId,
        name: String,
        created_by: Option<EntityId>,
        timestamp: DateTime<Utc>,
    },
    DocumentAdded {
        project_id: EntityId,
        document_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    DocumentRemoved {
        project_id: EntityId,
        document_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    StatusChanged {
        project_id: EntityId,
        old_status: ProjectStatus,
        new_status: ProjectStatus,
        timestamp: DateTime<Utc>,
    },
    PriorityChanged {
        project_id: EntityId,
        old_priority: ProjectPriority,
        new_priority: ProjectPriority,
        timestamp: DateTime<Utc>,
    },
    GoalAdded {
        project_id: EntityId,
        goal_type: crate::value_objects::GoalType,
        target_value: u32,
        timestamp: DateTime<Utc>,
    },
    GoalProgressUpdated {
        project_id: EntityId,
        goal_type: crate::value_objects::GoalType,
        old_value: u32,
        new_value: u32,
        timestamp: DateTime<Utc>,
    },
    GoalAchieved {
        project_id: EntityId,
        goal_type: crate::value_objects::GoalType,
        timestamp: DateTime<Utc>,
    },
    TagAdded {
        project_id: EntityId,
        tag: String,
        timestamp: DateTime<Utc>,
    },
    TagRemoved {
        project_id: EntityId,
        tag: String,
        timestamp: DateTime<Utc>,
    },
    WorkspaceConfigUpdated {
        project_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    ProjectArchived {
        project_id: EntityId,
        timestamp: DateTime<Utc>,
    },
}

/// Project statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub document_count: usize,
    pub total_goals: usize,
    pub achieved_goals: usize,
    pub average_goal_progress: f32,
    pub last_activity: DateTime<Utc>,
    pub tag_count: usize,
    pub status: ProjectStatus,
    pub priority: ProjectPriority,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::GoalType;

    #[test]
    fn test_create_project_aggregate() {
        let aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            Some("A test project".to_string()),
            Some(EntityId::new()),
        ).unwrap();
        
        assert_eq!(aggregate.project().name, "Test Project");
        assert_eq!(*aggregate.status(), ProjectStatus::Active);
        assert_eq!(*aggregate.priority(), ProjectPriority::Medium);
        assert_eq!(aggregate.version(), 1);
    }
    
    #[test]
    fn test_status_transitions() {
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        // Valid transition
        assert!(aggregate.update_status(ProjectStatus::Paused).is_ok());
        assert_eq!(*aggregate.status(), ProjectStatus::Paused);
        assert_eq!(aggregate.version(), 2);
        
        // Archive project
        assert!(aggregate.update_status(ProjectStatus::Archived).is_ok());
        
        // Cannot change archived project status
        assert!(aggregate.update_status(ProjectStatus::Active).is_err());
    }
    
    #[test]
    fn test_document_management() {
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        let doc_id = EntityId::new();
        
        // Add document
        assert!(aggregate.add_document(doc_id, None).is_ok());
        assert_eq!(aggregate.project().document_count(), 1);
        assert_eq!(aggregate.version(), 2);
        
        // Remove document
        assert!(aggregate.remove_document(&doc_id).is_ok());
        assert_eq!(aggregate.project().document_count(), 0);
        assert_eq!(aggregate.version(), 3);
        
        // Try to remove non-existent document
        assert!(aggregate.remove_document(&EntityId::new()).is_err());
    }
    
    #[test]
    fn test_goal_management() {
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        let goal = ProjectGoal::new(GoalType::WordCount, 1000);
        
        // Add goal
        assert!(aggregate.add_goal(goal).is_ok());
        assert_eq!(aggregate.goals().len(), 1);
        
        // Update goal progress
        assert!(aggregate.update_goal_progress(GoalType::WordCount, 500).is_ok());
        assert_eq!(aggregate.goals()[0].current_value, 500);
        
        // Achieve goal
        assert!(aggregate.update_goal_progress(GoalType::WordCount, 1000).is_ok());
        assert!(aggregate.goals()[0].is_achieved());
        
        // Try to add duplicate goal type
        let duplicate_goal = ProjectGoal::new(GoalType::WordCount, 2000);
        assert!(aggregate.add_goal(duplicate_goal).is_err());
    }
    
    #[test]
    fn test_tag_management() {
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        let tag = ProjectTag::new("writing".to_string()).unwrap();
        
        // Add tag
        assert!(aggregate.add_tag(tag.clone()).is_ok());
        assert_eq!(aggregate.tags().len(), 1);
        
        // Try to add duplicate tag
        assert!(aggregate.add_tag(tag).is_err());
        
        // Remove tag
        assert!(aggregate.remove_tag("writing").is_ok());
        assert_eq!(aggregate.tags().len(), 0);
        
        // Try to remove non-existent tag
        assert!(aggregate.remove_tag("nonexistent").is_err());
    }
    
    #[test]
    fn test_events() {
        let mut aggregate = ProjectAggregate::new(
            "Test Project".to_string(),
            None,
            None,
        ).unwrap();
        
        let initial_events = aggregate.events().len();
        
        // Add document should generate event
        aggregate.add_document(EntityId::new(), None).unwrap();
        assert_eq!(aggregate.events().len(), initial_events + 1);
        
        // Clear events
        aggregate.clear_events();
        assert_eq!(aggregate.events().len(), 0);
    }
}