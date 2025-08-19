//! Writing domain - Document management, content editing, and project organization

pub mod core_engine;
pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;
pub mod sqlite_repositories;
pub mod events;
pub mod ai_writing_integration;

// Re-export public types
pub use core_engine::*;
pub use entities::*;
pub use value_objects::*;
pub use aggregates::*;
pub use services::*;
pub use repositories::*;
pub use sqlite_repositories::*;
pub use events::*;
pub use ai_writing_integration::*;

// Re-export AI types for convenience
pub use writemagic_ai::{
    AIProvider, AIOrchestrationService, AIProviderRegistry,
    CompletionRequest, CompletionResponse, Message, MessageRole,
    ContextManagementService, ContentFilteringService
};