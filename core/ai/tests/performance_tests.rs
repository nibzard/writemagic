//! Tests for performance monitoring and optimization features

use writemagic_ai::{
    PerformanceMonitor, PerformanceThresholds, PerformanceAlerting, RequestBatcher, 
    BatchConfig, CompletionRequest, Message, RequestPriority,
    AIPerformanceMetrics,
};
use writemagic_shared::Result;
use std::time::Duration;
use tokio::sync::mpsc;

#[cfg(test)]
mod performance_monitor_tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(1000);
        assert_eq!(monitor.get_overall_stats().total_requests, 0);
    }

    #[test]
    fn test_performance_metric_tracking() {
        let monitor = PerformanceMonitor::new(1000);
        
        let mut metric = monitor.start_request(
            "claude".to_string(),
            "claude-3-sonnet".to_string(),
            "test-request-1".to_string(),
            RequestPriority::Normal,
        );
        
        // Simulate successful completion
        metric.input_tokens = 100;
        metric.output_tokens = 200;
        metric.total_tokens = 300;
        metric.cost = 0.05;
        
        monitor.complete_request(metric);
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.total_tokens, 300);
        assert!(stats.total_cost > 0.0);
    }

    #[test]
    fn test_performance_metric_failure_tracking() {
        let monitor = PerformanceMonitor::new(1000);
        
        let metric = monitor.start_request(
            "openai".to_string(),
            "gpt-4".to_string(),
            "test-request-fail".to_string(),
            RequestPriority::High,
        );
        
        monitor.fail_request(metric, "timeout_error".to_string());
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 1);
    }

    #[test]
    fn test_cache_hit_tracking() {
        let monitor = PerformanceMonitor::new(1000);
        
        let metric = monitor.start_request(
            "claude".to_string(),
            "claude-3-haiku".to_string(),
            "test-cache-hit".to_string(),
            RequestPriority::Low,
        );
        
        monitor.record_cache_hit(metric);
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.successful_requests, 1);
    }

    #[test]
    fn test_provider_stats() {
        let monitor = PerformanceMonitor::new(1000);
        
        // Add multiple requests for same provider
        for i in 0..5 {
            let mut metric = monitor.start_request(
                "claude".to_string(),
                "claude-3-sonnet".to_string(),
                format!("test-request-{}", i),
                RequestPriority::Normal,
            );
            
            metric.input_tokens = 50 + i as u32 * 10;
            metric.output_tokens = 100 + i as u32 * 20;
            metric.total_tokens = metric.input_tokens + metric.output_tokens;
            metric.cost = (metric.total_tokens as f64) * 0.0001;
            
            monitor.complete_request(metric);
        }
        
        let provider_stats = monitor.get_provider_stats("claude").unwrap();
        assert_eq!(provider_stats.total_requests, 5);
        assert_eq!(provider_stats.successful_requests, 5);
        assert!(provider_stats.total_tokens > 0);
        assert!(provider_stats.total_cost > 0.0);
    }

    #[test]
    fn test_recent_metrics_limit() {
        let monitor = PerformanceMonitor::new(10); // Small capacity
        
        // Add more metrics than capacity
        for i in 0..15 {
            let metric = monitor.start_request(
                "test-provider".to_string(),
                "test-model".to_string(),
                format!("request-{}", i),
                RequestPriority::Normal,
            );
            monitor.complete_request(metric);
        }
        
        let recent_metrics = monitor.get_recent_metrics(5);
        assert_eq!(recent_metrics.len(), 5);
        
        // Should have the most recent metrics
        let all_metrics = monitor.get_recent_metrics(20);
        assert!(all_metrics.len() <= 10); // Capped by monitor capacity
    }
}

#[cfg(test)]
mod performance_alerting_tests {
    use super::*;

    #[test]
    fn test_performance_alerting_creation() {
        let thresholds = PerformanceThresholds::default();
        let alerting = PerformanceAlerting::new(thresholds, 100);
        
        let alerts = alerting.get_recent_alerts(10);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_response_time_threshold() {
        let thresholds = PerformanceThresholds {
            max_response_time: Duration::from_millis(100),
            ..Default::default()
        };
        let alerting = PerformanceAlerting::new(thresholds, 100);
        
        let stats = writemagic_ai::PerformanceStats {
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            cache_hits: 2,
            total_tokens: 1000,
            total_cost: 0.50,
            avg_response_time: Duration::from_millis(200), // Exceeds threshold
            p50_response_time: Duration::from_millis(180),
            p95_response_time: Duration::from_millis(250),
            p99_response_time: Duration::from_millis(300),
        };
        
        alerting.check_thresholds("claude", "claude-3-sonnet", &stats);
        
        let alerts = alerting.get_recent_alerts(10);
        assert!(!alerts.is_empty());
        
        // Should have a high response time alert
        let has_response_time_alert = alerts.iter().any(|alert| {
            matches!(alert.alert_type, writemagic_ai::AlertType::HighResponseTime)
        });
        assert!(has_response_time_alert);
    }

    #[test]
    fn test_success_rate_threshold() {
        let thresholds = PerformanceThresholds {
            min_success_rate: 0.95,
            ..Default::default()
        };
        let alerting = PerformanceAlerting::new(thresholds, 100);
        
        let stats = writemagic_ai::PerformanceStats {
            total_requests: 100,
            successful_requests: 90, // 90% success rate, below threshold
            failed_requests: 10,
            cache_hits: 20,
            total_tokens: 10000,
            total_cost: 5.0,
            avg_response_time: Duration::from_millis(50),
            p50_response_time: Duration::from_millis(45),
            p95_response_time: Duration::from_millis(80),
            p99_response_time: Duration::from_millis(120),
        };
        
        alerting.check_thresholds("openai", "gpt-4", &stats);
        
        let alerts = alerting.get_recent_alerts(10);
        assert!(!alerts.is_empty());
        
        // Should have a low success rate alert
        let has_success_rate_alert = alerts.iter().any(|alert| {
            matches!(alert.alert_type, writemagic_ai::AlertType::LowSuccessRate)
        });
        assert!(has_success_rate_alert);
    }

    #[test]
    fn test_cost_threshold() {
        let thresholds = PerformanceThresholds {
            max_cost_per_request: 0.10,
            ..Default::default()
        };
        let alerting = PerformanceAlerting::new(thresholds, 100);
        
        let stats = writemagic_ai::PerformanceStats {
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            cache_hits: 0,
            total_tokens: 10000,
            total_cost: 2.0, // $0.20 per request, exceeds threshold
            avg_response_time: Duration::from_millis(100),
            p50_response_time: Duration::from_millis(90),
            p95_response_time: Duration::from_millis(150),
            p99_response_time: Duration::from_millis(200),
        };
        
        alerting.check_thresholds("claude", "claude-3-opus", &stats);
        
        let alerts = alerting.get_recent_alerts(10);
        assert!(!alerts.is_empty());
        
        // Should have a high cost alert
        let has_cost_alert = alerts.iter().any(|alert| {
            matches!(alert.alert_type, writemagic_ai::AlertType::HighCost)
        });
        assert!(has_cost_alert);
    }
}

#[cfg(test)]
mod request_batcher_tests {
    use super::*;

    fn create_test_request(content: &str, priority: RequestPriority) -> CompletionRequest {
        CompletionRequest::new(
            vec![Message::user(content)],
            "test-model".to_string(),
        ).with_priority(priority)
    }

    #[tokio::test]
    async fn test_request_batcher_creation() {
        let config = BatchConfig::default();
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Verify batcher was created
        let stats = batcher.get_stats().await;
        assert_eq!(stats.pending_requests, 0);
        assert_eq!(stats.cache_entries, 0);
    }

    #[tokio::test]
    async fn test_request_deduplication() {
        let config = BatchConfig {
            enable_deduplication: true,
            max_batch_size: 5,
            max_wait_time: Duration::from_millis(50),
            ..Default::default()
        };
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Create identical requests
        let request1 = create_test_request("Hello world", RequestPriority::Normal);
        let request2 = create_test_request("Hello world", RequestPriority::Normal);
        
        // Submit first request
        let batcher_clone = batcher.clone();
        let handle1 = tokio::spawn(async move {
            batcher_clone.submit_request(request1).await
        });
        
        // Cache a response for deduplication
        let test_response = writemagic_ai::CompletionResponse {
            id: "test-response".to_string(),
            choices: vec![writemagic_ai::Choice {
                index: 0,
                message: Message::assistant("Test response"),
                finish_reason: Some(writemagic_ai::FinishReason::Stop),
            }],
            usage: writemagic_ai::Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            model: "test-model".to_string(),
            created: chrono::Utc::now().timestamp(),
            metadata: std::collections::HashMap::new(),
        };
        
        let request_hash = batcher.calculate_request_hash(&request2);
        batcher.cache_response(request_hash, test_response.clone()).await;
        
        // Submit second identical request (should hit cache)
        let result = batcher.submit_request(request2).await;
        
        // Should get cached response
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, "test-response");
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = BatchConfig {
            priority_ordering: true,
            max_batch_size: 3,
            max_wait_time: Duration::from_millis(10),
            ..Default::default()
        };
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Create requests with different priorities
        let low_priority = create_test_request("Low priority", RequestPriority::Low);
        let high_priority = create_test_request("High priority", RequestPriority::High);
        let normal_priority = create_test_request("Normal priority", RequestPriority::Normal);
        
        // Submit in mixed order
        tokio::spawn({
            let batcher = batcher.clone();
            async move {
                let _ = batcher.submit_request(low_priority).await;
            }
        });
        
        tokio::spawn({
            let batcher = batcher.clone();
            async move {
                let _ = batcher.submit_request(normal_priority).await;
            }
        });
        
        tokio::spawn({
            let batcher = batcher.clone();
            async move {
                let _ = batcher.submit_request(high_priority).await;
            }
        });
        
        // Wait for batch to be created
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Try to receive batch
        if let Ok(batch) = receiver.try_recv() {
            // Verify batch has requests and they're sorted by priority
            assert!(!batch.requests.is_empty());
            assert_eq!(batch.priority, RequestPriority::High); // Highest priority wins
            
            // Requests should be sorted by priority (high to low)
            let priorities: Vec<RequestPriority> = batch.requests.iter()
                .map(|r| r.priority.clone())
                .collect();
            
            let mut sorted_priorities = priorities.clone();
            sorted_priorities.sort_by(|a, b| b.cmp(a)); // Descending order
            assert_eq!(priorities, sorted_priorities);
        }
    }

    #[tokio::test]
    async fn test_batch_size_limit() {
        let config = BatchConfig {
            max_batch_size: 2,
            max_wait_time: Duration::from_millis(100),
            ..Default::default()
        };
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Submit more requests than batch size
        for i in 0..5 {
            let request = create_test_request(&format!("Request {}", i), RequestPriority::Normal);
            tokio::spawn({
                let batcher = batcher.clone();
                async move {
                    let _ = batcher.submit_request(request).await;
                }
            });
        }
        
        // Should create multiple batches
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let mut batch_count = 0;
        let mut total_requests = 0;
        
        while let Ok(batch) = receiver.try_recv() {
            batch_count += 1;
            total_requests += batch.requests.len();
            assert!(batch.requests.len() <= 2); // Respect batch size limit
        }
        
        assert!(batch_count > 1); // Should have created multiple batches
    }

    #[tokio::test]
    async fn test_time_based_batching() {
        let config = BatchConfig {
            max_batch_size: 10, // Large batch size
            max_wait_time: Duration::from_millis(20), // Short wait time
            ..Default::default()
        };
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Submit a single request
        let request = create_test_request("Single request", RequestPriority::Normal);
        tokio::spawn({
            let batcher = batcher.clone();
            async move {
                let _ = batcher.submit_request(request).await;
            }
        });
        
        // Wait for time-based batching
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        // Should have created a batch due to timeout
        if let Ok(batch) = receiver.try_recv() {
            assert_eq!(batch.requests.len(), 1);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use writemagic_ai::{AIOrchestrationService, ClaudeProvider, OpenAIProvider};

    #[tokio::test]
    async fn test_performance_monitoring_integration() -> Result<()> {
        let mut service = AIOrchestrationService::new()?;
        
        // Add a mock provider for testing
        let claude_provider = ClaudeProvider::new("test-key".to_string())?;
        service.add_provider(std::sync::Arc::new(claude_provider)).await;
        
        // Get initial performance stats
        let initial_stats = service.get_performance_stats().await;
        assert_eq!(initial_stats.total_requests, 0);
        
        // Create a test request
        let request = CompletionRequest::new(
            vec![Message::user("Test performance monitoring")],
            "claude-3-haiku-20240307".to_string(),
        ).with_priority(RequestPriority::High);
        
        // Note: We can't actually complete the request without a real API key,
        // but we can test that the performance monitoring structure is in place
        let performance_monitor = service.performance_monitor();
        let test_metric = performance_monitor.start_request(
            "claude".to_string(),
            "claude-3-haiku".to_string(),
            "test-123".to_string(),
            RequestPriority::High,
        );
        
        performance_monitor.complete_request(test_metric);
        
        let updated_stats = service.get_performance_stats().await;
        assert_eq!(updated_stats.total_requests, 1);
        assert_eq!(updated_stats.successful_requests, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_alerting_integration() -> Result<()> {
        let service = AIOrchestrationService::new()?;
        
        // Get performance alerting service
        let alerting = service.performance_alerting();
        
        // Should start with no alerts
        let initial_alerts = alerting.get_recent_alerts(10);
        assert!(initial_alerts.is_empty());
        
        // Simulate some performance data that would trigger alerts
        let poor_stats = writemagic_ai::PerformanceStats {
            total_requests: 100,
            successful_requests: 85, // 85% success rate (below default 95% threshold)
            failed_requests: 15,
            cache_hits: 10,
            total_tokens: 50000,
            total_cost: 25.0,
            avg_response_time: Duration::from_secs(45), // 45 seconds (above default 30s threshold)
            p50_response_time: Duration::from_secs(30),
            p95_response_time: Duration::from_secs(60),
            p99_response_time: Duration::from_secs(90),
        };
        
        alerting.check_thresholds("test-provider", "test-model", &poor_stats);
        
        let alerts = alerting.get_recent_alerts(10);
        assert!(!alerts.is_empty());
        
        // Should have both success rate and response time alerts
        let alert_types: std::collections::HashSet<_> = alerts.iter()
            .map(|alert| std::mem::discriminant(&alert.alert_type))
            .collect();
        
        assert!(alert_types.len() >= 2); // Should have multiple alert types
        
        Ok(())
    }
}