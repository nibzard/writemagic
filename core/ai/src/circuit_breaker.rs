//! Circuit breaker patterns for provider isolation and intelligent fallback

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use std::collections::HashMap;
use writemagic_shared::{Result, WritemagicError};
use tokio::sync::Semaphore;
use metrics::{counter, gauge, histogram};

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen { attempts: usize },
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,
    /// Number of successful requests to close from half-open
    pub success_threshold: usize,
    /// Time to wait before transitioning from open to half-open
    pub timeout: Duration,
    /// Request timeout for individual operations
    pub request_timeout: Duration,
    /// Maximum concurrent requests in half-open state
    pub half_open_max_calls: usize,
    /// Minimum number of requests before evaluating failure rate
    pub minimum_throughput: usize,
    /// Window size for failure rate calculation (in seconds)
    pub failure_rate_window: Duration,
    /// Failure rate threshold (0.0 to 1.0)
    pub failure_rate_threshold: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
            minimum_throughput: 10,
            failure_rate_window: Duration::from_secs(60),
            failure_rate_threshold: 0.5,
        }
    }
}

impl CircuitBreakerConfig {
    /// Conservative configuration for critical services
    pub fn conservative() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 5,
            timeout: Duration::from_secs(120),
            request_timeout: Duration::from_secs(45),
            half_open_max_calls: 2,
            minimum_throughput: 20,
            failure_rate_window: Duration::from_secs(120),
            failure_rate_threshold: 0.3,
        }
    }

    /// Aggressive configuration for faster recovery
    pub fn aggressive() -> Self {
        Self {
            failure_threshold: 8,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(15),
            half_open_max_calls: 5,
            minimum_throughput: 5,
            failure_rate_window: Duration::from_secs(30),
            failure_rate_threshold: 0.7,
        }
    }
}

/// Request outcome for tracking
#[derive(Debug, Clone)]
pub struct RequestOutcome {
    pub timestamp: Instant,
    pub duration: Duration,
    pub success: bool,
    pub error_type: Option<String>,
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub circuit_opens: u64,
    pub circuit_closes: u64,
    pub half_open_transitions: u64,
    pub requests_blocked: u64,
    pub avg_response_time: Duration,
    pub current_failure_rate: f64,
}

/// Advanced circuit breaker with sliding window failure rate
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    outcomes: Arc<RwLock<Vec<RequestOutcome>>>,
    metrics: Arc<RwLock<CircuitMetrics>>,
    half_open_semaphore: Arc<Semaphore>,
    consecutive_failures: Arc<RwLock<usize>>,
    consecutive_successes: Arc<RwLock<usize>>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        let half_open_permits = config.half_open_max_calls;
        
        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            outcomes: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(CircuitMetrics::default())),
            half_open_semaphore: Arc::new(Semaphore::new(half_open_permits)),
            consecutive_failures: Arc::new(RwLock::new(0)),
            consecutive_successes: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if request can be executed
    pub async fn can_execute(&self) -> bool {
        let mut state = self.state.write();
        
        match &*state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.timeout {
                    *state = CircuitState::HalfOpen { attempts: 0 };
                    self.update_metrics_half_open_transition();
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Check if we can acquire a permit for half-open requests
                self.half_open_semaphore.try_acquire().is_ok()
            }
        }
    }

    /// Execute operation with circuit breaker protection
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display + Send + 'static,
    {
        if !self.can_execute().await {
            self.update_metrics_request_blocked();
            return Err(WritemagicError::circuit_breaker(format!(
                "Circuit breaker '{}' is open",
                self.name
            )));
        }

        let start = Instant::now();
        let _permit = if matches!(*self.state.read(), CircuitState::HalfOpen { .. }) {
            Some(self.half_open_semaphore.acquire().await.unwrap())
        } else {
            None
        };

        // Execute with timeout
        let result = tokio::time::timeout(self.config.request_timeout, operation()).await;
        let duration = start.elapsed();

        match result {
            Ok(Ok(value)) => {
                self.record_success(duration).await;
                Ok(value)
            }
            Ok(Err(e)) => {
                self.record_failure(duration, Some(e.to_string())).await;
                Err(WritemagicError::external(e.to_string()))
            }
            Err(_) => {
                self.record_failure(duration, Some("timeout".to_string())).await;
                Err(WritemagicError::timeout(self.config.request_timeout.as_millis() as u64))
            }
        }
    }

    /// Record successful operation
    pub async fn record_success(&self, duration: Duration) {
        let outcome = RequestOutcome {
            timestamp: Instant::now(),
            duration,
            success: true,
            error_type: None,
        };

        self.add_outcome(outcome);
        
        let mut consecutive_failures = self.consecutive_failures.write();
        let mut consecutive_successes = self.consecutive_successes.write();
        
        *consecutive_failures = 0;
        *consecutive_successes += 1;
        
        // Update state based on successes
        let mut state = self.state.write();
        match &*state {
            CircuitState::HalfOpen { attempts } => {
                if *consecutive_successes >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    *consecutive_successes = 0;
                    self.update_metrics_circuit_close();
                    tracing::info!("Circuit breaker '{}' closed after successful recovery", self.name);
                } else {
                    *state = CircuitState::HalfOpen { attempts: attempts + 1 };
                }
            }
            _ => {}
        }
        
        self.update_metrics_success(duration);
        
        // Emit metrics
        counter!("circuit_breaker_requests_total", &[("name", &self.name), ("result", "success")]).increment(1);
        histogram!("circuit_breaker_request_duration", &[("name", &self.name)]).record(duration.as_millis() as f64);
    }

    /// Record failed operation
    pub async fn record_failure(&self, duration: Duration, error_type: Option<String>) {
        let outcome = RequestOutcome {
            timestamp: Instant::now(),
            duration,
            success: false,
            error_type,
        };

        self.add_outcome(outcome);
        
        let mut consecutive_failures = self.consecutive_failures.write();
        let mut consecutive_successes = self.consecutive_successes.write();
        
        *consecutive_failures += 1;
        *consecutive_successes = 0;
        
        // Update state based on failures
        let should_open = self.should_open_circuit(*consecutive_failures);
        
        if should_open {
            let mut state = self.state.write();
            match &*state {
                CircuitState::Closed | CircuitState::HalfOpen { .. } => {
                    *state = CircuitState::Open { opened_at: Instant::now() };
                    self.update_metrics_circuit_open();
                    tracing::warn!(
                        "Circuit breaker '{}' opened after {} consecutive failures",
                        self.name, *consecutive_failures
                    );
                }
                _ => {}
            }
        }
        
        self.update_metrics_failure(duration);
        
        // Emit metrics
        counter!("circuit_breaker_requests_total", &[("name", &self.name), ("result", "failure")]).increment(1);
        histogram!("circuit_breaker_request_duration", &[("name", &self.name)]).record(duration.as_millis() as f64);
    }

    /// Check if circuit should open based on failure rate
    fn should_open_circuit(&self, consecutive_failures: usize) -> bool {
        // Simple consecutive failure threshold
        if consecutive_failures >= self.config.failure_threshold {
            return true;
        }
        
        // Advanced failure rate calculation
        let outcomes = self.outcomes.read();
        let now = Instant::now();
        let window_start = now - self.config.failure_rate_window;
        
        let recent_outcomes: Vec<&RequestOutcome> = outcomes
            .iter()
            .filter(|o| o.timestamp >= window_start)
            .collect();
        
        if recent_outcomes.len() < self.config.minimum_throughput {
            return false;
        }
        
        let failures = recent_outcomes.iter().filter(|o| !o.success).count();
        let failure_rate = failures as f64 / recent_outcomes.len() as f64;
        
        failure_rate >= self.config.failure_rate_threshold
    }

    /// Add outcome to sliding window
    fn add_outcome(&self, outcome: RequestOutcome) {
        let mut outcomes = self.outcomes.write();
        outcomes.push(outcome);
        
        // Keep only recent outcomes within window
        let cutoff = Instant::now() - self.config.failure_rate_window * 2;
        outcomes.retain(|o| o.timestamp >= cutoff);
        
        // Limit total outcomes to prevent unbounded growth
        if outcomes.len() > 1000 {
            outcomes.drain(0..outcomes.len() - 800);
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitState {
        self.state.read().clone()
    }

    /// Get circuit breaker metrics
    pub fn metrics(&self) -> CircuitMetrics {
        self.metrics.read().clone()
    }

    /// Get failure rate in the current window
    pub fn current_failure_rate(&self) -> f64 {
        let outcomes = self.outcomes.read();
        let now = Instant::now();
        let window_start = now - self.config.failure_rate_window;
        
        let recent_outcomes: Vec<&RequestOutcome> = outcomes
            .iter()
            .filter(|o| o.timestamp >= window_start)
            .collect();
        
        if recent_outcomes.is_empty() {
            return 0.0;
        }
        
        let failures = recent_outcomes.iter().filter(|o| !o.success).count();
        failures as f64 / recent_outcomes.len() as f64
    }

    /// Force circuit to open (for testing or manual intervention)
    pub fn force_open(&self) {
        let mut state = self.state.write();
        *state = CircuitState::Open { opened_at: Instant::now() };
        self.update_metrics_circuit_open();
        tracing::warn!("Circuit breaker '{}' manually forced open", self.name);
    }

    /// Force circuit to close (for testing or manual intervention)
    pub fn force_close(&self) {
        let mut state = self.state.write();
        *state = CircuitState::Closed;
        *self.consecutive_failures.write() = 0;
        *self.consecutive_successes.write() = 0;
        self.update_metrics_circuit_close();
        tracing::info!("Circuit breaker '{}' manually forced closed", self.name);
    }

    /// Reset all metrics and outcomes
    pub fn reset(&self) {
        self.outcomes.write().clear();
        *self.metrics.write() = CircuitMetrics::default();
        *self.consecutive_failures.write() = 0;
        *self.consecutive_successes.write() = 0;
        tracing::info!("Circuit breaker '{}' reset", self.name);
    }

    // Metric update helpers
    fn update_metrics_success(&self, duration: Duration) {
        let mut metrics = self.metrics.write();
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        metrics.current_failure_rate = self.current_failure_rate();
        
        // Update average response time with exponential smoothing
        let alpha = 0.1;
        if metrics.avg_response_time.is_zero() {
            metrics.avg_response_time = duration;
        } else {
            let new_avg = Duration::from_nanos(
                (alpha * duration.as_nanos() as f64 + 
                 (1.0 - alpha) * metrics.avg_response_time.as_nanos() as f64) as u64
            );
            metrics.avg_response_time = new_avg;
        }
    }

    fn update_metrics_failure(&self, duration: Duration) {
        let mut metrics = self.metrics.write();
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        metrics.current_failure_rate = self.current_failure_rate();
        
        // Update average response time
        let alpha = 0.1;
        if metrics.avg_response_time.is_zero() {
            metrics.avg_response_time = duration;
        } else {
            let new_avg = Duration::from_nanos(
                (alpha * duration.as_nanos() as f64 + 
                 (1.0 - alpha) * metrics.avg_response_time.as_nanos() as f64) as u64
            );
            metrics.avg_response_time = new_avg;
        }
    }

    fn update_metrics_circuit_open(&self) {
        self.metrics.write().circuit_opens += 1;
        gauge!("circuit_breaker_state", &[("name", &self.name)]).set(1.0); // 1.0 = open
    }

    fn update_metrics_circuit_close(&self) {
        self.metrics.write().circuit_closes += 1;
        gauge!("circuit_breaker_state", &[("name", &self.name)]).set(0.0); // 0.0 = closed
    }

    fn update_metrics_half_open_transition(&self) {
        self.metrics.write().half_open_transitions += 1;
        gauge!("circuit_breaker_state", &[("name", &self.name)]).set(0.5); // 0.5 = half-open
    }

    fn update_metrics_request_blocked(&self) {
        self.metrics.write().requests_blocked += 1;
        counter!("circuit_breaker_requests_blocked", &[("name", &self.name)]).increment(1);
    }
}

/// Circuit breaker registry for managing multiple circuit breakers
#[derive(Debug)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register circuit breaker
    pub fn register(&self, name: String, config: CircuitBreakerConfig) -> Arc<CircuitBreaker> {
        let breaker = Arc::new(CircuitBreaker::new(name.clone(), config));
        self.breakers.write().insert(name, breaker.clone());
        breaker
    }

    /// Get circuit breaker by name
    pub fn get(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.read().get(name).cloned()
    }

    /// Get all circuit breaker states
    pub fn get_all_states(&self) -> HashMap<String, CircuitState> {
        let breakers = self.breakers.read();
        breakers
            .iter()
            .map(|(name, breaker)| (name.clone(), breaker.state()))
            .collect()
    }

    /// Get all circuit breaker metrics
    pub fn get_all_metrics(&self) -> HashMap<String, CircuitMetrics> {
        let breakers = self.breakers.read();
        breakers
            .iter()
            .map(|(name, breaker)| (name.clone(), breaker.metrics()))
            .collect()
    }

    /// Force open all circuit breakers
    pub fn force_open_all(&self) {
        let breakers = self.breakers.read();
        for breaker in breakers.values() {
            breaker.force_open();
        }
    }

    /// Force close all circuit breakers
    pub fn force_close_all(&self) {
        let breakers = self.breakers.read();
        for breaker in breakers.values() {
            breaker.force_close();
        }
    }

    /// Reset all circuit breakers
    pub fn reset_all(&self) {
        let breakers = self.breakers.read();
        for breaker in breakers.values() {
            breaker.reset();
        }
    }

    /// Get list of all registered circuit breaker names
    pub fn list_names(&self) -> Vec<String> {
        self.breakers.read().keys().cloned().collect()
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for easy circuit breaker usage
#[macro_export]
macro_rules! with_circuit_breaker {
    ($breaker:expr, $operation:expr) => {
        $breaker.execute(|| async { $operation }).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Initially closed
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute().await);
        
        // Record failures to open circuit
        cb.record_failure(Duration::from_millis(10), Some("error".to_string())).await;
        cb.record_failure(Duration::from_millis(10), Some("error".to_string())).await;
        
        assert!(matches!(cb.state(), CircuitState::Open { .. }));
        assert!(!cb.can_execute().await);
        
        // Wait for timeout and transition to half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.can_execute().await);
        assert!(matches!(cb.state(), CircuitState::HalfOpen { .. }));
        
        // Record successes to close circuit
        cb.record_success(Duration::from_millis(10)).await;
        cb.record_success(Duration::from_millis(10)).await;
        
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_execution() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(50),
            request_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Successful execution
        let result = cb.execute(|| async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // Failed execution that opens circuit
        let result = cb.execute(|| async { Err::<i32, String>("error".to_string()) }).await;
        assert!(result.is_err());
        
        // Circuit should be open now
        let result = cb.execute(|| async { Ok::<i32, String>(42) }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("circuit breaker"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_registry() {
        let registry = CircuitBreakerRegistry::new();
        
        let cb1 = registry.register("test1".to_string(), CircuitBreakerConfig::default());
        let cb2 = registry.register("test2".to_string(), CircuitBreakerConfig::default());
        
        assert_eq!(registry.list_names().len(), 2);
        
        let retrieved = registry.get("test1").unwrap();
        assert!(Arc::ptr_eq(&cb1, &retrieved));
        
        let states = registry.get_all_states();
        assert_eq!(states.len(), 2);
        assert_eq!(states["test1"], CircuitState::Closed);
        assert_eq!(states["test2"], CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_failure_rate_calculation() {
        let config = CircuitBreakerConfig {
            failure_rate_threshold: 0.5,
            minimum_throughput: 3,
            failure_rate_window: Duration::from_millis(1000),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Add mixed outcomes
        cb.record_success(Duration::from_millis(10)).await;
        cb.record_failure(Duration::from_millis(10), Some("error".to_string())).await;
        cb.record_failure(Duration::from_millis(10), Some("error".to_string())).await;
        
        let failure_rate = cb.current_failure_rate();
        assert!((failure_rate - 0.666).abs() < 0.01); // Approximately 2/3
    }
}