//! Shared types, errors, and utilities across all WriteMagic domains

pub mod error;
pub mod events;
pub mod repository;
pub mod types;
pub mod traits;
pub mod validation;

// Re-export commonly used types
pub use error::{Result, WritemagicError};
pub use events::{DomainEvent, EventBus};
pub use repository::{Repository, RepositoryError};
pub use types::*;
pub use traits::*;