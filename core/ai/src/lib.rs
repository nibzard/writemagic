//! AI domain - LLM integration, context management, and response processing

pub mod providers;
pub mod entities;
pub mod value_objects;
pub mod services;
pub mod repositories;
pub mod examples;
pub mod writing_service;
pub mod writing_service_examples;

#[cfg(test)]
mod test_basic;
#[cfg(test)]
mod lib_test;

// Re-export public types
pub use providers::*;
pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use repositories::*;
pub use writing_service::*;