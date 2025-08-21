//! AI domain services

use writemagic_shared::{Result, WritemagicError};
use crate::providers::{AIProvider, CompletionRequest, CompletionResponse, Message, ClaudeProvider, OpenAIProvider, ResponseCache};
use std::sync::Arc;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub consecutive_failures: u32,
    pub avg_response_time: Duration,
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderHealth {
    pub fn new() -> Self {
        Self {
            is_healthy: true,
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            avg_response_time: Duration::from_millis(1000),
        }
    }

    pub fn record_success(&mut self, response_time: Duration) {
        self.is_healthy = true;
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;
        
        // Update average response time with exponential smoothing
        let alpha = 0.3;
        self.avg_response_time = Duration::from_millis(
            (alpha * response_time.as_millis() as f64 + 
             (1.0 - alpha) * self.avg_response_time.as_millis() as f64) as u64
        );
    }

    pub fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        self.consecutive_failures += 1;
        
        // Mark as unhealthy after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
        }
    }

    pub fn should_retry(&self) -> bool {
        if self.is_healthy {
            return true;
        }
        
        // Allow retry after 5 minutes for unhealthy providers
        if let Some(last_failure) = self.last_failure {
            last_failure.elapsed() > Duration::from_secs(300)
        } else {
            true
        }
    }
}

/// Provider candidate for optimal selection
#[derive(Debug, Clone)]
struct ProviderCandidate {
    name: String,
    health: ProviderHealth,
    circuit_state: crate::circuit_breaker::CircuitState,
    estimated_cost: f64,
    #[allow(dead_code)] // Used in future provider comparison logic
    capabilities: crate::providers::ModelCapabilities,
}

/// Advanced AI orchestration service with circuit breakers and security
pub struct AIOrchestrationService {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    fallback_order: Vec<String>,
    provider_health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    global_cache: Arc<ResponseCache>,
    circuit_breakers: Arc<crate::circuit_breaker::CircuitBreakerRegistry>,
    #[allow(dead_code)] // Used for API key rotation and security auditing
    key_manager: Arc<crate::security::SecureKeyManager>,
    content_sanitizer: Arc<crate::security::ContentSanitizationService>,
    tokenization_service: Arc<crate::tokenization::TokenizationService>,
    context_manager: Arc<ContextManagementService>,
    security_logger: Arc<crate::security::SecurityAuditLogger>,
    performance_monitor: Arc<crate::performance_monitor::PerformanceMonitor>,
    performance_alerting: Arc<crate::performance_monitor::PerformanceAlerting>,
    request_scheduler: Arc<RwLock<crate::request_batcher::RequestScheduler>>,
}

impl AIOrchestrationService {
    /// Create new orchestration service with all security and performance features
    pub fn new() -> Result<Self> {
        let key_manager = Arc::new(crate::security::SecureKeyManager::new());
        let content_sanitizer = Arc::new(crate::security::ContentSanitizationService::new(key_manager.clone())?);
        let tokenization_service = Arc::new(crate::tokenization::TokenizationService::new()?);
        let context_manager = Arc::new(ContextManagementService::with_tokenization_service(
            100000, // 100k token default limit
            tokenization_service.clone()
        ));

        let performance_monitor = Arc::new(crate::performance_monitor::PerformanceMonitor::new(50000));
        let performance_alerting = Arc::new(crate::performance_monitor::PerformanceAlerting::new(
            crate::performance_monitor::PerformanceThresholds::default(),
            1000
        ));

        Ok(Self {
            providers: HashMap::new(),
            fallback_order: Vec::new(),
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            global_cache: Arc::new(ResponseCache::new(600)), // 10 minute global cache
            circuit_breakers: Arc::new(crate::circuit_breaker::CircuitBreakerRegistry::new()),
            key_manager,
            content_sanitizer,
            tokenization_service,
            context_manager,
            security_logger: Arc::new(crate::security::SecurityAuditLogger::new(1000)),
            performance_monitor,
            performance_alerting,
            request_scheduler: Arc::new(RwLock::new(crate::request_batcher::RequestScheduler::new())),
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        cache_ttl_seconds: u64,
        max_context_tokens: u32,
        key_manager: Arc<crate::security::SecureKeyManager>,
    ) -> Result<Self> {
        let content_sanitizer = Arc::new(crate::security::ContentSanitizationService::new(key_manager.clone())?);
        let tokenization_service = Arc::new(crate::tokenization::TokenizationService::new()?);
        let context_manager = Arc::new(ContextManagementService::with_tokenization_service(
            max_context_tokens,
            tokenization_service.clone()
        ));

        let performance_monitor = Arc::new(crate::performance_monitor::PerformanceMonitor::new(50000));
        let performance_alerting = Arc::new(crate::performance_monitor::PerformanceAlerting::new(
            crate::performance_monitor::PerformanceThresholds::default(),
            1000
        ));

        Ok(Self {
            providers: HashMap::new(),
            fallback_order: Vec::new(),
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            global_cache: Arc::new(ResponseCache::new(cache_ttl_seconds)),
            circuit_breakers: Arc::new(crate::circuit_breaker::CircuitBreakerRegistry::new()),
            key_manager,
            content_sanitizer,
            tokenization_service,
            context_manager,
            security_logger: Arc::new(crate::security::SecurityAuditLogger::new(1000)),
            performance_monitor,
            performance_alerting,
            request_scheduler: Arc::new(RwLock::new(crate::request_batcher::RequestScheduler::new())),
        })
    }

    pub async fn add_provider(&mut self, provider: Arc<dyn AIProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name.clone(), provider);
        self.fallback_order.push(name.clone());
        
        // Initialize health tracking
        let mut health_map = self.provider_health.write().await;
        health_map.insert(name, ProviderHealth::new());
    }

    pub fn set_fallback_order(&mut self, order: Vec<String>) {
        self.fallback_order = order;
    }

    /// Get the best available provider based on health and performance
    pub async fn get_best_provider(&self) -> Option<String> {
        let health_map = self.provider_health.read().await;
        
        let mut available_providers: Vec<(String, &ProviderHealth)> = self.fallback_order
            .iter()
            .filter_map(|name| {
                health_map.get(name).and_then(|health| {
                    if health.should_retry() {
                        Some((name.clone(), health))
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Sort by health (healthy first) then by average response time
        available_providers.sort_by(|a, b| {
            match (a.1.is_healthy, b.1.is_healthy) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.1.avg_response_time.cmp(&b.1.avg_response_time),
            }
        });

        available_providers.first().map(|(name, _)| name.clone())
    }

    /// Complete with comprehensive security, tokenization, and circuit breaker protection
    pub async fn complete_with_fallback(&self, mut request: CompletionRequest) -> Result<CompletionResponse> {
        let request_id = Uuid::new_v4().to_string();
        let request_priority = request.priority.clone();
        
        // Start performance tracking
        let mut perf_metric = self.performance_monitor.start_request(
            "orchestration".to_string(),
            request.model.clone(),
            request_id.clone(),
            request_priority.clone(),
        );

        // Security: Sanitize request first
        request = self.content_sanitizer.sanitize_request(&request).map_err(|e| {
            self.security_logger.log_event(
                crate::security::SecurityEventType::SecurityViolation,
                format!("Request sanitization failed: {}", e),
                crate::security::PIISeverity::High,
            );
            self.performance_monitor.fail_request(perf_metric.clone(), "security_violation".to_string());
            e
        })?;

        // Tokenization: Validate request fits within model constraints
        self.tokenization_service.validate_request(&request).map_err(|e| {
            self.performance_monitor.fail_request(perf_metric.clone(), "tokenization_validation".to_string());
            e
        })?;

        // Context Management: Apply context optimization
        let optimized_messages = self.context_manager
            .manage_context(request.messages.clone(), &request.model)
            .map_err(|e| {
                self.performance_monitor.fail_request(perf_metric.clone(), "context_management".to_string());
                e
            })?;
        request.messages = optimized_messages;

        // Generate secure cache key
        let cache_key = self.generate_secure_cache_key(&request);
        
        // Check cache first
        if let Some(cached_response) = self.global_cache.get(&cache_key) {
            log::debug!("Global cache hit for model: {}", request.model);
            self.performance_monitor.record_cache_hit(perf_metric);
            return Ok(cached_response);
        }

        let mut last_error = None;
        let mut providers_tried = Vec::new();
        let request_start = Instant::now();

        // Get providers with circuit breaker and cost consideration
        let ordered_providers = self.get_optimal_providers_for_request(&request).await;
        
        for provider_name in ordered_providers {
            if let Some(provider) = self.providers.get(&provider_name) {
                // Circuit breaker check
                let circuit_breaker = self.circuit_breakers
                    .get(&provider_name)
                    .unwrap_or_else(|| {
                        // Create circuit breaker for provider if not exists
                        let config = self.get_circuit_breaker_config(&provider_name);
                        self.circuit_breakers.register(provider_name.clone(), config)
                    });

                if !circuit_breaker.can_execute().await {
                    log::debug!("Circuit breaker open for provider: {}", provider_name);
                    providers_tried.push(format!("{} (circuit-breaker-open)", provider_name));
                    continue;
                }

                let provider_start = Instant::now();
                
                // Execute with circuit breaker protection
                let result = circuit_breaker.execute(|| {
                    let req = request.clone();
                    let prov = provider.clone();
                    async move { prov.complete(&req).await }
                }).await;

                match result {
                    Ok(mut response) => {
                        let duration = provider_start.elapsed();
                        
                        // Security: Sanitize response
                        response = self.content_sanitizer.sanitize_response(&response)?;
                        
                        // Calculate accurate usage and cost
                        let usage = self.tokenization_service.calculate_usage(
                            &request,
                            response.choices.first().map(|c| &c.message.content).unwrap_or(&String::new()),
                            provider.capabilities().input_cost_per_token,
                            provider.capabilities().output_cost_per_token,
                        )?;

                        // Update response with accurate usage
                        response.usage.prompt_tokens = usage.input_tokens;
                        response.usage.completion_tokens = usage.output_tokens;
                        response.usage.total_tokens = usage.total_tokens;

                        // Record success
                        self.record_provider_success(&provider_name, duration).await;
                        
                        // Update performance metrics
                        perf_metric.input_tokens = usage.input_tokens;
                        perf_metric.output_tokens = usage.output_tokens;
                        perf_metric.total_tokens = usage.total_tokens;
                        perf_metric.cost = usage.estimated_cost;
                        
                        self.performance_monitor.complete_request(perf_metric);
                        
                        // Check performance thresholds and generate alerts if needed
                        if let Some(provider_stats) = self.performance_monitor.get_provider_stats(&provider_name) {
                            self.performance_alerting.check_thresholds(&provider_name, &request.model, &provider_stats);
                        }
                        
                        // Cache with content-sensitive TTL
                        let cache_ttl = self.calculate_cache_ttl(&response);
                        self.global_cache.insert(cache_key, response.clone(), cache_ttl);
                        
                        // Log performance metrics
                        tracing::info!(
                            provider = provider_name,
                            duration_ms = duration.as_millis(),
                            input_tokens = usage.input_tokens,
                            output_tokens = usage.output_tokens,
                            estimated_cost = usage.estimated_cost,
                            "AI request completed successfully"
                        );
                        
                        return Ok(response);
                    }
                    Err(e) => {
                        let duration = provider_start.elapsed();
                        
                        // Record failure - circuit breaker already recorded it
                        self.record_provider_failure(&provider_name).await;
                        
                        providers_tried.push(provider_name.clone());
                        
                        // Log sanitized error (no sensitive data)
                        let sanitized_error = self.content_sanitizer.sanitize_for_logging(&e.to_string());
                        tracing::warn!(
                            provider = provider_name,
                            duration_ms = duration.as_millis(),
                            error = sanitized_error,
                            "Provider request failed"
                        );
                        
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        // All providers failed - record performance failure and log security event
        self.performance_monitor.fail_request(perf_metric.clone(), "all_providers_failed".to_string());
        
        let total_duration = request_start.elapsed();
        let sanitized_error = last_error.as_ref()
            .map(|e| self.content_sanitizer.sanitize_for_logging(&e.to_string()))
            .unwrap_or_else(|| "No providers available".to_string());

        self.security_logger.log_event(
            crate::security::SecurityEventType::SuspiciousActivity,
            format!("All AI providers failed after {} attempts in {:?}", providers_tried.len(), total_duration),
            crate::security::PIISeverity::Medium,
        );

        let error_msg = format!(
            "All providers failed. Tried: {} providers in {:?}. Error: {}",
            providers_tried.len(),
            total_duration,
            sanitized_error
        );
        
        Err(WritemagicError::ai_provider(error_msg))
    }

    /// Generate secure cache key using BLAKE3 hash
    fn generate_secure_cache_key(&self, request: &CompletionRequest) -> String {
        
        let mut key_data = Vec::new();
        
        // Hash model and parameters
        key_data.extend(request.model.as_bytes());
        key_data.extend(&request.max_tokens.unwrap_or(0).to_le_bytes());
        key_data.extend(&request.temperature.unwrap_or(0.0).to_le_bytes());
        
        // Hash messages content (not including metadata which might contain sensitive data)
        for message in &request.messages {
            key_data.push(match message.role {
                crate::providers::MessageRole::System => 0,
                crate::providers::MessageRole::User => 1,
                crate::providers::MessageRole::Assistant => 2,
                crate::providers::MessageRole::Function => 3,
            });
            key_data.extend(message.content.as_bytes());
        }
        
        // Use BLAKE3 for secure, fast hashing
        blake3::hash(&key_data).to_hex().to_string()
    }

    /// Calculate content-sensitive cache TTL
    fn calculate_cache_ttl(&self, response: &CompletionResponse) -> Option<Duration> {
        // Check if response might contain sensitive or time-sensitive content
        let default_content = String::new();
        let content = response.choices.first()
            .map(|c| &c.message.content)
            .unwrap_or(&default_content);
        
        // Shorter TTL for potentially sensitive content
        let contains_sensitive = self.content_sanitizer
            .contains_sensitive_content(content);

        if contains_sensitive {
            Some(Duration::from_secs(60)) // 1 minute for sensitive content
        } else {
            Some(Duration::from_secs(600)) // 10 minutes for regular content
        }
    }

    /// Get circuit breaker configuration for provider
    fn get_circuit_breaker_config(&self, provider_name: &str) -> crate::circuit_breaker::CircuitBreakerConfig {
        match provider_name {
            "claude" => crate::circuit_breaker::CircuitBreakerConfig::conservative(),
            "openai" => crate::circuit_breaker::CircuitBreakerConfig::default(),
            _ => crate::circuit_breaker::CircuitBreakerConfig::aggressive(),
        }
    }

    #[allow(dead_code)] // Used by load balancing logic in full implementation
    async fn get_ordered_providers_for_request(&self, _request: &CompletionRequest) -> Vec<String> {
        let health_map = self.provider_health.read().await;
        
        let mut available_providers: Vec<(String, &ProviderHealth)> = self.fallback_order
            .iter()
            .filter_map(|name| {
                health_map.get(name).and_then(|health| {
                    if health.should_retry() {
                        Some((name.clone(), health))
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Sort by health status first, then by average response time
        available_providers.sort_by(|a, b| {
            match (a.1.is_healthy, b.1.is_healthy) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.1.avg_response_time.cmp(&b.1.avg_response_time),
            }
        });

        available_providers.into_iter().map(|(name, _)| name).collect()
    }

    /// Get optimal providers considering circuit breaker state, cost, and performance
    async fn get_optimal_providers_for_request(&self, request: &CompletionRequest) -> Vec<String> {
        let health_map = self.provider_health.read().await;
        
        let mut available_providers = Vec::new();
        
        for provider_name in &self.fallback_order {
            if let Some(health) = health_map.get(provider_name) {
                if !health.should_retry() {
                    continue;
                }
                
                // Check circuit breaker state
                let circuit_state = self.circuit_breakers
                    .get(provider_name)
                    .map(|cb| cb.state())
                    .unwrap_or(crate::circuit_breaker::CircuitState::Closed);
                
                let is_available = match circuit_state {
                    crate::circuit_breaker::CircuitState::Closed => true,
                    crate::circuit_breaker::CircuitState::HalfOpen { .. } => true,
                    crate::circuit_breaker::CircuitState::Open { .. } => false,
                };
                
                if is_available {
                    // Get provider for cost calculation
                    if let Some(provider) = self.providers.get(provider_name) {
                        let capabilities = provider.capabilities();
                        
                        // Estimate cost for this request
                        let estimated_input_tokens = self.tokenization_service
                            .count_request_tokens(request)
                            .unwrap_or(1000); // Fallback estimate
                        
                        let estimated_output_tokens = request.max_tokens.unwrap_or(capabilities.max_tokens.min(1000));
                        let estimated_cost = (estimated_input_tokens as f64 * capabilities.input_cost_per_token) +
                            (estimated_output_tokens as f64 * capabilities.output_cost_per_token);
                        
                        available_providers.push(ProviderCandidate {
                            name: provider_name.clone(),
                            health: health.clone(),
                            circuit_state,
                            estimated_cost,
                            capabilities: capabilities.clone(),
                        });
                    }
                }
            }
        }
        
        // Sort providers by multiple criteria
        available_providers.sort_by(|a, b| {
            use std::cmp::Ordering;
            
            // 1. Prioritize healthy providers
            match (a.health.is_healthy, b.health.is_healthy) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => {}
            }
            
            // 2. Prioritize closed circuits over half-open
            match (&a.circuit_state, &b.circuit_state) {
                (crate::circuit_breaker::CircuitState::Closed, crate::circuit_breaker::CircuitState::HalfOpen { .. }) => return Ordering::Less,
                (crate::circuit_breaker::CircuitState::HalfOpen { .. }, crate::circuit_breaker::CircuitState::Closed) => return Ordering::Greater,
                _ => {}
            }
            
            // 3. Consider cost (prefer lower cost, but not at expense of performance)
            let cost_diff = a.estimated_cost - b.estimated_cost;
            if cost_diff.abs() > 0.001 { // Significant cost difference
                // If one is significantly cheaper (>20% difference), prefer it
                if cost_diff > a.estimated_cost * 0.2 {
                    return Ordering::Greater;
                } else if cost_diff < -a.estimated_cost * 0.2 {
                    return Ordering::Less;
                }
            }
            
            // 4. Performance (response time)
            a.health.avg_response_time.cmp(&b.health.avg_response_time)
        });
        
        let result = available_providers.into_iter().map(|p| p.name).collect();
        
        tracing::debug!(
            providers = ?result,
            model = request.model,
            "Selected provider order for request"
        );
        
        result
    }

    async fn record_provider_success(&self, provider_name: &str, response_time: Duration) {
        let mut health_map = self.provider_health.write().await;
        if let Some(health) = health_map.get_mut(provider_name) {
            health.record_success(response_time);
        }
    }

    async fn record_provider_failure(&self, provider_name: &str) {
        let mut health_map = self.provider_health.write().await;
        if let Some(health) = health_map.get_mut(provider_name) {
            health.record_failure();
        }
    }

    /// Get health status of all providers
    pub async fn get_provider_health(&self) -> HashMap<String, ProviderHealth> {
        self.provider_health.read().await.clone()
    }

    /// Force health check on all providers
    pub async fn health_check_all_providers(&self) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();
        
        for provider_name in &self.fallback_order {
            if let Some(provider) = self.providers.get(provider_name) {
                let is_healthy = provider.validate_credentials().await.unwrap_or(false);
                results.insert(provider_name.clone(), is_healthy);
                
                if is_healthy {
                    self.record_provider_success(provider_name, Duration::from_millis(100)).await;
                } else {
                    self.record_provider_failure(provider_name).await;
                }
            }
        }
        
        Ok(results)
    }

    /// Get comprehensive service health status
    pub async fn get_comprehensive_health(&self) -> ServiceHealthReport {
        let provider_health = self.get_provider_health().await;
        let circuit_states = self.circuit_breakers.get_all_states();
        let circuit_metrics = self.circuit_breakers.get_all_metrics();
        
        ServiceHealthReport {
            provider_health,
            circuit_states,
            circuit_metrics,
            security_events: self.security_logger.get_recent_events(10),
            tokenization_models: self.tokenization_service.available_models(),
        }
    }

    /// Get cost estimates for request with different providers
    pub async fn estimate_costs(&self, request: &CompletionRequest) -> Result<HashMap<String, CostEstimate>> {
        let mut estimates = HashMap::new();
        
        let input_tokens = self.tokenization_service.count_request_tokens(request)?;
        let output_tokens = request.max_tokens.unwrap_or(1000);
        
        for (provider_name, provider) in &self.providers {
            let capabilities = provider.capabilities();
            let input_cost = input_tokens as f64 * capabilities.input_cost_per_token;
            let output_cost = output_tokens as f64 * capabilities.output_cost_per_token;
            let total_cost = input_cost + output_cost;
            
            estimates.insert(provider_name.clone(), CostEstimate {
                input_tokens,
                output_tokens,
                input_cost,
                output_cost,
                total_cost,
                provider_available: self.circuit_breakers
                    .get(provider_name)
                    .map(|cb| matches!(cb.state(), crate::circuit_breaker::CircuitState::Closed))
                    .unwrap_or(true),
            });
        }
        
        Ok(estimates)
    }

    /// Force circuit breaker states for testing or emergency
    pub fn emergency_circuit_control(&self, action: EmergencyAction) {
        match action {
            EmergencyAction::OpenAll => {
                self.circuit_breakers.force_open_all();
                self.security_logger.log_event(
                    crate::security::SecurityEventType::SuspiciousActivity,
                    "Emergency: All circuit breakers forced open".to_string(),
                    crate::security::PIISeverity::High,
                );
            }
            EmergencyAction::CloseAll => {
                self.circuit_breakers.force_close_all();
                self.security_logger.log_event(
                    crate::security::SecurityEventType::KeyRotated,
                    "Emergency: All circuit breakers forced closed".to_string(),
                    crate::security::PIISeverity::Medium,
                );
            }
            EmergencyAction::ResetAll => {
                self.circuit_breakers.reset_all();
                self.context_manager.clear_cache();
                self.global_cache.clear_expired();
            }
        }
    }

    /// Get tokenization service for external use
    pub fn tokenization_service(&self) -> &crate::tokenization::TokenizationService {
        &self.tokenization_service
    }

    /// Get security audit logger
    pub fn security_logger(&self) -> &crate::security::SecurityAuditLogger {
        &self.security_logger
    }

    /// Get context manager
    pub fn context_manager(&self) -> &ContextManagementService {
        &self.context_manager
    }

    /// Get comprehensive performance statistics
    pub async fn get_performance_stats(&self) -> crate::performance_monitor::PerformanceStats {
        self.performance_monitor.get_overall_stats()
    }

    /// Get performance statistics for a specific provider
    pub async fn get_provider_performance(&self, provider_name: &str) -> Option<crate::performance_monitor::PerformanceStats> {
        self.performance_monitor.get_provider_stats(provider_name)
    }

    /// Get recent performance alerts
    pub async fn get_performance_alerts(&self, limit: usize) -> Vec<crate::performance_monitor::PerformanceAlert> {
        self.performance_alerting.get_recent_alerts(limit)
    }

    /// Get performance trends over specified hours
    pub async fn get_performance_trends(&self, hours: u64) -> HashMap<String, Vec<f64>> {
        self.performance_monitor.get_performance_trends(hours)
    }

    /// Get request batcher statistics
    pub async fn get_batcher_stats(&self) -> HashMap<String, crate::request_batcher::BatcherStats> {
        self.request_scheduler.read().await.get_all_stats().await
    }

    /// Stream a completion request (returns async stream of partial responses)
    pub async fn stream_completion(&self, request: CompletionRequest) -> Result<Box<dyn crate::providers::StreamingResponse>> {
        // Use best available provider for streaming
        let providers = self.get_optimal_providers_for_request(&request).await;
        let provider_name = providers.first().cloned()
            .ok_or_else(|| WritemagicError::internal("No providers available for streaming"))?;
        
        if let Some(provider) = self.providers.get(&provider_name) {
            if !provider.supports_streaming() {
                return Err(WritemagicError::validation("Selected provider does not support streaming"));
            }
            
            // For now, just call the provider directly - circuit breaker implementation needed
            provider.stream(&request).await
        } else {
            Err(WritemagicError::internal(format!("Provider '{}' not found", provider_name)))
        }
    }

    /// Batch multiple completion requests for efficient processing
    pub async fn batch_complete(&self, requests: Vec<CompletionRequest>) -> Result<Vec<Result<CompletionResponse>>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // Group requests by preferred provider or model compatibility
        let mut provider_batches: HashMap<String, Vec<CompletionRequest>> = HashMap::new();
        
        for request in requests {
            let providers = self.get_optimal_providers_for_request(&request).await;
            let provider_name = providers.first().cloned()
                .unwrap_or_else(|| "claude".to_string()); // Fallback to Claude
            
            provider_batches.entry(provider_name).or_default().push(request);
        }

        // Process batches concurrently
        let mut handles = Vec::new();
        
        for (provider_name, batch_requests) in provider_batches {
            if let Some(provider) = self.providers.get(&provider_name).cloned() {
                let _circuit_breaker = self.circuit_breakers.get(&provider_name).map(|cb| cb.clone());
                
                let handle = tokio::spawn(async move {
                    // For now, just call the provider directly - circuit breaker implementation needed
                    provider.batch_complete(batch_requests).await
                });
                
                handles.push(handle);
            }
        }

        // Collect results
        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(batch_results) => {
                    match batch_results {
                        Ok(results) => all_results.extend(results),
                        Err(e) => {
                            // If entire batch failed, create error for each request
                            // We'd need to know how many requests were in this batch
                            all_results.push(Err(e));
                        }
                    }
                }
                Err(join_error) => {
                    all_results.push(Err(WritemagicError::internal(format!("Batch task failed: {}", join_error))));
                }
            }
        }

        Ok(all_results)
    }

    /// Get performance monitor for direct access
    pub fn performance_monitor(&self) -> &crate::performance_monitor::PerformanceMonitor {
        &self.performance_monitor
    }

    /// Get performance alerting service
    pub fn performance_alerting(&self) -> &crate::performance_monitor::PerformanceAlerting {
        &self.performance_alerting
    }
}

/// Service health report
#[derive(Debug, Clone)]
pub struct ServiceHealthReport {
    pub provider_health: HashMap<String, ProviderHealth>,
    pub circuit_states: HashMap<String, crate::circuit_breaker::CircuitState>,
    pub circuit_metrics: HashMap<String, crate::circuit_breaker::CircuitMetrics>,
    pub security_events: Vec<crate::security::SecurityEvent>,
    pub tokenization_models: Vec<String>,
}

/// Cost estimate for a provider
#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub provider_available: bool,
}

/// Emergency circuit breaker actions
#[derive(Debug, Clone)]
pub enum EmergencyAction {
    OpenAll,
    CloseAll,
    ResetAll,
}

/// Provider registry and factory service with secure key management
pub struct AIProviderRegistry {
    key_manager: Arc<crate::security::SecureKeyManager>,
}

impl Default for AIProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AIProviderRegistry {
    pub fn new() -> Self {
        Self {
            key_manager: Arc::new(crate::security::SecureKeyManager::new()),
        }
    }

    pub fn with_key_manager(key_manager: Arc<crate::security::SecureKeyManager>) -> Self {
        Self { key_manager }
    }

    pub fn add_claude_key(&self, api_key: String) -> Result<()> {
        let secure_key = crate::security::SecureApiKey::new("claude".to_string(), api_key);
        self.key_manager.add_key("claude".to_string(), secure_key)
    }

    pub fn add_openai_key(&self, api_key: String) -> Result<()> {
        let secure_key = crate::security::SecureApiKey::new("openai".to_string(), api_key);
        self.key_manager.add_key("openai".to_string(), secure_key)
    }

    pub fn add_key_with_rotation(&self, provider: String, api_key: String, max_usage: u64) -> Result<()> {
        let secure_key = crate::security::SecureApiKey::with_usage_limit(provider.clone(), api_key, max_usage);
        self.key_manager.add_key(provider, secure_key)
    }

    pub async fn create_orchestration_service(&self) -> Result<AIOrchestrationService> {
        let mut service = AIOrchestrationService::with_config(
            600, // 10 minute cache
            100000, // 100k token context limit
            self.key_manager.clone()
        )?;

        let mut fallback_order = Vec::new();

        // Try to create Claude provider if key exists
        if let Ok(claude_key) = self.key_manager.get_key("claude") {
            match ClaudeProvider::new(claude_key.value().to_string()) {
                Ok(provider) => {
                    let claude_provider = Arc::new(provider);
                    service.add_provider(claude_provider).await;
                    fallback_order.push("claude".to_string());
                    
                    // Register circuit breaker
                    service.circuit_breakers.register(
                        "claude".to_string(),
                        crate::circuit_breaker::CircuitBreakerConfig::conservative(),
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to create Claude provider: {}", e);
                }
            }
        }

        // Try to create OpenAI provider if key exists
        if let Ok(openai_key) = self.key_manager.get_key("openai") {
            match OpenAIProvider::new(openai_key.value().to_string()) {
                Ok(provider) => {
                    let openai_provider = Arc::new(provider);
                    service.add_provider(openai_provider).await;
                    fallback_order.push("openai".to_string());
                    
                    // Register circuit breaker
                    service.circuit_breakers.register(
                        "openai".to_string(),
                        crate::circuit_breaker::CircuitBreakerConfig::default(),
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to create OpenAI provider: {}", e);
                }
            }
        }
        
        service.set_fallback_order(fallback_order);

        Ok(service)
    }

    pub fn check_key_rotation_needed(&self) -> Vec<String> {
        self.key_manager.check_rotation_needed()
    }

    pub fn create_claude_provider(&self) -> Result<ClaudeProvider> {
        let key = self.key_manager.get_key("claude")?;
        ClaudeProvider::new(key.value().to_string())
    }

    pub fn create_openai_provider(&self) -> Result<OpenAIProvider> {
        let key = self.key_manager.get_key("openai")?;
        OpenAIProvider::new(key.value().to_string())
    }

    /// Get the underlying key manager
    pub fn key_manager(&self) -> &crate::security::SecureKeyManager {
        &self.key_manager
    }
}

/// Type alias for context cache to reduce complexity
type ContextCache = Arc<std::sync::RwLock<HashMap<String, (Vec<Message>, std::time::Instant)>>>;

/// Context management service with accurate tokenization
#[derive(Clone)]
pub struct ContextManagementService {
    max_context_tokens: u32,
    tokenization_service: Arc<crate::tokenization::TokenizationService>,
    context_cache: ContextCache,
    cache_ttl: std::time::Duration,
}

impl ContextManagementService {
    pub fn new(max_context_tokens: u32) -> Result<Self> {
        let tokenization_service = Arc::new(crate::tokenization::TokenizationService::new()?);
        
        Ok(Self { 
            max_context_tokens,
            tokenization_service,
            context_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300),
        })
    }

    pub fn with_tokenization_service(max_context_tokens: u32, tokenization_service: Arc<crate::tokenization::TokenizationService>) -> Self {
        Self {
            max_context_tokens,
            tokenization_service,
            context_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300),
        }
    }

    /// Manage context with accurate token counting for specific model
    pub fn manage_context(&self, messages: Vec<Message>, model_name: &str) -> Result<Vec<Message>> {
        // Create cache key
        let cache_key = self.create_cache_key(&messages, model_name);
        
        // Check cache first
        {
            let cache = self.context_cache.read()
                .map_err(|e| WritemagicError::internal(format!("Failed to read context cache: {}", e)))?;
            if let Some((cached_messages, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(cached_messages.clone());
                }
            }
        }

        let tokenizer = self.tokenization_service.get_tokenizer(model_name);
        let mut managed_messages = Vec::new();
        let mut current_tokens = 0u32;

        // Always include system messages first
        let mut system_messages = Vec::new();
        let mut non_system_messages = Vec::new();

        for msg in messages {
            match msg.role {
                crate::providers::MessageRole::System => system_messages.push(msg),
                _ => non_system_messages.push(msg),
            }
        }

        // Add system messages and count their tokens
        for msg in system_messages {
            let msg_tokens = tokenizer.count_tokens(&msg.content)?;
            if current_tokens + msg_tokens <= self.max_context_tokens {
                current_tokens += msg_tokens;
                managed_messages.push(msg);
            } else {
                log::warn!("System message too long, truncating context");
                break;
            }
        }

        // Add non-system messages in reverse order (most recent first)
        for msg in non_system_messages.into_iter().rev() {
            let msg_tokens = tokenizer.count_tokens(&msg.content)?;
            
            // Add message formatting overhead
            let total_msg_tokens = msg_tokens + match msg.role {
                crate::providers::MessageRole::User => 4,
                crate::providers::MessageRole::Assistant => 4,
                crate::providers::MessageRole::Function => 6,
                crate::providers::MessageRole::System => 4, // Already handled above
            };
            
            if current_tokens + total_msg_tokens <= self.max_context_tokens {
                current_tokens += total_msg_tokens;
                managed_messages.push(msg);
            } else {
                log::debug!("Dropping message to fit context window. Current tokens: {}, Message tokens: {}, Max: {}", 
                    current_tokens, total_msg_tokens, self.max_context_tokens);
                break;
            }
        }

        // Reverse non-system messages back to chronological order
        let mut final_messages = Vec::new();
        let system_count = managed_messages.iter().filter(|m| matches!(m.role, crate::providers::MessageRole::System)).count();
        
        // Add system messages first
        for (i, msg) in managed_messages.iter().enumerate() {
            if matches!(msg.role, crate::providers::MessageRole::System) {
                final_messages.push(msg.clone());
            } else if i >= system_count {
                break;
            }
        }
        
        // Add non-system messages in reverse order to restore chronology
        let mut non_system: Vec<Message> = managed_messages
            .into_iter()
            .filter(|m| !matches!(m.role, crate::providers::MessageRole::System))
            .collect();
        non_system.reverse();
        final_messages.extend(non_system);

        // Cache result
        {
            let mut cache = self.context_cache.write()
                .map_err(|e| WritemagicError::internal(format!("Failed to write context cache: {}", e)))?;
            cache.insert(cache_key, (final_messages.clone(), std::time::Instant::now()));
            
            // Clean expired entries
            let now = std::time::Instant::now();
            cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.cache_ttl);
        }

        log::debug!("Context managed: {} messages, ~{} tokens for model {}", 
            final_messages.len(), current_tokens, model_name);

        Ok(final_messages)
    }

    /// Get optimal context window for a specific model
    pub fn get_optimal_context_size(&self, model_name: &str) -> u32 {
        let tokenizer = self.tokenization_service.get_tokenizer(model_name);
        let config = tokenizer.config();
        
        // Use 75% of context window for input, leaving 25% for output
        ((config.context_window as f64) * 0.75) as u32
    }

    /// Validate that messages fit within context window
    pub fn validate_context_fit(&self, messages: &[Message], model_name: &str) -> Result<()> {
        let tokenizer = self.tokenization_service.get_tokenizer(model_name);
        let mut total_tokens = 0u32;

        for msg in messages {
            total_tokens += tokenizer.count_tokens(&msg.content)?;
        }

        if total_tokens > self.max_context_tokens {
            return Err(WritemagicError::validation(format!(
                "Messages exceed context window: {} tokens (max: {})",
                total_tokens, self.max_context_tokens
            )));
        }

        Ok(())
    }

    /// Clear context cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.context_cache.write() {
            cache.clear();
        }
    }

    /// Get context statistics
    pub fn get_context_stats(&self, messages: &[Message], model_name: &str) -> Result<ContextStats> {
        let tokenizer = self.tokenization_service.get_tokenizer(model_name);
        let mut stats = ContextStats::default();

        for msg in messages {
            let tokens = tokenizer.count_tokens(&msg.content)?;
            stats.total_tokens += tokens;
            
            match msg.role {
                crate::providers::MessageRole::System => {
                    stats.system_tokens += tokens;
                    stats.system_messages += 1;
                }
                crate::providers::MessageRole::User => {
                    stats.user_tokens += tokens;
                    stats.user_messages += 1;
                }
                crate::providers::MessageRole::Assistant => {
                    stats.assistant_tokens += tokens;
                    stats.assistant_messages += 1;
                }
                crate::providers::MessageRole::Function => {
                    stats.function_tokens += tokens;
                    stats.function_messages += 1;
                }
            }
        }

        stats.total_messages = messages.len() as u32;
        stats.utilization = (stats.total_tokens as f64 / self.max_context_tokens as f64) * 100.0;

        Ok(stats)
    }

    fn create_cache_key(&self, messages: &[Message], model_name: &str) -> String {
        let mut hasher = DefaultHasher::new();
        model_name.hash(&mut hasher);
        
        for msg in messages {
            msg.role.hash(&mut hasher);
            msg.content.hash(&mut hasher);
        }
        
        format!("{:x}", hasher.finish())
    }
}

/// Context statistics
#[derive(Debug, Default, Clone)]
pub struct ContextStats {
    pub total_messages: u32,
    pub total_tokens: u32,
    pub system_messages: u32,
    pub system_tokens: u32,
    pub user_messages: u32,
    pub user_tokens: u32,
    pub assistant_messages: u32,
    pub assistant_tokens: u32,
    pub function_messages: u32,
    pub function_tokens: u32,
    pub utilization: f64, // Percentage of max context used
}

/// Content filtering service
pub struct ContentFilteringService {
    prohibited_patterns: Vec<regex::Regex>,
}

impl ContentFilteringService {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            r"(?i)(password|api[_-]?key|secret|token)\s*[:=]\s*[^\s]+",
            r"(?i)(credit[_-]?card|ssn|social[_-]?security)",
        ];

        let mut prohibited_patterns = Vec::new();
        for pattern in patterns {
            prohibited_patterns.push(regex::Regex::new(pattern)
                .map_err(|e| WritemagicError::internal(format!("Invalid regex: {}", e)))?);
        }

        Ok(Self { prohibited_patterns })
    }

    pub fn filter_content(&self, content: &str) -> Result<String> {
        for pattern in &self.prohibited_patterns {
            if pattern.is_match(content) {
                return Err(WritemagicError::validation("Content contains sensitive information"));
            }
        }
        Ok(content.to_string())
    }

    pub fn detect_sensitive_info(&self, content: &str) -> Vec<String> {
        let mut findings = Vec::new();
        for (i, pattern) in self.prohibited_patterns.iter().enumerate() {
            if pattern.is_match(content) {
                findings.push(format!("Pattern {} matched", i));
            }
        }
        findings
    }
}

impl Default for ContentFilteringService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            log::error!("Failed to create content filtering service, using minimal implementation");
            Self { prohibited_patterns: Vec::new() }
        })
    }
}