//! Test the core AI module compilation without network dependencies

#[cfg(test)]
mod tests {
    use crate::providers::*;
    use std::collections::HashMap;

    #[test]
    fn test_message_role_enum() {
        assert_eq!(MessageRole::System as u8, MessageRole::System as u8);
        assert_ne!(MessageRole::System as u8, MessageRole::User as u8);
    }

    #[test]
    fn test_completion_request_builder() {
        let messages = vec![
            Message::system("Test system message"),
            Message::user("Test user message"),
        ];
        
        let request = CompletionRequest::new(messages, "test-model".to_string())
            .with_max_tokens(100)
            .with_temperature(0.7)
            .with_metadata("key".to_string(), "value".to_string());

        assert_eq!(request.model, "test-model");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.messages.len(), 2);
        assert!(request.metadata.contains_key("key"));
    }

    #[test]
    fn test_usage_struct() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_model_capabilities() {
        let caps = ModelCapabilities {
            max_tokens: 4096,
            supports_streaming: true,
            supports_functions: false,
            supports_vision: true,
            context_window: 128000,
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00003,
        };
        
        assert_eq!(caps.max_tokens, 4096);
        assert!(caps.supports_streaming);
        assert!(!caps.supports_functions);
        assert!(caps.supports_vision);
    }
}