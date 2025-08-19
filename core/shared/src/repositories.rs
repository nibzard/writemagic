//! Concrete repository implementations

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::{EntityId, Pagination, Repository, Result, WritemagicError};

/// In-memory repository implementation for testing and development
#[derive(Debug, Default)]
pub struct InMemoryRepository<T> {
    entities: Arc<RwLock<HashMap<EntityId, T>>>,
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> Clone for InMemoryRepository<T> {
    fn clone(&self) -> Self {
        Self {
            entities: Arc::clone(&self.entities),
        }
    }
}

#[async_trait]
impl<T> Repository<T, EntityId> for InMemoryRepository<T>
where
    T: Clone + Send + Sync + 'static,
    T: crate::Entity<Id = EntityId>,
{
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<T>> {
        let entities = self.entities.read().map_err(|_| {
            WritemagicError::internal("Failed to acquire read lock")
        })?;
        Ok(entities.get(id).cloned())
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<T>> {
        let entities = self.entities.read().map_err(|_| {
            WritemagicError::internal("Failed to acquire read lock")
        })?;
        
        let mut items: Vec<T> = entities.values().cloned().collect();
        
        // Simple pagination by skipping and taking
        let start = pagination.offset as usize;
        let end = start + pagination.limit as usize;
        
        if start >= items.len() {
            return Ok(Vec::new());
        }
        
        items.truncate(end.min(items.len()));
        if start > 0 {
            items.drain(0..start);
        }
        
        Ok(items)
    }

    async fn save(&self, entity: &T) -> Result<T> {
        let mut entities = self.entities.write().map_err(|_| {
            WritemagicError::internal("Failed to acquire write lock")
        })?;
        
        let id = *entity.id();
        entities.insert(id, entity.clone());
        Ok(entity.clone())
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let mut entities = self.entities.write().map_err(|_| {
            WritemagicError::internal("Failed to acquire write lock")
        })?;
        
        Ok(entities.remove(id).is_some())
    }

    async fn exists(&self, id: &EntityId) -> Result<bool> {
        let entities = self.entities.read().map_err(|_| {
            WritemagicError::internal("Failed to acquire read lock")
        })?;
        Ok(entities.contains_key(id))
    }

    async fn count(&self) -> Result<u64> {
        let entities = self.entities.read().map_err(|_| {
            WritemagicError::internal("Failed to acquire read lock")
        })?;
        Ok(entities.len() as u64)
    }
}
