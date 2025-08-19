//! AI domain services

use async_trait::async_trait;
use writemagic_shared::{EntityId, DomainService, Result, WritemagicError};
use crate::providers::{AIProvider, CompletionRequest, CompletionResponse, Message, ClaudeProvider, OpenAIProvider, ResponseCache};
use crate::value_objects::{Prompt, ModelConfiguration};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub consecutive_failures: u32,
    pub avg_response_time: Duration,
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

/// AI orchestration service with intelligent fallback
pub struct AIOrchestrationService {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    fallback_order: Vec<String>,
    provider_health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    global_cache: Arc<ResponseCache>,
}

impl AIOrchestrationService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            fallback_order: Vec::new(),
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            global_cache: Arc::new(ResponseCache::new(600)), // 10 minute global cache
        }
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

    pub async fn complete_with_fallback(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Try global cache first
        let cache_key = ResponseCache::generate_cache_key(&request);
        if let Some(cached_response) = self.global_cache.get(&cache_key) {
            log::debug!("Global cache hit");
            return Ok(cached_response);
        }

        let mut last_error = None;
        let mut providers_tried = Vec::new();

        // Try providers in order of health and performance
        let ordered_providers = self.get_ordered_providers_for_request(&request).await;
        
        for provider_name in ordered_providers {
            if let Some(provider) = self.providers.get(&provider_name) {
                let start_time = Instant::now();
                
                match provider.complete(&request).await {
                    Ok(response) => {
                        let duration = start_time.elapsed();
                        
                        // Record success
                        self.record_provider_success(&provider_name, duration).await;
                        
                        // Cache globally
                        self.global_cache.insert(cache_key, response.clone(), None);
                        
                        log::info!("Request completed with provider {} in {:?}", provider_name, duration);
                        return Ok(response);
                    }
                    Err(e) => {
                        // Record failure
                        self.record_provider_failure(&provider_name).await;
                        
                        providers_tried.push(provider_name.clone());
                        log::warn!("Provider {} failed: {}", provider_name, e);
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        let error_msg = format!(
            "All providers failed. Tried: {}. Last error: {}",
            providers_tried.join(", "),
            last_error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "No providers available".to_string())
        );
        
        Err(WritemagicError::ai_provider(error_msg))
    }

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
}

/// Provider registry and factory service
pub struct AIProviderRegistry {
    claude_api_key: Option<String>,
    openai_api_key: Option<String>,
}

impl AIProviderRegistry {
    pub fn new() -> Self {
        Self {
            claude_api_key: None,
            openai_api_key: None,
        }
    }

    pub fn with_claude_key(mut self, api_key: String) -> Self {
        self.claude_api_key = Some(api_key);
        self
    }

    pub fn with_openai_key(mut self, api_key: String) -> Self {
        self.openai_api_key = Some(api_key);
        self
    }

    pub fn create_orchestration_service(&self) -> Result<AIOrchestrationService> {
        let mut service = AIOrchestrationService::new();

        if let Some(claude_key) = &self.claude_api_key {
            let claude_provider = Arc::new(ClaudeProvider::new(claude_key.clone()));
            futures::executor::block_on(service.add_provider(claude_provider));
        }

        if let Some(openai_key) = &self.openai_api_key {
            let openai_provider = Arc::new(OpenAIProvider::new(openai_key.clone()));
            futures::executor::block_on(service.add_provider(openai_provider));
        }

        // Set fallback order (Claude first, then OpenAI)
        let mut fallback_order = Vec::new();
        if self.claude_api_key.is_some() {
            fallback_order.push("claude".to_string());
        }
        if self.openai_api_key.is_some() {
            fallback_order.push("openai".to_string());
        }
        
        service.set_fallback_order(fallback_order);

        Ok(service)
    }

    pub fn create_claude_provider(&self) -> Result<ClaudeProvider> {
        match &self.claude_api_key {
            Some(key) => Ok(ClaudeProvider::new(key.clone())),
            None => Err(WritemagicError::configuration("Claude API key not configured")),
        }
    }

    pub fn create_openai_provider(&self) -> Result<OpenAIProvider> {
        match &self.openai_api_key {
            Some(key) => Ok(OpenAIProvider::new(key.clone())),
            None => Err(WritemagicError::configuration("OpenAI API key not configured")),
        }
    }
}

/// Context management service
pub struct ContextManagementService {
    max_context_length: usize,
}

impl ContextManagementService {
    pub fn new(max_context_length: usize) -> Self {
        Self { max_context_length }
    }

    pub fn manage_context(&self, messages: Vec<Message>) -> Vec<Message> {
        let total_length: usize = messages.iter().map(|m| m.content.len()).sum();
        
        if total_length <= self.max_context_length {
            return messages;
        }

        // Keep system message and recent messages
        let mut result = Vec::new();
        let mut current_length = 0;

        // Add system messages first
        for msg in &messages {
            if matches!(msg.role, crate::providers::MessageRole::System) {
                result.push(msg.clone());
                current_length += msg.content.len();
            }
        }

        // Add recent messages in reverse order
        for msg in messages.iter().rev() {
            if !matches!(msg.role, crate::providers::MessageRole::System) {
                if current_length + msg.content.len() <= self.max_context_length {
                    result.insert(result.len(), msg.clone());
                    current_length += msg.content.len();
                } else {
                    break;
                }
            }
        }

        result
    }
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
        Self::new().expect("Failed to create content filtering service")
    }
}