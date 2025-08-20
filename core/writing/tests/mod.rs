//! Writing domain tests module
//!
//! This module contains comprehensive unit tests for the writing domain,
//! including entities, value objects, aggregates, services, and repositories.

mod entities_tests;
mod value_objects_tests;
mod aggregate_tests; // Existing test file
mod services_tests;
mod repositories_tests;

// Re-export test modules for external access if needed
pub use entities_tests::*;
pub use value_objects_tests::*;
pub use services_tests::*;
pub use repositories_tests::*;