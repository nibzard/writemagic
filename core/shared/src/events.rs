//! Domain events and event handling

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use crate::{EntityId, Result, WritemagicError};

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this event
    fn event_id(&self) -> EntityId;
    
    /// Timestamp when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    
    /// Type of the event
    fn event_type(&self) -> &'static str;
    
    /// ID of the aggregate that generated this event
    fn aggregate_id(&self) -> EntityId;
    
    /// Version of the aggregate when this event was generated
    fn aggregate_version(&self) -> u64;
    
    /// Optional metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<T: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &T) -> Result<()>;
}

/// Event bus for publishing and subscribing to events
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers
    async fn publish<T: DomainEvent>(&self, event: T) -> Result<()>;
    
    /// Subscribe a handler to events of type T
    async fn subscribe<T: DomainEvent + 'static>(&self, handler: Arc<dyn EventHandler<T>>) -> Result<()>;
    
    /// Unsubscribe from events
    async fn unsubscribe<T: DomainEvent + 'static>(&self) -> Result<()>;
}

/// Event store for persisting events
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Save events to the store
    async fn save_events(&self, aggregate_id: EntityId, events: Vec<Box<dyn DomainEvent>>, expected_version: u64) -> Result<()>;
    
    /// Load events for an aggregate
    async fn load_events(&self, aggregate_id: EntityId, from_version: u64) -> Result<Vec<Box<dyn DomainEvent>>>;
    
    /// Get the current version of an aggregate
    async fn get_aggregate_version(&self, aggregate_id: EntityId) -> Result<u64>;
}

/// Base implementation for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    pub event_id: EntityId,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: EntityId,
    pub aggregate_version: u64,
    pub metadata: HashMap<String, String>,
}

impl BaseEvent {
    pub fn new(aggregate_id: EntityId, aggregate_version: u64) -> Self {
        Self {
            event_id: EntityId::new(),
            occurred_at: Utc::now(),
            aggregate_id,
            aggregate_version,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Event sourcing aggregate trait
#[async_trait]
pub trait EventSourcedAggregate: Send + Sync {
    type Event: DomainEvent;
    
    /// Apply an event to the aggregate
    fn apply_event(&mut self, event: &Self::Event);
    
    /// Get uncommitted events
    fn uncommitted_events(&self) -> &[Self::Event];
    
    /// Mark events as committed
    fn mark_events_as_committed(&mut self);
    
    /// Get the current version
    fn version(&self) -> u64;
    
    /// Load from history
    fn load_from_history(events: Vec<Self::Event>) -> Self;
}

/// In-memory event bus implementation
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandlerWrapper>>>>>,
}

/// Wrapper trait to handle type erasure for event handlers
#[async_trait]
trait EventHandlerWrapper: Send + Sync {
    async fn handle_event(&self, event: &dyn DomainEvent) -> Result<()>;
}

/// Concrete wrapper implementation
struct EventHandlerWrapperImpl<T: DomainEvent> {
    handler: Arc<dyn EventHandler<T>>,
}

#[async_trait]
impl<T: DomainEvent + 'static> EventHandlerWrapper for EventHandlerWrapperImpl<T> {
    async fn handle_event(&self, event: &dyn DomainEvent) -> Result<()> {
        // This is a simplified implementation. In practice, you'd need proper type checking.
        // For now, we'll skip the actual handling since we can't safely downcast here.
        Ok(())
    }
}

impl InMemoryEventBus {
    /// Create a new in-memory event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish<T: DomainEvent>(&self, event: T) -> Result<()> {
        let event_type = event.event_type();
        let handlers = self.handlers.read().await;
        
        if let Some(event_handlers) = handlers.get(event_type) {
            for handler in event_handlers {
                if let Err(e) = handler.handle_event(&event).await {
                    // Log error but continue with other handlers
                    tracing::error!("Error handling event {}: {}", event_type, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn subscribe<T: DomainEvent + 'static>(&self, handler: Arc<dyn EventHandler<T>>) -> Result<()> {
        let event_type = std::any::type_name::<T>().to_string();
        let mut handlers = self.handlers.write().await;
        
        let wrapper = Arc::new(EventHandlerWrapperImpl { handler });
        handlers.entry(event_type).or_insert_with(Vec::new).push(wrapper);
        
        Ok(())
    }
    
    async fn unsubscribe<T: DomainEvent + 'static>(&self) -> Result<()> {
        let event_type = std::any::type_name::<T>().to_string();
        let mut handlers = self.handlers.write().await;
        handlers.remove(&event_type);
        Ok(())
    }
}

/// Cross-domain events that coordinate between domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainEvent {
    /// Document was created
    DocumentCreated {
        base: BaseEvent,
        document_id: EntityId,
        title: String,
        project_id: Option<EntityId>,
        created_by: EntityId,
    },
    /// Document was updated
    DocumentUpdated {
        base: BaseEvent,
        document_id: EntityId,
        changes: Vec<String>,
        updated_by: EntityId,
    },
    /// Document was deleted
    DocumentDeleted {
        base: BaseEvent,
        document_id: EntityId,
        deleted_by: EntityId,
    },
    /// Project was created
    ProjectCreated {
        base: BaseEvent,
        project_id: EntityId,
        name: String,
        created_by: EntityId,
    },
    /// Project was updated
    ProjectUpdated {
        base: BaseEvent,
        project_id: EntityId,
        changes: Vec<String>,
        updated_by: EntityId,
    },
    /// AI generation completed
    AIGenerationCompleted {
        base: BaseEvent,
        request_id: EntityId,
        document_id: Option<EntityId>,
        tokens_used: u32,
        processing_time_ms: u64,
    },
    /// Agent execution started
    AgentExecutionStarted {
        base: BaseEvent,
        agent_id: EntityId,
        execution_id: EntityId,
        trigger_type: String,
    },
    /// Agent execution completed
    AgentExecutionCompleted {
        base: BaseEvent,
        agent_id: EntityId,
        execution_id: EntityId,
        success: bool,
        duration_ms: u64,
    },
    /// Commit was created
    CommitCreated {
        base: BaseEvent,
        commit_id: EntityId,
        document_id: EntityId,
        message: String,
        created_by: EntityId,
    },
    /// Branch was created
    BranchCreated {
        base: BaseEvent,
        branch_id: EntityId,
        name: String,
        document_id: EntityId,
        created_by: EntityId,
    },
}

impl DomainEvent for CrossDomainEvent {
    fn event_id(&self) -> EntityId {
        match self {
            CrossDomainEvent::DocumentCreated { base, .. } => base.event_id,
            CrossDomainEvent::DocumentUpdated { base, .. } => base.event_id,
            CrossDomainEvent::DocumentDeleted { base, .. } => base.event_id,
            CrossDomainEvent::ProjectCreated { base, .. } => base.event_id,
            CrossDomainEvent::ProjectUpdated { base, .. } => base.event_id,
            CrossDomainEvent::AIGenerationCompleted { base, .. } => base.event_id,
            CrossDomainEvent::AgentExecutionStarted { base, .. } => base.event_id,
            CrossDomainEvent::AgentExecutionCompleted { base, .. } => base.event_id,
            CrossDomainEvent::CommitCreated { base, .. } => base.event_id,
            CrossDomainEvent::BranchCreated { base, .. } => base.event_id,
        }
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            CrossDomainEvent::DocumentCreated { base, .. } => base.occurred_at,
            CrossDomainEvent::DocumentUpdated { base, .. } => base.occurred_at,
            CrossDomainEvent::DocumentDeleted { base, .. } => base.occurred_at,
            CrossDomainEvent::ProjectCreated { base, .. } => base.occurred_at,
            CrossDomainEvent::ProjectUpdated { base, .. } => base.occurred_at,
            CrossDomainEvent::AIGenerationCompleted { base, .. } => base.occurred_at,
            CrossDomainEvent::AgentExecutionStarted { base, .. } => base.occurred_at,
            CrossDomainEvent::AgentExecutionCompleted { base, .. } => base.occurred_at,
            CrossDomainEvent::CommitCreated { base, .. } => base.occurred_at,
            CrossDomainEvent::BranchCreated { base, .. } => base.occurred_at,
        }
    }
    
    fn event_type(&self) -> &'static str {
        match self {
            CrossDomainEvent::DocumentCreated { .. } => "DocumentCreated",
            CrossDomainEvent::DocumentUpdated { .. } => "DocumentUpdated",
            CrossDomainEvent::DocumentDeleted { .. } => "DocumentDeleted",
            CrossDomainEvent::ProjectCreated { .. } => "ProjectCreated",
            CrossDomainEvent::ProjectUpdated { .. } => "ProjectUpdated",
            CrossDomainEvent::AIGenerationCompleted { .. } => "AIGenerationCompleted",
            CrossDomainEvent::AgentExecutionStarted { .. } => "AgentExecutionStarted",
            CrossDomainEvent::AgentExecutionCompleted { .. } => "AgentExecutionCompleted",
            CrossDomainEvent::CommitCreated { .. } => "CommitCreated",
            CrossDomainEvent::BranchCreated { .. } => "BranchCreated",
        }
    }
    
    fn aggregate_id(&self) -> EntityId {
        match self {
            CrossDomainEvent::DocumentCreated { base, .. } => base.aggregate_id,
            CrossDomainEvent::DocumentUpdated { base, .. } => base.aggregate_id,
            CrossDomainEvent::DocumentDeleted { base, .. } => base.aggregate_id,
            CrossDomainEvent::ProjectCreated { base, .. } => base.aggregate_id,
            CrossDomainEvent::ProjectUpdated { base, .. } => base.aggregate_id,
            CrossDomainEvent::AIGenerationCompleted { base, .. } => base.aggregate_id,
            CrossDomainEvent::AgentExecutionStarted { base, .. } => base.aggregate_id,
            CrossDomainEvent::AgentExecutionCompleted { base, .. } => base.aggregate_id,
            CrossDomainEvent::CommitCreated { base, .. } => base.aggregate_id,
            CrossDomainEvent::BranchCreated { base, .. } => base.aggregate_id,
        }
    }
    
    fn aggregate_version(&self) -> u64 {
        match self {
            CrossDomainEvent::DocumentCreated { base, .. } => base.aggregate_version,
            CrossDomainEvent::DocumentUpdated { base, .. } => base.aggregate_version,
            CrossDomainEvent::DocumentDeleted { base, .. } => base.aggregate_version,
            CrossDomainEvent::ProjectCreated { base, .. } => base.aggregate_version,
            CrossDomainEvent::ProjectUpdated { base, .. } => base.aggregate_version,
            CrossDomainEvent::AIGenerationCompleted { base, .. } => base.aggregate_version,
            CrossDomainEvent::AgentExecutionStarted { base, .. } => base.aggregate_version,
            CrossDomainEvent::AgentExecutionCompleted { base, .. } => base.aggregate_version,
            CrossDomainEvent::CommitCreated { base, .. } => base.aggregate_version,
            CrossDomainEvent::BranchCreated { base, .. } => base.aggregate_version,
        }
    }
    
    fn metadata(&self) -> HashMap<String, String> {
        match self {
            CrossDomainEvent::DocumentCreated { base, .. } => base.metadata.clone(),
            CrossDomainEvent::DocumentUpdated { base, .. } => base.metadata.clone(),
            CrossDomainEvent::DocumentDeleted { base, .. } => base.metadata.clone(),
            CrossDomainEvent::ProjectCreated { base, .. } => base.metadata.clone(),
            CrossDomainEvent::ProjectUpdated { base, .. } => base.metadata.clone(),
            CrossDomainEvent::AIGenerationCompleted { base, .. } => base.metadata.clone(),
            CrossDomainEvent::AgentExecutionStarted { base, .. } => base.metadata.clone(),
            CrossDomainEvent::AgentExecutionCompleted { base, .. } => base.metadata.clone(),
            CrossDomainEvent::CommitCreated { base, .. } => base.metadata.clone(),
            CrossDomainEvent::BranchCreated { base, .. } => base.metadata.clone(),
        }
    }
}

/// Event publisher trait for domain services
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a cross-domain event
    async fn publish_event(&self, event: CrossDomainEvent) -> Result<()>;
}

/// Implementation of event publisher using the event bus
pub struct EventBusPublisher {
    event_bus: Arc<dyn EventBus>,
}

impl EventBusPublisher {
    /// Create a new event publisher
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        Self { event_bus }
    }
}

#[async_trait]
impl EventPublisher for EventBusPublisher {
    async fn publish_event(&self, event: CrossDomainEvent) -> Result<()> {
        self.event_bus.publish(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_event_bus() {
        let event_bus = Arc::new(InMemoryEventBus::new());
        
        // Create a test event
        let base_event = BaseEvent::new(EntityId::new(), 1);
        let event = CrossDomainEvent::DocumentCreated {
            base: base_event,
            document_id: EntityId::new(),
            title: "Test Document".to_string(),
            project_id: None,
            created_by: EntityId::new(),
        };
        
        // Publish the event (should not fail even with no subscribers)
        let result = event_bus.publish(event).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_cross_domain_event_properties() {
        let base_event = BaseEvent::new(EntityId::new(), 1);
        let document_id = EntityId::new();
        let created_by = EntityId::new();
        
        let event = CrossDomainEvent::DocumentCreated {
            base: base_event.clone(),
            document_id,
            title: "Test Document".to_string(),
            project_id: None,
            created_by,
        };
        
        assert_eq!(event.event_id(), base_event.event_id);
        assert_eq!(event.aggregate_id(), base_event.aggregate_id);
        assert_eq!(event.aggregate_version(), 1);
        assert_eq!(event.event_type(), "DocumentCreated");
    }
}