//! Basic functionality tests that don't require network calls

use crate::providers::{CompletionRequest, Message, MessageRole, ClaudeProvider, OpenAIProvider, ResponseCache};
use crate::services::{ProviderHealth, ContextManagementService, ContentFilteringService, AIProviderRegistry};
use crate::value_objects::{Prompt, ModelConfiguration};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_creation() {
        let messages = vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello world"),
        ];
        
        let request = CompletionRequest::new(messages.clone(), "test-model".to_string())
            .with_max_tokens(100)
            .with_temperature(0.7)
            .with_metadata("test_key".to_string(), "test_value".to_string());

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.model, "test-model");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.metadata.contains_key("test_key"));
    }

    #[test]
    fn test_message_creation() {
        let system_msg = Message::system("System prompt")
            .with_name("system");
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(system_msg.content, "System prompt");
        assert_eq!(system_msg.name, Some("system".to_string()));

        let user_msg = Message::user("User input");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "User input");

        let assistant_msg = Message::assistant("Assistant response");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Assistant response");
    }

    #[test]
    fn test_model_capabilities() {
        let claude_provider = ClaudeProvider::new("test-key".to_string()).expect("Failed to create Claude provider");
        let capabilities = claude_provider.capabilities();
        
        assert_eq!(capabilities.max_tokens, 100000);
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_vision);
        assert!(!capabilities.supports_functions);
        assert_eq!(capabilities.context_window, 200000);
    }

    #[test]
    fn test_openai_capabilities() {
        let openai_provider = OpenAIProvider::new("test-key".to_string()).expect("Failed to create OpenAI provider");
        let capabilities = openai_provider.capabilities();
        
        assert_eq!(capabilities.max_tokens, 4096);
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_vision);
        assert!(capabilities.supports_functions);
        assert_eq!(capabilities.context_window, 128000);
    }

    #[test]
    fn test_provider_health() {
        let mut health = ProviderHealth::new();
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
        
        // Test failure recording
        health.record_failure();
        assert_eq!(health.consecutive_failures, 1);
        assert!(health.is_healthy); // Still healthy after 1 failure
        
        health.record_failure();
        health.record_failure();
        assert_eq!(health.consecutive_failures, 3);
        assert!(!health.is_healthy); // Unhealthy after 3 failures
        
        // Test success recording
        health.record_success(std::time::Duration::from_millis(500));
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_response_cache_key_generation() {
        let request = CompletionRequest::new(
            vec![Message::user("Test message")],
            "test-model".to_string(),
        ).with_temperature(0.7);
        
        let key1 = ResponseCache::generate_cache_key(&request);
        let key2 = ResponseCache::generate_cache_key(&request);
        
        // Same request should generate same key
        assert_eq!(key1, key2);
        
        let different_request = CompletionRequest::new(
            vec![Message::user("Different message")],
            "test-model".to_string(),
        ).with_temperature(0.7);
        
        let key3 = ResponseCache::generate_cache_key(&different_request);
        
        // Different request should generate different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_context_management() {
        let context_manager = ContextManagementService::new(50).expect("Failed to create context manager"); // Very short limit for testing
        
        let messages = vec![
            Message::system("System"), // 6 chars
            Message::user("This is a very long message that should be truncated"), // 54 chars
            Message::assistant("Short"), // 5 chars
            Message::user("Recent"), // 6 chars
        ];
        
        let managed = context_manager.manage_context(messages, "test-model").expect("Failed to manage context");
        
        // Should keep system message and recent messages within limit
        let total_length: usize = managed.iter().map(|m| m.content.len()).sum();
        assert!(total_length <= 50);
        
        // Should always keep system messages
        assert!(managed.iter().any(|m| matches!(m.role, MessageRole::System)));
        
        // Should prefer recent messages
        assert!(managed.iter().any(|m| m.content == "Recent"));
    }

    #[test]
    fn test_content_filtering_creation() {
        let filter = ContentFilteringService::new();
        assert!(filter.is_ok());
    }

    #[tokio::test]
    async fn test_provider_registry() {
        let mut registry = AIProviderRegistry::new();
        registry.add_claude_key("claude-key".to_string()).expect("Failed to add Claude key");
        registry.add_openai_key("openai-key".to_string()).expect("Failed to add OpenAI key");
        
        // Test individual provider creation
        let claude_result = registry.create_claude_provider();
        assert!(claude_result.is_ok());
        
        let openai_result = registry.create_openai_provider();
        assert!(openai_result.is_ok());
        
        // Test orchestration service creation
        let service_result = registry.create_orchestration_service().await;
        assert!(service_result.is_ok());
    }

    #[test]
    fn test_prompt_value_object() {
        let prompt = Prompt::new("Hello {name}, welcome to {app}!");
        assert!(prompt.is_ok());
        
        let prompt = prompt.unwrap()
            .with_variable("name", "Alice")
            .with_variable("app", "WriteMagic");
        
        let rendered = prompt.render();
        assert_eq!(rendered, "Hello Alice, welcome to WriteMagic!");
    }

    #[test]
    fn test_model_configuration() {
        let config = ModelConfiguration::new("gpt-4");
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.model_name, "gpt-4");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
    }
}