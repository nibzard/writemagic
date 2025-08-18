//! Agent domain - File-based YAML agents for background processing

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod repositories;

pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use repositories::*;

/// YAML-based agent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub id: writemagic_shared::EntityId,
    pub name: String,
    pub description: String,
    pub config_file_path: std::path::PathBuf,
    pub status: AgentStatus,
    pub triggers: Vec<AgentTrigger>,
    pub actions: Vec<AgentAction>,
    pub schedule: Option<AgentSchedule>,
    pub created_at: writemagic_shared::Timestamp,
    pub updated_at: writemagic_shared::Timestamp,
}

/// Agent execution status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Running,
    Failed,
    Disabled,
}

/// Agent trigger conditions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentTrigger {
    FileChanged { path: String },
    DocumentSaved { document_id: writemagic_shared::EntityId },
    TimeBased { cron_expression: String },
    Manual,
}

/// Agent actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentAction {
    RunCommand { command: String, args: Vec<String> },
    ProcessDocument { document_id: writemagic_shared::EntityId },
    SendNotification { message: String },
    UpdateFile { path: String, content: String },
}

/// Agent scheduling configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSchedule {
    pub cron_expression: String,
    pub timezone: String,
    pub enabled: bool,
}