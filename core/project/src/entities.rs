//! Project domain entities

use writemagic_shared::{EntityId, WritemagicError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Project entity representing a collection of documents and workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Project {
    pub id: EntityId,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub description: Option<String>,
    pub document_ids: Vec<EntityId>,
    pub workspace_config: WorkspaceConfig,
    pub metadata: ProjectMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<EntityId>,
    pub is_archived: bool,
}

/// Workspace configuration for the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub layout: WorkspaceLayout,
    pub panes: Vec<PaneConfig>,
    pub theme: Option<String>,
    pub auto_save_enabled: bool,
    pub focus_mode_enabled: bool,
}

/// Workspace layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceLayout {
    Single,
    SplitVertical,
    SplitHorizontal,
    MultiPane,
    Custom(String),
}

/// Individual pane configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    pub id: String,
    pub pane_type: PaneType,
    pub size_percentage: f32,
    pub document_id: Option<EntityId>,
    pub position: PanePosition,
}

/// Types of panes available in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaneType {
    Editor,
    Preview,
    Outline,
    Notes,
    Reference,
    AIAssistant,
    FileExplorer,
}

/// Position of a pane in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanePosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Project metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub total_documents: usize,
    pub total_word_count: usize,
    pub last_activity: DateTime<Utc>,
    pub collaborators: Vec<EntityId>,
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl Project {
    /// Create a new project
    pub fn new(
        name: String,
        description: Option<String>,
        created_by: Option<EntityId>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: EntityId::new(),
            name,
            description,
            document_ids: Vec::new(),
            workspace_config: WorkspaceConfig::default(),
            metadata: ProjectMetadata::default(),
            created_at: now,
            updated_at: now,
            created_by,
            is_archived: false,
        }
    }
    
    /// Add a document to the project
    pub fn add_document(&mut self, document_id: EntityId, pane_position: Option<usize>) -> Result<()> {
        if self.document_ids.contains(&document_id) {
            return Err(WritemagicError::validation("Document already exists in project"));
        }
        
        if let Some(pos) = pane_position {
            if pos <= self.document_ids.len() {
                self.document_ids.insert(pos, document_id);
            } else {
                self.document_ids.push(document_id);
            }
        } else {
            self.document_ids.push(document_id);
        }
        
        self.updated_at = Utc::now();
        self.metadata.total_documents = self.document_ids.len();
        self.metadata.last_activity = self.updated_at;
        
        Ok(())
    }
    
    /// Remove a document from the project
    pub fn remove_document(&mut self, document_id: &EntityId) -> Result<()> {
        let original_len = self.document_ids.len();
        self.document_ids.retain(|id| id != document_id);
        
        if self.document_ids.len() == original_len {
            return Err(WritemagicError::not_found("Document not found in project"));
        }
        
        self.updated_at = Utc::now();
        self.metadata.total_documents = self.document_ids.len();
        self.metadata.last_activity = self.updated_at;
        
        // Remove document from any pane configurations
        for pane in &mut self.workspace_config.panes {
            if pane.document_id == Some(*document_id) {
                pane.document_id = None;
            }
        }
        
        Ok(())
    }
    
    /// Update workspace configuration
    pub fn update_workspace_config(&mut self, config: WorkspaceConfig) {
        self.workspace_config = config;
        self.updated_at = Utc::now();
        self.metadata.last_activity = self.updated_at;
    }
    
    /// Add a pane to the workspace
    pub fn add_pane(&mut self, pane: PaneConfig) -> Result<()> {
        // Check for duplicate pane IDs
        if self.workspace_config.panes.iter().any(|p| p.id == pane.id) {
            return Err(WritemagicError::validation("Pane ID already exists"));
        }
        
        self.workspace_config.panes.push(pane);
        self.updated_at = Utc::now();
        self.metadata.last_activity = self.updated_at;
        
        Ok(())
    }
    
    /// Remove a pane from the workspace
    pub fn remove_pane(&mut self, pane_id: &str) -> Result<()> {
        let original_len = self.workspace_config.panes.len();
        self.workspace_config.panes.retain(|p| p.id != pane_id);
        
        if self.workspace_config.panes.len() == original_len {
            return Err(WritemagicError::not_found("Pane not found"));
        }
        
        self.updated_at = Utc::now();
        self.metadata.last_activity = self.updated_at;
        
        Ok(())
    }
    
    /// Archive the project
    pub fn archive(&mut self) {
        self.is_archived = true;
        self.updated_at = Utc::now();
        self.metadata.last_activity = self.updated_at;
    }
    
    /// Check if the project is empty (no documents)
    pub fn is_empty(&self) -> bool {
        self.document_ids.is_empty()
    }
    
    /// Get the number of documents in the project
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            layout: WorkspaceLayout::Single,
            panes: vec![
                PaneConfig {
                    id: "main-editor".to_string(),
                    pane_type: PaneType::Editor,
                    size_percentage: 100.0,
                    document_id: None,
                    position: PanePosition {
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                    },
                }
            ],
            theme: None,
            auto_save_enabled: true,
            focus_mode_enabled: false,
        }
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            total_documents: 0,
            total_word_count: 0,
            last_activity: Utc::now(),
            collaborators: Vec::new(),
            tags: Vec::new(),
            color: None,
            icon: None,
        }
    }
}

/// Project template for creating common project types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub description: String,
    pub workspace_config: WorkspaceConfig,
    pub default_documents: Vec<String>,
    pub tags: Vec<String>,
}

impl ProjectTemplate {
    /// Create a writing template
    pub fn writing_template() -> Self {
        Self {
            name: "Writing Project".to_string(),
            description: "Template for general writing projects".to_string(),
            workspace_config: WorkspaceConfig {
                layout: WorkspaceLayout::SplitVertical,
                panes: vec![
                    PaneConfig {
                        id: "main-editor".to_string(),
                        pane_type: PaneType::Editor,
                        size_percentage: 70.0,
                        document_id: None,
                        position: PanePosition { x: 0.0, y: 0.0, width: 70.0, height: 100.0 },
                    },
                    PaneConfig {
                        id: "notes".to_string(),
                        pane_type: PaneType::Notes,
                        size_percentage: 30.0,
                        document_id: None,
                        position: PanePosition { x: 70.0, y: 0.0, width: 30.0, height: 100.0 },
                    }
                ],
                auto_save_enabled: true,
                focus_mode_enabled: false,
                theme: None,
            },
            default_documents: vec!["Main Document".to_string(), "Notes".to_string()],
            tags: vec!["writing".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        let project = Project::new(
            "Test Project".to_string(),
            Some("A test project".to_string()),
            Some(EntityId::new()),
        );
        
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert!(project.document_ids.is_empty());
        assert!(!project.is_archived);
    }
    
    #[test]
    fn test_add_document() {
        let mut project = Project::new(
            "Test Project".to_string(),
            None,
            None,
        );
        
        let doc_id = EntityId::new();
        assert!(project.add_document(doc_id, None).is_ok());
        assert_eq!(project.document_ids.len(), 1);
        assert_eq!(project.document_ids[0], doc_id);
        assert_eq!(project.metadata.total_documents, 1);
        
        // Test duplicate document
        assert!(project.add_document(doc_id, None).is_err());
    }
}