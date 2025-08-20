//! Request batching and deduplication for AI providers

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::interval;
use writemagic_shared::{Result, WritemagicError};
use crate::providers::{CompletionRequest, CompletionResponse, RequestPriority};

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
    pub max_concurrent_batches: usize,
    pub enable_deduplication: bool,
    pub priority_ordering: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_wait_time: Duration::from_millis(100),
            max_concurrent_batches: 5,
            enable_deduplication: true,
            priority_ordering: true,
        }
    }
}

/// Pending request with response channel
struct PendingRequest {
    request: CompletionRequest,
    response_tx: tokio::sync::oneshot::Sender<Result<CompletionResponse>>,
    received_at: Instant,
    priority: RequestPriority,
    request_hash: u64,
}

/// Batch of requests ready for processing
#[derive(Debug)]
pub struct RequestBatch {
    pub requests: Vec<CompletionRequest>,
    pub batch_id: String,
    pub created_at: Instant,
    pub priority: RequestPriority,
}

/// Request deduplication cache entry
#[derive(Debug)]
struct CacheEntry {
    response: CompletionResponse,
    created_at: Instant,
    ttl: Duration,
    waiting_requests: Vec<tokio::sync::oneshot::Sender<Result<CompletionResponse>>>,
}

/// Advanced request batcher with deduplication and priority queuing
pub struct RequestBatcher {
    config: BatchConfig,
    pending_requests: Arc<RwLock<VecDeque<PendingRequest>>>,
    dedup_cache: Arc<RwLock<HashMap<u64, CacheEntry>>>,
    batch_semaphore: Arc<Semaphore>,
    batch_sender: mpsc::UnboundedSender<RequestBatch>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl RequestBatcher {
    /// Create new request batcher
    pub fn new(
        config: BatchConfig,
        _batch_processor: mpsc::UnboundedReceiver<RequestBatch>,
    ) -> (Self, mpsc::UnboundedReceiver<RequestBatch>) {
        let (batch_tx, batch_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let batcher = Self {
            config: config.clone(),
            pending_requests: Arc::new(RwLock::new(VecDeque::new())),
            dedup_cache: Arc::new(RwLock::new(HashMap::new())),
            batch_semaphore: Arc::new(Semaphore::new(config.max_concurrent_batches)),
            batch_sender: batch_tx,
            shutdown_tx: Some(shutdown_tx),
        };

        // Start background tasks
        let batcher_clone = batcher.clone();
        tokio::spawn(async move {
            batcher_clone.run_batch_processor(shutdown_rx).await;
        });

        let batcher_clone = batcher.clone();
        tokio::spawn(async move {
            batcher_clone.run_cache_cleaner().await;
        });

        (batcher, batch_rx)
    }

    /// Submit a request for batching
    pub async fn submit_request(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let request_hash = self.calculate_request_hash(&request);
        let priority = request.priority.clone();

        // Check for deduplication if enabled
        if self.config.enable_deduplication {
            if let Some(cached_response) = self.check_dedup_cache(request_hash).await {
                return Ok(cached_response);
            }
        }

        // Create response channel
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Add to pending requests
        let pending_request = PendingRequest {
            request,
            response_tx,
            received_at: Instant::now(),
            priority,
            request_hash,
        };

        {
            let mut pending = self.pending_requests.write().await;
            pending.push_back(pending_request);
        }

        // Try to create batch immediately if conditions are met
        self.try_create_batch().await;

        // Wait for response
        match response_rx.await {
            Ok(result) => result,
            Err(_) => Err(WritemagicError::internal("Request cancelled")),
        }
    }

    /// Calculate hash for request deduplication
    pub fn calculate_request_hash(&self, request: &CompletionRequest) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        
        // Hash the essential parts of the request
        request.model.hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);
        
        // Convert f32 to bits for hashing
        if let Some(temp) = request.temperature {
            temp.to_bits().hash(&mut hasher);
        }
        if let Some(top_p) = request.top_p {
            top_p.to_bits().hash(&mut hasher);
        }
        
        for message in &request.messages {
            // Note: We'll need to implement Hash for MessageRole if not already done
            format!("{:?}", message.role).hash(&mut hasher);
            message.content.hash(&mut hasher);
        }
        
        hasher.finish()
    }

    /// Check deduplication cache
    async fn check_dedup_cache(&self, request_hash: u64) -> Option<CompletionResponse> {
        let cache = self.dedup_cache.read().await;
        if let Some(entry) = cache.get(&request_hash) {
            if !entry.is_expired() {
                return Some(entry.response.clone());
            }
        }
        None
    }

    /// Try to create a batch if conditions are met
    async fn try_create_batch(&self) -> bool {
        let mut pending = self.pending_requests.write().await;
        
        if pending.is_empty() {
            return false;
        }

        let should_batch = pending.len() >= self.config.max_batch_size ||
            pending.front().map_or(false, |req| {
                req.received_at.elapsed() >= self.config.max_wait_time
            });

        if !should_batch {
            return false;
        }

        // Check if we can acquire a batch processing permit
        if self.batch_semaphore.available_permits() == 0 {
            return false;
        }

        // Create batch
        let batch_size = pending.len().min(self.config.max_batch_size);
        let mut batch_requests = Vec::with_capacity(batch_size);
        let mut response_channels = Vec::with_capacity(batch_size);

        // Extract requests for batch
        for _ in 0..batch_size {
            if let Some(pending_req) = pending.pop_front() {
                batch_requests.push(pending_req.request);
                response_channels.push(pending_req.response_tx);
            }
        }

        drop(pending);

        if batch_requests.is_empty() {
            return false;
        }

        // Determine batch priority (highest priority wins)
        let batch_priority = batch_requests.iter()
            .map(|r| &r.priority)
            .max()
            .cloned()
            .unwrap_or(RequestPriority::Normal);

        // Sort by priority if enabled
        if self.config.priority_ordering {
            let mut indexed_requests: Vec<(usize, &CompletionRequest)> = batch_requests.iter().enumerate().collect();
            indexed_requests.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
            
            let sorted_requests: Vec<CompletionRequest> = indexed_requests.into_iter()
                .map(|(_, req)| req.clone())
                .collect();
            batch_requests = sorted_requests;
        }

        let batch = RequestBatch {
            requests: batch_requests.clone(),
            batch_id: uuid::Uuid::new_v4().to_string(),
            created_at: Instant::now(),
            priority: batch_priority,
        };

        // Send batch for processing
        if let Err(_) = self.batch_sender.send(batch) {
            // If sending fails, respond with error to all requests
            for response_tx in response_channels {
                let _ = response_tx.send(Err(WritemagicError::internal("Batch processing unavailable")));
            }
            return false;
        }

        true
    }

    /// Background task to process batches based on time
    async fn run_batch_processor(&self, mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) {
        let mut interval = interval(self.config.max_wait_time / 2);
        
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    log::info!("Shutting down request batcher");
                    break;
                }
                _ = interval.tick() => {
                    self.try_create_batch().await;
                }
            }
        }
    }

    /// Background task to clean expired cache entries
    async fn run_cache_cleaner(&self) {
        let mut interval = interval(Duration::from_secs(60)); // Clean every minute
        
        loop {
            interval.tick().await;
            
            let mut cache = self.dedup_cache.write().await;
            let before_size = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            let after_size = cache.len();
            
            if before_size != after_size {
                log::debug!("Cleaned {} expired cache entries", before_size - after_size);
            }
        }
    }

    /// Add response to deduplication cache
    pub async fn cache_response(&self, request_hash: u64, response: CompletionResponse) {
        if !self.config.enable_deduplication {
            return;
        }

        let entry = CacheEntry {
            response,
            created_at: Instant::now(),
            ttl: Duration::from_secs(300), // 5 minutes default TTL
            waiting_requests: Vec::new(),
        };

        let mut cache = self.dedup_cache.write().await;
        cache.insert(request_hash, entry);
    }

    /// Get statistics about the batcher
    pub async fn get_stats(&self) -> BatcherStats {
        let pending = self.pending_requests.read().await;
        let cache = self.dedup_cache.read().await;
        
        BatcherStats {
            pending_requests: pending.len(),
            cache_entries: cache.len(),
            available_batch_permits: self.batch_semaphore.available_permits(),
        }
    }
}

impl Clone for RequestBatcher {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pending_requests: self.pending_requests.clone(),
            dedup_cache: self.dedup_cache.clone(),
            batch_semaphore: self.batch_semaphore.clone(),
            batch_sender: self.batch_sender.clone(),
            shutdown_tx: None, // Only the original has the shutdown sender
        }
    }
}

impl Drop for RequestBatcher {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Batcher statistics
#[derive(Debug, Clone)]
pub struct BatcherStats {
    pub pending_requests: usize,
    pub cache_entries: usize,
    pub available_batch_permits: usize,
}

/// Intelligent request scheduler that optimizes batch processing
pub struct RequestScheduler {
    batchers: HashMap<String, RequestBatcher>, // Per-provider batchers
    load_balancer: Arc<RwLock<LoadBalancer>>,
}

impl RequestScheduler {
    pub fn new() -> Self {
        Self {
            batchers: HashMap::new(),
            load_balancer: Arc::new(RwLock::new(LoadBalancer::new())),
        }
    }

    /// Add a provider batcher
    pub fn add_provider_batcher(&mut self, provider_name: String, batcher: RequestBatcher) {
        self.batchers.insert(provider_name, batcher);
    }

    /// Schedule a request to the best available provider
    pub async fn schedule_request(&self, request: CompletionRequest, preferred_provider: Option<String>) -> Result<CompletionResponse> {
        let provider_name = if let Some(preferred) = preferred_provider {
            preferred
        } else {
            let load_balancer = self.load_balancer.read().await;
            load_balancer.select_provider(&self.batchers.keys().collect::<Vec<_>>())
                .ok_or_else(|| WritemagicError::internal("No providers available"))?
        };

        if let Some(batcher) = self.batchers.get(&provider_name) {
            let result = batcher.submit_request(request).await;
            
            // Update load balancer metrics
            let mut load_balancer = self.load_balancer.write().await;
            match &result {
                Ok(_) => load_balancer.record_success(&provider_name),
                Err(_) => load_balancer.record_failure(&provider_name),
            }
            
            result
        } else {
            Err(WritemagicError::internal(format!("Provider '{}' not found", provider_name)))
        }
    }

    /// Get statistics for all batchers
    pub async fn get_all_stats(&self) -> HashMap<String, BatcherStats> {
        let mut stats = HashMap::new();
        
        for (provider_name, batcher) in &self.batchers {
            stats.insert(provider_name.clone(), batcher.get_stats().await);
        }
        
        stats
    }
}

/// Simple load balancer for provider selection
#[derive(Debug)]
struct LoadBalancer {
    provider_weights: HashMap<String, f64>,
    provider_stats: HashMap<String, ProviderStats>,
}

#[derive(Debug, Default)]
struct ProviderStats {
    success_count: u64,
    failure_count: u64,
    avg_response_time: Duration,
}

impl LoadBalancer {
    fn new() -> Self {
        Self {
            provider_weights: HashMap::new(),
            provider_stats: HashMap::new(),
        }
    }

    /// Select best provider based on weights and statistics
    fn select_provider(&self, available_providers: &[&String]) -> Option<String> {
        if available_providers.is_empty() {
            return None;
        }

        // Simple weighted random selection based on success rate
        let mut best_provider = None;
        let mut best_score = f64::NEG_INFINITY;

        for provider in available_providers {
            let default_stats = ProviderStats::default();
            let stats = self.provider_stats.get(*provider).unwrap_or(&default_stats);
            let total_requests = stats.success_count + stats.failure_count;
            
            let success_rate = if total_requests > 0 {
                stats.success_count as f64 / total_requests as f64
            } else {
                0.5 // Neutral score for untested providers
            };

            let response_time_penalty = stats.avg_response_time.as_millis() as f64 / 10000.0; // Normalize to 0-1 scale
            let score = success_rate - response_time_penalty;

            if score > best_score {
                best_score = score;
                best_provider = Some((*provider).clone());
            }
        }

        best_provider
    }

    fn record_success(&mut self, provider_name: &str) {
        let stats = self.provider_stats.entry(provider_name.to_string()).or_default();
        stats.success_count += 1;
    }

    fn record_failure(&mut self, provider_name: &str) {
        let stats = self.provider_stats.entry(provider_name.to_string()).or_default();
        stats.failure_count += 1;
    }
}