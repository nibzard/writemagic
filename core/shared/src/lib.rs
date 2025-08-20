//! Shared types, errors, and utilities across all WriteMagic domains

#![feature(error_generic_member_access)]

#[cfg(not(target_arch = "wasm32"))]
pub mod database;
pub mod error;
pub mod events;
pub mod repository;
pub mod repositories;
pub mod services;
pub mod types;
pub mod traits;
pub mod validation;
pub mod buffer_pool;
#[cfg(not(target_arch = "wasm32"))]
pub mod shutdown;
pub mod service_container;
pub mod ffi_safety;
pub mod simd_optimizations;
pub mod allocators;
#[cfg(not(target_arch = "wasm32"))]
pub mod advanced_performance;
#[cfg(not(target_arch = "wasm32"))]
pub mod observability;

#[cfg(test)]
pub mod property_testing;

#[cfg(test)]
pub mod error_handling_tests;

#[cfg(test)]
mod tests;

// Re-export commonly used types
#[cfg(not(target_arch = "wasm32"))]
pub use database::{DatabaseManager, DatabaseConfig, MigrationStatus};
pub use error::{Result, WritemagicError, ErrorResponse, ErrorCode};
pub use events::{DomainEvent, EventBus, EventHandler, EventStore, InMemoryEventBus, CrossDomainEvent, EventPublisher, EventBusPublisher};
pub use repository::{Repository, RepositoryError};
pub use repositories::InMemoryRepository;
pub use services::{
    CrossDomainServiceRegistry, CrossDomainCoordinator, 
    WritingDomainService, AIDomainService, ProjectDomainService, 
    VersionControlDomainService, AgentDomainService
};
pub use types::*;
pub use traits::*;
pub use buffer_pool::{BufferPool, PooledBuffer, WorkingMemory, with_working_memory};
#[cfg(not(target_arch = "wasm32"))]
pub use shutdown::{ShutdownCoordinator, ShutdownSubscriber, GracefulShutdown};
pub use service_container::{ServiceContainer, ServiceRef, ProviderRegistry, StaticServiceRegistry};
pub use ffi_safety::{FFIResult, FFIError, SafeCString, SafeStringReader, FFIHandle};
pub use simd_optimizations::{text_processing, numerical};
pub use allocators::{ArenaAllocator, StackAllocator, PoolAllocator, alloc_in_thread_arena, reset_thread_arena};

#[cfg(not(target_arch = "wasm32"))]
pub use advanced_performance::{MappedFile, MappedFileMut, fast_serialization, batch_processing, lock_free};

#[cfg(not(target_arch = "wasm32"))]
pub use observability::{MetricsCollector, PerformanceProfiler, HealthChecker, tracing_setup};

// WASM-specific exports
#[cfg(target_arch = "wasm32")]
pub mod wasm_utils;

#[cfg(target_arch = "wasm32")]
pub use wasm_utils::*;