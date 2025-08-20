//! Advanced async retry patterns with circuit breaker

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time::Sleep;
use writemagic_shared::WritemagicError;

/// Exponential backoff retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f32,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 3.0,
            jitter: false,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout: Duration,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            reset_timeout: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: usize,
    success_count: usize,
    #[allow(dead_code)] // TODO: Implement circuit breaker scheduling in Phase 2
    next_attempt: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            next_attempt: None,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match &self.state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.reset_timeout {
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Open { .. } => {
                // Should not happen
            }
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        
        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open { opened_at: Instant::now() };
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open { opened_at: Instant::now() };
                self.success_count = 0;
            }
            CircuitState::Open { .. } => {
                // Already open
            }
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}

/// Retry future with exponential backoff
pub struct RetryFuture<F, Fut, T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
{
    operation: F,
    config: RetryConfig,
    current_attempt: usize,
    current_delay: Duration,
    current_future: Option<Fut>,
    sleep_future: Option<Pin<Box<Sleep>>>,
    circuit_breaker: Option<CircuitBreaker>,
}

impl<F, Fut, T, E> RetryFuture<F, Fut, T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
{
    pub fn new(operation: F, config: RetryConfig) -> Self {
        let initial_delay = config.initial_delay;
        Self {
            operation,
            config,
            current_attempt: 0,
            current_delay: initial_delay,
            current_future: None,
            sleep_future: None,
            circuit_breaker: None,
        }
    }

    pub fn with_circuit_breaker(mut self, circuit_breaker: CircuitBreaker) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    fn calculate_delay(&self) -> Duration {
        let mut delay = self.current_delay;

        if self.config.jitter {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            self.current_attempt.hash(&mut hasher);
            let hash = hasher.finish();
            
            let jitter_factor = (hash % 100) as f64 / 100.0; // 0.0 to 0.99
            let jitter_amount = delay.as_millis() as f64 * 0.1 * jitter_factor;
            delay += Duration::from_millis(jitter_amount as u64);
        }

        delay.min(self.config.max_delay)
    }
}

impl<F, Fut, T, E> Future for RetryFuture<F, Fut, T, E>
where
    F: FnMut() -> Fut + Unpin,
    Fut: Future<Output = std::result::Result<T, E>> + Unpin,
    E: std::fmt::Display,
{
    type Output = std::result::Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            // Check if we're currently sleeping
            if let Some(sleep) = &mut this.sleep_future {
                match sleep.as_mut().poll(cx) {
                    Poll::Ready(()) => {
                        this.sleep_future = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Check if we have a current operation running
            if let Some(future) = &mut this.current_future {
                match Pin::new(future).poll(cx) {
                    Poll::Ready(Ok(result)) => {
                        // Success! Record it with circuit breaker if present
                        if let Some(cb) = &mut this.circuit_breaker {
                            cb.record_success();
                        }
                        return Poll::Ready(Ok(result));
                    }
                    Poll::Ready(Err(error)) => {
                        this.current_future = None;
                        this.current_attempt += 1;

                        // Record failure with circuit breaker
                        if let Some(cb) = &mut this.circuit_breaker {
                            cb.record_failure();
                        }

                        // Check if we should retry
                        if this.current_attempt >= this.config.max_attempts {
                            return Poll::Ready(Err(error));
                        }

                        // Check circuit breaker
                        if let Some(cb) = &mut this.circuit_breaker {
                            if !cb.can_execute() {
                                return Poll::Ready(Err(error));
                            }
                        }

                        // Schedule next attempt
                        let delay = this.calculate_delay();
                        this.sleep_future = Some(Box::pin(tokio::time::sleep(delay)));
                        
                        // Update delay for next time
                        let next_delay = Duration::from_millis(
                            (this.current_delay.as_millis() as f32 * this.config.backoff_multiplier) as u64
                        );
                        this.current_delay = next_delay.min(this.config.max_delay);
                        
                        continue;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Check circuit breaker before starting new attempt
            if let Some(cb) = &mut this.circuit_breaker {
                if !cb.can_execute() {
                    return Poll::Ready(Err(
                        // We need a way to convert to the error type
                        // This is a limitation of the current design
                        // In practice, you'd need to adapt this based on your error types
                        unsafe { std::mem::zeroed() }
                    ));
                }
            }

            // Start new attempt
            this.current_future = Some((this.operation)());
        }
    }
}

/// Create a retry future with the given configuration
pub fn with_retry<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
) -> RetryFuture<F, Fut, T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
{
    RetryFuture::new(operation, config)
}

/// Create a retry future with circuit breaker
pub fn with_retry_and_circuit_breaker<F, Fut, T, E>(
    operation: F,
    retry_config: RetryConfig,
    circuit_config: CircuitBreakerConfig,
) -> RetryFuture<F, Fut, T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
{
    RetryFuture::new(operation, retry_config)
        .with_circuit_breaker(CircuitBreaker::new(circuit_config))
}

/// Timeout wrapper for futures
pub struct TimeoutFuture<F> {
    future: Pin<Box<F>>,
    timeout: Pin<Box<Sleep>>,
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = std::result::Result<F::Output, WritemagicError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Poll the main future first
        if let Poll::Ready(result) = self.future.as_mut().poll(cx) {
            return Poll::Ready(Ok(result));
        }

        // Check timeout
        if let Poll::Ready(()) = self.timeout.as_mut().poll(cx) {
            return Poll::Ready(Err(WritemagicError::timeout(30000)));
        }

        Poll::Pending
    }
}

/// Add timeout to any future
pub fn with_timeout<F: Future>(future: F, duration: Duration) -> TimeoutFuture<F> {
    TimeoutFuture {
        future: Box::pin(future),
        timeout: Box::pin(tokio::time::sleep(duration)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = with_retry(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 2 {
                        Err("failure")
                    } else {
                        Ok("success")
                    }
                }
            },
            RetryConfig::default(),
        ).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = with_retry(
            move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                async move { Err("always fails") }
            },
            RetryConfig {
                max_attempts: 2,
                ..Default::default()
            },
        ).await;

        assert_eq!(result, Err("always fails"));
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_circuit_breaker_states() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        });

        // Initially closed
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert!(cb.can_execute());

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Closed);
        
        cb.record_failure();
        assert!(matches!(cb.state(), CircuitState::Open { .. }));
        assert!(!cb.can_execute());

        // Success should reset when half-open
        cb.state = CircuitState::HalfOpen;
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), &CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_timeout_future() {
        // Fast operation should succeed
        let result = with_timeout(
            async { tokio::time::sleep(Duration::from_millis(10)).await; "done" },
            Duration::from_millis(100)
        ).await;
        assert!(result.is_ok());

        // Slow operation should timeout
        let result = with_timeout(
            async { tokio::time::sleep(Duration::from_millis(100)).await; "done" },
            Duration::from_millis(10)
        ).await;
        assert!(result.is_err());
    }
}