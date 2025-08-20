//! Writing domain events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use writemagic_shared::{EntityId, Timestamp, DomainEvent};
use std::collections::HashMap;

/// Document domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentEvent {
    DocumentCreated {
        document_id: EntityId,
        title: String,
        content_type: writemagic_shared::ContentType,
        created_by: Option<EntityId>,
        created_at: Timestamp,
    },
    DocumentTitleUpdated {
        document_id: EntityId,
        old_title: String,
        new_title: String,
        updated_by: Option<EntityId>,
        updated_at: Timestamp,
    },
    DocumentContentUpdated {
        document_id: EntityId,
        old_content: String,
        new_content: String,
        old_word_count: u32,
        new_word_count: u32,
        updated_by: Option<EntityId>,
        updated_at: Timestamp,
    },
    DocumentFilePathSet {
        document_id: EntityId,
        file_path: String,
        updated_by: Option<EntityId>,
        updated_at: Timestamp,
    },
    DocumentDeleted {
        document_id: EntityId,
        deleted_by: Option<EntityId>,
        deleted_at: Timestamp,
    },
    DocumentRestored {
        document_id: EntityId,
        restored_by: Option<EntityId>,
        restored_at: Timestamp,
    },
}

impl DomainEvent for DocumentEvent {
    fn event_id(&self) -> EntityId {
        // In a real implementation, this would be stored with the event
        EntityId::new()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DocumentEvent::DocumentCreated { created_at, .. } => created_at.as_datetime(),
            DocumentEvent::DocumentTitleUpdated { updated_at, .. } => updated_at.as_datetime(),
            DocumentEvent::DocumentContentUpdated { updated_at, .. } => updated_at.as_datetime(),
            DocumentEvent::DocumentFilePathSet { updated_at, .. } => updated_at.as_datetime(),
            DocumentEvent::DocumentDeleted { deleted_at, .. } => deleted_at.as_datetime(),
            DocumentEvent::DocumentRestored { restored_at, .. } => restored_at.as_datetime(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            DocumentEvent::DocumentCreated { .. } => "DocumentCreated",
            DocumentEvent::DocumentTitleUpdated { .. } => "DocumentTitleUpdated",
            DocumentEvent::DocumentContentUpdated { .. } => "DocumentContentUpdated",
            DocumentEvent::DocumentFilePathSet { .. } => "DocumentFilePathSet",
            DocumentEvent::DocumentDeleted { .. } => "DocumentDeleted",
            DocumentEvent::DocumentRestored { .. } => "DocumentRestored",
        }
    }

    fn aggregate_id(&self) -> EntityId {
        match self {
            DocumentEvent::DocumentCreated { document_id, .. } => *document_id,
            DocumentEvent::DocumentTitleUpdated { document_id, .. } => *document_id,
            DocumentEvent::DocumentContentUpdated { document_id, .. } => *document_id,
            DocumentEvent::DocumentFilePathSet { document_id, .. } => *document_id,
            DocumentEvent::DocumentDeleted { document_id, .. } => *document_id,
            DocumentEvent::DocumentRestored { document_id, .. } => *document_id,
        }
    }

    fn aggregate_version(&self) -> u64 {
        // In a real implementation, this would be tracked properly
        1
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), "writing".to_string());
        metadata.insert("aggregate_type".to_string(), "document".to_string());
        metadata
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
}

/// Project domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectEvent {
    ProjectCreated {
        project_id: EntityId,
        name: String,
        description: Option<String>,
        created_by: Option<EntityId>,
        created_at: Timestamp,
    },
    ProjectNameUpdated {
        project_id: EntityId,
        old_name: String,
        new_name: String,
        updated_by: Option<EntityId>,
        updated_at: Timestamp,
    },
    ProjectDescriptionUpdated {
        project_id: EntityId,
        old_description: Option<String>,
        new_description: Option<String>,
        updated_by: Option<EntityId>,
        updated_at: Timestamp,
    },
    DocumentAdded {
        project_id: EntityId,
        document_id: EntityId,
        document_title: String,
        added_by: Option<EntityId>,
        added_at: Timestamp,
    },
    DocumentRemoved {
        project_id: EntityId,
        document_id: EntityId,
        document_title: String,
        removed_by: Option<EntityId>,
        removed_at: Timestamp,
    },
}

impl DomainEvent for ProjectEvent {
    fn event_id(&self) -> EntityId {
        // In a real implementation, this would be stored with the event
        EntityId::new()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            ProjectEvent::ProjectCreated { created_at, .. } => created_at.as_datetime(),
            ProjectEvent::ProjectNameUpdated { updated_at, .. } => updated_at.as_datetime(),
            ProjectEvent::ProjectDescriptionUpdated { updated_at, .. } => updated_at.as_datetime(),
            ProjectEvent::DocumentAdded { added_at, .. } => added_at.as_datetime(),
            ProjectEvent::DocumentRemoved { removed_at, .. } => removed_at.as_datetime(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            ProjectEvent::ProjectCreated { .. } => "ProjectCreated",
            ProjectEvent::ProjectNameUpdated { .. } => "ProjectNameUpdated",
            ProjectEvent::ProjectDescriptionUpdated { .. } => "ProjectDescriptionUpdated",
            ProjectEvent::DocumentAdded { .. } => "DocumentAdded",
            ProjectEvent::DocumentRemoved { .. } => "DocumentRemoved",
        }
    }

    fn aggregate_id(&self) -> EntityId {
        match self {
            ProjectEvent::ProjectCreated { project_id, .. } => *project_id,
            ProjectEvent::ProjectNameUpdated { project_id, .. } => *project_id,
            ProjectEvent::ProjectDescriptionUpdated { project_id, .. } => *project_id,
            ProjectEvent::DocumentAdded { project_id, .. } => *project_id,
            ProjectEvent::DocumentRemoved { project_id, .. } => *project_id,
        }
    }

    fn aggregate_version(&self) -> u64 {
        // In a real implementation, this would be tracked properly
        1
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), "writing".to_string());
        metadata.insert("aggregate_type".to_string(), "project".to_string());
        metadata
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
}