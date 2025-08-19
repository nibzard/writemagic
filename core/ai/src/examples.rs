//! AI integration examples and usage patterns

use crate::providers::*;
use crate::services::*;
use writemagic_shared::Result;
use std::sync::Arc;

/// Example of setting up AI providers with fallback
pub async fn setup_ai_system() -> Result<AIOrchestrationService> {
    // Create provider registry
    let registry = AIProviderRegistry::new()
        .with_claude_key(std::env::var("CLAUDE_API_KEY").unwrap_or_default())
        .with_openai_key(std::env::var("OPENAI_API_KEY").unwrap_or_default());
    
    // Create orchestration service with automatic provider setup
    registry.create_orchestration_service()
}

/// Example of making a simple completion request
pub async fn simple_completion_example(service: &AIOrchestrationService) -> Result<String> {
    let request = CompletionRequest::new(
        vec![
            Message::system("You are a helpful writing assistant."),
            Message::user("Help me write a compelling opening sentence for a blog post about sustainable living."),
        ],
        "claude-3-sonnet-20240229".to_string(),
    )
    .with_max_tokens(200)
    .with_temperature(0.7);

    let response = service.complete_with_fallback(request).await?;
    
    Ok(response.choices.into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default())
}

/// Example of conversation with context management
pub async fn conversation_example(
    service: &AIOrchestrationService, 
    context_manager: &ContextManagementService,
) -> Result<String> {
    // Simulate a conversation history
    let mut messages = vec![
        Message::system("You are an expert writing coach helping with creative writing."),
        Message::user("I'm writing a mystery novel. Can you help me develop a compelling character?"),
        Message::assistant("I'd be happy to help you develop a compelling character for your mystery novel! Let's start with the basics. What role will this character play in your story? Are they the detective, a suspect, a witness, or perhaps the victim?"),
        Message::user("They'll be the detective. I want them to have a unique personality trait that affects how they solve cases."),
        Message::assistant("Excellent! A unique personality trait can really set your detective apart. Here are some intriguing possibilities:\n\n1. **Synesthesia** - They might see sounds as colors or associate numbers with personalities, giving them unusual pattern recognition abilities.\n\n2. **Compulsive honesty** - They're incapable of lying, which creates interesting challenges when dealing with suspects who expect deception.\n\n3. **Eidetic memory for emotions** - They remember exactly how people felt in past situations, helping them predict behavior.\n\nWhich direction interests you, or would you like to explore something different?"),
        Message::user("I love the synesthesia idea! How could that specifically help them solve cases?"),
    ];

    // Apply context management
    let managed_messages = context_manager.manage_context(messages);

    let request = CompletionRequest::new(
        managed_messages,
        "claude-3-sonnet-20240229".to_string(),
    )
    .with_max_tokens(300)
    .with_temperature(0.8);

    let response = service.complete_with_fallback(request).await?;
    
    Ok(response.choices.into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default())
}

/// Example of provider health monitoring
pub async fn health_monitoring_example(service: &AIOrchestrationService) -> Result<()> {
    // Check health of all providers
    let health_results = service.health_check_all_providers().await?;
    
    println!("Provider Health Check Results:");
    for (provider, is_healthy) in &health_results {
        println!("  {}: {}", provider, if *is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
    }

    // Get detailed health statistics
    let provider_health = service.get_provider_health().await;
    
    println!("\nDetailed Health Statistics:");
    for (provider, health) in &provider_health {
        println!("  {}:", provider);
        println!("    Healthy: {}", health.is_healthy);
        println!("    Consecutive failures: {}", health.consecutive_failures);
        println!("    Avg response time: {:?}", health.avg_response_time);
        
        if let Some(last_success) = health.last_success {
            println!("    Last success: {:?} ago", last_success.elapsed());
        }
        
        if let Some(last_failure) = health.last_failure {
            println!("    Last failure: {:?} ago", last_failure.elapsed());
        }
    }

    Ok(())
}

/// Example of content filtering
pub async fn content_filtering_example() -> Result<()> {
    let filter = ContentFilteringService::new()?;
    
    // Test various content samples
    let test_content = vec![
        "Write a story about a magical forest.",
        "My API key is sk-1234567890abcdef",
        "The credit card number is 4111-1111-1111-1111",
        "Help me improve this essay about climate change.",
    ];

    println!("Content Filtering Results:");
    for content in &test_content {
        match filter.filter_content(content) {
            Ok(_) => println!("  ✅ Safe: {}", content),
            Err(e) => println!("  ❌ Filtered: {} - Error: {}", content, e),
        }
        
        let findings = filter.detect_sensitive_info(content);
        if !findings.is_empty() {
            println!("    Sensitive patterns detected: {:?}", findings);
        }
    }

    Ok(())
}

/// Example of usage statistics tracking
pub async fn usage_statistics_example(service: &AIOrchestrationService) -> Result<()> {
    // Make several requests to generate statistics
    for i in 1..=3 {
        let request = CompletionRequest::new(
            vec![Message::user(format!("Generate a short poem about the number {}", i))],
            "claude-3-haiku-20240307".to_string(),
        ).with_max_tokens(100);

        let _response = service.complete_with_fallback(request).await?;
    }

    // Get usage statistics from the orchestration service
    println!("Usage Statistics:");
    let health_stats = service.get_provider_health().await;
    
    for (provider_name, health) in &health_stats {
        println!("  Provider: {}", provider_name);
        println!("    Requests made: (tracked via health monitoring)");
        println!("    Average response time: {:?}", health.avg_response_time);
        println!("    Health status: {}", if health.is_healthy { "Healthy" } else { "Unhealthy" });
    }

    Ok(())
}

/// Example of different model configurations
pub async fn model_configuration_example(service: &AIOrchestrationService) -> Result<()> {
    let base_prompt = vec![
        Message::system("You are a creative writing assistant."),
        Message::user("Write one sentence about a sunset."),
    ];

    // Creative configuration (high temperature)
    let creative_request = CompletionRequest::new(
        base_prompt.clone(),
        "claude-3-sonnet-20240229".to_string(),
    )
    .with_temperature(0.9)
    .with_max_tokens(100);

    let creative_response = service.complete_with_fallback(creative_request).await?;
    println!("Creative (temp=0.9): {}", 
        creative_response.choices.first().unwrap().message.content);

    // Balanced configuration (medium temperature)
    let balanced_request = CompletionRequest::new(
        base_prompt.clone(),
        "claude-3-sonnet-20240229".to_string(),
    )
    .with_temperature(0.7)
    .with_max_tokens(100);

    let balanced_response = service.complete_with_fallback(balanced_request).await?;
    println!("Balanced (temp=0.7): {}", 
        balanced_response.choices.first().unwrap().message.content);

    // Precise configuration (low temperature)
    let precise_request = CompletionRequest::new(
        base_prompt,
        "claude-3-sonnet-20240229".to_string(),
    )
    .with_temperature(0.1)
    .with_max_tokens(100);

    let precise_response = service.complete_with_fallback(precise_request).await?;
    println!("Precise (temp=0.1): {}", 
        precise_response.choices.first().unwrap().message.content);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_provider_registry() {
        let registry = AIProviderRegistry::new()
            .with_claude_key("test-claude-key".to_string())
            .with_openai_key("test-openai-key".to_string());
        
        // This would fail with invalid keys, but tests the structure
        assert!(registry.create_orchestration_service().is_ok());
    }
    
    #[tokio::test]
    async fn test_content_filtering() {
        let filter = ContentFilteringService::new().unwrap();
        
        // Safe content should pass
        assert!(filter.filter_content("Write a story about dragons").is_ok());
        
        // Sensitive content should be detected
        let findings = filter.detect_sensitive_info("API key: sk-12345");
        assert!(!findings.is_empty());
    }
    
    #[tokio::test]
    async fn test_context_management() {
        let context_manager = ContextManagementService::new(1000); // 1000 char limit
        
        let long_messages = vec![
            Message::system("System message"),
            Message::user("This is a very long user message that exceeds the context limit. ".repeat(50)),
            Message::assistant("Short response"),
            Message::user("Recent message"),
        ];
        
        let managed = context_manager.manage_context(long_messages);
        
        // Should keep system message and recent messages within limit
        let total_length: usize = managed.iter().map(|m| m.content.len()).sum();
        assert!(total_length <= 1000);
        
        // Should always keep system messages
        assert!(managed.iter().any(|m| matches!(m.role, MessageRole::System)));
    }
}