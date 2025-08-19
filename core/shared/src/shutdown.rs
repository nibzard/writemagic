//! Graceful shutdown patterns for async services

use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use std::time::Duration;
use tracing::{info, warn, error};

/// Graceful shutdown coordinator for async services
pub struct ShutdownCoordinator {
    /// Cancellation token to signal shutdown
    pub cancellation_token: CancellationToken,
    /// Channel for services to signal completion
    completion_tx: mpsc::Sender<ServiceShutdown>,
    /// Receiver for shutdown completions
    completion_rx: mpsc::Receiver<ServiceShutdown>,
    /// Broadcast channel for shutdown notifications
    shutdown_tx: broadcast::Sender<ShutdownSignal>,
}

/// Signal sent to all services during shutdown
#[derive(Debug, Clone)]
pub enum ShutdownSignal {
    /// Graceful shutdown requested
    Graceful,
    /// Immediate shutdown required (after timeout)
    Immediate,
}

/// Service completion notification
#[derive(Debug)]
pub struct ServiceShutdown {
    pub service_name: String,
    pub success: bool,
    pub duration: Duration,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (completion_tx, completion_rx) = mpsc::channel(32);
        let (shutdown_tx, _) = broadcast::channel(16);
        
        Self {
            cancellation_token: CancellationToken::new(),
            completion_tx,
            completion_rx,
            shutdown_tx,
        }
    }
    
    /// Get a shutdown subscriber for a service
    pub fn subscriber(&self) -> ShutdownSubscriber {
        ShutdownSubscriber {
            cancellation_token: self.cancellation_token.clone(),
            completion_tx: self.completion_tx.clone(),
            shutdown_rx: self.shutdown_tx.subscribe(),
        }
    }
    
    /// Initiate graceful shutdown
    pub async fn shutdown(&mut self, timeout: Duration) -> bool {
        info!("Initiating graceful shutdown with timeout {:?}", timeout);
        
        // Send graceful shutdown signal
        if let Err(e) = self.shutdown_tx.send(ShutdownSignal::Graceful) {
            warn!("Failed to send graceful shutdown signal: {}", e);
        }
        
        // Cancel all operations
        self.cancellation_token.cancel();
        
        let start = std::time::Instant::now();
        let mut services_remaining = 0;
        
        // Wait for services to complete or timeout
        while start.elapsed() < timeout {
            tokio::select! {
                Some(completion) = self.completion_rx.recv() => {
                    if completion.success {
                        info!("Service '{}' shut down successfully in {:?}", 
                              completion.service_name, completion.duration);
                    } else {
                        warn!("Service '{}' failed to shut down gracefully in {:?}", 
                              completion.service_name, completion.duration);
                    }
                    services_remaining = services_remaining.saturating_sub(1);
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Continue waiting
                }
                else => break,
            }
        }
        
        if start.elapsed() >= timeout && services_remaining > 0 {
            warn!("Shutdown timeout reached, forcing immediate shutdown");
            if let Err(e) = self.shutdown_tx.send(ShutdownSignal::Immediate) {
                error!("Failed to send immediate shutdown signal: {}", e);
            }
            return false;
        }
        
        info!("Graceful shutdown completed successfully");
        true
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Subscriber for shutdown signals that services can use
pub struct ShutdownSubscriber {
    cancellation_token: CancellationToken,
    completion_tx: mpsc::Sender<ServiceShutdown>,
    shutdown_rx: broadcast::Receiver<ShutdownSignal>,
}

impl ShutdownSubscriber {
    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&mut self) -> ShutdownSignal {
        tokio::select! {
            _ = self.cancellation_token.cancelled() => ShutdownSignal::Graceful,
            signal = self.shutdown_rx.recv() => {
                signal.unwrap_or(ShutdownSignal::Immediate)
            }
        }
    }
    
    /// Check if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
    
    /// Report service shutdown completion
    pub async fn report_shutdown(&self, service_name: String, success: bool, duration: Duration) {
        let completion = ServiceShutdown {
            service_name,
            success,
            duration,
        };
        
        if let Err(e) = self.completion_tx.send(completion).await {
            error!("Failed to report service shutdown: {}", e);
        }
    }
    
    /// Get cancellation token for integration with other async operations
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

/// Helper trait for services to implement graceful shutdown
#[async_trait::async_trait]
pub trait GracefulShutdown {
    /// Service name for logging
    fn service_name(&self) -> &str;
    
    /// Perform graceful shutdown
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Run the service with graceful shutdown support
    async fn run_with_shutdown(mut self, mut subscriber: ShutdownSubscriber) 
    where 
        Self: Sized + Send,
    {
        let service_name = self.service_name().to_string();
        let start_time = std::time::Instant::now();
        
        info!("Starting service: {}", service_name);
        
        // Wait for shutdown signal
        let shutdown_signal = subscriber.wait_for_shutdown().await;
        info!("Service '{}' received shutdown signal: {:?}", service_name, shutdown_signal);
        
        // Perform shutdown
        let shutdown_start = std::time::Instant::now();
        let success = match self.shutdown().await {
            Ok(()) => {
                info!("Service '{}' shut down successfully", service_name);
                true
            }
            Err(e) => {
                error!("Service '{}' failed to shut down: {}", service_name, e);
                false
            }
        };
        
        let shutdown_duration = shutdown_start.elapsed();
        
        // Report completion
        subscriber.report_shutdown(service_name, success, shutdown_duration).await;
    }
}

/// Utility macro for creating services that support graceful shutdown
#[macro_export]
macro_rules! shutdown_service {
    ($service:expr, $coordinator:expr) => {{
        let subscriber = $coordinator.subscriber();
        tokio::spawn($service.run_with_shutdown(subscriber))
    }};
}