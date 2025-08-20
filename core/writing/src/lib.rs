//! Writing domain - Document management, content editing, and project organization

pub mod core_engine;
pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;
#[cfg(feature = "database")]
pub mod sqlite_repositories;
pub mod events;
pub mod conversions;
#[cfg(feature = "ai")]
pub mod ai_writing_integration;

// Web persistence layer for IndexedDB
#[cfg(target_arch = "wasm32")]
pub mod web_persistence;

// Re-export public types
pub use core_engine::*;
pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use repositories::*;
#[cfg(feature = "database")]
pub use sqlite_repositories::*;
pub use events::*;
pub use conversions::*;
#[cfg(feature = "ai")]
pub use ai_writing_integration::*;

// Re-export web persistence types for WASM builds
#[cfg(target_arch = "wasm32")]
pub use web_persistence::*;

// Re-export AI types for convenience
#[cfg(feature = "ai")]
pub use writemagic_ai::{
    AIProvider, AIOrchestrationService, AIProviderRegistry,
    CompletionRequest, CompletionResponse, Message, MessageRole,
    ContextManagementService, ContentFilteringService
};

#[cfg(test)]
mod tests;