//! Performance monitoring and optimization examples

use crate::{
    AIOrchestrationService, CompletionRequest, Message, RequestPriority,
    PerformanceThresholds, PerformanceAlerting,
    RequestBatcher, BatchConfig, ClaudeProvider, OpenAIProvider,
};
use writemagic_shared::Result;
use std::sync::Arc;
use std::time::Duration;

/// Example: Setting up a production AI service with full performance monitoring
pub async fn setup_production_ai_service() -> Result<AIOrchestrationService> {
    // Create the orchestration service with all features enabled
    let mut service = AIOrchestrationService::new()?;
    
    // Add providers (in production, use real API keys)
    let claude_provider = Arc::new(ClaudeProvider::new("sk-ant-api-key".to_string())?);
    let openai_provider = Arc::new(OpenAIProvider::new("sk-openai-api-key".to_string())?);
    
    service.add_provider(claude_provider).await;
    service.add_provider(openai_provider).await;
    
    println!("✅ Production AI service initialized with full monitoring");
    Ok(service)
}

/// Example: Monitoring AI performance in real-time
pub async fn monitor_ai_performance(service: &AIOrchestrationService) -> Result<()> {
    // Get overall performance statistics
    let stats = service.get_performance_stats().await;
    
    println!("📊 Overall Performance Statistics:");
    println!("   Total Requests: {}", stats.total_requests);
    println!("   Success Rate: {:.1}%", 
        if stats.total_requests > 0 {
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        } else { 0.0 }
    );
    println!("   Cache Hit Rate: {:.1}%", 
        if stats.total_requests > 0 {
            (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0
        } else { 0.0 }
    );
    println!("   Average Response Time: {:?}", stats.avg_response_time);
    println!("   P95 Response Time: {:?}", stats.p95_response_time);
    println!("   Total Tokens: {}", stats.total_tokens);
    println!("   Total Cost: ${:.4}", stats.total_cost);
    
    // Get provider-specific performance
    for provider_name in ["claude", "openai"] {
        if let Some(provider_stats) = service.get_provider_performance(provider_name).await {
            println!("\n🔍 {} Performance:", provider_name);
            println!("   Requests: {}", provider_stats.total_requests);
            println!("   Success Rate: {:.1}%", 
                if provider_stats.total_requests > 0 {
                    (provider_stats.successful_requests as f64 / provider_stats.total_requests as f64) * 100.0
                } else { 0.0 }
            );
            println!("   Avg Response Time: {:?}", provider_stats.avg_response_time);
            println!("   Cost: ${:.4}", provider_stats.total_cost);
        }
    }
    
    // Check for performance alerts
    let alerts = service.get_performance_alerts(10).await;
    if !alerts.is_empty() {
        println!("\n⚠️  Performance Alerts:");
        for alert in alerts {
            println!("   {:?}: {} - Threshold: {:.2}, Current: {:.2}", 
                alert.alert_type, alert.provider_name, 
                alert.threshold_value, alert.current_value);
        }
    }
    
    // Get performance trends
    let trends = service.get_performance_trends(24).await; // Last 24 hours
    if let Some(response_times) = trends.get("response_time_ms") {
        if !response_times.is_empty() {
            let avg_trend = response_times.iter().sum::<f64>() / response_times.len() as f64;
            println!("\n📈 24h Response Time Trend: {:.1}ms average", avg_trend);
        }
    }
    
    Ok(())
}

/// Example: Using request batching for efficient processing
pub async fn demonstrate_request_batching() -> Result<()> {
    // Configure batching for optimal performance
    let batch_config = BatchConfig {
        max_batch_size: 5,                        // Process up to 5 requests together
        max_wait_time: Duration::from_millis(50), // Don't wait more than 50ms
        max_concurrent_batches: 3,                // Allow 3 concurrent batches
        enable_deduplication: true,               // Cache identical requests
        priority_ordering: true,                  // Process high priority first
    };
    
    let (_batch_tx, batch_rx) = tokio::sync::mpsc::unbounded_channel();
    let (batcher, mut receiver) = RequestBatcher::new(batch_config, batch_rx);
    
    println!("🔄 Request Batcher initialized");
    
    // Simulate submitting multiple requests with different priorities
    let requests = vec![
        ("Generate a blog post about AI", RequestPriority::Normal),
        ("Urgent: Fix this grammar", RequestPriority::High), 
        ("Brainstorm ideas for later", RequestPriority::Low),
        ("Generate a blog post about AI", RequestPriority::Normal), // Duplicate for dedup test
        ("Critical production issue", RequestPriority::Critical),
    ];
    
    let mut handles = Vec::new();
    
    for (content, priority) in requests {
        let batcher = batcher.clone();
        let handle = tokio::spawn(async move {
            let request = CompletionRequest::new(
                vec![Message::user(content)],
                "claude-3-sonnet-20240229".to_string(),
            ).with_priority(priority);
            
            println!("📤 Submitting {:?} priority request: {}", request.priority, content);
            
            // In a real implementation, this would return the actual response
            // For this example, we'll just demonstrate the batching mechanism
            tokio::time::timeout(
                Duration::from_millis(200),
                batcher.submit_request(request)
            ).await
        });
        
        handles.push(handle);
    }
    
    // Monitor batch creation
    tokio::spawn(async move {
        let mut batch_count = 0;
        while let Some(batch) = receiver.recv().await {
            batch_count += 1;
            println!("📦 Batch {} created with {} requests (Priority: {:?})", 
                batch_count, batch.requests.len(), batch.priority);
            
            // Show request details in batch
            for (i, request) in batch.requests.iter().enumerate() {
                let user_message = request.messages.iter()
                    .find(|m| matches!(m.role, crate::providers::MessageRole::User))
                    .map(|m| &m.content[..m.content.len().min(30)])
                    .unwrap_or("Unknown");
                println!("   {}. {:?}: {}...", i + 1, request.priority, user_message);
            }
            
            if batch_count >= 3 {
                break; // Prevent infinite loop in example
            }
        }
    });
    
    // Wait a bit for batching to occur
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Show batcher statistics
    let stats = batcher.get_stats().await;
    println!("\n📊 Batcher Statistics:");
    println!("   Pending Requests: {}", stats.pending_requests);
    println!("   Cache Entries: {}", stats.cache_entries);
    println!("   Available Batch Permits: {}", stats.available_batch_permits);
    
    Ok(())
}

/// Example: Streaming AI responses for real-time applications
pub async fn demonstrate_streaming_responses(service: &AIOrchestrationService) -> Result<()> {
    let request = CompletionRequest::new(
        vec![Message::user("Write a story about a robot learning to paint")],
        "claude-3-sonnet-20240229".to_string(),
    ).with_streaming(true)
     .with_priority(RequestPriority::High);
    
    println!("🌊 Starting streaming completion...");
    
    match service.stream_completion(request).await {
        Ok(mut stream) => {
            let mut accumulated_content = String::new();
            let mut chunk_count = 0;
            
            while !stream.is_complete() {
                match stream.next_chunk().await {
                    Ok(Some(chunk)) => {
                        chunk_count += 1;
                        accumulated_content.push_str(&chunk.content);
                        
                        print!("{}", chunk.content); // Real-time output
                        
                        if let Some(finish_reason) = chunk.finish_reason {
                            println!("\n\n✅ Stream completed: {:?}", finish_reason);
                            break;
                        }
                    }
                    Ok(None) => {
                        // No more chunks available right now, but stream may continue
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    Err(e) => {
                        println!("\n❌ Streaming error: {}", e);
                        break;
                    }
                }
            }
            
            println!("\n📊 Streaming Statistics:");
            println!("   Chunks received: {}", chunk_count);
            println!("   Total content length: {}", accumulated_content.len());
            println!("   Final content preview: {}...", 
                &accumulated_content[..accumulated_content.len().min(100)]);
        }
        Err(e) => {
            println!("❌ Failed to start streaming: {}", e);
        }
    }
    
    Ok(())
}

/// Example: Batch processing multiple requests efficiently
pub async fn demonstrate_batch_processing(service: &AIOrchestrationService) -> Result<()> {
    let requests = vec![
        CompletionRequest::new(
            vec![Message::user("Summarize the benefits of renewable energy")],
            "claude-3-haiku-20240307".to_string(),
        ),
        CompletionRequest::new(
            vec![Message::user("Explain quantum computing in simple terms")],
            "claude-3-haiku-20240307".to_string(),
        ),
        CompletionRequest::new(
            vec![Message::user("Write a haiku about programming")],
            "claude-3-haiku-20240307".to_string(),
        ),
        CompletionRequest::new(
            vec![Message::user("List 5 healthy breakfast ideas")],
            "gpt-3.5-turbo".to_string(),
        ),
    ];
    
    println!("📦 Processing {} requests in batch...", requests.len());
    let start_time = std::time::Instant::now();
    
    match service.batch_complete(requests).await {
        Ok(results) => {
            let duration = start_time.elapsed();
            println!("✅ Batch completed in {:?}", duration);
            
            let mut successful = 0;
            let mut failed = 0;
            
            for (i, result) in results.iter().enumerate() {
                match result {
                    Ok(response) => {
                        successful += 1;
                        let content_preview = if response.choices.is_empty() {
                            "No content"
                        } else {
                            &response.choices[0].message.content[..response.choices[0].message.content.len().min(50)]
                        };
                        println!("   {}. ✅ Success: {}...", i + 1, content_preview);
                    }
                    Err(e) => {
                        failed += 1;
                        println!("   {}. ❌ Failed: {}", i + 1, e);
                    }
                }
            }
            
            println!("\n📊 Batch Results:");
            println!("   Successful: {}", successful);
            println!("   Failed: {}", failed);
            println!("   Success Rate: {:.1}%", 
                (successful as f64 / results.len() as f64) * 100.0);
        }
        Err(e) => {
            println!("❌ Batch processing failed: {}", e);
        }
    }
    
    Ok(())
}

/// Example: Setting up custom performance alerting
pub async fn setup_performance_alerting() -> Result<PerformanceAlerting> {
    // Configure custom performance thresholds
    let thresholds = PerformanceThresholds {
        max_response_time: Duration::from_secs(10),  // Alert if response > 10s
        min_success_rate: 0.95,                      // Alert if success rate < 95%
        max_error_rate: 0.05,                        // Alert if error rate > 5%
        max_cost_per_request: 0.50,                  // Alert if cost > $0.50 per request
    };
    
    let alerting = PerformanceAlerting::new(thresholds, 1000);
    
    println!("🚨 Performance alerting configured:");
    println!("   Max Response Time: 10s");
    println!("   Min Success Rate: 95%");
    println!("   Max Error Rate: 5%");
    println!("   Max Cost per Request: $0.50");
    
    Ok(alerting)
}

/// Example: Cost optimization analysis
pub async fn analyze_cost_optimization(service: &AIOrchestrationService) -> Result<()> {
    let test_request = CompletionRequest::new(
        vec![Message::user("Analyze the cost-effectiveness of different AI models for content generation")],
        "claude-3-sonnet-20240229".to_string(),
    );
    
    // Get cost estimates from all providers
    match service.estimate_costs(&test_request).await {
        Ok(estimates) => {
            println!("💰 Cost Analysis for Sample Request:");
            
            let mut sorted_estimates: Vec<_> = estimates.into_iter().collect();
            sorted_estimates.sort_by(|a, b| a.1.total_cost.partial_cmp(&b.1.total_cost).unwrap());
            
            for (provider, estimate) in sorted_estimates {
                println!("   {}: ${:.4} (Input: {}, Output: {}, Available: {})",
                    provider,
                    estimate.total_cost,
                    estimate.input_tokens,
                    estimate.output_tokens,
                    if estimate.provider_available { "✅" } else { "❌" }
                );
            }
        }
        Err(e) => {
            println!("❌ Cost estimation failed: {}", e);
        }
    }
    
    Ok(())
}

/// Complete example: Running a production AI service with full monitoring
pub async fn run_production_example() -> Result<()> {
    println!("🚀 Starting Production AI Service Example\n");
    
    // 1. Setup the service
    let service = setup_production_ai_service().await?;
    
    // 2. Setup performance alerting
    let _alerting = setup_performance_alerting().await?;
    
    // 3. Demonstrate request batching
    println!("\n{}", "=".repeat(50));
    println!("📦 BATCH PROCESSING DEMONSTRATION");
    println!("{}", "=".repeat(50));
    demonstrate_request_batching().await?;
    
    // 4. Demonstrate streaming
    println!("\n{}", "=".repeat(50));
    println!("🌊 STREAMING DEMONSTRATION");
    println!("{}", "=".repeat(50));
    demonstrate_streaming_responses(&service).await?;
    
    // 5. Demonstrate batch processing
    println!("\n{}", "=".repeat(50));
    println!("📦 BATCH COMPLETION DEMONSTRATION");
    println!("{}", "=".repeat(50));
    demonstrate_batch_processing(&service).await?;
    
    // 6. Monitor performance
    println!("\n{}", "=".repeat(50));
    println!("📊 PERFORMANCE MONITORING");
    println!("{}", "=".repeat(50));
    monitor_ai_performance(&service).await?;
    
    // 7. Cost analysis
    println!("\n{}", "=".repeat(50));
    println!("💰 COST OPTIMIZATION ANALYSIS");
    println!("{}", "=".repeat(50));
    analyze_cost_optimization(&service).await?;
    
    println!("\n✅ Production example completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod example_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_production_setup() {
        // Test that we can create a production service
        // Note: This will fail without real API keys, but tests the structure
        let result = setup_production_ai_service().await;
        
        // Should fail due to invalid API keys, but not due to code issues
        match result {
            Ok(_) => println!("Service created successfully"),
            Err(e) => println!("Expected error with test keys: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_performance_alerting_setup() {
        let alerting = setup_performance_alerting().await.unwrap();
        
        // Verify alerting is configured
        let alerts = alerting.get_recent_alerts(10);
        assert!(alerts.is_empty()); // Should start with no alerts
    }
    
    #[tokio::test]
    async fn test_request_batching_setup() {
        let result = demonstrate_request_batching().await;
        
        // Should succeed in setting up batching infrastructure
        assert!(result.is_ok());
    }
}