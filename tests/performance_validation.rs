//! Performance and Stress Testing Suite for WriteMagic
//!
//! Validates performance characteristics under various loads and stress conditions.
//! Tests memory usage, concurrent access patterns, large document handling, and AI performance.

use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use writemagic_shared::{EntityId, Result, WritemagicError, ContentType, Pagination};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, DocumentTitle, DocumentContent, ProjectName,
    entities::{Document, Project},
};

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub warm_up_iterations: usize,
    pub test_iterations: usize,
    pub concurrent_users: usize,
    pub large_document_size_mb: usize,
    pub batch_sizes: Vec<usize>,
    pub timeout_seconds: u64,
    pub enable_ai_tests: bool,
    pub memory_pressure_test: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            warm_up_iterations: 10,
            test_iterations: 100,
            concurrent_users: 50,
            large_document_size_mb: 10,
            batch_sizes: vec![1, 10, 50, 100],
            timeout_seconds: 60,
            enable_ai_tests: false,
            memory_pressure_test: true,
        }
    }
}

/// Performance metrics collection
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub operation_times: Vec<f64>,
    pub throughput_ops_per_sec: f64,
    pub memory_usage_mb: f64,
    pub concurrent_success_rate: f64,
    pub error_rate: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
}

impl PerformanceMetrics {
    pub fn calculate_from_times(&mut self, times: Vec<f64>) {
        if times.is_empty() {
            return;
        }

        self.operation_times = times.clone();
        let mut sorted_times = times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.min_latency_ms = sorted_times[0];
        self.max_latency_ms = sorted_times[sorted_times.len() - 1];
        
        let len = sorted_times.len();
        self.p50_latency_ms = sorted_times[len / 2];
        self.p95_latency_ms = sorted_times[(len as f64 * 0.95) as usize];
        self.p99_latency_ms = sorted_times[(len as f64 * 0.99) as usize];

        // Calculate throughput (ops/sec)
        let total_time_sec = times.iter().sum::<f64>() / 1000.0;
        self.throughput_ops_per_sec = if total_time_sec > 0.0 {
            times.len() as f64 / total_time_sec
        } else {
            0.0
        };
    }
}

/// Performance test results
#[derive(Debug, Default)]
pub struct PerformanceResults {
    pub document_creation: PerformanceMetrics,
    pub document_retrieval: PerformanceMetrics,
    pub document_update: PerformanceMetrics,
    pub document_search: PerformanceMetrics,
    pub project_operations: PerformanceMetrics,
    pub concurrent_access: PerformanceMetrics,
    pub large_document_handling: PerformanceMetrics,
    pub ai_completion: PerformanceMetrics,
    pub sqlite_operations: PerformanceMetrics,
    pub memory_stress: PerformanceMetrics,
}

/// Main performance validation engine
pub struct PerformanceValidator {
    config: PerformanceConfig,
    results: Arc<RwLock<PerformanceResults>>,
    operation_counter: Arc<AtomicUsize>,
    error_counter: Arc<AtomicUsize>,
}

impl PerformanceValidator {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(PerformanceResults::default())),
            operation_counter: Arc::new(AtomicUsize::new(0)),
            error_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Run complete performance validation suite
    pub async fn run_performance_validation(&self) -> Result<PerformanceResults> {
        println!("⚡ WriteMagic Performance Validation Suite");
        println!("=========================================\n");

        let engine = self.create_test_engine().await?;

        // Warm-up phase
        self.warm_up_phase(&engine).await?;

        // Core performance tests
        self.test_document_creation_performance(&engine).await?;
        self.test_document_retrieval_performance(&engine).await?;
        self.test_document_update_performance(&engine).await?;
        self.test_document_search_performance(&engine).await?;
        self.test_project_operations_performance(&engine).await?;

        // Concurrent access tests
        self.test_concurrent_access_performance(&engine).await?;

        // Large document handling
        self.test_large_document_performance(&engine).await?;

        // SQLite performance
        self.test_sqlite_performance(&engine).await?;

        // AI performance (if enabled)
        if self.config.enable_ai_tests {
            self.test_ai_performance(&engine).await?;
        }

        // Memory stress tests
        if self.config.memory_pressure_test {
            self.test_memory_stress_performance(&engine).await?;
        }

        let results = self.results.read().await.clone();
        self.print_performance_summary(&results).await;

        Ok(results)
    }

    /// Warm-up phase to initialize caches and connections
    async fn warm_up_phase(&self, engine: &CoreEngine) -> Result<()> {
        println!("🔥 Warming up system caches...");
        let doc_service = engine.document_management_service();

        for i in 0..self.config.warm_up_iterations {
            let title = DocumentTitle::new(&format!("Warmup Doc {}", i))?;
            let content = DocumentContent::new("Warmup content")?;
            
            let _ = doc_service
                .create_document(title, content, ContentType::PlainText, None)
                .await?;
        }

        // Clear warm-up data
        let doc_repo = engine.document_repository();
        let docs = doc_repo.find_all(Pagination::new(0, 1000)?).await?;
        for doc in docs {
            let _ = doc_repo.delete(&doc.id).await;
        }

        println!("✅ Warm-up completed\n");
        Ok(())
    }

    /// Test document creation performance
    async fn test_document_creation_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("📄 Testing Document Creation Performance...");
        let doc_service = engine.document_management_service();
        let mut operation_times = Vec::new();

        for i in 0..self.config.test_iterations {
            let start = Instant::now();
            
            let title = DocumentTitle::new(&format!("Perf Test Doc {}", i))?;
            let content = DocumentContent::new(&format!("Performance test content for document {}", i))?;
            
            match doc_service.create_document(title, content, ContentType::Markdown, None).await {
                Ok(_) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            // Progress indicator
            if i % (self.config.test_iterations / 10).max(1) == 0 {
                let progress = (i as f64 / self.config.test_iterations as f64) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.document_creation = metrics.clone();
        println!("   ✅ Document Creation - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test document retrieval performance
    async fn test_document_retrieval_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🔍 Testing Document Retrieval Performance...");
        let doc_repo = engine.document_repository();
        
        // Get some document IDs to test with
        let existing_docs = doc_repo.find_all(Pagination::new(0, 50)?).await?;
        if existing_docs.is_empty() {
            println!("   ⚠️  No existing documents for retrieval test");
            return Ok(());
        }

        let mut operation_times = Vec::new();

        for i in 0..self.config.test_iterations {
            let doc_id = &existing_docs[i % existing_docs.len()].id;
            let start = Instant::now();
            
            match doc_repo.find_by_id(doc_id).await {
                Ok(Some(_)) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Ok(None) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % (self.config.test_iterations / 10).max(1) == 0 {
                let progress = (i as f64 / self.config.test_iterations as f64) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.document_retrieval = metrics.clone();
        println!("   ✅ Document Retrieval - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test document update performance
    async fn test_document_update_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n✏️  Testing Document Update Performance...");
        let doc_service = engine.document_management_service();
        let doc_repo = engine.document_repository();
        
        // Get some document IDs to test with
        let existing_docs = doc_repo.find_all(Pagination::new(0, 20)?).await?;
        if existing_docs.is_empty() {
            println!("   ⚠️  No existing documents for update test");
            return Ok(());
        }

        let mut operation_times = Vec::new();

        for i in 0..self.config.test_iterations.min(existing_docs.len() * 10) {
            let doc_id = existing_docs[i % existing_docs.len()].id;
            let start = Instant::now();
            
            let new_content = DocumentContent::new(&format!("Updated content {}", i))?;
            
            match doc_service.update_document_content(doc_id, new_content, None, None).await {
                Ok(_) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % (self.config.test_iterations / 10).max(1) == 0 {
                let progress = (i as f64 / self.config.test_iterations as f64) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.document_update = metrics.clone();
        println!("   ✅ Document Update - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test document search performance
    async fn test_document_search_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🔎 Testing Document Search Performance...");
        let doc_repo = engine.document_repository();
        
        let search_terms = vec!["test", "performance", "document", "content", "Perf"];
        let mut operation_times = Vec::new();

        for i in 0..self.config.test_iterations {
            let search_term = &search_terms[i % search_terms.len()];
            let start = Instant::now();
            
            match doc_repo.search_by_title(search_term, Pagination::new(0, 10)?).await {
                Ok(_) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % (self.config.test_iterations / 10).max(1) == 0 {
                let progress = (i as f64 / self.config.test_iterations as f64) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.document_search = metrics.clone();
        println!("   ✅ Document Search - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test project operations performance
    async fn test_project_operations_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n📁 Testing Project Operations Performance...");
        let proj_service = engine.project_management_service();
        let mut operation_times = Vec::new();

        // Create projects
        for i in 0..self.config.test_iterations / 2 {
            let start = Instant::now();
            
            let name = ProjectName::new(&format!("Perf Project {}", i))?;
            let description = Some(format!("Performance test project {}", i));
            
            match proj_service.create_project(name, description, None).await {
                Ok(_) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Retrieve projects
        let proj_repo = engine.project_repository();
        for i in 0..self.config.test_iterations / 2 {
            let start = Instant::now();
            
            match proj_repo.find_all(Pagination::new(0, 10)?).await {
                Ok(_) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % (self.config.test_iterations / 20).max(1) == 0 {
                let progress = (i as f64 / (self.config.test_iterations as f64 / 2.0)) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.project_operations = metrics.clone();
        println!("   ✅ Project Operations - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test concurrent access performance
    async fn test_concurrent_access_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🔄 Testing Concurrent Access Performance...");
        let doc_service = engine.document_management_service();
        
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_users));
        let operation_times = Arc::new(RwLock::new(Vec::new()));
        let success_counter = Arc::new(AtomicUsize::new(0));
        let total_ops = self.config.test_iterations;

        let start_time = Instant::now();
        
        let mut handles = Vec::new();
        for i in 0..total_ops {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let service = doc_service.clone();
            let times = operation_times.clone();
            let counter = success_counter.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit
                let op_start = Instant::now();
                
                let title = DocumentTitle::new(&format!("Concurrent Doc {}", i)).unwrap();
                let content = DocumentContent::new(&format!("Concurrent content {}", i)).unwrap();
                
                match service.create_document(title, content, ContentType::Markdown, None).await {
                    Ok(_) => {
                        let duration = op_start.elapsed().as_secs_f64() * 1000.0;
                        times.write().await.push(duration);
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {}
                }
            });
            
            handles.push(handle);

            // Progress indicator
            if i % (total_ops / 10).max(1) == 0 {
                let progress = (i as f64 / total_ops as f64) * 100.0;
                print!("\r   Spawned: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }

        println!("\n   Waiting for concurrent operations to complete...");
        futures::future::join_all(handles).await;
        
        let total_time = start_time.elapsed().as_secs_f64();
        let successful_ops = success_counter.load(Ordering::Relaxed);
        let success_rate = successful_ops as f64 / total_ops as f64;

        let times = operation_times.read().await.clone();
        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(times);
        metrics.concurrent_success_rate = success_rate;

        self.results.write().await.concurrent_access = metrics.clone();
        println!("   ✅ Concurrent Access - {} users, {:.1}% success rate, {:.1} ops/sec total throughput",
                 self.config.concurrent_users, success_rate * 100.0, successful_ops as f64 / total_time);

        Ok(())
    }

    /// Test large document handling performance
    async fn test_large_document_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n📚 Testing Large Document Performance...");
        let doc_service = engine.document_management_service();
        
        // Create large content (configurable size in MB)
        let content_size = self.config.large_document_size_mb * 1024 * 1024; // Convert MB to bytes
        let large_content = "A".repeat(content_size);
        
        let mut operation_times = Vec::new();
        let iterations = 5; // Fewer iterations for large documents

        for i in 0..iterations {
            let start = Instant::now();
            
            let title = DocumentTitle::new(&format!("Large Doc {} ({}MB)", i, self.config.large_document_size_mb))?;
            let content = DocumentContent::new(&large_content)?;
            
            match doc_service.create_document(title, content, ContentType::PlainText, None).await {
                Ok(aggregate) => {
                    let create_duration = start.elapsed().as_secs_f64() * 1000.0;
                    
                    // Also test retrieval of large document
                    let retrieve_start = Instant::now();
                    let doc_repo = engine.document_repository();
                    match doc_repo.find_by_id(&aggregate.document().id).await {
                        Ok(Some(_)) => {
                            let retrieve_duration = retrieve_start.elapsed().as_secs_f64() * 1000.0;
                            operation_times.push(create_duration + retrieve_duration);
                            self.operation_counter.fetch_add(2, Ordering::Relaxed);
                        }
                        _ => {
                            operation_times.push(create_duration);
                            self.operation_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            println!("   Progress: {}/{} large documents", i + 1, iterations);
        }

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        // Estimate memory usage
        metrics.memory_usage_mb = (content_size * iterations) as f64 / (1024.0 * 1024.0);

        self.results.write().await.large_document_handling = metrics.clone();
        println!("   ✅ Large Documents ({}MB) - Avg: {:.2}ms, Max: {:.2}ms, Memory: {:.1}MB",
                 self.config.large_document_size_mb, metrics.p50_latency_ms, 
                 metrics.max_latency_ms, metrics.memory_usage_mb);

        Ok(())
    }

    /// Test SQLite-specific performance
    async fn test_sqlite_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🗄️  Testing SQLite Performance...");
        let doc_repo = engine.document_repository();
        
        let mut operation_times = Vec::new();

        // Test batch operations with different batch sizes
        for &batch_size in &self.config.batch_sizes {
            println!("   Testing batch size: {}", batch_size);
            
            let batch_start = Instant::now();
            let mut batch_docs = Vec::new();
            
            // Create batch of documents
            for i in 0..batch_size {
                let doc = Document::new(
                    format!("Batch Doc {} (size {})", i, batch_size),
                    format!("Batch content {} for size {}", i, batch_size),
                    ContentType::Markdown,
                    Some(EntityId::new()),
                );
                batch_docs.push(doc);
            }

            // Time the batch save operations
            for doc in batch_docs {
                let save_start = Instant::now();
                match doc_repo.save(&doc).await {
                    Ok(_) => {
                        let duration = save_start.elapsed().as_secs_f64() * 1000.0;
                        operation_times.push(duration);
                    }
                    Err(_) => {
                        self.error_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            let batch_duration = batch_start.elapsed().as_secs_f64() * 1000.0;
            println!("      Batch {} completed in {:.2}ms", batch_size, batch_duration);
        }

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.sqlite_operations = metrics.clone();
        println!("   ✅ SQLite Operations - Avg: {:.2}ms, P95: {:.2}ms, Throughput: {:.1} ops/sec",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, metrics.throughput_ops_per_sec);

        Ok(())
    }

    /// Test AI performance (if enabled)
    async fn test_ai_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🤖 Testing AI Performance...");
        
        let mut operation_times = Vec::new();
        let ai_prompts = vec![
            "Write a brief introduction to Rust programming.",
            "Explain the benefits of functional programming.",
            "Describe how to implement a binary search algorithm.",
            "What are the key principles of software architecture?",
            "How does async programming work in modern languages?",
        ];

        for i in 0..self.config.test_iterations.min(20) { // Limit AI tests
            let prompt = &ai_prompts[i % ai_prompts.len()];
            let start = Instant::now();
            
            match tokio::time::timeout(
                Duration::from_secs(self.config.timeout_seconds),
                engine.complete_text(prompt.to_string(), None)
            ).await {
                Ok(Ok(_)) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Err(_)) | Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % 5 == 0 {
                let progress = (i as f64 / 20.0) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);

        self.results.write().await.ai_completion = metrics.clone();
        println!("   ✅ AI Completion - Avg: {:.2}ms, P95: {:.2}ms, Success: {:.1}%",
                 metrics.p50_latency_ms, metrics.p95_latency_ms, 
                 (operation_times.len() as f64 / 20.0) * 100.0);

        Ok(())
    }

    /// Test memory stress performance
    async fn test_memory_stress_performance(&self, engine: &CoreEngine) -> Result<()> {
        println!("\n🧠 Testing Memory Stress Performance...");
        let doc_service = engine.document_management_service();
        
        let mut operation_times = Vec::new();
        let mut created_docs = Vec::new();

        // Create many documents to stress memory
        let stress_iterations = 500;
        
        for i in 0..stress_iterations {
            let start = Instant::now();
            
            // Vary document sizes to create memory pressure
            let content_size = match i % 4 {
                0 => 1024,      // 1KB
                1 => 10 * 1024, // 10KB
                2 => 100 * 1024, // 100KB
                _ => 512,       // 512B
            };
            
            let title = DocumentTitle::new(&format!("Stress Doc {}", i))?;
            let content = DocumentContent::new(&"X".repeat(content_size))?;
            
            match doc_service.create_document(title, content, ContentType::PlainText, None).await {
                Ok(aggregate) => {
                    let duration = start.elapsed().as_secs_f64() * 1000.0;
                    operation_times.push(duration);
                    created_docs.push(aggregate.document().id);
                    self.operation_counter.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.error_counter.fetch_add(1, Ordering::Relaxed);
                }
            }

            if i % (stress_iterations / 10) == 0 {
                let progress = (i as f64 / stress_iterations as f64) * 100.0;
                print!("\r   Progress: {:.1}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        // Test rapid cleanup
        let cleanup_start = Instant::now();
        let doc_repo = engine.document_repository();
        let mut cleanup_count = 0;
        
        for doc_id in &created_docs[..100.min(created_docs.len())] {
            if doc_repo.delete(doc_id).await.is_ok() {
                cleanup_count += 1;
            }
        }
        
        let cleanup_duration = cleanup_start.elapsed().as_secs_f64() * 1000.0;

        let mut metrics = PerformanceMetrics::default();
        metrics.calculate_from_times(operation_times);
        
        // Estimate memory usage
        let avg_doc_size = (1024 + 10*1024 + 100*1024 + 512) / 4;
        metrics.memory_usage_mb = (created_docs.len() * avg_doc_size) as f64 / (1024.0 * 1024.0);

        self.results.write().await.memory_stress = metrics.clone();
        println!("   ✅ Memory Stress - {} docs, Avg: {:.2}ms, Cleanup: {}ms, Est. Memory: {:.1}MB",
                 created_docs.len(), metrics.p50_latency_ms, cleanup_duration, metrics.memory_usage_mb);

        Ok(())
    }

    /// Print comprehensive performance summary
    async fn print_performance_summary(&self, results: &PerformanceResults) {
        println!("\n📊 WriteMagic Performance Validation Summary");
        println!("============================================\n");

        let total_operations = self.operation_counter.load(Ordering::Relaxed);
        let total_errors = self.error_counter.load(Ordering::Relaxed);
        let error_rate = if total_operations > 0 {
            (total_errors as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };

        println!("📈 Overall Statistics:");
        println!("   Total Operations: {}", total_operations);
        println!("   Total Errors: {} ({:.2}% error rate)", total_errors, error_rate);
        println!();

        println!("⚡ Performance Metrics by Operation:");
        
        self.print_operation_metrics("Document Creation", &results.document_creation);
        self.print_operation_metrics("Document Retrieval", &results.document_retrieval);
        self.print_operation_metrics("Document Update", &results.document_update);
        self.print_operation_metrics("Document Search", &results.document_search);
        self.print_operation_metrics("Project Operations", &results.project_operations);
        self.print_operation_metrics("Concurrent Access", &results.concurrent_access);
        self.print_operation_metrics("Large Documents", &results.large_document_handling);
        self.print_operation_metrics("SQLite Operations", &results.sqlite_operations);
        
        if self.config.enable_ai_tests {
            self.print_operation_metrics("AI Completion", &results.ai_completion);
        }
        
        if self.config.memory_pressure_test {
            self.print_operation_metrics("Memory Stress", &results.memory_stress);
        }

        // Performance assessment
        println!("\n🎯 Performance Assessment:");
        self.assess_performance(results).await;
    }

    fn print_operation_metrics(&self, operation: &str, metrics: &PerformanceMetrics) {
        if metrics.operation_times.is_empty() {
            return;
        }

        println!("   {}:", operation);
        println!("     - Latency: min={:.2}ms, p50={:.2}ms, p95={:.2}ms, p99={:.2}ms, max={:.2}ms",
                 metrics.min_latency_ms, metrics.p50_latency_ms, 
                 metrics.p95_latency_ms, metrics.p99_latency_ms, metrics.max_latency_ms);
        
        if metrics.throughput_ops_per_sec > 0.0 {
            println!("     - Throughput: {:.1} ops/sec", metrics.throughput_ops_per_sec);
        }
        
        if metrics.concurrent_success_rate > 0.0 {
            println!("     - Success Rate: {:.1}%", metrics.concurrent_success_rate * 100.0);
        }
        
        if metrics.memory_usage_mb > 0.0 {
            println!("     - Memory Usage: {:.1} MB", metrics.memory_usage_mb);
        }
        
        println!("     - Operations: {}", metrics.operation_times.len());
        println!();
    }

    async fn assess_performance(&self, results: &PerformanceResults) {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Document operations assessment
        if results.document_creation.p95_latency_ms > 100.0 {
            issues.push("Document creation P95 latency > 100ms");
            recommendations.push("Consider optimizing document validation and SQLite writes");
        }

        if results.document_retrieval.p95_latency_ms > 50.0 {
            issues.push("Document retrieval P95 latency > 50ms");
            recommendations.push("Consider adding database indexes or caching");
        }

        if results.concurrent_access.concurrent_success_rate < 0.95 {
            issues.push("Concurrent access success rate < 95%");
            recommendations.push("Review connection pooling and locking strategies");
        }

        if self.config.enable_ai_tests && results.ai_completion.p95_latency_ms > 10000.0 {
            issues.push("AI completion P95 latency > 10s");
            recommendations.push("Implement request timeout and caching for AI responses");
        }

        // Memory assessment
        let total_memory_mb = results.large_document_handling.memory_usage_mb + 
                             results.memory_stress.memory_usage_mb;
        if total_memory_mb > 500.0 {
            issues.push("High memory usage detected");
            recommendations.push("Consider implementing document streaming or pagination");
        }

        // Print assessment
        if issues.is_empty() {
            println!("   ✅ All performance metrics within acceptable ranges");
            println!("   🚀 System is ready for production deployment");
        } else {
            println!("   ⚠️  Performance issues identified:");
            for issue in &issues {
                println!("      - {}", issue);
            }
            
            println!("\n   💡 Recommendations:");
            for rec in &recommendations {
                println!("      - {}", rec);
            }
        }

        // Throughput summary
        let doc_throughput = results.document_creation.throughput_ops_per_sec;
        let retrieval_throughput = results.document_retrieval.throughput_ops_per_sec;
        
        println!("\n📋 Production Readiness Checklist:");
        println!("   Document Creation: {} ops/sec {}", 
                 doc_throughput, if doc_throughput > 100.0 { "✅" } else { "⚠️" });
        println!("   Document Retrieval: {} ops/sec {}", 
                 retrieval_throughput, if retrieval_throughput > 500.0 { "✅" } else { "⚠️" });
        println!("   Error Rate: {:.2}% {}", 
                 (self.error_counter.load(Ordering::Relaxed) as f64 / 
                  self.operation_counter.load(Ordering::Relaxed) as f64) * 100.0,
                 if self.error_counter.load(Ordering::Relaxed) == 0 { "✅" } else { "⚠️" });
    }

    async fn create_test_engine(&self) -> Result<CoreEngine> {
        ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .with_log_level("warn".to_string()) // Reduce log noise during performance tests
            .with_content_filtering(false)
            .build()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_validation() {
        let config = PerformanceConfig {
            warm_up_iterations: 5,
            test_iterations: 20,
            concurrent_users: 5,
            large_document_size_mb: 1,
            batch_sizes: vec![1, 5],
            timeout_seconds: 10,
            enable_ai_tests: false,
            memory_pressure_test: false,
        };

        let validator = PerformanceValidator::new(config);
        let results = validator.run_performance_validation().await.unwrap();
        
        assert!(results.document_creation.operation_times.len() > 0);
        assert!(results.document_retrieval.operation_times.len() > 0);
    }
}

/// CLI runner for performance validation
pub async fn run_performance_validation_suite() -> Result<()> {
    let config = PerformanceConfig {
        warm_up_iterations: std::env::var("PERF_WARMUP")
            .unwrap_or("10".to_string())
            .parse()
            .unwrap_or(10),
        test_iterations: std::env::var("PERF_ITERATIONS")
            .unwrap_or("100".to_string())
            .parse()
            .unwrap_or(100),
        concurrent_users: std::env::var("PERF_CONCURRENT_USERS")
            .unwrap_or("50".to_string())
            .parse()
            .unwrap_or(50),
        large_document_size_mb: std::env::var("PERF_LARGE_DOC_MB")
            .unwrap_or("10".to_string())
            .parse()
            .unwrap_or(10),
        batch_sizes: vec![1, 10, 50, 100],
        timeout_seconds: 60,
        enable_ai_tests: std::env::var("PERF_ENABLE_AI").is_ok(),
        memory_pressure_test: std::env::var("PERF_MEMORY_STRESS")
            .unwrap_or("true".to_string())
            .parse()
            .unwrap_or(true),
    };

    println!("🎯 WriteMagic Performance Validation Suite");
    println!("==========================================");
    println!("Configuration:");
    println!("  - Test Iterations: {}", config.test_iterations);
    println!("  - Concurrent Users: {}", config.concurrent_users);
    println!("  - Large Doc Size: {}MB", config.large_document_size_mb);
    println!("  - AI Testing: {}", config.enable_ai_tests);
    println!("  - Memory Stress: {}", config.memory_pressure_test);
    println!();

    let validator = PerformanceValidator::new(config);
    let _results = validator.run_performance_validation().await?;

    Ok(())
}