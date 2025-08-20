//! Unit tests for AI services

use writemagic_ai::{
    AIOrchestrationService, AIProviderRegistry, ContextManagementService,
    ContentFilteringService, AIProvider, CompletionRequest, CompletionResponse,
    Message, MessageRole, ClaudeProvider, OpenAIProvider, ProviderHealth,
    CircuitBreaker, CircuitBreakerConfig, TokenizationService
};
use writemagic_shared::{Result, WritemagicError};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Mock AI provider for testing
struct MockAIProvider {
    name: String,
    responses: Arc<Mutex<Vec<Result<CompletionResponse>>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockAIProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            responses: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }
    
    fn add_response(&self, response: Result<CompletionResponse>) {
        self.responses.lock().unwrap().push(response);
    }
    
    fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
    
    fn create_success_response(&self, content: &str) -> CompletionResponse {
        CompletionResponse {
            id: format!("{}-response-{}", self.name, self.get_call_count()),
            content: content.to_string(),
            model: format!("{}-model", self.name),
            finish_reason: Some("stop".to_string()),
            usage: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl AIProvider for MockAIProvider {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn complete(&self, _request: &CompletionRequest) -> Result<CompletionResponse> {
        let mut call_count = self.call_count.lock().unwrap();
        *call_count += 1;
        
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok(self.create_success_response("Mock response"))
        } else {
            responses.remove(0)
        }
    }
    
    fn capabilities(&self) -> writemagic_ai::ModelCapabilities {
        writemagic_ai::ModelCapabilities {
            max_tokens: 4096,
            context_window: 8192,
            supports_streaming: true,
            supports_vision: false,
            supports_functions: false,
        }
    }
    
    fn supported_models(&self) -> Vec<String> {
        vec![format!("{}-model", self.name)]
    }
    
    fn default_model(&self) -> String {
        format!("{}-model", self.name)
    }
    
    fn rate_limits(&self) -> writemagic_ai::RateLimits {
        writemagic_ai::RateLimits {
            requests_per_minute: 60,
            tokens_per_minute: 10000,
        }
    }
}

#[cfg(test)]
mod ai_provider_registry_tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AIProviderRegistry::new();
        assert!(registry.get_provider("claude").is_none());
        assert!(registry.get_provider("openai").is_none());
    }

    #[test]
    fn test_add_claude_key() -> Result<()> {
        let mut registry = AIProviderRegistry::new();
        registry.add_claude_key("test-claude-key".to_string())?;
        
        let provider = registry.create_claude_provider();
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "claude");
        
        Ok(())
    }

    #[test]
    fn test_add_openai_key() -> Result<()> {
        let mut registry = AIProviderRegistry::new();
        registry.add_openai_key("test-openai-key".to_string())?;
        
        let provider = registry.create_openai_provider();
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "openai");
        
        Ok(())
    }

    #[test]
    fn test_register_custom_provider() {
        let mut registry = AIProviderRegistry::new();
        let mock_provider = Arc::new(MockAIProvider::new("custom"));
        
        registry.register_provider("custom", mock_provider);
        
        let provider = registry.get_provider("custom");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "custom");
    }

    #[tokio::test]
    async fn test_create_orchestration_service() -> Result<()> {
        let mut registry = AIProviderRegistry::new();
        registry.add_claude_key("test-key".to_string())?;
        
        let service = registry.create_orchestration_service().await;
        assert!(service.is_ok());
        
        Ok(())
    }
}

#[cfg(test)]
mod context_management_tests {
    use super::*;

    #[test]
    fn test_context_management_creation() -> Result<()> {
        let service = ContextManagementService::new(1000)?;
        assert_eq!(service.max_tokens(), 1000);
        Ok(())
    }

    #[test]
    fn test_context_management_invalid_limit() {
        let result = ContextManagementService::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_manage_context_within_limit() -> Result<()> {
        let service = ContextManagementService::new(100)?;
        
        let messages = vec![
            Message::system("System"), // ~6 chars
            Message::user("Hello"), // ~5 chars
            Message::assistant("Hi"), // ~2 chars
        ];
        
        let managed = service.manage_context(messages.clone(), "test-model")?;
        
        // All messages should fit within limit
        assert_eq!(managed.len(), messages.len());
        
        Ok(())
    }

    #[test]
    fn test_manage_context_over_limit() -> Result<()> {
        let service = ContextManagementService::new(20)?; // Very small limit
        
        let messages = vec![
            Message::system("System message"), // ~14 chars
            Message::user("This is a very long user message that exceeds the limit"), // ~58 chars
            Message::assistant("Short reply"), // ~11 chars
            Message::user("Recent"), // ~6 chars
        ];
        
        let managed = service.manage_context(messages, "test-model")?;
        
        // Should keep system message and recent messages within limit
        assert!(!managed.is_empty());
        assert!(managed.iter().any(|m| matches!(m.role, MessageRole::System)));
        
        // Calculate total length
        let total_length: usize = managed.iter().map(|m| m.content.len()).sum();
        assert!(total_length <= 20);
        
        Ok(())
    }

    #[test]
    fn test_preserve_system_messages() -> Result<()> {
        let service = ContextManagementService::new(30)?;
        
        let messages = vec![
            Message::system("Important system message"),
            Message::user("Very long user message that should be truncated to fit within the limit"),
            Message::assistant("Reply"),
        ];
        
        let managed = service.manage_context(messages, "test-model")?;
        
        // System message should always be preserved
        assert!(managed.iter().any(|m| {
            matches!(m.role, MessageRole::System) && m.content == "Important system message"
        }));
        
        Ok(())
    }
}

#[cfg(test)]
mod content_filtering_tests {
    use super::*;

    #[test]
    fn test_content_filtering_creation() -> Result<()> {
        let service = ContentFilteringService::new();
        assert!(service.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_filter_safe_content() -> Result<()> {
        let service = ContentFilteringService::new()?;
        
        let safe_content = "This is a normal conversation about programming.";
        let result = service.filter_content(safe_content).await?;
        
        assert!(!result.is_blocked);
        assert!(result.filtered_content == safe_content);
        assert!(result.violations.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_detect_pii() -> Result<()> {
        let service = ContentFilteringService::new()?;
        
        let content_with_email = "My email is john.doe@example.com and my phone is 555-123-4567";
        let result = service.detect_pii(content_with_email).await?;
        
        assert!(!result.pii_entities.is_empty());
        
        // Should detect email and phone number
        let entity_types: Vec<&str> = result.pii_entities.iter().map(|e| e.entity_type.as_str()).collect();
        assert!(entity_types.contains(&"email"));
        assert!(entity_types.contains(&"phone"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_sanitize_content() -> Result<()> {
        let service = ContentFilteringService::new()?;
        
        let content = "Contact me at john@example.com or call 555-0123";
        let sanitized = service.sanitize_content(content).await?;
        
        // PII should be redacted
        assert!(!sanitized.contains("john@example.com"));
        assert!(!sanitized.contains("555-0123"));
        assert!(sanitized.contains("[REDACTED]") || sanitized.contains("***"));
        
        Ok(())
    }
}

#[cfg(test)]
mod ai_orchestration_service_tests {
    use super::*;

    fn create_test_orchestration_service() -> AIOrchestrationService {
        let mut providers: HashMap<String, Arc<dyn AIProvider>> = HashMap::new();
        
        let claude_mock = Arc::new(MockAIProvider::new("claude"));
        let openai_mock = Arc::new(MockAIProvider::new("openai"));
        
        providers.insert("claude".to_string(), claude_mock);
        providers.insert("openai".to_string(), openai_mock);
        
        AIOrchestrationService::new(providers)
    }

    #[tokio::test]
    async fn test_orchestration_service_creation() {
        let service = create_test_orchestration_service();
        let provider_names = service.available_providers();
        
        assert_eq!(provider_names.len(), 2);
        assert!(provider_names.contains(&"claude".to_string()));
        assert!(provider_names.contains(&"openai".to_string()));
    }

    #[tokio::test]
    async fn test_complete_with_preferred_provider() -> Result<()> {
        let service = create_test_orchestration_service();
        
        let request = CompletionRequest::new(
            vec![Message::user("Hello, world!")],
            "claude-model".to_string(),
        );
        
        let response = service.complete(&request, Some("claude".to_string())).await?;
        
        assert_eq!(response.model, "claude-model");
        assert!(!response.content.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_complete_with_fallback() -> Result<()> {
        let service = create_test_orchestration_service();
        
        // Get the claude provider and make it fail
        if let Some(claude_provider) = service.get_provider("claude") {
            let claude_mock = claude_provider.as_any().downcast_ref::<MockAIProvider>().unwrap();
            claude_mock.add_response(Err(WritemagicError::ai_provider("Provider unavailable")));
        }
        
        let request = CompletionRequest::new(
            vec![Message::user("Test with fallback")],
            "test-model".to_string(),
        );
        
        let response = service.complete(&request, Some("claude".to_string())).await?;
        
        // Should fall back to another provider
        assert!(!response.content.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_complete_no_preferred_provider() -> Result<()> {
        let service = create_test_orchestration_service();
        
        let request = CompletionRequest::new(
            vec![Message::user("No preference")],
            "any-model".to_string(),
        );
        
        let response = service.complete(&request, None).await?;
        
        assert!(!response.content.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_provider_health() {
        let service = create_test_orchestration_service();
        
        let health = service.get_provider_health("claude").await;
        assert!(health.is_some());
        
        let claude_health = health.unwrap();
        assert!(claude_health.is_healthy);
        assert_eq!(claude_health.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_provider_health_after_failure() -> Result<()> {
        let service = create_test_orchestration_service();
        
        // Get the claude provider and make it fail
        if let Some(claude_provider) = service.get_provider("claude") {
            let claude_mock = claude_provider.as_any().downcast_ref::<MockAIProvider>().unwrap();
            claude_mock.add_response(Err(WritemagicError::ai_provider("Simulated failure")));
        }
        
        let request = CompletionRequest::new(
            vec![Message::user("This should fail")],
            "claude-model".to_string(),
        );
        
        // This should trigger the failure and health update
        let _ = service.complete(&request, Some("claude".to_string())).await;
        
        let health = service.get_provider_health("claude").await;
        assert!(health.is_some());
        
        // Health status might be updated after failure (implementation dependent)
        
        Ok(())
    }
}

#[cfg(test)]
mod provider_health_tests {
    use super::*;

    #[test]
    fn test_provider_health_creation() {
        let health = ProviderHealth::new();
        
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_success.is_none());
        assert!(health.last_failure.is_none());
    }

    #[test]
    fn test_record_success() {
        let mut health = ProviderHealth::new();
        let response_time = Duration::from_millis(500);
        
        health.record_success(response_time);
        
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_success.is_some());
        
        // Average response time should be updated
        assert!(health.avg_response_time.as_millis() > 0);
    }

    #[test]
    fn test_record_failure() {
        let mut health = ProviderHealth::new();
        
        // Record multiple failures
        health.record_failure();
        assert_eq!(health.consecutive_failures, 1);
        assert!(health.is_healthy); // Still healthy after 1 failure
        
        health.record_failure();
        assert_eq!(health.consecutive_failures, 2);
        assert!(health.is_healthy); // Still healthy after 2 failures
        
        health.record_failure();
        assert_eq!(health.consecutive_failures, 3);
        assert!(!health.is_healthy); // Unhealthy after 3 failures
    }

    #[test]
    fn test_health_recovery_after_success() {
        let mut health = ProviderHealth::new();
        
        // Make it unhealthy
        health.record_failure();
        health.record_failure();
        health.record_failure();
        assert!(!health.is_healthy);
        
        // Record success should restore health
        health.record_success(Duration::from_millis(200));
        
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_average_response_time_calculation() {
        let mut health = ProviderHealth::new();
        
        // Record multiple response times
        health.record_success(Duration::from_millis(100));
        health.record_success(Duration::from_millis(200));
        health.record_success(Duration::from_millis(300));
        
        // Average should be calculated (with exponential smoothing)
        let avg_ms = health.avg_response_time.as_millis();
        assert!(avg_ms > 0);
        assert!(avg_ms <= 300); // Should be influenced by recent values
    }
}

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        };
        
        let circuit_breaker = CircuitBreaker::new("test-service", config);
        
        assert_eq!(circuit_breaker.name(), "test-service");
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_success() -> Result<()> {
        let config = CircuitBreakerConfig::default();
        let circuit_breaker = CircuitBreaker::new("test", config);
        
        let result = circuit_breaker.call(|| async { Ok::<String, WritemagicError>("Success".to_string()) }).await?;
        
        assert_eq!(result, "Success");
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::Closed);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() -> Result<()> {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            half_open_max_calls: 1,
        };
        
        let circuit_breaker = CircuitBreaker::new("failure-test", config);
        
        // Make calls that fail up to threshold
        for _ in 0..2 {
            let _ = circuit_breaker.call(|| async { 
                Err::<String, WritemagicError>(WritemagicError::internal("Test failure")) 
            }).await;
        }
        
        // Circuit should be open after threshold failures
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::Open);
        
        // Next call should fail fast
        let result = circuit_breaker.call(|| async { Ok::<String, WritemagicError>("Should not run".to_string()) }).await;
        assert!(result.is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() -> Result<()> {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(10),
            half_open_max_calls: 1,
        };
        
        let circuit_breaker = CircuitBreaker::new("recovery-test", config);
        
        // Trigger circuit to open
        let _ = circuit_breaker.call(|| async { 
            Err::<String, WritemagicError>(WritemagicError::internal("Failure")) 
        }).await;
        
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::Open);
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Circuit should be half-open
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::HalfOpen);
        
        // Successful call should close the circuit
        let result = circuit_breaker.call(|| async { Ok::<String, WritemagicError>("Recovery".to_string()) }).await?;
        
        assert_eq!(result, "Recovery");
        assert_eq!(circuit_breaker.state(), writemagic_ai::CircuitState::Closed);
        
        Ok(())
    }
}

#[cfg(test)]
mod tokenization_service_tests {
    use super::*;

    #[test]
    fn test_tokenization_service_creation() -> Result<()> {
        let service = TokenizationService::new();
        assert!(service.is_ok());
        Ok(())
    }

    #[test]
    fn test_estimate_tokens() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let text = "This is a test sentence with several words.";
        let token_count = service.estimate_tokens(text, "gpt-4")?;
        
        assert!(token_count > 0);
        assert!(token_count < text.len()); // Should be fewer tokens than characters
        
        Ok(())
    }

    #[test]
    fn test_count_tokens_messages() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello, how are you?"),
            Message::assistant("I'm doing well, thank you!"),
        ];
        
        let total_tokens = service.count_tokens(&messages, "gpt-4")?;
        
        assert!(total_tokens > 0);
        
        Ok(())
    }

    #[test]
    fn test_truncate_to_limit() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let long_text = "This is a very long text that should be truncated to fit within the specified token limit for testing purposes.";
        let truncated = service.truncate_to_limit(long_text, 10, "gpt-4")?;
        
        assert!(truncated.len() < long_text.len());
        
        // Verify truncated text is within token limit
        let token_count = service.estimate_tokens(&truncated, "gpt-4")?;
        assert!(token_count <= 10);
        
        Ok(())
    }

    #[test]
    fn test_empty_text_tokenization() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let empty_tokens = service.estimate_tokens("", "gpt-4")?;
        assert_eq!(empty_tokens, 0);
        
        let empty_messages_tokens = service.count_tokens(&[], "gpt-4")?;
        assert_eq!(empty_messages_tokens, 0);
        
        Ok(())
    }
}