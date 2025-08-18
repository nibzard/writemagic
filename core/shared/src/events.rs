//! Domain events and event handling

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{EntityId, Result};

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