//! Repository abstraction for data access

use async_trait::async_trait;
use crate::{EntityId, Pagination, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Entity not found with id: {id}")]
    NotFound { id: String },

    #[error("Entity already exists with id: {id}")]
    AlreadyExists { id: String },

    #[error("Optimistic lock error: entity was modified by another operation")]
    OptimisticLock,

    #[error("Database operation failed: {message}")]
    Database { message: String },

    #[error("Connection error: {message}")]
    Connection { message: String },
}

/// Generic repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, ID = EntityId>: Send + Sync {
    /// Find entity by ID
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;

    /// Find all entities with pagination
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<T>>;

    /// Save entity (insert or update)
    async fn save(&self, entity: &T) -> Result<T>;

    /// Delete entity by ID
    async fn delete(&self, id: &ID) -> Result<bool>;

    /// Check if entity exists
    async fn exists(&self, id: &ID) -> Result<bool>;

    /// Count total entities
    async fn count(&self) -> Result<u64>;
}

/// Unit of work pattern for transaction management
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a new transaction
    async fn begin(&mut self) -> Result<()>;

    /// Commit the current transaction
    async fn commit(&mut self) -> Result<()>;

    /// Rollback the current transaction
    async fn rollback(&mut self) -> Result<()>;

    /// Execute work within a transaction
    async fn with_transaction<F, R>(&mut self, work: F) -> Result<R>
    where
        F: FnOnce(&mut Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send>> + Send,
        R: Send;
}

/// Query specification pattern
#[async_trait]
pub trait Specification<T>: Send + Sync {
    async fn is_satisfied_by(&self, entity: &T) -> bool;
    fn to_sql(&self) -> (String, Vec<serde_json::Value>);
}

/// Read-only repository for queries
#[async_trait]
pub trait ReadRepository<T, ID = EntityId>: Send + Sync {
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<T>>;
    async fn count(&self) -> Result<u64>;
    async fn find_by_specification<S>(&self, spec: S, pagination: Pagination) -> Result<Vec<T>>
    where
        S: Specification<T>;
}

/// Write-only repository for commands
#[async_trait]
pub trait WriteRepository<T, ID = EntityId>: Send + Sync {
    async fn save(&self, entity: &T) -> Result<T>;
    async fn delete(&self, id: &ID) -> Result<bool>;
    async fn delete_by_specification<S>(&self, spec: S) -> Result<u64>
    where
        S: Specification<T>;
}