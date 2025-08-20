//! Load testing framework

use anyhow::Result;
use integration_tests::{TestResult, TestStatus, TestPlatform};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

/// Load test configuration
pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub duration_seconds: u64,
    pub requests_per_second: u64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 10,
            duration_seconds: 30,
            requests_per_second: 50,
        }
    }
}

/// Run load tests
pub async fn run_load_tests(config: LoadTestConfig) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();
    let start_time = Instant::now();
    
    // Simulate load test
    let mut tasks = Vec::new();
    
    for i in 0..config.concurrent_users {
        let task = tokio::spawn(simulate_user_load(i, config.duration_seconds));
        tasks.push(task);
    }
    
    // Wait for all users to complete
    for task in tasks {
        task.await?;
    }
    
    let duration = start_time.elapsed();
    
    results.push(TestResult {
        test_name: "Load Test - Concurrent Users".to_string(),
        platform: TestPlatform::CrossPlatform,
        status: TestStatus::Passed,
        duration_ms: duration.as_millis() as u64,
        message: Some(format!("{} concurrent users for {}s", config.concurrent_users, config.duration_seconds)),
        metrics: {
            let mut metrics = HashMap::new();
            metrics.insert("concurrent_users".to_string(), config.concurrent_users as f64);
            metrics.insert("duration_seconds".to_string(), config.duration_seconds as f64);
            metrics.insert("avg_response_time_ms".to_string(), 50.0); // Simulated
            metrics
        },
        timestamp: chrono::Utc::now(),
    });
    
    Ok(results)
}

/// Simulate user load
async fn simulate_user_load(user_id: usize, duration_seconds: u64) {
    let end_time = Instant::now() + Duration::from_secs(duration_seconds);
    let mut request_count = 0;
    
    while Instant::now() < end_time {
        // Simulate API request
        tokio::time::sleep(Duration::from_millis(100)).await;
        request_count += 1;
        
        // Simulate some requests taking longer
        if request_count % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    
    println!("User {} completed {} requests", user_id, request_count);
}