//! Project domain - Multi-pane workspaces and session management

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;

pub use entities::{Project, WorkspaceConfig, ProjectMetadata, ProjectTemplate, PaneConfig, PaneType};
pub use value_objects::{ProjectStatus, ProjectPriority, ProjectColor, ProjectTag, ProjectGoal, GoalType};
pub use aggregates::{ProjectAggregate, ProjectEvent};
pub use services::{ProjectManagementService, ProjectTemplateService, ProjectAnalyticsService, CreateProjectRequest, UpdateProjectRequest, ProjectAnalytics, ProductivityMetrics};
pub use repositories::{ProjectRepository, ProjectTemplateRepository, ProjectFilter, ProjectSearchCriteria, ProjectSortBy, SortOrder, RecentActivity, ActivityType};

/// Workspace entity for managing multiple panes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workspace {
    pub id: writemagic_shared::EntityId,
    pub name: String,
    pub panes: Vec<Pane>,
    pub active_pane_id: Option<writemagic_shared::EntityId>,
    pub layout: WorkspaceLayout,
    pub created_at: writemagic_shared::Timestamp,
    pub updated_at: writemagic_shared::Timestamp,
}

/// Individual pane within a workspace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pane {
    pub id: writemagic_shared::EntityId,
    pub document_id: Option<writemagic_shared::EntityId>,
    pub branch_name: Option<String>,
    pub position: PanePosition,
    pub size: PaneSize,
    pub is_active: bool,
}

/// Pane position and size
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PanePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaneSize {
    pub width: f32,
    pub height: f32,
}

/// Workspace layout configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkspaceLayout {
    Horizontal,
    Vertical,
    Grid { columns: u32, rows: u32 },
    Custom,
}