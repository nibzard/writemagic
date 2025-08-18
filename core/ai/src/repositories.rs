//! AI domain repositories

use async_trait::async_trait;
use writemagic_shared::{EntityId, Pagination, Repository, Result};
use crate::entities::{Conversation, Completion};

/// Conversation repository interface
#[async_trait]
pub trait ConversationRepository: Repository<Conversation, EntityId> + Send + Sync {
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Conversation>>;
    async fn find_by_provider(&self, provider_name: &str, pagination: Pagination) -> Result<Vec<Conversation>>;
    async fn find_recently_active(&self, pagination: Pagination) -> Result<Vec<Conversation>>;
    async fn get_total_cost_by_user(&self, user_id: &EntityId) -> Result<f64>;
}

/// Completion repository interface
#[async_trait]
pub trait CompletionRepository: Repository<Completion, EntityId> + Send + Sync {
    async fn find_by_conversation(&self, conversation_id: &EntityId, pagination: Pagination) -> Result<Vec<Completion>>;
    async fn find_by_status(&self, status: &crate::entities::CompletionStatus, pagination: Pagination) -> Result<Vec<Completion>>;
    async fn get_usage_stats(&self, user_id: Option<&EntityId>) -> Result<UsageStatistics>;
}

/// Usage statistics
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    pub total_completions: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub completions_by_provider: std::collections::HashMap<String, u64>,
    pub tokens_by_provider: std::collections::HashMap<String, u64>,
    pub cost_by_provider: std::collections::HashMap<String, f64>,
}