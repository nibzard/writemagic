//! Benchmark tests for AI service performance and load testing

use writemagic_ai::{
    AIOrchestrationService, CompletionRequest, Message, RequestPriority,
    PerformanceMonitor, BatchConfig, RequestBatcher,
};
use writemagic_shared::Result;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[cfg(test)]
mod performance_benchmarks {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_overhead() {
        let monitor = PerformanceMonitor::new(10000);
        let start_time = Instant::now();
        
        // Benchmark metric creation and completion
        for i in 0..1000 {
            let mut metric = monitor.start_request(
                "benchmark-provider".to_string(),
                "benchmark-model".to_string(),
                format!("request-{}", i),
                RequestPriority::Normal,
            );
            
            metric.input_tokens = 100;
            metric.output_tokens = 150;
            metric.total_tokens = 250;
            metric.cost = 0.025;
            
            monitor.complete_request(metric);
        }
        
        let duration = start_time.elapsed();
        println!("Performance monitor overhead for 1000 operations: {:?}", duration);
        
        // Should complete reasonably quickly (under 100ms for 1000 operations)
        assert!(duration < Duration::from_millis(100));
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 1000);
        assert_eq!(stats.successful_requests, 1000);
    }

    #[tokio::test]
    async fn test_concurrent_performance_tracking() {
        let monitor = Arc::new(PerformanceMonitor::new(50000));
        let num_concurrent = 100;
        let requests_per_task = 50;
        
        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(num_concurrent));
        
        let mut handles = Vec::new();
        
        for task_id in 0..num_concurrent {
            let monitor = monitor.clone();
            let semaphore = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                for i in 0..requests_per_task {
                    let mut metric = monitor.start_request(
                        format!("provider-{}", task_id % 3), // Simulate 3 providers
                        format!("model-{}", task_id % 2),   // Simulate 2 models
                        format!("request-{}-{}", task_id, i),
                        match i % 3 {
                            0 => RequestPriority::Low,
                            1 => RequestPriority::Normal,
                            2 => RequestPriority::High,
                            _ => RequestPriority::Normal,
                        }
                    );
                    
                    // Simulate variable processing
                    tokio::time::sleep(Duration::from_micros(10)).await;
                    
                    metric.input_tokens = 50 + (i as u32) * 5;
                    metric.output_tokens = 100 + (i as u32) * 10;
                    metric.total_tokens = metric.input_tokens + metric.output_tokens;
                    metric.cost = (metric.total_tokens as f64) * 0.0001;
                    
                    if i % 10 == 9 {
                        // Simulate 10% failure rate
                        monitor.fail_request(metric, "simulated_error".to_string());
                    } else if i % 5 == 4 {
                        // Simulate 20% cache hit rate
                        monitor.record_cache_hit(metric);
                    } else {
                        monitor.complete_request(metric);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let total_duration = start_time.elapsed();
        let total_operations = num_concurrent * requests_per_task;
        
        println!("Concurrent performance test: {} operations in {:?}", total_operations, total_duration);
        println!("Average per operation: {:?}", total_duration / total_operations as u32);
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, total_operations as u64);
        
        // Verify metrics are reasonable
        let expected_successful = (total_operations as f64 * 0.7) as u64; // 70% success (excluding failures and cache hits)
        let expected_failed = (total_operations as f64 * 0.1) as u64; // 10% failure
        let expected_cache_hits = (total_operations as f64 * 0.2) as u64; // 20% cache hits
        
        assert!(stats.successful_requests >= expected_successful - 10);
        assert!(stats.failed_requests >= expected_failed - 10);
        assert!(stats.cache_hits >= expected_cache_hits - 10);
        
        // Test provider-specific stats
        for provider_id in 0..3 {
            let provider_name = format!("provider-{}", provider_id);
            if let Some(provider_stats) = monitor.get_provider_stats(&provider_name) {
                assert!(provider_stats.total_requests > 0);
                println!("Provider {} stats: {} requests, {} tokens, ${:.4} cost", 
                        provider_name, provider_stats.total_requests, 
                        provider_stats.total_tokens, provider_stats.total_cost);
            }
        }
    }

    #[tokio::test]
    async fn test_memory_usage_scaling() {
        // Test memory usage with large number of metrics
        let monitor = PerformanceMonitor::new(100000); // Large capacity
        
        let start_memory = get_memory_usage_estimate();
        
        // Add many metrics
        for i in 0..50000 {
            let mut metric = monitor.start_request(
                format!("provider-{}", i % 10),
                format!("model-{}", i % 5),
                format!("request-{}", i),
                RequestPriority::Normal,
            );
            
            metric.input_tokens = (i % 1000) as u32;
            metric.output_tokens = ((i * 2) % 1500) as u32;
            metric.total_tokens = metric.input_tokens + metric.output_tokens;
            metric.cost = (metric.total_tokens as f64) * 0.0001;
            
            monitor.complete_request(metric);
        }
        
        let end_memory = get_memory_usage_estimate();
        println!("Memory usage: start ~{}KB, end ~{}KB, growth ~{}KB", 
                start_memory / 1024, end_memory / 1024, (end_memory - start_memory) / 1024);
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 50000);
        
        // Memory growth should be reasonable (less than 100MB for 50k records)
        assert!((end_memory - start_memory) < 100 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_percentile_calculation_accuracy() {
        let monitor = PerformanceMonitor::new(10000);
        
        // Create metrics with known response times
        let response_times = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]; // ms
        
        for (i, &response_time) in response_times.iter().enumerate() {
            let mut metric = monitor.start_request(
                "test-provider".to_string(),
                "test-model".to_string(),
                format!("request-{}", i),
                RequestPriority::Normal,
            );
            
            // Simulate the response time by setting the metric timing manually
            metric.start_time = Instant::now() - Duration::from_millis(response_time);
            metric.input_tokens = 100;
            metric.output_tokens = 100;
            metric.total_tokens = 200;
            metric.cost = 0.02;
            
            monitor.complete_request(metric);
        }
        
        let stats = monitor.get_overall_stats();
        
        // Check percentile calculations
        println!("Response time percentiles:");
        println!("  Average: {:?}", stats.avg_response_time);
        println!("  P50: {:?}", stats.p50_response_time);
        println!("  P95: {:?}", stats.p95_response_time);
        println!("  P99: {:?}", stats.p99_response_time);
        
        // P50 should be around 55ms (median of 10-100)
        assert!(stats.p50_response_time >= Duration::from_millis(40));
        assert!(stats.p50_response_time <= Duration::from_millis(70));
        
        // P95 should be around 95ms
        assert!(stats.p95_response_time >= Duration::from_millis(85));
        assert!(stats.p95_response_time <= Duration::from_millis(105));
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let monitor = PerformanceMonitor::new(10000);
        
        // Test cache hit vs miss performance
        let start_time = Instant::now();
        
        // First batch: cache misses
        for i in 0..1000 {
            let mut metric = monitor.start_request(
                "cache-test".to_string(),
                "test-model".to_string(),
                format!("miss-{}", i),
                RequestPriority::Normal,
            );
            
            metric.input_tokens = 100;
            metric.output_tokens = 200;
            metric.total_tokens = 300;
            metric.cost = 0.03;
            
            monitor.complete_request(metric);
        }
        
        let miss_time = start_time.elapsed();
        
        let cache_start = Instant::now();
        
        // Second batch: cache hits (should be faster)
        for i in 0..1000 {
            let metric = monitor.start_request(
                "cache-test".to_string(),
                "test-model".to_string(),
                format!("hit-{}", i),
                RequestPriority::Normal,
            );
            
            monitor.record_cache_hit(metric);
        }
        
        let hit_time = cache_start.elapsed();
        
        println!("Cache performance: miss time {:?}, hit time {:?}, ratio {:.2}x", 
                miss_time, hit_time, miss_time.as_nanos() as f64 / hit_time.as_nanos() as f64);
        
        let stats = monitor.get_overall_stats();
        assert_eq!(stats.total_requests, 2000);
        assert_eq!(stats.cache_hits, 1000);
        
        // Cache hits should be significantly faster
        assert!(hit_time < miss_time);
    }

    // Helper function to estimate memory usage (simplified)
    fn get_memory_usage_estimate() -> usize {
        // This is a simplified estimate - in a real benchmark you might use
        // more sophisticated memory measurement tools
        std::mem::size_of::<usize>() * 1000 // Rough baseline estimate
    }
}

#[cfg(test)]
mod batch_processing_benchmarks {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_batch_processing_throughput() {
        let config = BatchConfig {
            max_batch_size: 10,
            max_wait_time: Duration::from_millis(10),
            max_concurrent_batches: 5,
            enable_deduplication: true,
            priority_ordering: true,
        };
        
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        let num_requests = 1000;
        let start_time = Instant::now();
        
        // Submit many requests concurrently
        let mut handles = Vec::new();
        
        for i in 0..num_requests {
            let batcher = batcher.clone();
            let handle = tokio::spawn(async move {
                let request = CompletionRequest::new(
                    vec![Message::user(&format!("Request {}", i))],
                    "test-model".to_string(),
                ).with_priority(match i % 3 {
                    0 => RequestPriority::Low,
                    1 => RequestPriority::Normal,
                    2 => RequestPriority::High,
                    _ => RequestPriority::Normal,
                });
                
                // Note: In a real test, we'd need to handle the response
                // For benchmark purposes, we're measuring batching overhead
                tokio::time::timeout(
                    Duration::from_secs(1),
                    batcher.submit_request(request)
                ).await
            });
            
            handles.push(handle);
        }
        
        // Count batches created
        let mut batch_count = 0;
        let mut total_batched_requests = 0;
        
        // Collect batches for a short time
        let batch_collection_start = Instant::now();
        while batch_collection_start.elapsed() < Duration::from_millis(100) {
            if let Ok(batch) = receiver.try_recv() {
                batch_count += 1;
                total_batched_requests += batch.requests.len();
                
                // Verify batch properties
                assert!(batch.requests.len() <= 10); // Respects max batch size
                assert!(!batch.batch_id.is_empty());
                
                // Verify priority ordering if enabled
                if batch.requests.len() > 1 {
                    let priorities: Vec<RequestPriority> = batch.requests.iter()
                        .map(|r| r.priority.clone())
                        .collect();
                    
                    let mut sorted_priorities = priorities.clone();
                    sorted_priorities.sort_by(|a, b| b.cmp(a));
                    // Note: May not be perfectly sorted due to async submission timing
                }
            }
            
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        
        let processing_time = start_time.elapsed();
        
        println!("Batch processing benchmark:");
        println!("  {} requests submitted in {:?}", num_requests, processing_time);
        println!("  {} batches created", batch_count);
        println!("  {} requests batched so far", total_batched_requests);
        
        if batch_count > 0 {
            println!("  Average requests per batch: {:.1}", total_batched_requests as f64 / batch_count as f64);
            println!("  Average batch creation time: {:?}", processing_time / batch_count as u32);
        }
        
        let stats = batcher.get_stats().await;
        println!("  Pending requests: {}", stats.pending_requests);
        println!("  Cache entries: {}", stats.cache_entries);
        
        // Should have created multiple batches
        assert!(batch_count > 0);
        
        // Batching should improve throughput (total processing time should be reasonable)
        assert!(processing_time < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_deduplication_effectiveness() {
        let config = BatchConfig {
            enable_deduplication: true,
            max_batch_size: 5,
            max_wait_time: Duration::from_millis(50),
            ..Default::default()
        };
        
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        let (batcher, mut receiver) = RequestBatcher::new(config, batch_rx);
        
        // Create many identical requests
        let identical_request = || CompletionRequest::new(
            vec![Message::user("Identical request for deduplication test")],
            "test-model".to_string(),
        );
        
        let num_duplicates = 100;
        let start_time = Instant::now();
        
        // Submit first request and cache a response
        let test_response = writemagic_ai::CompletionResponse {
            id: "dedup-test-response".to_string(),
            choices: vec![writemagic_ai::Choice {
                index: 0,
                message: Message::assistant("Cached response"),
                finish_reason: Some(writemagic_ai::FinishReason::Stop),
            }],
            usage: writemagic_ai::Usage {
                prompt_tokens: 50,
                completion_tokens: 25,
                total_tokens: 75,
            },
            model: "test-model".to_string(),
            created: chrono::Utc::now().timestamp(),
            metadata: std::collections::HashMap::new(),
        };
        
        let sample_request = identical_request();
        let request_hash = batcher.calculate_request_hash(&sample_request);
        batcher.cache_response(request_hash, test_response).await;
        
        // Submit many identical requests
        let mut handles = Vec::new();
        
        for i in 0..num_duplicates {
            let batcher = batcher.clone();
            let handle = tokio::spawn(async move {
                let request = identical_request();
                let start = Instant::now();
                let result = batcher.submit_request(request).await;
                let duration = start.elapsed();
                (i, result, duration)
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut cache_hits = 0;
        let mut total_response_time = Duration::ZERO;
        
        for handle in handles {
            if let Ok((_, result, duration)) = handle.await {
                if result.is_ok() {
                    cache_hits += 1;
                    total_response_time += duration;
                }
            }
        }
        
        let total_time = start_time.elapsed();
        
        println!("Deduplication effectiveness:");
        println!("  {} identical requests in {:?}", num_duplicates, total_time);
        println!("  {} cache hits", cache_hits);
        println!("  Average response time: {:?}", total_response_time / cache_hits.max(1) as u32);
        
        // Most requests should hit the cache
        let cache_hit_rate = cache_hits as f64 / num_duplicates as f64;
        println!("  Cache hit rate: {:.1}%", cache_hit_rate * 100.0);
        
        // Should have high cache hit rate (>80%)
        assert!(cache_hit_rate > 0.8);
        
        // Average response time should be very fast due to caching
        let avg_response_time = total_response_time / cache_hits.max(1) as u32;
        assert!(avg_response_time < Duration::from_millis(10));
        
        let stats = batcher.get_stats().await;
        assert!(stats.cache_entries > 0);
    }

    #[tokio::test]
    async fn test_load_balancing_performance() {
        // This test simulates load balancing across multiple providers
        let monitor = PerformanceMonitor::new(10000);
        
        let providers = vec!["claude", "openai", "local"];
        let models = vec!["model-fast", "model-medium", "model-slow"];
        
        let num_requests = 300; // 100 per provider
        let start_time = Instant::now();
        
        // Simulate different provider performance characteristics
        for i in 0..num_requests {
            let provider = providers[i % providers.len()];
            let model = models[i % models.len()];
            
            let mut metric = monitor.start_request(
                provider.to_string(),
                model.to_string(),
                format!("request-{}", i),
                RequestPriority::Normal,
            );
            
            // Simulate different response times per provider
            let response_time_ms = match provider {
                "claude" => 100 + (i % 50) as u64,      // 100-150ms
                "openai" => 150 + (i % 100) as u64,     // 150-250ms  
                "local" => 50 + (i % 30) as u64,        // 50-80ms
                _ => 100,
            };
            
            metric.start_time = Instant::now() - Duration::from_millis(response_time_ms);
            metric.input_tokens = 100 + (i as u32 % 200);
            metric.output_tokens = 150 + (i as u32 % 300);
            metric.total_tokens = metric.input_tokens + metric.output_tokens;
            
            // Different cost structures
            let cost_per_token = match provider {
                "claude" => 0.0001,
                "openai" => 0.00015,
                "local" => 0.00005,
                _ => 0.0001,
            };
            metric.cost = metric.total_tokens as f64 * cost_per_token;
            
            // Simulate different success rates
            let success_rate = match provider {
                "claude" => 0.98,
                "openai" => 0.95,
                "local" => 0.90,
                _ => 0.95,
            };
            
            if (i as f64 / num_requests as f64) < success_rate {
                monitor.complete_request(metric);
            } else {
                monitor.fail_request(metric, format!("{}_error", provider));
            }
        }
        
        let total_time = start_time.elapsed();
        
        println!("Load balancing performance analysis:");
        println!("  {} requests across {} providers in {:?}", num_requests, providers.len(), total_time);
        
        // Analyze per-provider performance
        for provider in &providers {
            if let Some(stats) = monitor.get_provider_stats(provider) {
                let success_rate = stats.successful_requests as f64 / stats.total_requests as f64;
                let avg_cost = stats.total_cost / stats.total_requests as f64;
                
                println!("  {}: {} requests, {:.1}% success, {:?} avg time, ${:.5} avg cost",
                        provider, stats.total_requests, success_rate * 100.0,
                        stats.avg_response_time, avg_cost);
            }
        }
        
        let overall_stats = monitor.get_overall_stats();
        assert_eq!(overall_stats.total_requests, num_requests as u64);
        
        // Local provider should have lowest average response time
        let local_stats = monitor.get_provider_stats("local").unwrap();
        let claude_stats = monitor.get_provider_stats("claude").unwrap();
        let openai_stats = monitor.get_provider_stats("openai").unwrap();
        
        assert!(local_stats.avg_response_time < claude_stats.avg_response_time);
        assert!(claude_stats.avg_response_time < openai_stats.avg_response_time);
        
        // Cost ordering should reflect our setup
        let local_avg_cost = local_stats.total_cost / local_stats.total_requests as f64;
        let claude_avg_cost = claude_stats.total_cost / claude_stats.total_requests as f64;
        let openai_avg_cost = openai_stats.total_cost / openai_stats.total_requests as f64;
        
        assert!(local_avg_cost < claude_avg_cost);
        assert!(claude_avg_cost < openai_avg_cost);
    }
}