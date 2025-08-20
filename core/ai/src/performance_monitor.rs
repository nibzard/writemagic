//! Performance monitoring and metrics collection for AI services

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use parking_lot::RwLock;
use metrics::{counter, histogram};

/// Performance metrics for AI requests
#[derive(Debug, Clone)]
pub struct AIPerformanceMetrics {
    pub provider_name: String,
    pub model_name: String,
    pub request_id: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub cost: f64,
    pub success: bool,
    pub error_type: Option<String>,
    pub cache_hit: bool,
    pub priority: crate::providers::RequestPriority,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cache_hits: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_response_time: Duration,
    pub p50_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
}

/// Performance monitoring service
#[derive(Debug)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<AIPerformanceMetrics>>>,
    max_metrics: usize,
    provider_stats: Arc<RwLock<HashMap<String, PerformanceStats>>>,
    model_stats: Arc<RwLock<HashMap<String, PerformanceStats>>>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(max_metrics: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::with_capacity(max_metrics))),
            max_metrics,
            provider_stats: Arc::new(RwLock::new(HashMap::new())),
            model_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tracking a request
    pub fn start_request(
        &self,
        provider_name: String,
        model_name: String,
        request_id: String,
        priority: crate::providers::RequestPriority,
    ) -> AIPerformanceMetrics {
        AIPerformanceMetrics {
            provider_name,
            model_name,
            request_id,
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost: 0.0,
            success: false,
            error_type: None,
            cache_hit: false,
            priority,
        }
    }

    /// Complete request tracking
    pub fn complete_request(&self, mut metric: AIPerformanceMetrics) {
        metric.end_time = Some(Instant::now());
        metric.duration = Some(metric.end_time.unwrap() - metric.start_time);
        metric.success = true;

        // Emit metrics to monitoring system
        self.emit_metrics(&metric);

        // Store metric
        self.store_metric(metric);
    }

    /// Record request failure
    pub fn fail_request(&self, mut metric: AIPerformanceMetrics, error_type: String) {
        metric.end_time = Some(Instant::now());
        metric.duration = Some(metric.end_time.unwrap() - metric.start_time);
        metric.success = false;
        metric.error_type = Some(error_type);

        // Emit metrics to monitoring system
        self.emit_metrics(&metric);

        // Store metric
        self.store_metric(metric);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self, mut metric: AIPerformanceMetrics) {
        metric.end_time = Some(Instant::now());
        metric.duration = Some(metric.end_time.unwrap() - metric.start_time);
        metric.success = true;
        metric.cache_hit = true;

        // Emit metrics to monitoring system
        self.emit_metrics(&metric);

        // Store metric
        self.store_metric(metric);
    }

    /// Get performance statistics for a provider
    pub fn get_provider_stats(&self, provider_name: &str) -> Option<PerformanceStats> {
        self.provider_stats.read().get(provider_name).cloned()
    }

    /// Get performance statistics for a model
    pub fn get_model_stats(&self, model_name: &str) -> Option<PerformanceStats> {
        self.model_stats.read().get(model_name).cloned()
    }

    /// Get overall performance statistics
    pub fn get_overall_stats(&self) -> PerformanceStats {
        let metrics = self.metrics.read();
        self.calculate_stats(&metrics)
    }

    /// Get recent performance metrics
    pub fn get_recent_metrics(&self, limit: usize) -> Vec<AIPerformanceMetrics> {
        let metrics = self.metrics.read();
        let start = if metrics.len() > limit { metrics.len() - limit } else { 0 };
        metrics[start..].to_vec()
    }

    /// Get performance trends over time
    pub fn get_performance_trends(&self, hours: u64) -> HashMap<String, Vec<f64>> {
        let metrics = self.metrics.read();
        let cutoff = Instant::now() - Duration::from_secs(hours * 3600);
        
        let recent_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.start_time >= cutoff)
            .collect();

        let mut trends = HashMap::new();
        
        // Calculate hourly response time trends
        let mut response_times = Vec::new();
        for metric in &recent_metrics {
            if let Some(duration) = metric.duration {
                response_times.push(duration.as_millis() as f64);
            }
        }
        trends.insert("response_time_ms".to_string(), response_times);
        
        // Calculate hourly success rate trends
        let success_rates: Vec<f64> = recent_metrics
            .chunks(recent_metrics.len().max(1) / hours.max(1) as usize)
            .map(|chunk| {
                let total = chunk.len() as f64;
                let successful = chunk.iter().filter(|m| m.success).count() as f64;
                if total > 0.0 { successful / total } else { 1.0 }
            })
            .collect();
        trends.insert("success_rate".to_string(), success_rates);
        
        trends
    }

    /// Store performance metric
    fn store_metric(&self, metric: AIPerformanceMetrics) {
        let mut metrics = self.metrics.write();
        
        // Remove old metrics if we exceed capacity
        if metrics.len() >= self.max_metrics {
            let remove_count = metrics.len() - self.max_metrics + 1;
            metrics.drain(0..remove_count);
        }
        
        metrics.push(metric.clone());
        drop(metrics);
        
        // Update aggregated statistics
        self.update_provider_stats(&metric);
        self.update_model_stats(&metric);
    }

    /// Emit metrics to monitoring system
    fn emit_metrics(&self, metric: &AIPerformanceMetrics) {
        // Counter metrics
        counter!("ai_requests_total", 1, 
            &[("provider", metric.provider_name.clone()),
              ("model", metric.model_name.clone()),
              ("success", metric.success.to_string())]);

        if metric.cache_hit {
            counter!("ai_cache_hits_total", 1, 
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone())]);
        }

        // Duration histogram
        if let Some(duration) = metric.duration {
            histogram!("ai_request_duration_ms", duration.as_millis() as f64,
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone())]);
        }

        // Token usage
        if metric.total_tokens > 0 {
            histogram!("ai_tokens_used", metric.total_tokens as f64,
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone()),
                  ("type", "total".to_string())]);

            histogram!("ai_tokens_used", metric.input_tokens as f64,
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone()),
                  ("type", "input".to_string())]);

            histogram!("ai_tokens_used", metric.output_tokens as f64,
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone()),
                  ("type", "output".to_string())]);
        }

        // Cost tracking
        if metric.cost > 0.0 {
            histogram!("ai_request_cost", metric.cost,
                &[("provider", metric.provider_name.clone()),
                  ("model", metric.model_name.clone())]);
        }

        // Priority tracking
        counter!("ai_requests_by_priority", 1,
            &[("priority", format!("{:?}", metric.priority))]);
    }

    /// Update provider statistics
    fn update_provider_stats(&self, metric: &AIPerformanceMetrics) {
        let mut stats = self.provider_stats.write();
        let provider_stats = stats.entry(metric.provider_name.clone()).or_default();
        
        provider_stats.total_requests += 1;
        if metric.success {
            provider_stats.successful_requests += 1;
        } else {
            provider_stats.failed_requests += 1;
        }
        
        if metric.cache_hit {
            provider_stats.cache_hits += 1;
        }
        
        provider_stats.total_tokens += metric.total_tokens as u64;
        provider_stats.total_cost += metric.cost;
        
        // Update response time statistics
        if let Some(duration) = metric.duration {
            provider_stats.avg_response_time = self.calculate_weighted_average(
                provider_stats.avg_response_time,
                duration,
                provider_stats.total_requests,
            );
        }
    }

    /// Update model statistics
    fn update_model_stats(&self, metric: &AIPerformanceMetrics) {
        let mut stats = self.model_stats.write();
        let model_stats = stats.entry(metric.model_name.clone()).or_default();
        
        model_stats.total_requests += 1;
        if metric.success {
            model_stats.successful_requests += 1;
        } else {
            model_stats.failed_requests += 1;
        }
        
        if metric.cache_hit {
            model_stats.cache_hits += 1;
        }
        
        model_stats.total_tokens += metric.total_tokens as u64;
        model_stats.total_cost += metric.cost;
        
        // Update response time statistics
        if let Some(duration) = metric.duration {
            model_stats.avg_response_time = self.calculate_weighted_average(
                model_stats.avg_response_time,
                duration,
                model_stats.total_requests,
            );
        }
    }

    /// Calculate weighted average for response times
    fn calculate_weighted_average(&self, current_avg: Duration, new_value: Duration, count: u64) -> Duration {
        if count <= 1 {
            return new_value;
        }
        
        let current_weight = (count - 1) as f64;
        let new_weight = 1.0;
        let total_weight = current_weight + new_weight;
        
        let weighted_avg = (current_avg.as_millis() as f64 * current_weight + 
                          new_value.as_millis() as f64 * new_weight) / total_weight;
        
        Duration::from_millis(weighted_avg as u64)
    }

    /// Calculate statistics from metrics
    fn calculate_stats(&self, metrics: &[AIPerformanceMetrics]) -> PerformanceStats {
        if metrics.is_empty() {
            return PerformanceStats::default();
        }
        
        let total_requests = metrics.len() as u64;
        let successful_requests = metrics.iter().filter(|m| m.success).count() as u64;
        let failed_requests = total_requests - successful_requests;
        let cache_hits = metrics.iter().filter(|m| m.cache_hit).count() as u64;
        
        let total_tokens: u64 = metrics.iter().map(|m| m.total_tokens as u64).sum();
        let total_cost: f64 = metrics.iter().map(|m| m.cost).sum();
        
        // Calculate response time percentiles
        let mut durations: Vec<Duration> = metrics
            .iter()
            .filter_map(|m| m.duration)
            .collect();
        durations.sort();
        
        let avg_response_time = if durations.is_empty() {
            Duration::ZERO
        } else {
            let total_ms: u64 = durations.iter().map(|d| d.as_millis() as u64).sum();
            Duration::from_millis(total_ms / durations.len() as u64)
        };
        
        let p50_response_time = self.percentile(&durations, 0.5);
        let p95_response_time = self.percentile(&durations, 0.95);
        let p99_response_time = self.percentile(&durations, 0.99);
        
        PerformanceStats {
            total_requests,
            successful_requests,
            failed_requests,
            cache_hits,
            total_tokens,
            total_cost,
            avg_response_time,
            p50_response_time,
            p95_response_time,
            p99_response_time,
        }
    }

    /// Calculate percentile from sorted durations
    fn percentile(&self, durations: &[Duration], percentile: f64) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }
        
        let index = (durations.len() as f64 * percentile).floor() as usize;
        let clamped_index = index.min(durations.len() - 1);
        durations[clamped_index]
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(10000) // Default to keeping 10k metrics
    }
}

/// Performance alert thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_response_time: Duration,
    pub min_success_rate: f64,
    pub max_error_rate: f64,
    pub max_cost_per_request: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time: Duration::from_secs(30),
            min_success_rate: 0.95,
            max_error_rate: 0.05,
            max_cost_per_request: 1.0,
        }
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub provider_name: String,
    pub model_name: String,
    pub threshold_value: f64,
    pub current_value: f64,
    pub timestamp: SystemTime,
}

/// Alert types for performance monitoring
#[derive(Debug, Clone)]
pub enum AlertType {
    HighResponseTime,
    LowSuccessRate,
    HighErrorRate,
    HighCost,
    ProviderDown,
}

/// Performance alerting service
#[derive(Debug)]
pub struct PerformanceAlerting {
    thresholds: PerformanceThresholds,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    max_alerts: usize,
}

impl PerformanceAlerting {
    pub fn new(thresholds: PerformanceThresholds, max_alerts: usize) -> Self {
        Self {
            thresholds,
            alerts: Arc::new(RwLock::new(Vec::with_capacity(max_alerts))),
            max_alerts,
        }
    }

    /// Check performance stats against thresholds
    pub fn check_thresholds(&self, provider_name: &str, model_name: &str, stats: &PerformanceStats) {
        // Check response time
        if stats.avg_response_time > self.thresholds.max_response_time {
            self.create_alert(
                AlertType::HighResponseTime,
                provider_name,
                model_name,
                self.thresholds.max_response_time.as_millis() as f64,
                stats.avg_response_time.as_millis() as f64,
            );
        }

        // Check success rate
        let success_rate = if stats.total_requests > 0 {
            stats.successful_requests as f64 / stats.total_requests as f64
        } else {
            1.0
        };

        if success_rate < self.thresholds.min_success_rate {
            self.create_alert(
                AlertType::LowSuccessRate,
                provider_name,
                model_name,
                self.thresholds.min_success_rate,
                success_rate,
            );
        }

        // Check error rate
        let error_rate = if stats.total_requests > 0 {
            stats.failed_requests as f64 / stats.total_requests as f64
        } else {
            0.0
        };

        if error_rate > self.thresholds.max_error_rate {
            self.create_alert(
                AlertType::HighErrorRate,
                provider_name,
                model_name,
                self.thresholds.max_error_rate,
                error_rate,
            );
        }

        // Check cost per request
        let cost_per_request = if stats.total_requests > 0 {
            stats.total_cost / stats.total_requests as f64
        } else {
            0.0
        };

        if cost_per_request > self.thresholds.max_cost_per_request {
            self.create_alert(
                AlertType::HighCost,
                provider_name,
                model_name,
                self.thresholds.max_cost_per_request,
                cost_per_request,
            );
        }
    }

    /// Create and store performance alert
    fn create_alert(
        &self,
        alert_type: AlertType,
        provider_name: &str,
        model_name: &str,
        threshold_value: f64,
        current_value: f64,
    ) {
        let alert = PerformanceAlert {
            alert_type: alert_type.clone(),
            provider_name: provider_name.to_string(),
            model_name: model_name.to_string(),
            threshold_value,
            current_value,
            timestamp: SystemTime::now(),
        };

        // Log the alert
        log::warn!(
            "Performance alert: {:?} for {}/{} - threshold: {}, current: {}",
            alert_type, provider_name, model_name, threshold_value, current_value
        );

        // Store the alert
        let mut alerts = self.alerts.write();
        if alerts.len() >= self.max_alerts {
            alerts.remove(0);
        }
        alerts.push(alert);
    }

    /// Get recent alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read();
        let start = if alerts.len() > limit { alerts.len() - limit } else { 0 };
        alerts[start..].to_vec()
    }

    /// Get alerts by type
    pub fn get_alerts_by_type(&self, alert_type: AlertType) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read();
        alerts
            .iter()
            .filter(|alert| std::mem::discriminant(&alert.alert_type) == std::mem::discriminant(&alert_type))
            .cloned()
            .collect()
    }
}