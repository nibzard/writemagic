//! Writing domain tests
//!
//! This module includes comprehensive unit tests for the writing domain

// Include the tests directory as a module
#[path = "../tests/mod.rs"]
mod tests_module;

// Re-export all tests
pub use tests_module::*;