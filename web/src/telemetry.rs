use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info_span, Instrument};

use crate::extractors::RequestId;
use crate::state::AppState;

/// Initialize structured tracing for the application
pub fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Create a custom env filter with our default level
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            "writemagic_web=info,writemagic_core=info,sea_orm=warn,tower_http=info".into()
        });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .json(), // Use JSON format for structured logging
        )
        .try_init()?;

    tracing::info!("Structured tracing initialized successfully");
    Ok(())
}

/// Middleware for detailed request/response tracing with metrics
pub async fn trace_middleware(
    State(state): State<AppState>,
    request_id: RequestId,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract client IP for logging
    let client_ip = extract_client_ip(request.headers());

    // Create a span for this request
    let span = info_span!(
        "http_request",
        method = %method,
        uri = %uri,
        request_id = %request_id.get(),
        client_ip = %client_ip,
        user_agent = %user_agent,
    );

    async move {
        tracing::info!("Request started");

        // Process the request
        let response = next.run(request).await;
        
        let duration = start_time.elapsed();
        let status = response.status();
        
        // Record metrics
        record_request_metrics(&state, &method.to_string(), status, duration);
        
        // Log request completion with details
        tracing::info!(
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );

        // Log error responses with more detail
        if status.is_client_error() || status.is_server_error() {
            tracing::warn!(
                status = %status,
                duration_ms = duration.as_millis(),
                "Request completed with error"
            );
        }

        response
    }
    .instrument(span)
    .await
}

/// Extract client IP from request headers
fn extract_client_ip(headers: &axum::http::HeaderMap) -> String {
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip", 
        "cf-connecting-ip",
        "x-client-ip",
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(header_name) {
            if let Ok(header_str) = header_value.to_str() {
                let ip = header_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    "unknown".to_string()
}

/// Record request metrics (in a real app, this would use Prometheus or similar)
fn record_request_metrics(
    state: &AppState,
    method: &str,
    status: StatusCode,
    duration: std::time::Duration,
) {
    // For now, we'll store metrics in the app cache
    // In production, you'd use a proper metrics system like Prometheus
    
    let metrics_key = format!("metrics:requests:{}:{}", method, status.as_u16());
    
    // Increment request counter
    if let Some(current_count) = state.get_cached::<u64>(&metrics_key) {
        state.set_cached(metrics_key, current_count + 1, 3600); // Cache for 1 hour
    } else {
        state.set_cached(metrics_key, 1u64, 3600);
    }
    
    // Record response time histogram (simplified)
    let latency_key = format!("metrics:latency:{}:{}ms", method, duration.as_millis());
    if let Some(current_count) = state.get_cached::<u64>(&latency_key) {
        state.set_cached(latency_key, current_count + 1, 3600);
    } else {
        state.set_cached(latency_key, 1u64, 3600);
    }
}

/// Metrics collection and reporting
pub struct MetricsCollector {
    app_state: AppState,
}

impl MetricsCollector {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Get current application metrics
    pub fn get_metrics(&self) -> ApplicationMetrics {
        let cache = &self.app_state.cache;
        
        let mut request_counts = std::collections::HashMap::new();
        let mut response_times = std::collections::HashMap::new();
        let mut active_connections = 0;
        
        // Collect metrics from cache
        for entry in cache.iter() {
            let key = entry.key();
            if key.starts_with("metrics:requests:") {
                let count: u64 = self.app_state.get_cached(key).unwrap_or(0);
                request_counts.insert(key.clone(), count);
            } else if key.starts_with("metrics:latency:") {
                let count: u64 = self.app_state.get_cached(key).unwrap_or(0);
                response_times.insert(key.clone(), count);
            }
        }
        
        // Get rate limiter stats
        let rate_limit_stats = self.app_state.rate_limiter.stats();
        
        ApplicationMetrics {
            request_counts,
            response_times,
            active_connections,
            cache_entries: cache.len(),
            rate_limit_entries: rate_limit_stats.active_entries,
            uptime_seconds: 0, // Would track actual uptime in real implementation
        }
    }

    /// Get health check information
    pub async fn health_check(&self) -> HealthStatus {
        // Check database connectivity
        let db_healthy = self.check_database_health().await;
        
        // Check rate limiter
        let rate_limiter_healthy = self.app_state.rate_limiter.stats().active_entries < 100_000;
        
        // Check cache size
        let cache_healthy = self.app_state.cache.len() < 50_000;
        
        let overall_healthy = db_healthy && rate_limiter_healthy && cache_healthy;
        
        HealthStatus {
            healthy: overall_healthy,
            database: db_healthy,
            rate_limiter: rate_limiter_healthy,
            cache: cache_healthy,
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    async fn check_database_health(&self) -> bool {
        use sea_orm::{ConnectionTrait, Statement};
        
        // Simple query to check database connectivity
        let result = self.app_state.db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT 1".to_string()
            ))
            .await;
            
        result.is_ok()
    }
}

/// Application metrics structure
#[derive(Debug, serde::Serialize)]
pub struct ApplicationMetrics {
    pub request_counts: std::collections::HashMap<String, u64>,
    pub response_times: std::collections::HashMap<String, u64>,
    pub active_connections: u64,
    pub cache_entries: usize,
    pub rate_limit_entries: usize,
    pub uptime_seconds: u64,
}

/// Health check status
#[derive(Debug, serde::Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub database: bool,
    pub rate_limiter: bool,
    pub cache: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

/// Background task for metrics collection and cleanup
pub async fn metrics_collection_task(app_state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    let collector = MetricsCollector::new(app_state.clone());

    loop {
        interval.tick().await;
        
        // Collect and log metrics
        let metrics = collector.get_metrics();
        let health = collector.health_check().await;
        
        tracing::info!(
            cache_entries = metrics.cache_entries,
            rate_limit_entries = metrics.rate_limit_entries,
            healthy = health.healthy,
            "Metrics collected"
        );
        
        // Clean up expired cache entries
        app_state.cleanup_cache();
        
        // Log unhealthy status with more detail
        if !health.healthy {
            tracing::warn!(
                database = health.database,
                rate_limiter = health.rate_limiter,
                cache = health.cache,
                "Application health check failed"
            );
        }
    }
}

/// Performance monitoring utilities
pub mod performance {
    use std::time::Instant;
    
    /// Timer for measuring operation duration
    pub struct Timer {
        start: Instant,
        operation: String,
    }
    
    impl Timer {
        pub fn new(operation: impl Into<String>) -> Self {
            Self {
                start: Instant::now(),
                operation: operation.into(),
            }
        }
        
        pub fn finish(self) {
            let duration = self.start.elapsed();
            tracing::debug!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Operation completed"
            );
        }
    }
    
    /// Macro for easy timing of operations
    #[macro_export]
    macro_rules! time_operation {
        ($operation:expr, $code:block) => {{
            let timer = $crate::telemetry::performance::Timer::new($operation);
            let result = $code;
            timer.finish();
            result
        }};
    }
    
    pub use time_operation;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_ip_extraction() {
        let mut headers = axum::http::HeaderMap::new();
        
        // Test x-forwarded-for header
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "192.168.1.1");

        // Test with single IP
        headers.clear();
        headers.insert("x-real-ip", "203.0.113.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "203.0.113.1");

        // Test with no headers
        headers.clear();
        assert_eq!(extract_client_ip(&headers), "unknown");
    }

    #[tokio::test]
    async fn test_timer() {
        let timer = performance::Timer::new("test_operation");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        timer.finish();
        // Timer should complete without panic
    }
}