//! Production monitoring and observability patterns

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, instrument, Span};
use serde::{Serialize, Deserialize};

/// Comprehensive tracing setup for production systems
pub mod tracing_setup {
    use super::*;
    use tracing_subscriber::{
        layer::SubscriberExt, 
        util::SubscriberInitExt,
        EnvFilter, 
        fmt,
        Registry,
    };
    
    /// Initialize comprehensive tracing with multiple outputs
    pub fn init_production_tracing(service_name: &str, version: &str) {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,writemagic=debug".into());
        
        // Console output with JSON formatting
        let console_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_ansi(atty::is(atty::Stream::Stdout))
            .json();
        
        // File output with rotation
        #[cfg(feature = "file-logging")]
        {
            use tracing_appender::rolling::{RollingFileAppender, Rotation};
            let file_appender = RollingFileAppender::new(
                Rotation::daily(),
                "logs",
                format!("{}.log", service_name),
            );
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .json();
        }
        
        let registry = Registry::default()
            .with(filter)
            .with(console_layer);
        
        // Add OpenTelemetry if configured
        #[cfg(feature = "opentelemetry")]
        {
            if let Ok(otlp_endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
                use opentelemetry::sdk::trace::TracerProvider;
                use opentelemetry_otlp::WithExportConfig;
                use tracing_opentelemetry::OpenTelemetryLayer;
                
                let tracer_result = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(
                        opentelemetry_otlp::new_exporter()
                            .tonic()
                            .with_endpoint(otlp_endpoint)
                    )
                    .with_trace_config(
                        opentelemetry::sdk::trace::config()
                            .with_resource(opentelemetry::sdk::Resource::new(vec![
                                opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                                opentelemetry::KeyValue::new("service.version", version.to_string()),
                            ]))
                    )
                    .install_batch(opentelemetry::runtime::Tokio);
                
                let tracer = match tracer_result {
                    Ok(tracer) => tracer,
                    Err(e) => {
                        log::error!("Failed to initialize OpenTelemetry tracer: {}", e);
                        registry.init();
                        return;
                    }
                };
                
                let telemetry_layer = OpenTelemetryLayer::new(tracer);
                registry.with(telemetry_layer).init();
                return;
            }
        }
        
        registry.init();
        
        info!(
            service = service_name,
            version = version,
            "Tracing initialized"
        );
    }
    
    /// Create a structured span with common fields
    pub fn create_request_span(
        request_id: &str,
        operation: &str,
        user_id: Option<&str>,
    ) -> Span {
        let span = tracing::info_span!(
            "request",
            request_id = request_id,
            operation = operation,
            user_id = user_id,
        );
        
        span.in_scope(|| {
            debug!("Starting request processing");
        });
        
        span
    }
}

/// High-performance metrics collection
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Increment a counter
    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += value;
    }
    
    /// Record a histogram value
    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms
            .entry(name.to_string())
            .or_insert_with(Histogram::new)
            .record(value);
    }
    
    /// Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }
    
    /// Get all metrics as Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // Counters
        let counters = self.counters.read().await;
        for (name, value) in counters.iter() {
            output.push_str(&format!(
                "# TYPE {} counter\n{} {}\n",
                name, name, value
            ));
        }
        
        // Histograms
        let histograms = self.histograms.read().await;
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!(
                "# TYPE {} histogram\n",
                name
            ));
            
            let stats = histogram.stats();
            output.push_str(&format!(
                "{}_count {}\n{}_sum {}\n{}_avg {}\n{}_min {}\n{}_max {}\n",
                name, stats.count,
                name, stats.sum,
                name, stats.avg,
                name, stats.min,
                name, stats.max,
            ));
        }
        
        // Gauges
        let gauges = self.gauges.read().await;
        for (name, value) in gauges.iter() {
            output.push_str(&format!(
                "# TYPE {} gauge\n{} {}\n",
                name, name, value
            ));
        }
        
        // System metrics
        output.push_str(&format!(
            "# TYPE process_uptime_seconds gauge\nprocess_uptime_seconds {}\n",
            self.start_time.elapsed().as_secs()
        ));
        
        output
    }
    
    /// Export metrics as JSON
    pub async fn export_json(&self) -> serde_json::Value {
        let counters = self.counters.read().await.clone();
        let gauges = self.gauges.read().await.clone();
        
        let mut histograms_json = HashMap::new();
        let histograms = self.histograms.read().await;
        for (name, histogram) in histograms.iter() {
            histograms_json.insert(name.clone(), histogram.stats());
        }
        
        serde_json::json!({
            "counters": counters,
            "gauges": gauges,
            "histograms": histograms_json,
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple histogram implementation for metrics
#[derive(Debug, Clone)]
pub struct Histogram {
    values: Vec<f64>,
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            count: 0,
        }
    }
    
    pub fn record(&mut self, value: f64) {
        self.values.push(value);
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
        
        // Keep only recent values to prevent unbounded growth
        if self.values.len() > 10000 {
            self.values.drain(0..5000);
        }
    }
    
    pub fn stats(&self) -> HistogramStats {
        if self.count == 0 {
            return HistogramStats::default();
        }
        
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = percentile(&sorted, 0.5);
        let p95 = percentile(&sorted, 0.95);
        let p99 = percentile(&sorted, 0.99);
        
        HistogramStats {
            count: self.count,
            sum: self.sum,
            min: self.min,
            max: self.max,
            avg: self.sum / self.count as f64,
            p50,
            p95,
            p99,
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (p * (sorted_values.len() - 1) as f64) as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

/// Performance profiler for request tracing
pub struct PerformanceProfiler {
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
    metadata: HashMap<String, String>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a performance checkpoint
    pub fn checkpoint(&mut self, name: &str) {
        self.checkpoints.push((name.to_string(), Instant::now()));
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get performance report
    pub fn report(&self) -> PerformanceReport {
        let total_duration = self.start_time.elapsed();
        
        let mut segments = Vec::new();
        let mut last_time = self.start_time;
        
        for (name, checkpoint_time) in &self.checkpoints {
            let duration = checkpoint_time.duration_since(last_time);
            segments.push(PerformanceSegment {
                name: name.clone(),
                duration,
                elapsed_from_start: checkpoint_time.duration_since(self.start_time),
            });
            last_time = *checkpoint_time;
        }
        
        PerformanceReport {
            total_duration,
            segments,
            metadata: self.metadata.clone(),
        }
    }
    
    /// Log performance report with tracing
    #[instrument(skip(self))]
    pub fn log_report(&self, operation: &str) {
        let report = self.report();
        
        info!(
            operation = operation,
            total_ms = report.total_duration.as_millis(),
            segments = report.segments.len(),
            "Performance report"
        );
        
        for segment in &report.segments {
            debug!(
                segment = segment.name,
                duration_ms = segment.duration.as_millis(),
                elapsed_ms = segment.elapsed_from_start.as_millis(),
                "Performance segment"
            );
        }
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceReport {
    pub total_duration: Duration,
    pub segments: Vec<PerformanceSegment>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceSegment {
    pub name: String,
    #[serde(with = "duration_as_millis")]
    pub duration: Duration,
    #[serde(with = "duration_as_millis")]
    pub elapsed_from_start: Duration,
}

mod duration_as_millis {
    use serde::{Serialize, Serializer};
    use std::time::Duration;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }
}

/// Health check system for service monitoring
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheck + Send + Sync>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }
    
    /// Register a health check
    pub fn register<H>(&mut self, name: String, check: H)
    where
        H: HealthCheck + Send + Sync + 'static,
    {
        self.checks.insert(name, Box::new(check));
    }
    
    /// Run all health checks
    pub async fn check_all(&self) -> HealthReport {
        let start_time = Instant::now();
        let mut results = HashMap::new();
        
        for (name, check) in &self.checks {
            let check_start = Instant::now();
            let result = check.check().await;
            let check_duration = check_start.elapsed();
            
            results.insert(name.clone(), HealthCheckResult {
                status: result,
                duration: check_duration,
                timestamp: SystemTime::now(),
            });
        }
        
        let overall_status = if results.values().all(|r| r.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if results.values().any(|r| r.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };
        
        HealthReport {
            overall_status,
            checks: results,
            total_duration: start_time.elapsed(),
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check trait
#[async_trait::async_trait]
pub trait HealthCheck {
    async fn check(&self) -> HealthStatus;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    #[serde(with = "duration_as_millis")]
    pub duration: Duration,
    #[serde(with = "system_time_as_unix")]
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checks: HashMap<String, HealthCheckResult>,
    #[serde(with = "duration_as_millis")]
    pub total_duration: Duration,
}

mod system_time_as_unix {
    use serde::{Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let unix_timestamp = time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        unix_timestamp.serialize(serializer)
    }
}

/// Example health checks
pub mod health_checks {
    use super::*;
    use std::time::Duration;
    
    pub struct DatabaseHealthCheck {
        // In a real implementation, this would have a database connection
    }
    
    #[async_trait::async_trait]
    impl HealthCheck for DatabaseHealthCheck {
        async fn check(&self) -> HealthStatus {
            // Simulate database connectivity check
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            // In a real implementation:
            // match sqlx::query("SELECT 1").execute(&pool).await {
            //     Ok(_) => HealthStatus::Healthy,
            //     Err(_) => HealthStatus::Unhealthy,
            // }
            
            HealthStatus::Healthy
        }
    }
    
    pub struct MemoryHealthCheck {
        max_memory_mb: usize,
    }
    
    impl MemoryHealthCheck {
        pub fn new(max_memory_mb: usize) -> Self {
            Self { max_memory_mb }
        }
    }
    
    #[async_trait::async_trait]
    impl HealthCheck for MemoryHealthCheck {
        async fn check(&self) -> HealthStatus {
            // Check memory usage (simplified)
            let memory_usage = get_memory_usage_mb();
            
            if memory_usage > self.max_memory_mb {
                HealthStatus::Unhealthy
            } else if memory_usage > (self.max_memory_mb * 80 / 100) {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        }
    }
    
    fn get_memory_usage_mb() -> usize {
        // Simplified memory usage check
        // In a real implementation, use system APIs or process metrics
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.increment_counter("requests", 1).await;
        collector.record_histogram("response_time", 0.125).await;
        collector.set_gauge("memory_usage", 75.5).await;
        
        let json = collector.export_json().await;
        assert!(json["counters"]["requests"] == 1);
        assert!(json["gauges"]["memory_usage"] == 75.5);
        
        let prometheus = collector.export_prometheus().await;
        assert!(prometheus.contains("requests 1"));
        assert!(prometheus.contains("memory_usage 75.5"));
    }
    
    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();
        profiler.add_metadata("request_id", "123");
        
        std::thread::sleep(Duration::from_millis(10));
        profiler.checkpoint("validation");
        
        std::thread::sleep(Duration::from_millis(5));
        profiler.checkpoint("processing");
        
        let report = profiler.report();
        assert_eq!(report.segments.len(), 2);
        assert!(report.total_duration.as_millis() >= 15);
        assert_eq!(report.metadata["request_id"], "123");
    }
    
    #[tokio::test]
    async fn test_health_checker() {
        let mut checker = HealthChecker::new();
        checker.register(
            "test".to_string(),
            health_checks::MemoryHealthCheck::new(1000),
        );
        
        let report = checker.check_all().await;
        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert!(report.checks.contains_key("test"));
    }
    
    #[test]
    fn test_histogram() {
        let mut hist = Histogram::new();
        
        for i in 1..=100 {
            hist.record(i as f64);
        }
        
        let stats = hist.stats();
        assert_eq!(stats.count, 100);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 100.0);
        assert_eq!(stats.avg, 50.5);
    }
}