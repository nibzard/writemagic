//! AI domain - LLM integration, context management, and response processing

pub mod providers;
pub mod entities;
pub mod value_objects;
pub mod services;
pub mod repositories;
pub mod examples;
pub mod writing_service;
pub mod retry_patterns;
pub mod tokenization;
pub mod security;
pub mod circuit_breaker;

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
pub use retry_patterns::{RetryConfig, with_retry, with_timeout};
pub use tokenization::{TokenizationService, ModelTokenizer, TokenUsage, ModelTokenizerConfig};
pub use security::{SecureKeyManager, PIIDetectionService, ContentSanitizationService, SecurityAuditLogger};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerRegistry, CircuitBreakerConfig, CircuitState};