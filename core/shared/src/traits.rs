//! Shared traits and interfaces

use async_trait::async_trait;
use crate::{EntityId, Result, Timestamp};

/// Aggregate root marker trait
pub trait AggregateRoot {
    type Id;
    
    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> &Timestamp;
    fn updated_at(&self) -> &Timestamp;
}

/// Entity trait for domain entities
pub trait Entity {
    type Id;
    
    fn id(&self) -> &Self::Id;
}

/// Value object marker trait
pub trait ValueObject: Clone + PartialEq + Eq {}

/// Domain service trait
#[async_trait]
pub trait DomainService: Send + Sync {
    type Input;
    type Output;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Encryption service trait
#[async_trait]
pub trait EncryptionService: Send + Sync {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;
}

/// Configuration provider trait
pub trait ConfigurationProvider: Send + Sync {
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_i64(&self, key: &str) -> Option<i64>;
    fn get_f64(&self, key: &str) -> Option<f64>;
}

/// Cache trait for general caching operations
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Result<Option<V>>;
    async fn set(&self, key: K, value: V) -> Result<()>;
    async fn remove(&self, key: &K) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

/// Audit trail trait for tracking changes
pub trait Auditable {
    fn created_by(&self) -> Option<&EntityId>;
    fn updated_by(&self) -> Option<&EntityId>;
    fn created_at(&self) -> &Timestamp;
    fn updated_at(&self) -> &Timestamp;
}

/// Soft delete trait
pub trait SoftDeletable {
    fn is_deleted(&self) -> bool;
    fn deleted_at(&self) -> Option<&Timestamp>;
    fn deleted_by(&self) -> Option<&EntityId>;
    
    fn mark_deleted(&mut self, deleted_by: EntityId);
    fn restore(&mut self);
}

/// Versioned entity trait
pub trait Versioned {
    fn version(&self) -> u64;
    fn increment_version(&mut self);
}