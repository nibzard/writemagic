//! Writing domain aggregates

use crate::entities::{Document, Project};
use crate::events::{DocumentEvent, ProjectEvent};
use crate::value_objects::{DocumentTitle, DocumentContent, ProjectName, TextSelection};
use writemagic_shared::{EntityId, Timestamp, ContentType, FilePath, Result, WritemagicError};
use std::collections::HashMap;

/// Document aggregate with business logic and invariants
#[derive(Debug, Clone)]
pub struct DocumentAggregate {
    document: Document,
    uncommitted_events: Vec<DocumentEvent>,
    collaborators: HashMap<EntityId, String>, // user_id -> display_name
    edit_history: Vec<EditOperation>,
}

impl DocumentAggregate {
    pub fn new(title: DocumentTitle, content: DocumentContent, content_type: ContentType, created_by: Option<EntityId>) -> Self {
        let document = Document::new(title.value.clone(), content.value.clone(), content_type.clone(), created_by);
        let event = DocumentEvent::DocumentCreated {
            document_id: document.id,
            title: title.value.clone(),
            content_type: content_type.clone(),
            created_by,
            created_at: document.created_at.clone(),
        };

        Self {
            document,
            uncommitted_events: vec![event],
            collaborators: HashMap::new(),
            edit_history: Vec::new(),
        }
    }

    pub fn load_from_document(document: Document) -> Self {
        Self {
            document,
            uncommitted_events: Vec::new(),
            collaborators: HashMap::new(),
            edit_history: Vec::new(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn update_title(&mut self, title: DocumentTitle, updated_by: Option<EntityId>) -> Result<()> {
        if self.document.is_deleted {
            return Err(WritemagicError::validation("Cannot update deleted document"));
        }

        let old_title = self.document.title.clone();
        self.document.update_title(title.value.clone(), updated_by);

        let event = DocumentEvent::DocumentTitleUpdated {
            document_id: self.document.id,
            old_title,
            new_title: title.value,
            updated_by,
            updated_at: self.document.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn update_content(&mut self, content: DocumentContent, selection: Option<TextSelection>, updated_by: Option<EntityId>) -> Result<()> {
        if self.document.is_deleted {
            return Err(WritemagicError::validation("Cannot update deleted document"));
        }

        let old_content = self.document.content.clone();
        let old_word_count = self.document.word_count;
        
        self.document.update_content(content.value.clone(), updated_by);

        // Record edit operation
        let edit_op = EditOperation {
            id: EntityId::new(),
            document_id: self.document.id,
            operation_type: EditOperationType::ContentUpdate,
            selection,
            old_text: old_content.clone(),
            new_text: content.value.clone(),
            timestamp: Timestamp::now(),
            user_id: updated_by,
        };
        self.edit_history.push(edit_op);

        let event = DocumentEvent::DocumentContentUpdated {
            document_id: self.document.id,
            old_content,
            new_content: content.value,
            old_word_count,
            new_word_count: self.document.word_count,
            updated_by,
            updated_at: self.document.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn set_file_path(&mut self, file_path: FilePath, updated_by: Option<EntityId>) -> Result<()> {
        if self.document.is_deleted {
            return Err(WritemagicError::validation("Cannot update deleted document"));
        }

        self.document.set_file_path(file_path.clone(), updated_by);

        let event = DocumentEvent::DocumentFilePathSet {
            document_id: self.document.id,
            file_path: file_path.path,
            updated_by,
            updated_at: self.document.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn delete(&mut self, deleted_by: Option<EntityId>) -> Result<()> {
        if self.document.is_deleted {
            return Err(WritemagicError::validation("Document is already deleted"));
        }

        self.document.mark_deleted(deleted_by);

        let event = DocumentEvent::DocumentDeleted {
            document_id: self.document.id,
            deleted_by,
            deleted_at: self.document.deleted_at.clone().unwrap(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn restore(&mut self, restored_by: Option<EntityId>) -> Result<()> {
        if !self.document.is_deleted {
            return Err(WritemagicError::validation("Document is not deleted"));
        }

        self.document.restore(restored_by);

        let event = DocumentEvent::DocumentRestored {
            document_id: self.document.id,
            restored_by,
            restored_at: self.document.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn add_collaborator(&mut self, user_id: EntityId, display_name: String) {
        self.collaborators.insert(user_id, display_name);
    }

    pub fn remove_collaborator(&mut self, user_id: &EntityId) {
        self.collaborators.remove(user_id);
    }

    pub fn collaborators(&self) -> &HashMap<EntityId, String> {
        &self.collaborators
    }

    pub fn edit_history(&self) -> &[EditOperation] {
        &self.edit_history
    }

    pub fn uncommitted_events(&self) -> &[DocumentEvent] {
        &self.uncommitted_events
    }

    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
}

/// Project aggregate with business logic and invariants
#[derive(Debug, Clone)]
pub struct ProjectAggregate {
    project: Project,
    uncommitted_events: Vec<ProjectEvent>,
    document_metadata: HashMap<EntityId, DocumentMetadata>,
}

impl ProjectAggregate {
    pub fn new(name: ProjectName, description: Option<String>, created_by: Option<EntityId>) -> Self {
        let project = Project::new(name.value.clone(), description.clone(), created_by);
        let event = ProjectEvent::ProjectCreated {
            project_id: project.id,
            name: name.value,
            description,
            created_by,
            created_at: project.created_at.clone(),
        };

        Self {
            project,
            uncommitted_events: vec![event],
            document_metadata: HashMap::new(),
        }
    }

    pub fn load_from_project(project: Project) -> Self {
        Self {
            project,
            uncommitted_events: Vec::new(),
            document_metadata: HashMap::new(),
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn add_document(&mut self, document_id: EntityId, document_title: String, updated_by: Option<EntityId>) -> Result<()> {
        if self.project.is_deleted {
            return Err(WritemagicError::validation("Cannot add document to deleted project"));
        }

        if self.project.document_ids.len() >= 1000 {
            return Err(WritemagicError::validation("Project cannot have more than 1000 documents"));
        }

        self.project.add_document(document_id, updated_by);

        // Store document metadata
        self.document_metadata.insert(document_id, DocumentMetadata {
            title: document_title.clone(),
            added_at: Timestamp::now(),
            added_by: updated_by,
        });

        let event = ProjectEvent::DocumentAdded {
            project_id: self.project.id,
            document_id,
            document_title,
            added_by: updated_by,
            added_at: self.project.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn remove_document(&mut self, document_id: &EntityId, updated_by: Option<EntityId>) -> Result<()> {
        if self.project.is_deleted {
            return Err(WritemagicError::validation("Cannot remove document from deleted project"));
        }

        if !self.project.document_ids.contains(document_id) {
            return Err(WritemagicError::validation("Document is not part of this project"));
        }

        self.project.remove_document(document_id, updated_by);
        let document_metadata = self.document_metadata.remove(document_id);

        let event = ProjectEvent::DocumentRemoved {
            project_id: self.project.id,
            document_id: *document_id,
            document_title: document_metadata.map(|m| m.title).unwrap_or_default(),
            removed_by: updated_by,
            removed_at: self.project.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn update_name(&mut self, name: ProjectName, updated_by: Option<EntityId>) -> Result<()> {
        if self.project.is_deleted {
            return Err(WritemagicError::validation("Cannot update deleted project"));
        }

        let old_name = self.project.name.clone();
        self.project.update_name(name.value.clone(), updated_by);

        let event = ProjectEvent::ProjectNameUpdated {
            project_id: self.project.id,
            old_name,
            new_name: name.value,
            updated_by,
            updated_at: self.project.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn update_description(&mut self, description: Option<String>, updated_by: Option<EntityId>) -> Result<()> {
        if self.project.is_deleted {
            return Err(WritemagicError::validation("Cannot update deleted project"));
        }

        let old_description = self.project.description.clone();
        self.project.update_description(description.clone(), updated_by);

        let event = ProjectEvent::ProjectDescriptionUpdated {
            project_id: self.project.id,
            old_description,
            new_description: description,
            updated_by,
            updated_at: self.project.updated_at.clone(),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    pub fn document_metadata(&self) -> &HashMap<EntityId, DocumentMetadata> {
        &self.document_metadata
    }

    pub fn uncommitted_events(&self) -> &[ProjectEvent] {
        &self.uncommitted_events
    }

    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
}

/// Edit operation tracking
#[derive(Debug, Clone)]
pub struct EditOperation {
    pub id: EntityId,
    pub document_id: EntityId,
    pub operation_type: EditOperationType,
    pub selection: Option<TextSelection>,
    pub old_text: String,
    pub new_text: String,
    pub timestamp: Timestamp,
    pub user_id: Option<EntityId>,
}

#[derive(Debug, Clone)]
pub enum EditOperationType {
    ContentUpdate,
    Insert,
    Delete,
    Replace,
    Paste,
    Cut,
}

/// Document metadata within project
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: String,
    pub added_at: Timestamp,
    pub added_by: Option<EntityId>,
}