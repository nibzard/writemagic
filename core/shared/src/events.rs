//! Domain events and event handling

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{EntityId, Result};

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync + std::fmt::Debug + Any {
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
    
    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<T: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &T) -> Result<()>;
}

/// Type-erased event handler
type DynEventHandler = Arc<dyn Fn(&dyn DomainEvent) -> Result<()> + Send + Sync>;

/// Event bus for publishing and subscribing to events
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers
    async fn publish(&self, event: Box<dyn DomainEvent>) -> Result<()>;
    
    /// Subscribe a handler to events of a specific type
    async fn subscribe(&self, event_type: TypeId, handler: DynEventHandler) -> Result<()>;
    
    /// Unsubscribe from events of a specific type
    async fn unsubscribe(&self, event_type: TypeId) -> Result<()>;
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
    handlers: Arc<RwLock<HashMap<TypeId, Vec<DynEventHandler>>>>,
}

impl InMemoryEventBus {
    /// Create a new in-memory event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Type-safe helper for publishing events
    pub async fn publish_typed<T: DomainEvent + 'static>(&self, event: T) -> Result<()> {
        self.publish(Box::new(event)).await
    }
    
    /// Type-safe helper for subscribing to events
    pub async fn subscribe_typed<T: DomainEvent + 'static, F>(&self, handler: F) -> Result<()>
    where
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        let handler = Arc::new(move |event: &dyn DomainEvent| {
            if let Some(typed_event) = event.as_any().downcast_ref::<T>() {
                handler(typed_event)
            } else {
                Ok(())
            }
        });
        self.subscribe(TypeId::of::<T>(), handler).await
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: Box<dyn DomainEvent>) -> Result<()> {
        let event_type_id = event.as_any().type_id();
        let handlers = self.handlers.read().await;
        
        if let Some(event_handlers) = handlers.get(&event_type_id) {
            for handler in event_handlers {
                if let Err(e) = handler(event.as_ref()) {
                    // Log error but continue with other handlers
                    tracing::error!("Error handling event {}: {}", event.event_type(), e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn subscribe(&self, event_type: TypeId, handler: DynEventHandler) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.entry(event_type).or_insert_with(Vec::new).push(handler);
        Ok(())
    }
    
    async fn unsubscribe(&self, event_type: TypeId) -> Result<()> {
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
    fn as_any(&self) -> &dyn Any {
        self
    }
    
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
    event_bus: Arc<InMemoryEventBus>, // Use concrete type for now
}

impl EventBusPublisher {
    /// Create a new event publisher
    pub fn new(event_bus: Arc<InMemoryEventBus>) -> Self {
        Self { event_bus }
    }
}

#[async_trait]
impl EventPublisher for EventBusPublisher {
    async fn publish_event(&self, event: CrossDomainEvent) -> Result<()> {
        self.event_bus.publish(Box::new(event)).await
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
        let result = event_bus.publish(Box::new(event)).await;
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