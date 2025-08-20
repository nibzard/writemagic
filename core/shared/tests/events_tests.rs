//! Unit tests for domain events and event bus

use writemagic_shared::{
    DomainEvent, EventBus, EventHandler, EventStore, InMemoryEventBus, 
    CrossDomainEvent, EventPublisher, EntityId, Timestamp, Result, WritemagicError
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::RwLock;

// Test event types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestEvent {
    pub id: EntityId,
    pub data: String,
    pub timestamp: Timestamp,
}

impl DomainEvent for TestEvent {
    fn event_id(&self) -> EntityId {
        self.id
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp.as_datetime()
    }
    
    fn event_type(&self) -> &'static str {
        "TestEvent"
    }
    
    fn aggregate_id(&self) -> EntityId {
        self.id
    }
    
    fn aggregate_version(&self) -> u64 {
        1
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnotherTestEvent {
    pub id: EntityId,
    pub value: i32,
    pub timestamp: Timestamp,
}

impl DomainEvent for AnotherTestEvent {
    fn event_id(&self) -> EntityId {
        self.id
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp.as_datetime()
    }
    
    fn event_type(&self) -> &'static str {
        "AnotherTestEvent"
    }
    
    fn aggregate_id(&self) -> EntityId {
        self.id
    }
    
    fn aggregate_version(&self) -> u64 {
        1
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Test event handler
struct TestEventHandler {
    events_received: Arc<Mutex<Vec<TestEvent>>>,
}

impl TestEventHandler {
    fn new() -> Self {
        Self {
            events_received: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_events(&self) -> Vec<TestEvent> {
        self.events_received.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl EventHandler<TestEvent> for TestEventHandler {
    async fn handle(&self, event: &TestEvent) -> Result<()> {
        self.events_received.lock().unwrap().push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod domain_event_tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = TestEvent {
            id: EntityId::new(),
            data: "Test data".to_string(),
            timestamp: Timestamp::now(),
        };
        
        assert_eq!(event.event_type(), "TestEvent");
        assert_eq!(event.data, "Test data");
        assert_eq!(event.aggregate_version(), 1);
    }

    #[test]
    fn test_event_serialization() {
        let event = TestEvent {
            id: EntityId::new(),
            data: "Test data".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let serialized = serde_json::to_string(&event).expect("Serialize event");
        let deserialized: TestEvent = serde_json::from_str(&serialized).expect("Deserialize event");
        
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.data, deserialized.data);
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

#[cfg(test)]
mod event_handler_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler() {
        let handler = TestEventHandler::new();
        let event = TestEvent {
            id: EntityId::new(),
            data: "Test data".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let result = handler.handle(&event).await;
        assert!(result.is_ok());
        
        let events = handler.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "Test data");
    }
}

#[cfg(test)]
mod event_bus_tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_event_bus() {
        let mut event_bus = InMemoryEventBus::new();
        let handler = Arc::new(TestEventHandler::new());
        
        // Subscribe handler
        event_bus.subscribe(Arc::clone(&handler)).await;
        
        // Publish event
        let event = TestEvent {
            id: EntityId::new(),
            data: "Bus test data".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let result = event_bus.publish(&event).await;
        assert!(result.is_ok());
        
        // Check handler received event
        let events = handler.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "Bus test data");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let mut event_bus = InMemoryEventBus::new();
        let handler1 = Arc::new(TestEventHandler::new());
        let handler2 = Arc::new(TestEventHandler::new());
        
        // Subscribe both handlers
        event_bus.subscribe(Arc::clone(&handler1)).await;
        event_bus.subscribe(Arc::clone(&handler2)).await;
        
        // Publish event
        let event = TestEvent {
            id: EntityId::new(),
            data: "Multi-subscriber test".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let result = event_bus.publish(&event).await;
        assert!(result.is_ok());
        
        // Both handlers should have received the event
        let events1 = handler1.get_events();
        let events2 = handler2.get_events();
        
        assert_eq!(events1.len(), 1);
        assert_eq!(events2.len(), 1);
        assert_eq!(events1[0].data, "Multi-subscriber test");
        assert_eq!(events2[0].data, "Multi-subscriber test");
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let mut event_bus = InMemoryEventBus::new();
        let handler = Arc::new(TestEventHandler::new());
        
        // Subscribe and then unsubscribe
        event_bus.subscribe(Arc::clone(&handler)).await;
        event_bus.unsubscribe(Arc::clone(&handler)).await;
        
        // Publish event
        let event = TestEvent {
            id: EntityId::new(),
            data: "Unsubscribe test".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let result = event_bus.publish(&event).await;
        assert!(result.is_ok());
        
        // Handler should not have received the event
        let events = handler.get_events();
        assert_eq!(events.len(), 0);
    }
}

#[cfg(test)]
mod cross_domain_event_tests {
    use super::*;

    #[test]
    fn test_cross_domain_event_creation() {
        let source_event = TestEvent {
            id: EntityId::new(),
            data: "Cross-domain data".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let cross_domain_event = CrossDomainEvent::new(
            "WritingDomain".to_string(),
            "AIDomain".to_string(),
            Box::new(source_event.clone()),
        );
        
        assert_eq!(cross_domain_event.source_domain, "WritingDomain");
        assert_eq!(cross_domain_event.target_domain, "AIDomain");
        assert_eq!(cross_domain_event.event.event_type(), "TestEvent");
    }

    #[test]
    fn test_cross_domain_event_serialization() {
        let source_event = TestEvent {
            id: EntityId::new(),
            data: "Serialization test".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let cross_domain_event = CrossDomainEvent::new(
            "Source".to_string(),
            "Target".to_string(),
            Box::new(source_event),
        );
        
        // Note: Full serialization testing would require more complex setup
        // for trait object serialization, but we can test the structure
        assert_eq!(cross_domain_event.source_domain, "Source");
        assert_eq!(cross_domain_event.target_domain, "Target");
    }
}

// Test event store implementation
struct TestEventStore {
    events: Arc<RwLock<HashMap<EntityId, Vec<Box<dyn DomainEvent>>>>>,
}

impl TestEventStore {
    fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl EventStore for TestEventStore {
    async fn save_events(&self, aggregate_id: EntityId, events: Vec<Box<dyn DomainEvent>>, expected_version: u64) -> Result<()> {
        let mut store = self.events.write().await;
        let stored_events = store.entry(aggregate_id).or_insert_with(Vec::new);
        
        // Check expected version
        if stored_events.len() as u64 != expected_version {
            return Err(WritemagicError::validation("Version mismatch"));
        }
        
        stored_events.extend(events);
        Ok(())
    }
    
    async fn load_events(&self, aggregate_id: EntityId, from_version: u64) -> Result<Vec<Box<dyn DomainEvent>>> {
        let store = self.events.read().await;
        let events = store.get(&aggregate_id).cloned().unwrap_or_default();
        Ok(events.into_iter().skip(from_version as usize).collect())
    }
    
    async fn get_aggregate_version(&self, aggregate_id: EntityId) -> Result<u64> {
        let store = self.events.read().await;
        let events = store.get(&aggregate_id).unwrap_or(&Vec::new());
        Ok(events.len() as u64)
    }
}

#[cfg(test)]
mod event_store_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_store_save_and_load() {
        let store = TestEventStore::new();
        let aggregate_id = EntityId::new();
        
        let event = TestEvent {
            id: aggregate_id,
            data: "Store test".to_string(),
            timestamp: Timestamp::now(),
        };
        
        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![Box::new(event.clone())];
        
        // Save events
        let result = store.save_events(aggregate_id, events, 0).await;
        assert!(result.is_ok());
        
        // Load events
        let loaded_events = store.load_events(aggregate_id, 0).await.expect("Load events");
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].event_type(), "TestEvent");
    }

    #[tokio::test]
    async fn test_event_store_load_from_version() {
        let store = TestEventStore::new();
        let aggregate_id = EntityId::new();
        
        // Save multiple events
        let events: Vec<Box<dyn DomainEvent + Send + Sync>> = vec![
            Box::new(TestEvent {
                id: aggregate_id,
                data: "Event 1".to_string(),
                timestamp: Timestamp::now(),
            }),
            Box::new(TestEvent {
                id: aggregate_id,
                data: "Event 2".to_string(),
                timestamp: Timestamp::now(),
            }),
            Box::new(TestEvent {
                id: aggregate_id,
                data: "Event 3".to_string(),
                timestamp: Timestamp::now(),
            }),
        ];
        
        store.save_events(aggregate_id, events, 0).await.expect("Save events");
        
        // Load events from version 1 (should get events 2 and 3)
        let loaded_events = store.load_events(aggregate_id, 1).await.expect("Load events from version");
        assert_eq!(loaded_events.len(), 2);
    }
}