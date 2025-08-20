//! Unit tests for AI providers

use writemagic_ai::{
    AIProvider, CompletionRequest, CompletionResponse, Message, MessageRole,
    ClaudeProvider, OpenAIProvider, ModelCapabilities, ResponseCache,
    ProviderConfiguration, ModelConfiguration
};
use writemagic_shared::{Result, WritemagicError};
use std::collections::HashMap;

#[cfg(test)]
mod completion_request_tests {
    use super::*;

    #[test]
    fn test_completion_request_creation() {
        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello, how are you?"),
        ];
        
        let request = CompletionRequest::new(messages.clone(), "test-model".to_string());
        
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.model, "test-model");
        assert!(request.max_tokens.is_none());
        assert!(request.temperature.is_none());
        assert!(request.metadata.is_empty());
    }

    #[test]
    fn test_completion_request_with_options() {
        let messages = vec![Message::user("Test message")];
        
        let request = CompletionRequest::new(messages, "gpt-4".to_string())
            .with_max_tokens(500)
            .with_temperature(0.8)
            .with_top_p(0.9)
            .with_stop_sequences(vec!["END".to_string(), "STOP".to_string()])
            .with_metadata("session_id".to_string(), "abc123".to_string());
        
        assert_eq!(request.max_tokens, Some(500));
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.top_p, Some(0.9));
        assert_eq!(request.stop_sequences, vec!["END", "STOP"]);
        assert_eq!(request.metadata.get("session_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest::new(
            vec![Message::user("Test")],
            "test-model".to_string(),
        ).with_temperature(0.5);
        
        let serialized = serde_json::to_string(&request).expect("Serialize request");
        let deserialized: CompletionRequest = serde_json::from_str(&serialized).expect("Deserialize request");
        
        assert_eq!(request.messages.len(), deserialized.messages.len());
        assert_eq!(request.model, deserialized.model);
        assert_eq!(request.temperature, deserialized.temperature);
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let system_msg = Message::system("You are an AI assistant.");
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(system_msg.content, "You are an AI assistant.");
        assert!(system_msg.name.is_none());

        let user_msg = Message::user("Hello!");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello!");

        let assistant_msg = Message::assistant("Hi there!");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there!");
    }

    #[test]
    fn test_message_with_name() {
        let message = Message::user("Test message").with_name("user123");
        
        assert_eq!(message.name, Some("user123".to_string()));
        assert_eq!(message.content, "Test message");
    }

    #[test]
    fn test_message_role_serialization() {
        let roles = vec![
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Function,
        ];
        
        for role in roles {
            let serialized = serde_json::to_string(&role).expect("Serialize role");
            let deserialized: MessageRole = serde_json::from_str(&serialized).expect("Deserialize role");
            assert_eq!(role, deserialized);
        }
    }

    #[test]
    fn test_message_serialization() {
        let message = Message::system("System prompt")
            .with_name("system");
        
        let serialized = serde_json::to_string(&message).expect("Serialize message");
        let deserialized: Message = serde_json::from_str(&serialized).expect("Deserialize message");
        
        assert_eq!(message.role, deserialized.role);
        assert_eq!(message.content, deserialized.content);
        assert_eq!(message.name, deserialized.name);
    }
}

#[cfg(test)]
mod completion_response_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_completion_response_creation() {
        let response = CompletionResponse {
            id: "test-id".to_string(),
            content: "Test response content".to_string(),
            model: "test-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        assert_eq!(response.id, "test-id");
        assert_eq!(response.content, "Test response content");
        assert_eq!(response.model, "test-model");
        assert_eq!(response.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_completion_response_with_usage() {
        use writemagic_ai::TokenUsage;
        
        let usage = TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        
        let response = CompletionResponse {
            id: "usage-test".to_string(),
            content: "Response with usage".to_string(),
            model: "test-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: Some(usage.clone()),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };
        
        assert!(response.usage.is_some());
        let response_usage = response.usage.unwrap();
        assert_eq!(response_usage.prompt_tokens, 10);
        assert_eq!(response_usage.completion_tokens, 20);
        assert_eq!(response_usage.total_tokens, 30);
    }

    #[test]
    fn test_completion_response_serialization() {
        let response = CompletionResponse {
            id: "ser-test".to_string(),
            content: "Serialization test".to_string(),
            model: "test-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };
        
        let serialized = serde_json::to_string(&response).expect("Serialize response");
        let deserialized: CompletionResponse = serde_json::from_str(&serialized).expect("Deserialize response");
        
        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.content, deserialized.content);
        assert_eq!(response.model, deserialized.model);
    }
}

#[cfg(test)]
mod claude_provider_tests {
    use super::*;

    #[test]
    fn test_claude_provider_creation() {
        let provider = ClaudeProvider::new("test-api-key".to_string());
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "claude");
    }

    #[test]
    fn test_claude_provider_invalid_key() {
        let provider = ClaudeProvider::new("".to_string());
        assert!(provider.is_err());
        assert!(provider.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_claude_provider_capabilities() {
        let provider = ClaudeProvider::new("test-key".to_string()).expect("Valid provider");
        let capabilities = provider.capabilities();
        
        assert_eq!(capabilities.max_tokens, 100000);
        assert_eq!(capabilities.context_window, 200000);
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_vision);
        assert!(!capabilities.supports_functions);
    }

    #[test]
    fn test_claude_provider_models() {
        let provider = ClaudeProvider::new("test-key".to_string()).expect("Valid provider");
        let models = provider.supported_models();
        
        assert!(!models.is_empty());
        assert!(models.contains(&"claude-3-sonnet-20240229".to_string()));
        assert!(models.contains(&"claude-3-opus-20240229".to_string()));
    }

    #[test]
    fn test_claude_provider_default_model() {
        let provider = ClaudeProvider::new("test-key".to_string()).expect("Valid provider");
        let default_model = provider.default_model();
        
        assert!(!default_model.is_empty());
        assert!(provider.supported_models().contains(&default_model));
    }
}

#[cfg(test)]
mod openai_provider_tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test-api-key".to_string());
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_openai_provider_capabilities() {
        let provider = OpenAIProvider::new("test-key".to_string()).expect("Valid provider");
        let capabilities = provider.capabilities();
        
        assert_eq!(capabilities.max_tokens, 4096);
        assert_eq!(capabilities.context_window, 128000);
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_vision);
        assert!(capabilities.supports_functions);
    }

    #[test]
    fn test_openai_provider_models() {
        let provider = OpenAIProvider::new("test-key".to_string()).expect("Valid provider");
        let models = provider.supported_models();
        
        assert!(!models.is_empty());
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
    }

    #[test]
    fn test_openai_provider_rate_limits() {
        let provider = OpenAIProvider::new("test-key".to_string()).expect("Valid provider");
        let rate_limits = provider.rate_limits();
        
        assert!(rate_limits.requests_per_minute > 0);
        assert!(rate_limits.tokens_per_minute > 0);
    }
}

#[cfg(test)]
mod model_capabilities_tests {
    use super::*;

    #[test]
    fn test_model_capabilities_creation() {
        let capabilities = ModelCapabilities {
            max_tokens: 4096,
            context_window: 8192,
            supports_streaming: true,
            supports_vision: false,
            supports_functions: true,
        };
        
        assert_eq!(capabilities.max_tokens, 4096);
        assert_eq!(capabilities.context_window, 8192);
        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_vision);
        assert!(capabilities.supports_functions);
    }

    #[test]
    fn test_model_capabilities_comparison() {
        let caps1 = ModelCapabilities {
            max_tokens: 4096,
            context_window: 8192,
            supports_streaming: true,
            supports_vision: false,
            supports_functions: true,
        };
        
        let caps2 = ModelCapabilities {
            max_tokens: 4096,
            context_window: 8192,
            supports_streaming: true,
            supports_vision: false,
            supports_functions: true,
        };
        
        assert_eq!(caps1, caps2);
    }

    #[test]
    fn test_model_capabilities_serialization() {
        let capabilities = ModelCapabilities {
            max_tokens: 2048,
            context_window: 4096,
            supports_streaming: false,
            supports_vision: true,
            supports_functions: false,
        };
        
        let serialized = serde_json::to_string(&capabilities).expect("Serialize capabilities");
        let deserialized: ModelCapabilities = serde_json::from_str(&serialized).expect("Deserialize capabilities");
        
        assert_eq!(capabilities, deserialized);
    }
}

#[cfg(test)]
mod response_cache_tests {
    use super::*;

    #[test]
    fn test_response_cache_creation() {
        let cache = ResponseCache::new(100, std::time::Duration::from_secs(300));
        assert_eq!(cache.capacity(), 100);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_response_cache_key_generation() {
        let request1 = CompletionRequest::new(
            vec![Message::user("Hello")],
            "gpt-4".to_string(),
        ).with_temperature(0.7);
        
        let request2 = CompletionRequest::new(
            vec![Message::user("Hello")],
            "gpt-4".to_string(),
        ).with_temperature(0.7);
        
        let key1 = ResponseCache::generate_cache_key(&request1);
        let key2 = ResponseCache::generate_cache_key(&request2);
        
        assert_eq!(key1, key2); // Same request should generate same key
        
        let request3 = CompletionRequest::new(
            vec![Message::user("Hello")],
            "gpt-4".to_string(),
        ).with_temperature(0.8); // Different temperature
        
        let key3 = ResponseCache::generate_cache_key(&request3);
        assert_ne!(key1, key3); // Different request should generate different key
    }

    #[test]
    fn test_response_cache_put_and_get() {
        let mut cache = ResponseCache::new(10, std::time::Duration::from_secs(60));
        
        let request = CompletionRequest::new(
            vec![Message::user("Test")],
            "test-model".to_string(),
        );
        
        let response = CompletionResponse {
            id: "test-id".to_string(),
            content: "Test response".to_string(),
            model: "test-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        // Put response in cache
        cache.put(&request, response.clone());
        
        // Get response from cache
        let cached_response = cache.get(&request);
        assert!(cached_response.is_some());
        
        let cached = cached_response.unwrap();
        assert_eq!(cached.id, response.id);
        assert_eq!(cached.content, response.content);
    }

    #[test]
    fn test_response_cache_capacity_limit() {
        let mut cache = ResponseCache::new(2, std::time::Duration::from_secs(60));
        
        // Add responses up to capacity
        for i in 0..3 {
            let request = CompletionRequest::new(
                vec![Message::user(&format!("Test {}", i))],
                "test-model".to_string(),
            );
            
            let response = CompletionResponse {
                id: format!("test-id-{}", i),
                content: format!("Response {}", i),
                model: "test-model".to_string(),
                finish_reason: Some("stop".to_string()),
                usage: None,
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
            };
            
            cache.put(&request, response);
        }
        
        // Cache should not exceed capacity
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_response_cache_expiration() {
        let mut cache = ResponseCache::new(10, std::time::Duration::from_millis(1));
        
        let request = CompletionRequest::new(
            vec![Message::user("Expire test")],
            "test-model".to_string(),
        );
        
        let response = CompletionResponse {
            id: "expire-test".to_string(),
            content: "This will expire".to_string(),
            model: "test-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };
        
        cache.put(&request, response);
        
        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_millis(5));
        
        // Response should be expired and not found
        let cached_response = cache.get(&request);
        assert!(cached_response.is_none());
    }
}

#[cfg(test)]
mod provider_configuration_tests {
    use super::*;

    #[test]
    fn test_provider_configuration_creation() {
        let config = ProviderConfiguration {
            api_key: "test-key".to_string(),
            base_url: Some("https://api.example.com".to_string()),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(1000),
        };
        
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
        assert_eq!(config.timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_provider_configuration_default() {
        let config = ProviderConfiguration::default();
        
        assert_eq!(config.api_key, "");
        assert!(config.base_url.is_none());
        assert_eq!(config.timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }
}

#[cfg(test)]
mod model_configuration_tests {
    use super::*;

    #[test]
    fn test_model_configuration_creation() -> Result<()> {
        let config = ModelConfiguration::new("gpt-4")?;
        
        assert_eq!(config.model_name, "gpt-4");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
        assert!(config.stop_sequences.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_model_configuration_invalid_model() {
        let result = ModelConfiguration::new("");
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_model_configuration_with_options() -> Result<()> {
        let config = ModelConfiguration::new("claude-3-sonnet")?
            .with_temperature(0.5)
            .with_max_tokens(2048)
            .with_top_p(0.9)
            .with_stop_sequences(vec!["END".to_string()]);
        
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.top_p, Some(0.9));
        assert_eq!(config.stop_sequences, vec!["END"]);
        
        Ok(())
    }

    #[test]
    fn test_model_configuration_validation() {
        // Test invalid temperature
        let result = ModelConfiguration::new("gpt-4")
            .unwrap()
            .with_temperature(1.5); // Invalid: > 1.0
        assert!(ModelConfiguration::validate_temperature(1.5).is_err());
        
        // Test invalid top_p
        assert!(ModelConfiguration::validate_top_p(1.5).is_err()); // Invalid: > 1.0
        assert!(ModelConfiguration::validate_top_p(-0.1).is_err()); // Invalid: < 0.0
        
        // Test valid values
        assert!(ModelConfiguration::validate_temperature(0.5).is_ok());
        assert!(ModelConfiguration::validate_top_p(0.9).is_ok());
    }
}