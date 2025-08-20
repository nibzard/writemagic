//! AI domain entities

use serde::{Deserialize, Serialize};
use writemagic_shared::{EntityId, Timestamp, Entity, AggregateRoot, Versioned};
use crate::providers::{CompletionRequest, CompletionResponse};

/// AI conversation entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: EntityId,
    pub title: String,
    pub provider_name: String,
    pub model_name: String,
    pub message_count: u32,
    pub total_tokens: u32,
    pub total_cost: f64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: Option<EntityId>,
    pub updated_by: Option<EntityId>,
    pub version: u64,
    pub is_deleted: bool,
    pub deleted_at: Option<Timestamp>,
}

impl Conversation {
    pub fn new(title: String, provider_name: String, model_name: String, created_by: Option<EntityId>) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(),
            title,
            provider_name,
            model_name,
            message_count: 0,
            total_tokens: 0,
            total_cost: 0.0,
            created_at: now.clone(),
            updated_at: now,
            created_by,
            updated_by: created_by,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        }
    }

    pub fn add_exchange(&mut self, request: &CompletionRequest, response: &CompletionResponse, cost: f64) {
        self.message_count += request.messages.len() as u32 + 1; // +1 for response
        self.total_tokens += response.usage.total_tokens;
        self.total_cost += cost;
        self.updated_at = Timestamp::now();
        self.increment_version();
    }
}

impl Entity for Conversation {
    type Id = EntityId;
    fn id(&self) -> &Self::Id { &self.id }
}

impl AggregateRoot for Conversation {
    type Id = EntityId;
    fn id(&self) -> &Self::Id { &self.id }
    fn created_at(&self) -> &Timestamp { &self.created_at }
    fn updated_at(&self) -> &Timestamp { &self.updated_at }
}

impl Versioned for Conversation {
    fn version(&self) -> u64 { self.version }
    fn increment_version(&mut self) { self.version += 1; }
}

/// AI completion entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub id: EntityId,
    pub conversation_id: EntityId,
    pub provider_name: String,
    pub model_name: String,
    pub request: CompletionRequest,
    pub response: Option<CompletionResponse>,
    pub status: CompletionStatus,
    pub error_message: Option<String>,
    pub cost: f64,
    pub created_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl Entity for Completion {
    type Id = EntityId;
    fn id(&self) -> &Self::Id { &self.id }
}