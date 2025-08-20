//! AI domain tests module
//!
//! This module contains comprehensive unit tests for the AI domain,
//! including providers, services, security, and integration functionality.

mod providers_tests;
mod services_tests;
mod atomic_stats_tests; // Existing test file
mod performance_tests;
mod benchmarks;

// Re-export test modules for external access if needed
pub use providers_tests::*;
pub use services_tests::*;