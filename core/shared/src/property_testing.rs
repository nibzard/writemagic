//! Property-based testing utilities and strategies

use proptest::prelude::*;
use crate::types::EntityId;

/// Generate valid entity IDs for testing
pub fn entity_id_strategy() -> impl Strategy<Value = EntityId> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
        .prop_map(|s| s.parse::<uuid::Uuid>().unwrap().into())
}

/// Generate realistic text content for documents
pub fn document_content_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Short content
        "[A-Za-z0-9 ]{1,100}",
        // Medium content with punctuation
        "[A-Za-z0-9.,!? ]{100,1000}",
        // Long content with line breaks
        "[A-Za-z0-9.,!? \n]{1000,10000}",
        // Empty content
        Just(String::new()),
        // Unicode content
        "[\u{00A0}-\u{024F} ]{10,100}",
    ]
}

/// Generate document titles
pub fn document_title_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Normal titles
        "[A-Za-z0-9 ]{1,100}",
        // Titles with special characters
        "[A-Za-z0-9.,!?()\\[\\] ]{1,100}",
        // Unicode titles
        "[\u{00A0}-\u{024F} ]{5,50}",
        // Edge cases
        Just("".to_string()),
        Just("A".repeat(1000)), // Very long title
    ]
}

/// Generate AI completion requests
pub fn completion_request_strategy() -> impl Strategy<Value = crate::ai::CompletionRequest> {
    (
        prop::collection::vec(message_strategy(), 1..10),
        model_name_strategy(),
        prop::option::of(1u32..4000),
        prop::option::of(0.0f32..2.0f32),
    ).prop_map(|(messages, model, max_tokens, temperature)| {
        let mut request = crate::ai::CompletionRequest::new(messages, model);
        request.max_tokens = max_tokens;
        request.temperature = temperature;
        request
    })
}

/// Generate AI messages
pub fn message_strategy() -> impl Strategy<Value = crate::ai::Message> {
    (
        message_role_strategy(),
        document_content_strategy(),
    ).prop_map(|(role, content)| crate::ai::Message { role, content })
}

/// Generate message roles
pub fn message_role_strategy() -> impl Strategy<Value = crate::ai::MessageRole> {
    prop_oneof![
        Just(crate::ai::MessageRole::System),
        Just(crate::ai::MessageRole::User),
        Just(crate::ai::MessageRole::Assistant),
    ]
}

/// Generate model names
pub fn model_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("claude-3-haiku-20240307".to_string()),
        Just("claude-3-sonnet-20240229".to_string()),
        Just("claude-3-opus-20240229".to_string()),
        Just("gpt-4-turbo-preview".to_string()),
        Just("gpt-3.5-turbo".to_string()),
        // Invalid model names for error testing
        Just("".to_string()),
        Just("invalid-model".to_string()),
    ]
}

/// Generate realistic file paths for testing
pub fn file_path_strategy() -> impl Strategy<Value = std::path::PathBuf> {
    prop_oneof![
        // Normal paths
        "[a-zA-Z0-9_-]+(/[a-zA-Z0-9_.-]+){0,5}\\.(md|txt|doc)",
        // Paths with spaces
        "[a-zA-Z0-9 _-]+(/[a-zA-Z0-9 _.-]+){0,3}\\.(md|txt)",
        // Edge cases
        Just(".".to_string()),
        Just("..".to_string()),
        Just("/".to_string()),
        Just("".to_string()),
        // Very long paths
        format!("{}/file.txt", "a".repeat(200)),
    ].prop_map(|s| std::path::PathBuf::from(s))
}

/// Generate buffer pool configurations for testing
pub fn buffer_pool_config_strategy() -> impl Strategy<Value = (usize, usize)> {
    (
        1usize..=65536, // buffer_size: 1 byte to 64KB
        0usize..=100,   // initial_capacity: 0 to 100 buffers
    )
}

/// Generate concurrent operation scenarios
pub fn concurrent_operations_strategy() -> impl Strategy<Value = Vec<Operation>> {
    prop::collection::vec(operation_strategy(), 1..100)
}

/// Different types of operations for concurrent testing
#[derive(Debug, Clone)]
pub enum Operation {
    CreateDocument { title: String, content: String },
    UpdateDocument { id: EntityId, content: String },
    DeleteDocument { id: EntityId },
    CreateProject { name: String },
    AddDocumentToProject { project_id: EntityId, doc_id: EntityId },
    QueryDocuments { limit: Option<u32> },
}

pub fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (document_title_strategy(), document_content_strategy())
            .prop_map(|(title, content)| Operation::CreateDocument { title, content }),
        (entity_id_strategy(), document_content_strategy())
            .prop_map(|(id, content)| Operation::UpdateDocument { id, content }),
        entity_id_strategy()
            .prop_map(|id| Operation::DeleteDocument { id }),
        document_title_strategy()
            .prop_map(|name| Operation::CreateProject { name }),
        (entity_id_strategy(), entity_id_strategy())
            .prop_map(|(project_id, doc_id)| Operation::AddDocumentToProject { project_id, doc_id }),
        prop::option::of(1u32..1000)
            .prop_map(|limit| Operation::QueryDocuments { limit }),
    ]
}

/// Generate error conditions for testing error handling
pub fn error_condition_strategy() -> impl Strategy<Value = ErrorCondition> {
    prop_oneof![
        Just(ErrorCondition::NetworkTimeout),
        Just(ErrorCondition::InvalidInput),
        Just(ErrorCondition::DatabaseError),
        Just(ErrorCondition::AuthenticationError),
        Just(ErrorCondition::RateLimitExceeded),
        Just(ErrorCondition::ServiceUnavailable),
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCondition {
    NetworkTimeout,
    InvalidInput,
    DatabaseError,
    AuthenticationError,
    RateLimitExceeded,
    ServiceUnavailable,
}

/// Property testing utilities for round-trip serialization
pub fn test_round_trip_serialization<T>(item: &T) -> bool
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = match serde_json::to_string(item) {
        Ok(json) => json,
        Err(_) => return false,
    };

    let deserialized: T = match serde_json::from_str(&json) {
        Ok(item) => item,
        Err(_) => return false,
    };

    *item == deserialized
}

/// Property testing utilities for testing invariants
pub struct InvariantTester<T> {
    invariants: Vec<Box<dyn Fn(&T) -> bool>>,
}

impl<T> InvariantTester<T> {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
        }
    }

    pub fn add_invariant<F>(mut self, invariant: F) -> Self 
    where
        F: Fn(&T) -> bool + 'static,
    {
        self.invariants.push(Box::new(invariant));
        self
    }

    pub fn test(&self, item: &T) -> bool {
        self.invariants.iter().all(|invariant| invariant(item))
    }
}

impl<T> Default for InvariantTester<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for creating proptest cases with custom strategies
#[macro_export]
macro_rules! proptest_case {
    ($test_name:ident, $strategy:expr, $test_fn:expr) => {
        #[cfg(test)]
        mod $test_name {
            use super::*;
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn property_test(input in $strategy) {
                    ($test_fn)(input);
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::test_runner::{Config, TestRunner};

    #[test]
    fn test_entity_id_generation() {
        let mut runner = TestRunner::new(Config::default());
        
        runner.run(&entity_id_strategy(), |id| {
            // Entity IDs should always be valid UUIDs
            let uuid_str = id.to_string();
            assert!(uuid::Uuid::parse_str(&uuid_str).is_ok());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_document_content_strategies() {
        let mut runner = TestRunner::new(Config::default());
        
        runner.run(&document_content_strategy(), |content| {
            // Content should never exceed reasonable bounds
            assert!(content.len() <= 100_000);
            Ok(())
        }).unwrap();
    }

    #[test] 
    fn test_completion_request_generation() {
        let mut runner = TestRunner::new(Config::default());
        
        runner.run(&completion_request_strategy(), |request| {
            // All requests should have at least one message
            assert!(!request.messages.is_empty());
            
            // Temperature should be in valid range if set
            if let Some(temp) = request.temperature {
                assert!(temp >= 0.0 && temp <= 2.0);
            }
            
            // Max tokens should be positive if set
            if let Some(max_tokens) = request.max_tokens {
                assert!(max_tokens > 0);
            }
            
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_invariant_tester() {
        #[derive(Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }

        let tester = InvariantTester::new()
            .add_invariant(|s: &TestStruct| s.value >= 0)
            .add_invariant(|s: &TestStruct| s.value <= 100);

        assert!(tester.test(&TestStruct { value: 50 }));
        assert!(!tester.test(&TestStruct { value: -1 }));
        assert!(!tester.test(&TestStruct { value: 101 }));
    }
}