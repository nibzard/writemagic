//! Version control domain - Git integration with timeline visualization

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod repositories;

pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use repositories::*;

/// Git repository abstraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Repository {
    pub id: writemagic_shared::EntityId,
    pub name: String,
    pub path: std::path::PathBuf,
    pub remote_url: Option<String>,
    pub current_branch: String,
    pub branches: Vec<Branch>,
    pub commits: Vec<Commit>,
    pub created_at: writemagic_shared::Timestamp,
    pub updated_at: writemagic_shared::Timestamp,
}

/// Git branch representation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit_id: String,
    pub is_current: bool,
    pub upstream: Option<String>,
    pub created_at: writemagic_shared::Timestamp,
}

/// Git commit representation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub timestamp: writemagic_shared::Timestamp,
    pub parent_ids: Vec<String>,
    pub changes: Vec<FileChange>,
}

/// File change in a commit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { old_path: String },
    Copied { source_path: String },
}