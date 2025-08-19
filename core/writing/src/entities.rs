//! Writing domain entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use writemagic_shared::{EntityId, Timestamp, ContentHash, FilePath, ContentType, Entity, AggregateRoot, Auditable, Versioned};

/// Document entity representing a single document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: EntityId,
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub content_hash: ContentHash,
    pub file_path: Option<FilePath>,
    pub word_count: u32,
    pub character_count: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: Option<EntityId>,
    pub updated_by: Option<EntityId>,
    pub version: u64,
    pub is_deleted: bool,
    pub deleted_at: Option<Timestamp>,
}

impl Document {
    pub fn new(title: String, content: String, content_type: ContentType, created_by: Option<EntityId>) -> Self {
        let now = Timestamp::now();
        let content_hash = ContentHash::new(&content);
        let word_count = Self::count_words(&content);
        let character_count = content.len() as u32;

        Self {
            id: EntityId::new(),
            title,
            content,
            content_type,
            content_hash,
            file_path: None,
            word_count,
            character_count,
            created_at: now.clone(),
            updated_at: now,
            created_by,
            updated_by: created_by,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        }
    }

    pub fn update_content(&mut self, content: String, updated_by: Option<EntityId>) {
        if self.content != content {
            // Calculate metrics before moving content
            let content_hash = ContentHash::new(&content);
            let word_count = Self::count_words(&content);
            let character_count = content.len() as u32;
            
            // Move content to avoid clone
            self.content = content;
            self.content_hash = content_hash;
            self.word_count = word_count;
            self.character_count = character_count;
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }

    pub fn update_title(&mut self, title: String, updated_by: Option<EntityId>) {
        if self.title != title {
            self.title = title;
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }

    pub fn set_file_path(&mut self, file_path: FilePath, updated_by: Option<EntityId>) {
        self.file_path = Some(file_path);
        self.updated_at = Timestamp::now();
        self.updated_by = updated_by;
        self.increment_version();
    }

    pub fn mark_deleted(&mut self, deleted_by: Option<EntityId>) {
        if !self.is_deleted {
            self.is_deleted = true;
            self.deleted_at = Some(Timestamp::now());
            self.updated_by = deleted_by;
            self.increment_version();
        }
    }

    pub fn restore(&mut self, restored_by: Option<EntityId>) {
        if self.is_deleted {
            self.is_deleted = false;
            self.deleted_at = None;
            self.updated_at = Timestamp::now();
            self.updated_by = restored_by;
            self.increment_version();
        }
    }

    fn count_words(content: &str) -> u32 {
        content
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .count() as u32
    }
}

impl Entity for Document {
    type Id = EntityId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Document {
    type Id = EntityId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }
}

impl Auditable for Document {
    fn created_by(&self) -> Option<&EntityId> {
        self.created_by.as_ref()
    }

    fn updated_by(&self) -> Option<&EntityId> {
        self.updated_by.as_ref()
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }
}

impl Versioned for Document {
    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

/// Project entity representing a collection of documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: EntityId,
    pub name: String,
    pub description: Option<String>,
    pub document_ids: Vec<EntityId>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: Option<EntityId>,
    pub updated_by: Option<EntityId>,
    pub version: u64,
    pub is_deleted: bool,
    pub deleted_at: Option<Timestamp>,
}

impl Project {
    pub fn new(name: String, description: Option<String>, created_by: Option<EntityId>) -> Self {
        let now = Timestamp::now();

        Self {
            id: EntityId::new(),
            name,
            description,
            document_ids: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            created_by,
            updated_by: created_by,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        }
    }

    pub fn add_document(&mut self, document_id: EntityId, updated_by: Option<EntityId>) {
        if !self.document_ids.contains(&document_id) {
            self.document_ids.push(document_id);
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }

    pub fn remove_document(&mut self, document_id: &EntityId, updated_by: Option<EntityId>) {
        if let Some(pos) = self.document_ids.iter().position(|id| id == document_id) {
            self.document_ids.remove(pos);
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }

    pub fn update_name(&mut self, name: String, updated_by: Option<EntityId>) {
        if self.name != name {
            self.name = name;
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }

    pub fn update_description(&mut self, description: Option<String>, updated_by: Option<EntityId>) {
        if self.description != description {
            self.description = description;
            self.updated_at = Timestamp::now();
            self.updated_by = updated_by;
            self.increment_version();
        }
    }
}

impl Entity for Project {
    type Id = EntityId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Project {
    type Id = EntityId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }
}

impl Auditable for Project {
    fn created_by(&self) -> Option<&EntityId> {
        self.created_by.as_ref()
    }

    fn updated_by(&self) -> Option<&EntityId> {
        self.updated_by.as_ref()
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }
}

impl Versioned for Project {
    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}