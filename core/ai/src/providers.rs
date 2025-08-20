//! AI provider abstractions and implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use writemagic_shared::{Result, WritemagicError};
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use dashmap::DashMap;

/// AI provider trait following the pattern from CLAUDE.md
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Complete a request with the AI provider
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;

    /// Stream a completion request (returns async stream of partial responses)
    async fn stream(&self, request: &CompletionRequest) -> Result<Box<dyn StreamingResponse>>;

    /// Batch multiple requests for efficient processing
    async fn batch_complete(&self, requests: Vec<CompletionRequest>) -> Result<Vec<Result<CompletionResponse>>>;

    /// Get provider capabilities
    fn capabilities(&self) -> ModelCapabilities;

    /// Validate API key or credentials
    async fn validate_credentials(&self) -> Result<bool>;

    /// Get usage statistics
    async fn get_usage_stats(&self) -> Result<UsageStats>;

    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool {
        self.capabilities().supports_streaming
    }

    /// Check if provider supports batching
    fn supports_batching(&self) -> bool {
        true // Most providers support batching at the API level
    }

    /// Get provider health metrics
    async fn health_check(&self) -> Result<ProviderHealthMetrics>;
}

/// Streaming response trait for real-time completions
#[async_trait]
pub trait StreamingResponse: Send + Sync {
    /// Get next streaming chunk
    async fn next_chunk(&mut self) -> Result<Option<StreamingChunk>>;
    
    /// Check if stream is complete
    fn is_complete(&self) -> bool;
    
    /// Get accumulated response so far
    fn get_partial_response(&self) -> String;
}

/// Streaming chunk from AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    pub content: String,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<Usage>,
}


/// Provider health metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthMetrics {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub success_rate: f64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub timestamp: std::time::SystemTime,
}

/// Completion request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
    pub metadata: HashMap<String, String>,
    /// Request priority for load balancing
    pub priority: RequestPriority,
    /// Request timeout override
    pub timeout: Option<Duration>,
    /// Enable response compression
    pub compress_response: bool,
    /// Request batching hint
    pub batchable: bool,
}

/// Request priority levels for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>, model: String) -> Self {
        Self {
            messages,
            model,
            max_tokens: None,
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
            metadata: HashMap::new(),
            priority: RequestPriority::Normal,
            timeout: None,
            compress_response: false,
            batchable: false,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress_response = compress;
        self
    }

    pub fn with_batching(mut self, batchable: bool) -> Self {
        self.batchable = batchable;
        self
    }

    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Completion response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub model: String,
    pub created: i64,
    pub metadata: HashMap<String, String>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Message role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Choice in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<FinishReason>,
}

/// Finish reason enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    FunctionCall,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub context_window: u32,
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub requests_today: u64,
    pub tokens_today: u64,
    pub cost_today: f64,
}

/// Thread-safe usage statistics with atomic operations
#[derive(Debug)]
pub struct AtomicUsageStats {
    pub total_requests: AtomicU64,
    pub total_tokens: AtomicU64,
    pub total_cost: RwLock<f64>,
    pub requests_today: AtomicU64,
    pub tokens_today: AtomicU64,
    pub cost_today: RwLock<f64>,
}

impl Default for AtomicUsageStats {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicUsageStats {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_cost: RwLock::new(0.0),
            requests_today: AtomicU64::new(0),
            tokens_today: AtomicU64::new(0),
            cost_today: RwLock::new(0.0),
        }
    }

    pub async fn increment_request(&self, tokens: u64, cost: f64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_tokens.fetch_add(tokens, Ordering::Relaxed);
        self.requests_today.fetch_add(1, Ordering::Relaxed);
        self.tokens_today.fetch_add(tokens, Ordering::Relaxed);

        // Update costs atomically
        {
            let mut total_cost = self.total_cost.write().await;
            *total_cost += cost;
        }
        {
            let mut cost_today = self.cost_today.write().await;
            *cost_today += cost;
        }
    }

    pub async fn to_usage_stats(&self) -> UsageStats {
        UsageStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
            total_cost: *self.total_cost.read().await,
            requests_today: self.requests_today.load(Ordering::Relaxed),
            tokens_today: self.tokens_today.load(Ordering::Relaxed),
            cost_today: *self.cost_today.read().await,
        }
    }
}

/// Claude AI provider implementation
#[derive(Clone)]
pub struct ClaudeProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ResponseCache>,
    usage_stats: Arc<AtomicUsageStats>,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| WritemagicError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            client,
            rate_limiter: Arc::new(RateLimiter::new(5, 200)), // 5 concurrent, 200ms min interval
            cache: Arc::new(ResponseCache::new(300)), // 5 minute cache
            usage_stats: Arc::new(AtomicUsageStats::new()),
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_rate_limit(mut self, max_concurrent: usize, min_interval_ms: u64) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(max_concurrent, min_interval_ms));
        self
    }

    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache = Arc::new(ResponseCache::new(ttl_seconds));
        self
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        // Check cache first
        let cache_key = ResponseCache::generate_cache_key(request);
        if let Some(cached_response) = self.cache.get(&cache_key) {
            log::debug!("Cache hit for Claude request");
            return Ok(cached_response);
        }

        // Rate limiting
        let _permit = self.rate_limiter.acquire().await?;

        let url = format!("{}/v1/messages", self.base_url);
        
        // Convert to Claude API format
        let claude_request = self.convert_to_claude_format(request)?;
        
        log::debug!("Making Claude API request to: {}", url);
        let start_time = Instant::now();
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .header("x-api-key", &self.api_key)
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Claude API network error: {}", e);
                WritemagicError::network(format!("Claude API request failed: {}", e))
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            log::error!("Claude API error (status {}): {}", status, response_text);
            
            // Handle specific error types
            match status.as_u16() {
                401 => return Err(WritemagicError::authentication("Invalid Claude API key")),
                429 => return Err(WritemagicError::ai_provider("Claude API rate limit exceeded")),
                500..=599 => return Err(WritemagicError::ai_provider("Claude API server error")),
                _ => return Err(WritemagicError::ai_provider(format!("Claude API error: {}", response_text))),
            }
        }

        let claude_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to parse Claude response: {}", e);
                WritemagicError::ai_provider(format!("Failed to parse Claude response: {}", e))
            })?;

        let completion_response = self.convert_from_claude_format(&claude_response)?;
        
        // Update usage stats
        let request_duration = start_time.elapsed();
        self.update_usage_stats(&completion_response, request_duration).await;

        // Cache the response
        self.cache.insert(cache_key, completion_response.clone(), None);

        log::debug!("Claude request completed in {:?}", request_duration);
        Ok(completion_response)
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            max_tokens: 100000,
            supports_streaming: true,
            supports_functions: false,
            supports_vision: true,
            context_window: 200000,
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00003,
        }
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // Simple validation by making a minimal request
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "claude-3-haiku-20240307".to_string(),
        ).with_max_tokens(1);

        match self.complete(&test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_usage_stats(&self) -> Result<UsageStats> {
        Ok(self.usage_stats.to_usage_stats().await)
    }

    async fn stream(&self, request: &CompletionRequest) -> Result<Box<dyn StreamingResponse>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let url = format!("{}/v1/messages", self.base_url);
        let mut claude_request = self.convert_to_claude_format(request)?;
        claude_request["stream"] = serde_json::Value::Bool(true);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Anthropic-Version", "2023-06-01")
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| WritemagicError::network(format!("Claude API request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(Box::new(ClaudeStreamingResponse::new(response)))
        } else {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(WritemagicError::ai_provider(format!("Claude streaming failed: {}", error_text)))
        }
    }

    async fn batch_complete(&self, requests: Vec<CompletionRequest>) -> Result<Vec<Result<CompletionResponse>>> {
        let mut results = Vec::new();
        for request in requests {
            let result = self.complete(&request).await;
            results.push(result);
        }
        Ok(results)
    }

    async fn health_check(&self) -> Result<ProviderHealthMetrics> {
        let start_time = Instant::now();
        
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "claude-3-haiku-20240307".to_string(),
        ).with_max_tokens(1);

        let result = self.complete(&test_request).await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ProviderHealthMetrics {
            is_healthy: result.is_ok(),
            response_time_ms,
            success_rate: if result.is_ok() { 1.0 } else { 0.0 },
            error_count: if result.is_ok() { 0 } else { 1 },
            last_error: result.err().map(|e| e.to_string()),
            timestamp: std::time::SystemTime::now(),
        })
    }
}

impl ClaudeProvider {
    async fn update_usage_stats(&self, response: &CompletionResponse, _duration: Duration) {
        // Calculate cost based on model capabilities
        let capabilities = self.capabilities();
        let input_cost = response.usage.prompt_tokens as f64 * capabilities.input_cost_per_token;
        let output_cost = response.usage.completion_tokens as f64 * capabilities.output_cost_per_token;
        let total_cost = input_cost + output_cost;
        
        // Atomically update all statistics
        self.usage_stats.increment_request(response.usage.total_tokens as u64, total_cost).await;
    }

    fn convert_to_claude_format(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        let mut claude_messages = Vec::new();
        let mut system_message = None;

        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    system_message = Some(msg.content.clone());
                }
                MessageRole::User | MessageRole::Assistant => {
                    claude_messages.push(serde_json::json!({
                        "role": msg.role,
                        "content": msg.content
                    }));
                }
                MessageRole::Function => {
                    // Skip function messages for Claude
                    continue;
                }
            }
        }

        let mut claude_request = serde_json::json!({
            "model": request.model,
            "messages": claude_messages,
            "max_tokens": request.max_tokens.unwrap_or(4096)
        });

        if let Some(system) = system_message {
            claude_request["system"] = serde_json::Value::String(system);
        }

        if let Some(temperature) = request.temperature {
            claude_request["temperature"] = serde_json::Value::from(temperature);
        }

        if let Some(top_p) = request.top_p {
            claude_request["top_p"] = serde_json::Value::from(top_p);
        }

        if let Some(stop) = &request.stop {
            claude_request["stop_sequences"] = serde_json::Value::Array(
                stop.iter().map(|s| serde_json::Value::String(s.clone())).collect()
            );
        }

        Ok(claude_request)
    }

    fn convert_from_claude_format(&self, response: &serde_json::Value) -> Result<CompletionResponse> {
        let id = response["id"].as_str().unwrap_or_default().to_string();
        let model = response["model"].as_str().unwrap_or_default().to_string();
        
        let content = response["content"][0]["text"].as_str().unwrap_or_default();
        let message = Message::assistant(content);

        let choice = Choice {
            index: 0,
            message,
            finish_reason: Some(FinishReason::Stop),
        };

        let usage = Usage {
            prompt_tokens: response["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: response["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (response["usage"]["input_tokens"].as_u64().unwrap_or(0) + 
                          response["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
        };

        Ok(CompletionResponse {
            id,
            choices: vec![choice],
            usage,
            model,
            created: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        })
    }
}

/// OpenAI GPT provider implementation
#[derive(Clone)]
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ResponseCache>,
    usage_stats: Arc<AtomicUsageStats>,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| WritemagicError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            api_key,
            base_url: "https://api.openai.com".to_string(),
            client,
            rate_limiter: Arc::new(RateLimiter::new(10, 100)), // 10 concurrent, 100ms min interval
            cache: Arc::new(ResponseCache::new(300)), // 5 minute cache
            usage_stats: Arc::new(AtomicUsageStats::new()),
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_rate_limit(mut self, max_concurrent: usize, min_interval_ms: u64) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(max_concurrent, min_interval_ms));
        self
    }

    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache = Arc::new(ResponseCache::new(ttl_seconds));
        self
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        // Check cache first
        let cache_key = ResponseCache::generate_cache_key(request);
        if let Some(cached_response) = self.cache.get(&cache_key) {
            log::debug!("Cache hit for OpenAI request");
            return Ok(cached_response);
        }

        // Rate limiting
        let _permit = self.rate_limiter.acquire().await?;

        let url = format!("{}/v1/chat/completions", self.base_url);
        
        log::debug!("Making OpenAI API request to: {}", url);
        let start_time = Instant::now();

        let openai_request = self.convert_to_openai_format(request);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| {
                log::error!("OpenAI API network error: {}", e);
                WritemagicError::network(format!("OpenAI API request failed: {}", e))
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            log::error!("OpenAI API error (status {}): {}", status, response_text);
            
            // Handle specific error types
            match status.as_u16() {
                401 => return Err(WritemagicError::authentication("Invalid OpenAI API key")),
                429 => return Err(WritemagicError::ai_provider("OpenAI API rate limit exceeded")),
                500..=599 => return Err(WritemagicError::ai_provider("OpenAI API server error")),
                _ => return Err(WritemagicError::ai_provider(format!("OpenAI API error: {}", response_text))),
            }
        }

        let completion_response: CompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to parse OpenAI response: {}", e);
                WritemagicError::ai_provider(format!("Failed to parse OpenAI response: {}", e))
            })?;

        // Update usage stats
        let request_duration = start_time.elapsed();
        self.update_usage_stats(&completion_response, request_duration).await;

        // Cache the response
        self.cache.insert(cache_key, completion_response.clone(), None);

        log::debug!("OpenAI request completed in {:?}", request_duration);
        Ok(completion_response)
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            max_tokens: 4096,
            supports_streaming: true,
            supports_functions: true,
            supports_vision: true,
            context_window: 128000,
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00003,
        }
    }

    async fn validate_credentials(&self) -> Result<bool> {
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "gpt-3.5-turbo".to_string(),
        ).with_max_tokens(1);

        match self.complete(&test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_usage_stats(&self) -> Result<UsageStats> {
        Ok(self.usage_stats.to_usage_stats().await)
    }

    async fn stream(&self, request: &CompletionRequest) -> Result<Box<dyn StreamingResponse>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        let mut openai_request = self.convert_to_openai_format(request);
        openai_request["stream"] = serde_json::Value::Bool(true);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| WritemagicError::network(format!("OpenAI API request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(Box::new(OpenAIStreamingResponse::new(response)))
        } else {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(WritemagicError::ai_provider(format!("OpenAI streaming failed: {}", error_text)))
        }
    }

    async fn batch_complete(&self, requests: Vec<CompletionRequest>) -> Result<Vec<Result<CompletionResponse>>> {
        let mut results = Vec::new();
        for request in requests {
            let result = self.complete(&request).await;
            results.push(result);
        }
        Ok(results)
    }

    async fn health_check(&self) -> Result<ProviderHealthMetrics> {
        let start_time = Instant::now();
        
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "gpt-3.5-turbo".to_string(),
        ).with_max_tokens(1);

        let result = self.complete(&test_request).await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ProviderHealthMetrics {
            is_healthy: result.is_ok(),
            response_time_ms,
            success_rate: if result.is_ok() { 1.0 } else { 0.0 },
            error_count: if result.is_ok() { 0 } else { 1 },
            last_error: result.err().map(|e| e.to_string()),
            timestamp: std::time::SystemTime::now(),
        })
    }
}

impl OpenAIProvider {
    async fn update_usage_stats(&self, response: &CompletionResponse, _duration: Duration) {
        // Calculate cost based on model capabilities
        let capabilities = self.capabilities();
        let input_cost = response.usage.prompt_tokens as f64 * capabilities.input_cost_per_token;
        let output_cost = response.usage.completion_tokens as f64 * capabilities.output_cost_per_token;
        let total_cost = input_cost + output_cost;
        
        // Atomically update all statistics
        self.usage_stats.increment_request(response.usage.total_tokens as u64, total_cost).await;
    }

    fn convert_to_openai_format(&self, request: &CompletionRequest) -> serde_json::Value {
        serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
            "top_p": request.top_p.unwrap_or(1.0),
            "frequency_penalty": request.frequency_penalty.unwrap_or(0.0),
            "presence_penalty": request.presence_penalty.unwrap_or(0.0),
            "stop": request.stop,
            "stream": request.stream,
        })
    }
}

/// Rate limiter for API requests
#[derive(Debug)]
pub struct RateLimiter {
    semaphore: Semaphore,
    last_request: RwLock<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize, min_interval_ms: u64) -> Self {
        Self {
            semaphore: Semaphore::new(max_concurrent),
            last_request: RwLock::new(Instant::now() - Duration::from_secs(60)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub async fn acquire(&self) -> writemagic_shared::Result<tokio::sync::SemaphorePermit<'_>> {
        let permit = self.semaphore.acquire().await
            .map_err(|_| WritemagicError::network("Rate limiter semaphore closed".to_string()))?;
        
        // Enforce minimum interval between requests
        let last = self.last_request.write().await;
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            let sleep_duration = self.min_interval - elapsed;
            drop(last);
            tokio::time::sleep(sleep_duration).await;
        }
        *self.last_request.write().await = Instant::now();
        
        Ok(permit)
    }
}

/// Response cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    response: CompletionResponse,
    created_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Response cache for AI providers
#[derive(Debug)]
pub struct ResponseCache {
    entries: DashMap<String, CacheEntry>,
    default_ttl: Duration,
}

impl ResponseCache {
    pub fn new(default_ttl_seconds: u64) -> Self {
        Self {
            entries: DashMap::new(),
            default_ttl: Duration::from_secs(default_ttl_seconds),
        }
    }

    pub fn get(&self, key: &str) -> Option<CompletionResponse> {
        if let Some(entry) = self.entries.get(key) {
            if !entry.is_expired() {
                return Some(entry.response.clone());
            } else {
                // Remove expired entry
                drop(entry);
                self.entries.remove(key);
            }
        }
        None
    }

    pub fn insert(&self, key: String, response: CompletionResponse, ttl: Option<Duration>) {
        let entry = CacheEntry {
            response,
            created_at: Instant::now(),
            ttl: ttl.unwrap_or(self.default_ttl),
        };
        self.entries.insert(key, entry);
    }

    pub fn clear_expired(&self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    pub fn generate_cache_key(request: &CompletionRequest) -> String {
        // Create a deterministic cache key from request
        let mut key_parts = Vec::new();
        key_parts.push(request.model.clone());
        key_parts.push(format!("{:?}", request.max_tokens));
        key_parts.push(format!("{:?}", request.temperature));
        
        for message in &request.messages {
            let role_str = match message.role {
                MessageRole::System => "system",
                MessageRole::User => "user", 
                MessageRole::Assistant => "assistant",
                MessageRole::Function => "function",
            };
            key_parts.push(format!("{}:{}", role_str, message.content));
        }
        
        let mut hasher = DefaultHasher::new();
        key_parts.join("|").hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Claude streaming response implementation
pub struct ClaudeStreamingResponse {
    response: reqwest::Response,
    buffer: String,
    is_complete: bool,
    accumulated_content: String,
}

impl ClaudeStreamingResponse {
    pub fn new(response: reqwest::Response) -> Self {
        Self {
            response,
            buffer: String::new(),
            is_complete: false,
            accumulated_content: String::new(),
        }
    }
}

#[async_trait]
impl StreamingResponse for ClaudeStreamingResponse {
    async fn next_chunk(&mut self) -> Result<Option<StreamingChunk>> {
        if self.is_complete {
            return Ok(None);
        }

        // Read next bytes from response
        let chunk = match self.response.chunk().await {
            Ok(Some(chunk)) => {
                self.buffer.push_str(&String::from_utf8_lossy(&chunk));
                chunk
            }
            Ok(None) => {
                self.is_complete = true;
                return Ok(None);
            }
            Err(e) => {
                self.is_complete = true;
                return Err(WritemagicError::network(format!("Streaming error: {}", e)));
            }
        };

        if chunk.is_empty() {
            self.is_complete = true;
            return Ok(None);
        }

        // Parse Server-Sent Events format
        if let Some(event_end) = self.buffer.find("\n\n") {
            let event_data = self.buffer[..event_end].to_string();
            self.buffer.drain(..event_end + 2);

            // Parse the event data
            for line in event_data.lines() {
                if line.starts_with("data: ") {
                    let json_data = &line[6..];
                    if json_data == "[DONE]" {
                        self.is_complete = true;
                        return Ok(None);
                    }

                    match serde_json::from_str::<serde_json::Value>(json_data) {
                        Ok(parsed) => {
                            if let Some(content) = parsed["delta"]["text"].as_str() {
                                self.accumulated_content.push_str(content);
                                
                                return Ok(Some(StreamingChunk {
                                    content: content.to_string(),
                                    finish_reason: parsed["delta"]["stop_reason"].as_str()
                                        .and_then(|r| match r {
                                            "end_turn" => Some(FinishReason::Stop),
                                            "max_tokens" => Some(FinishReason::Length),
                                            _ => None,
                                        }),
                                    usage: None, // Usage typically comes at the end
                                }));
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to parse streaming JSON: {}", e);
                        }
                    }
                }
            }
        }

        // If no complete event found, continue reading
        Ok(None)
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn get_partial_response(&self) -> String {
        self.accumulated_content.clone()
    }
}

/// OpenAI streaming response implementation
pub struct OpenAIStreamingResponse {
    response: reqwest::Response,
    buffer: String,
    is_complete: bool,
    accumulated_content: String,
}

impl OpenAIStreamingResponse {
    pub fn new(response: reqwest::Response) -> Self {
        Self {
            response,
            buffer: String::new(),
            is_complete: false,
            accumulated_content: String::new(),
        }
    }
}

#[async_trait]
impl StreamingResponse for OpenAIStreamingResponse {
    async fn next_chunk(&mut self) -> Result<Option<StreamingChunk>> {
        if self.is_complete {
            return Ok(None);
        }

        // Read next bytes from response
        let chunk = match self.response.chunk().await {
            Ok(Some(chunk)) => {
                self.buffer.push_str(&String::from_utf8_lossy(&chunk));
                chunk
            }
            Ok(None) => {
                self.is_complete = true;
                return Ok(None);
            }
            Err(e) => {
                self.is_complete = true;
                return Err(WritemagicError::network(format!("Streaming error: {}", e)));
            }
        };

        if chunk.is_empty() {
            self.is_complete = true;
            return Ok(None);
        }

        // Parse Server-Sent Events format
        if let Some(event_end) = self.buffer.find("\n\n") {
            let event_data = self.buffer[..event_end].to_string();
            self.buffer.drain(..event_end + 2);

            // Parse the event data
            for line in event_data.lines() {
                if line.starts_with("data: ") {
                    let json_data = &line[6..];
                    if json_data == "[DONE]" {
                        self.is_complete = true;
                        return Ok(None);
                    }

                    match serde_json::from_str::<serde_json::Value>(json_data) {
                        Ok(parsed) => {
                            if let Some(choices) = parsed["choices"].as_array() {
                                if let Some(choice) = choices.first() {
                                    if let Some(content) = choice["delta"]["content"].as_str() {
                                        self.accumulated_content.push_str(content);
                                        
                                        return Ok(Some(StreamingChunk {
                                            content: content.to_string(),
                                            finish_reason: choice["finish_reason"].as_str()
                                                .and_then(|r| match r {
                                                    "stop" => Some(FinishReason::Stop),
                                                    "length" => Some(FinishReason::Length),
                                                    "content_filter" => Some(FinishReason::ContentFilter),
                                                    "tool_calls" => Some(FinishReason::ToolCalls),
                                                    "function_call" => Some(FinishReason::FunctionCall),
                                                    _ => None,
                                                }),
                                            usage: parsed["usage"].as_object().map(|usage| {
                                                Usage {
                                                    prompt_tokens: usage.get("prompt_tokens")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0) as u32,
                                                    completion_tokens: usage.get("completion_tokens")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0) as u32,
                                                    total_tokens: usage.get("total_tokens")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0) as u32,
                                                }
                                            }),
                                        }));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to parse OpenAI streaming JSON: {}", e);
                        }
                    }
                }
            }
        }

        // If no complete event found, continue reading
        Ok(None)
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn get_partial_response(&self) -> String {
        self.accumulated_content.clone()
    }
}