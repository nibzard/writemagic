//! Shared types, errors, and utilities across all WriteMagic domains

pub mod database;
pub mod error;
pub mod events;
pub mod repository;
pub mod repositories;
pub mod types;
pub mod traits;
pub mod validation;
pub mod buffer_pool;
pub mod shutdown;

// Re-export commonly used types
pub use database::{DatabaseManager, DatabaseConfig, MigrationStatus};
pub use error::{Result, WritemagicError, ErrorResponse, ErrorCode};
pub use events::{DomainEvent, EventBus};
pub use repository::{Repository, RepositoryError};
pub use repositories::InMemoryRepository;
pub use types::*;
pub use traits::*;
pub use buffer_pool::{BufferPool, PooledBuffer, WorkingMemory, with_working_memory};
pub use shutdown::{ShutdownCoordinator, ShutdownSubscriber, GracefulShutdown};