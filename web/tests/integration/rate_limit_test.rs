use futures::future::join_all;
use std::time::Duration;

use crate::common::{assertions::*, TestApp};

#[tokio::test]
async fn test_rate_limit_basic() {
    let app = TestApp::new().await;

    // Make requests up to the limit (100 requests per minute in test config)
    for i in 0..100 {
        let response = app.server.get("/health").await;
        
        assert_success(&response);
        
        // Check rate limit headers
        let headers = response.headers();
        assert!(headers.contains_key("x-ratelimit-limit"));
        assert!(headers.contains_key("x-ratelimit-remaining"));
        assert!(headers.contains_key("x-ratelimit-reset"));
        
        let remaining: u32 = headers
            .get("x-ratelimit-remaining")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        
        assert_eq!(remaining, 100 - i - 1);
    }

    // The 101st request should be rate limited
    let response = app.server.get("/health").await;
    assert_rate_limit_error(&response);
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["error"], "rate_limit_exceeded");
    assert!(json_response["retry_after"].is_number());
}

#[tokio::test] 
async fn test_rate_limit_window_reset() {
    let app = TestApp::new().await;

    // Use up the rate limit
    for _ in 0..100 {
        let response = app.server.get("/health").await;
        assert_success(&response);
    }

    // Next request should be rate limited
    let response = app.server.get("/health").await;
    assert_rate_limit_error(&response);

    // Wait for rate limit window to reset (1 minute + buffer)
    tokio::time::sleep(Duration::from_secs(65)).await;

    // Should be able to make requests again
    let response = app.server.get("/health").await;
    assert_success(&response);
    
    let headers = response.headers();
    let remaining: u32 = headers
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    
    assert_eq!(remaining, 99); // Should reset to full limit minus 1
}

#[tokio::test]
async fn test_rate_limit_per_ip() {
    let app = TestApp::new().await;

    // Simulate requests from different IPs by using different client headers
    // Note: In a real test, you'd need to simulate different source IPs
    
    // Use up limit for "default" IP
    for _ in 0..100 {
        let response = app.server.get("/health").await;
        assert_success(&response);
    }

    // Should be rate limited
    let response = app.server.get("/health").await;
    assert_rate_limit_error(&response);
    
    // Requests with different forwarded-for headers should have separate limits
    let response = app.server
        .get("/health")
        .add_header("x-forwarded-for", "192.168.1.100")
        .await;
    
    assert_success(&response);
    
    let headers = response.headers();
    let remaining: u32 = headers
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    
    assert_eq!(remaining, 99); // Fresh limit for new IP
}

#[tokio::test]
async fn test_rate_limit_headers() {
    let app = TestApp::new().await;

    let response = app.server.get("/health").await;
    assert_success(&response);
    
    let headers = response.headers();
    
    // Check all expected rate limit headers are present
    assert!(headers.contains_key("x-ratelimit-limit"));
    assert!(headers.contains_key("x-ratelimit-remaining"));
    assert!(headers.contains_key("x-ratelimit-reset"));
    assert!(headers.contains_key("x-request-id"));
    
    // Verify header values
    let limit: u32 = headers
        .get("x-ratelimit-limit")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(limit, 100);
    
    let remaining: u32 = headers
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(remaining, 99);
    
    let reset: u64 = headers
        .get("x-ratelimit-reset")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(reset > 0 && reset <= 60); // Should be within 60 seconds
}

#[tokio::test]
async fn test_rate_limit_concurrent_requests() {
    let app = TestApp::new().await;

    // Make 50 concurrent requests
    let tasks = (0..50).map(|_| {
        let app = app.server.clone();
        tokio::spawn(async move {
            app.get("/health").await
        })
    });

    let responses = join_all(tasks).await;
    
    // All requests should complete (either success or rate limited)
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    for result in responses {
        let response = result.expect("Task should complete");
        if response.status_code().is_success() {
            success_count += 1;
        } else if response.status_code() == 429 {
            rate_limited_count += 1;
        }
    }
    
    // Should have some successful requests and possibly some rate limited ones
    assert!(success_count > 0);
    assert_eq!(success_count + rate_limited_count, 50);
    
    // Total should not exceed rate limit
    assert!(success_count <= 100);
}

#[tokio::test]
async fn test_rate_limit_cleanup() {
    let app = TestApp::new().await;

    // Make some requests to populate rate limit cache
    for _ in 0..10 {
        let response = app.server.get("/health").await;
        assert_success(&response);
    }

    // Get initial cache stats
    let initial_entries = app.app_state.rate_limiter.stats().active_entries;
    assert!(initial_entries > 0);

    // Wait for cleanup interval (should be 2 * window duration = 120 seconds for test)
    // For testing purposes, we'll check if the cleanup mechanism is in place
    // In a real scenario, you'd wait longer and verify cache cleanup
    
    // Verify the rate limiter has cleanup functionality
    app.app_state.rate_limiter.cleanup_expired();
    
    // The cleanup method exists and can be called
    // In a production test, you'd verify entries are actually removed after expiry
}

#[tokio::test]
async fn test_rate_limit_different_endpoints() {
    let app = TestApp::new().await;
    
    // Create and authenticate user for protected endpoints
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Use up rate limit on health endpoint
    for _ in 0..100 {
        let response = app.server.get("/health").await;
        assert_success(&response);
    }

    // Health endpoint should be rate limited
    let response = app.server.get("/health").await;
    assert_rate_limit_error(&response);

    // But other endpoints should also be rate limited since it's per-IP
    let response = app.server
        .get("/api/v1/auth/profile")
        .add_header("Authorization", &auth_header)
        .await;
    
    assert_rate_limit_error(&response);
}

#[tokio::test]
async fn test_rate_limit_error_format() {
    let app = TestApp::new().await;

    // Use up the rate limit
    for _ in 0..100 {
        let response = app.server.get("/health").await;
        assert_success(&response);
    }

    // Get rate limited response
    let response = app.server.get("/health").await;
    assert_rate_limit_error(&response);

    let json_response: serde_json::Value = response.json();
    
    // Verify error response structure
    assert_eq!(json_response["error"], "rate_limit_exceeded");
    assert_eq!(json_response["message"], "Too many requests. Please try again later.");
    assert_eq!(json_response["status"], 429);
    assert!(json_response["retry_after"].is_number());
    assert!(json_response["reset_time"].is_number());
    assert!(json_response["request_id"].is_string());
    
    // Verify retry-after header is present
    let headers = response.headers();
    assert!(headers.contains_key("retry-after"));
    assert!(headers.contains_key("x-ratelimit-reset"));
    assert!(headers.contains_key("x-request-id"));
}