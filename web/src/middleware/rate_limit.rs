use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::extractors::RequestId;

/// Rate limiting state that tracks requests per IP/user
#[derive(Clone)]
pub struct RateLimitState {
    /// Tracks rate limits by key (IP address or user ID)
    limits: Arc<DashMap<String, RateLimitEntry>>,
    /// Maximum requests per window
    max_requests: u32,
    /// Time window for rate limiting
    window_duration: Duration,
    /// How long to remember rate limit entries after they expire
    cleanup_interval: Duration,
}

/// Individual rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Number of requests made in current window
    count: u32,
    /// When the current window started
    window_start: Instant,
    /// When this entry was last accessed (for cleanup)
    last_access: Instant,
}

impl RateLimitState {
    /// Create a new rate limiter with specified limits
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            limits: Arc::new(DashMap::new()),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
            cleanup_interval: Duration::from_secs(window_seconds * 2),
        }
    }

    /// Check if a request should be rate limited
    pub fn check_rate_limit(&self, key: &str) -> RateLimitResult {
        let now = Instant::now();
        
        let mut entry = self.limits.entry(key.to_string()).or_insert_with(|| {
            RateLimitEntry {
                count: 0,
                window_start: now,
                last_access: now,
            }
        });

        // Check if we need to reset the window
        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.last_access = now;

        // Check if limit is exceeded
        if entry.count >= self.max_requests {
            let reset_time = entry.window_start + self.window_duration;
            let retry_after = reset_time.saturating_duration_since(now);
            
            RateLimitResult::Limited {
                retry_after: retry_after.as_secs(),
                reset_time: reset_time.duration_since(Instant::now()).as_secs(),
            }
        } else {
            entry.count += 1;
            RateLimitResult::Allowed {
                remaining: self.max_requests - entry.count,
                reset_time: (entry.window_start + self.window_duration)
                    .duration_since(now)
                    .as_secs(),
            }
        }
    }

    /// Clean up expired entries to prevent memory leaks
    pub fn cleanup_expired(&self) {
        let cutoff = Instant::now() - self.cleanup_interval;
        self.limits.retain(|_key, entry| entry.last_access > cutoff);
    }

    /// Get current statistics
    pub fn stats(&self) -> RateLimitStats {
        RateLimitStats {
            active_entries: self.limits.len(),
            max_requests: self.max_requests,
            window_seconds: self.window_duration.as_secs(),
        }
    }
}

/// Result of rate limit check
#[derive(Debug)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        reset_time: u64,
    },
    Limited {
        retry_after: u64,
        reset_time: u64,
    },
}

/// Rate limit statistics
#[derive(Debug, serde::Serialize)]
pub struct RateLimitStats {
    pub active_entries: usize,
    pub max_requests: u32,
    pub window_seconds: u64,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimitState>,
    request_id: RequestId,
    mut request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Extract client identifier (IP address from headers or connection info)
    let client_ip = extract_client_ip(request.headers())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    let result = rate_limiter.check_rate_limit(&client_ip);

    match result {
        RateLimitResult::Allowed { remaining, reset_time } => {
            // Add rate limit headers to the request for downstream handlers
            if let Ok(remaining_header) = remaining.to_string().parse() {
                request.headers_mut().insert("x-ratelimit-remaining", remaining_header);
            }
            if let Ok(reset_header) = reset_time.to_string().parse() {
                request.headers_mut().insert("x-ratelimit-reset", reset_header);
            }

            // Process the request
            let mut response = next.run(request).await;

            // Add rate limit headers to response
            if let Ok(limit_header) = rate_limiter.max_requests.to_string().parse() {
                response.headers_mut().insert("x-ratelimit-limit", limit_header);
            }
            if let Ok(remaining_header) = remaining.to_string().parse() {
                response.headers_mut().insert("x-ratelimit-remaining", remaining_header);
            }
            if let Ok(reset_header) = reset_time.to_string().parse() {
                response.headers_mut().insert("x-ratelimit-reset", reset_header);
            }
            if let Ok(request_id_header) = request_id.get().parse() {
                response.headers_mut().insert("x-request-id", request_id_header);
            }

            Ok(response)
        }
        RateLimitResult::Limited { retry_after, reset_time } => {
            tracing::warn!(
                "Rate limit exceeded for client: {} (request_id: {})",
                client_ip,
                request_id.get()
            );

            Err(RateLimitError::TooManyRequests {
                retry_after,
                reset_time,
                request_id: request_id.get().to_string(),
            })
        }
    }
}

/// Extract client IP from request headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try various headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip", // Cloudflare
        "x-client-ip",
        "forwarded",
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(header_str) = header_value.to_str() {
                // Handle comma-separated IPs (take the first one)
                let ip = header_str.split(',').next()?.trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Rate limiting errors
#[derive(Debug)]
pub enum RateLimitError {
    TooManyRequests {
        retry_after: u64,
        reset_time: u64,
        request_id: String,
    },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::TooManyRequests {
                retry_after,
                reset_time,
                request_id,
            } => {
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": "Too many requests. Please try again later.",
                        "retry_after": retry_after,
                        "reset_time": reset_time,
                        "request_id": request_id,
                        "status": 429
                    })),
                )
                    .into_response();

                // Add retry-after header
                if let Ok(retry_header) = retry_after.to_string().parse() {
                    response.headers_mut().insert("retry-after", retry_header);
                }
                if let Ok(reset_header) = reset_time.to_string().parse() {
                    response.headers_mut().insert("x-ratelimit-reset", reset_header);
                }
                if let Ok(request_id_header) = request_id.parse() {
                    response.headers_mut().insert("x-request-id", request_id_header);
                }

                response
            }
        }
    }
}

/// Background task to clean up expired rate limit entries
pub async fn rate_limit_cleanup_task(rate_limiter: RateLimitState) {
    let cleanup_interval = rate_limiter.cleanup_interval;
    let mut interval = tokio::time::interval(cleanup_interval);

    loop {
        interval.tick().await;
        rate_limiter.cleanup_expired();
        
        tracing::debug!(
            "Rate limiter cleanup completed. Active entries: {}",
            rate_limiter.limits.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limit_basic() {
        let rate_limiter = RateLimitState::new(5, 60); // 5 requests per 60 seconds
        let key = "test_client";

        // First 5 requests should be allowed
        for i in 0..5 {
            match rate_limiter.check_rate_limit(key) {
                RateLimitResult::Allowed { remaining, .. } => {
                    assert_eq!(remaining, 5 - i - 1);
                }
                RateLimitResult::Limited { .. } => panic!("Request {} should not be limited", i),
            }
        }

        // 6th request should be limited
        match rate_limiter.check_rate_limit(key) {
            RateLimitResult::Limited { .. } => {}
            RateLimitResult::Allowed { .. } => panic!("6th request should be limited"),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let rate_limiter = RateLimitState::new(2, 1); // 2 requests per 1 second
        let key = "test_client";

        // Use up the limit
        rate_limiter.check_rate_limit(key);
        rate_limiter.check_rate_limit(key);

        // Should be limited
        match rate_limiter.check_rate_limit(key) {
            RateLimitResult::Limited { .. } => {}
            RateLimitResult::Allowed { .. } => panic!("Should be limited"),
        }

        // Wait for window to reset
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be allowed again
        match rate_limiter.check_rate_limit(key) {
            RateLimitResult::Allowed { remaining, .. } => {
                assert_eq!(remaining, 1);
            }
            RateLimitResult::Limited { .. } => panic!("Should be allowed after window reset"),
        }
    }

    #[test]
    fn test_client_ip_extraction() {
        let mut headers = HeaderMap::new();
        
        // Test x-forwarded-for header
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.1".to_string()));

        // Test x-real-ip header (should override x-forwarded-for due to order)
        headers.insert("x-real-ip", "203.0.113.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.1".to_string())); // x-forwarded-for is checked first

        // Test with no headers
        let empty_headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&empty_headers), None);
    }
}