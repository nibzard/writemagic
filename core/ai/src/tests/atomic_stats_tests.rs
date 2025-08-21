//! Tests for atomic AI provider statistics
//! 
//! These tests ensure thread-safe statistics updates under concurrent access

use crate::providers::{AtomicUsageStats, ClaudeProvider, OpenAIProvider, AIProvider};
use std::sync::Arc;
use tokio::task;

#[tokio::test]
async fn test_atomic_stats_concurrent_updates() {
    let stats = Arc::new(AtomicUsageStats::new());
    let mut handles = Vec::new();
    
    // Spawn 10 concurrent tasks that each increment stats 100 times
    for i in 0..10 {
        let stats_clone = Arc::clone(&stats);
        let handle = task::spawn(async move {
            for j in 0..100 {
                let tokens = (i * 100 + j) as u64;
                let cost = tokens as f64 * 0.00001;
                stats_clone.increment_request(tokens, cost).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify final counts are correct
    let final_stats = stats.to_usage_stats().await;
    assert_eq!(final_stats.total_requests, 1000);
    assert_eq!(final_stats.requests_today, 1000);
    
    // Verify token counts (sum of 0..999)
    let expected_tokens: u64 = (0..1000).sum();
    assert_eq!(final_stats.total_tokens, expected_tokens);
    assert_eq!(final_stats.tokens_today, expected_tokens);
    
    // Verify costs
    let expected_cost = expected_tokens as f64 * 0.00001;
    assert!((final_stats.total_cost - expected_cost).abs() < 0.001);
    assert!((final_stats.cost_today - expected_cost).abs() < 0.001);
}

#[tokio::test]
async fn test_atomic_stats_no_race_conditions() {
    let stats = Arc::new(AtomicUsageStats::new());
    let mut handles = Vec::new();
    
    // Spawn tasks that read and write simultaneously
    // Writers
    for i in 0..5 {
        let stats_clone = Arc::clone(&stats);
        let handle = task::spawn(async move {
            for j in 0..50 {
                let tokens = (i * 50 + j + 1) as u64;
                let cost = tokens as f64 * 0.00002;
                stats_clone.increment_request(tokens, cost).await;
                
                // Add small delay to increase chance of interleaving
                tokio::time::sleep(tokio::time::Duration::from_nanos(1)).await;
            }
        });
        handles.push(handle);
    }
    
    // Readers
    for _ in 0..5 {
        let stats_clone = Arc::clone(&stats);
        let handle = task::spawn(async move {
            for _ in 0..50 {
                let current_stats = stats_clone.to_usage_stats().await;
                
                // Verify internal consistency (total >= today)
                assert!(current_stats.total_requests >= current_stats.requests_today);
                assert!(current_stats.total_tokens >= current_stats.tokens_today);
                assert!(current_stats.total_cost >= current_stats.cost_today);
                
                // Add small delay
                tokio::time::sleep(tokio::time::Duration::from_nanos(1)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Final consistency check
    let final_stats = stats.to_usage_stats().await;
    assert_eq!(final_stats.total_requests, 250);
    assert_eq!(final_stats.requests_today, 250);
    
    // Verify token sum (1+2+...+250)
    let expected_tokens: u64 = (1..=250).sum();
    assert_eq!(final_stats.total_tokens, expected_tokens);
}

#[tokio::test]
async fn test_provider_stats_initialization() {
    // Test that both providers initialize with atomic stats
    let claude_result = ClaudeProvider::new("test-key".to_string());
    assert!(claude_result.is_ok());
    
    let openai_result = OpenAIProvider::new("test-key".to_string());
    assert!(openai_result.is_ok());
    
    if let Ok(claude) = claude_result {
        let stats = claude.get_usage_stats().await.unwrap();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.total_cost, 0.0);
    }
}

#[tokio::test]
async fn test_atomic_stats_memory_ordering() {
    let stats = Arc::new(AtomicUsageStats::new());
    
    // Test that atomic operations maintain proper memory ordering
    // This is a stress test with rapid updates
    let mut handles = Vec::new();
    
    for i in 0..20 {
        let stats_clone = Arc::clone(&stats);
        let handle = task::spawn(async move {
            for j in 0..1000 {
                // Rapid fire updates
                stats_clone.increment_request(1, 0.001).await;
                
                // Occasionally yield to scheduler
                if j % 100 == 0 {
                    task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }
    
    // Also spawn readers during updates
    for _ in 0..5 {
        let stats_clone = Arc::clone(&stats);
        let handle = task::spawn(async move {
            for _ in 0..100 {
                let current = stats_clone.to_usage_stats().await;
                // Should never see negative or inconsistent values
                assert!(current.total_requests <= 20000);
                assert!(current.total_tokens <= 20000);
                assert!(current.total_cost >= 0.0);
                
                task::yield_now().await;
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify final state
    let final_stats = stats.to_usage_stats().await;
    assert_eq!(final_stats.total_requests, 20000);
    assert_eq!(final_stats.total_tokens, 20000);
    assert!((final_stats.total_cost - 20.0).abs() < 0.001);
}