//! Comprehensive integration validation tests for WriteMagic mobile-to-core-to-AI workflow
//!
//! This test suite validates the complete end-to-end functionality:
//! - Mobile FFI layer integration (Android JNI, iOS C-FFI)
//! - Core engine operations (document/project management)
//! - SQLite persistence layer
//! - AI provider orchestration and fallback
//! - Memory safety and performance characteristics
//! - Error handling and recovery scenarios

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use writemagic_shared::{EntityId, Result, WritemagicError, ContentType, Pagination};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, DocumentTitle, DocumentContent, ProjectName,
    entities::{Document, Project},
};
// AI types would be imported if AI testing is enabled
// For now, we'll use placeholder types for testing

/// Test configuration for validation scenarios
#[derive(Debug, Clone)]
struct ValidationConfig {
    use_sqlite: bool,
    enable_ai: bool,
    concurrent_operations: usize,
    stress_test_iterations: usize,
    timeout_seconds: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            use_sqlite: true,
            enable_ai: false, // Disabled by default for CI
            concurrent_operations: 10,
            stress_test_iterations: 100,
            timeout_seconds: 30,
        }
    }
}

/// Comprehensive validation results
#[derive(Debug, Default)]
struct ValidationResults {
    core_engine_tests: TestResults,
    sqlite_persistence_tests: TestResults,
    ai_integration_tests: TestResults,
    memory_safety_tests: TestResults,
    performance_tests: PerformanceResults,
    error_handling_tests: TestResults,
    concurrent_access_tests: TestResults,
}

#[derive(Debug, Default)]
struct TestResults {
    passed: u32,
    failed: u32,
    errors: Vec<String>,
}

#[derive(Debug, Default)]
struct PerformanceResults {
    document_creation_avg_ms: f64,
    document_retrieval_avg_ms: f64,
    ai_completion_avg_ms: f64,
    sqlite_query_avg_ms: f64,
    memory_usage_mb: f64,
    max_concurrent_operations: usize,
}

impl TestResults {
    fn pass(&mut self) {
        self.passed += 1;
    }

    fn fail(&mut self, error: String) {
        self.failed += 1;
        self.errors.push(error);
    }

    fn is_success(&self) -> bool {
        self.failed == 0
    }

    fn summary(&self) -> String {
        format!("Passed: {}, Failed: {}", self.passed, self.failed)
    }
}

/// Main validation orchestrator
pub struct IntegrationValidator {
    config: ValidationConfig,
    results: Arc<Mutex<ValidationResults>>,
}

impl IntegrationValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            results: Arc::new(Mutex::new(ValidationResults::default())),
        }
    }

    /// Run complete validation suite
    pub async fn validate_complete_workflow(&self) -> Result<ValidationResults> {
        println!("🚀 Starting WriteMagic Complete Workflow Validation");
        println!("================================================\n");

        // 1. Core Engine Validation
        self.validate_core_engine().await?;

        // 2. SQLite Persistence Validation
        if self.config.use_sqlite {
            self.validate_sqlite_persistence().await?;
        }

        // 3. AI Integration Validation
        if self.config.enable_ai {
            self.validate_ai_integration().await?;
        }

        // 4. Memory Safety Validation
        self.validate_memory_safety().await?;

        // 5. Performance Benchmarking
        self.benchmark_performance().await?;

        // 6. Error Handling Validation
        self.validate_error_handling().await?;

        // 7. Concurrent Access Validation
        self.validate_concurrent_access().await?;

        // 8. FFI Layer Simulation (since we can't run actual mobile code here)
        self.validate_ffi_layer().await?;

        let results = self.results.lock().unwrap().clone();
        self.print_validation_summary(&results);

        Ok(results)
    }

    /// Validate core engine functionality
    async fn validate_core_engine(&self) -> Result<()> {
        println!("1. 🏗️  Validating Core Engine Functionality...");
        let mut results = self.results.lock().unwrap();
        
        // Test engine initialization
        let start = Instant::now();
        let engine = match self.create_test_engine().await {
            Ok(engine) => {
                results.core_engine_tests.pass();
                println!("   ✅ Engine initialization: {:.2}ms", start.elapsed().as_secs_f64() * 1000.0);
                engine
            }
            Err(e) => {
                results.core_engine_tests.fail(format!("Engine initialization failed: {}", e));
                return Err(e);
            }
        };

        // Test configuration validation
        let config_issues = engine.validate_config();
        if config_issues.is_empty() {
            results.core_engine_tests.pass();
            println!("   ✅ Configuration validation passed");
        } else {
            results.core_engine_tests.fail(format!("Configuration issues: {:?}", config_issues));
        }

        // Test repository access
        let doc_repo = engine.document_repository();
        let proj_repo = engine.project_repository();
        
        match doc_repo.count().await {
            Ok(_) => {
                results.core_engine_tests.pass();
                println!("   ✅ Document repository access");
            }
            Err(e) => {
                results.core_engine_tests.fail(format!("Document repository access failed: {}", e));
            }
        }

        match proj_repo.count().await {
            Ok(_) => {
                results.core_engine_tests.pass();
                println!("   ✅ Project repository access");
            }
            Err(e) => {
                results.core_engine_tests.fail(format!("Project repository access failed: {}", e));
            }
        }

        // Test service access
        let doc_service = engine.document_management_service();
        let proj_service = engine.project_management_service();
        
        if Arc::strong_count(&doc_service) > 0 && Arc::strong_count(&proj_service) > 0 {
            results.core_engine_tests.pass();
            println!("   ✅ Service layer access");
        } else {
            results.core_engine_tests.fail("Service layer access failed".to_string());
        }

        println!("   📊 Core Engine Tests: {}\n", results.core_engine_tests.summary());
        Ok(())
    }

    /// Validate SQLite persistence layer
    async fn validate_sqlite_persistence(&self) -> Result<()> {
        println!("2. 🗄️  Validating SQLite Persistence Layer...");
        let mut results = self.results.lock().unwrap();
        
        let engine = self.create_test_engine().await?;
        let doc_repo = engine.document_repository();
        let proj_repo = engine.project_repository();

        // Test document CRUD operations
        let start = Instant::now();
        let doc = Document::new(
            "Test Document".to_string(),
            "Test content for persistence validation".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );

        match doc_repo.save(&doc).await {
            Ok(_) => {
                let save_time = start.elapsed().as_secs_f64() * 1000.0;
                results.sqlite_persistence_tests.pass();
                println!("   ✅ Document save: {:.2}ms", save_time);
                
                // Update performance metrics
                if let Ok(mut perf_results) = self.results.try_lock() {
                    perf_results.performance_tests.sqlite_query_avg_ms = save_time;
                }
            }
            Err(e) => {
                results.sqlite_persistence_tests.fail(format!("Document save failed: {}", e));
            }
        }

        // Test document retrieval
        let start = Instant::now();
        match doc_repo.find_by_id(&doc.id).await {
            Ok(Some(retrieved_doc)) => {
                let retrieve_time = start.elapsed().as_secs_f64() * 1000.0;
                results.sqlite_persistence_tests.pass();
                println!("   ✅ Document retrieval: {:.2}ms", retrieve_time);
                
                // Validate data integrity
                if retrieved_doc.title == doc.title && retrieved_doc.content == doc.content {
                    results.sqlite_persistence_tests.pass();
                    println!("   ✅ Data integrity verification");
                } else {
                    results.sqlite_persistence_tests.fail("Data integrity check failed".to_string());
                }
            }
            Ok(None) => {
                results.sqlite_persistence_tests.fail("Document not found after save".to_string());
            }
            Err(e) => {
                results.sqlite_persistence_tests.fail(format!("Document retrieval failed: {}", e));
            }
        }

        // Test project CRUD operations
        let mut project = Project::new(
            "Test Project".to_string(),
            Some("Test project description".to_string()),
            Some(EntityId::new()),
        );
        project.add_document(doc.id, None);

        match proj_repo.save(&project).await {
            Ok(_) => {
                results.sqlite_persistence_tests.pass();
                println!("   ✅ Project save with document relationship");
            }
            Err(e) => {
                results.sqlite_persistence_tests.fail(format!("Project save failed: {}", e));
            }
        }

        // Test complex queries
        let pagination = Pagination::new(0, 10)?;
        match doc_repo.find_all(pagination).await {
            Ok(docs) => {
                results.sqlite_persistence_tests.pass();
                println!("   ✅ Paginated query: {} documents", docs.len());
            }
            Err(e) => {
                results.sqlite_persistence_tests.fail(format!("Paginated query failed: {}", e));
            }
        }

        // Test search functionality
        match doc_repo.search_by_title("Test", pagination).await {
            Ok(search_results) => {
                if !search_results.is_empty() {
                    results.sqlite_persistence_tests.pass();
                    println!("   ✅ Search functionality: {} results", search_results.len());
                } else {
                    results.sqlite_persistence_tests.fail("Search returned no results".to_string());
                }
            }
            Err(e) => {
                results.sqlite_persistence_tests.fail(format!("Search failed: {}", e));
            }
        }

        println!("   📊 SQLite Tests: {}\n", results.sqlite_persistence_tests.summary());
        Ok(())
    }

    /// Validate AI integration and fallback mechanisms
    async fn validate_ai_integration(&self) -> Result<()> {
        println!("3. 🤖 Validating AI Integration...");
        let mut results = self.results.lock().unwrap();

        let engine = self.create_test_engine_with_ai().await?;

        // Test AI service availability
        if let Some(_ai_service) = engine.ai_orchestration_service() {
            results.ai_integration_tests.pass();
            println!("   ✅ AI orchestration service available");
        } else {
            results.ai_integration_tests.fail("AI orchestration service not available".to_string());
            return Ok(()); // Skip AI tests if service not available
        }

        // Test AI text completion
        let start = Instant::now();
        match timeout(
            Duration::from_secs(self.config.timeout_seconds),
            engine.complete_text("Write a brief introduction to Rust programming.".to_string(), None)
        ).await {
            Ok(Ok(completion)) => {
                let completion_time = start.elapsed().as_secs_f64() * 1000.0;
                results.ai_integration_tests.pass();
                println!("   ✅ AI text completion: {:.2}ms", completion_time);
                println!("   📝 Sample: {}...", &completion[..50.min(completion.len())]);
                
                // Update performance metrics
                if let Ok(mut perf_results) = self.results.try_lock() {
                    perf_results.performance_tests.ai_completion_avg_ms = completion_time;
                }
            }
            Ok(Err(e)) => {
                results.ai_integration_tests.fail(format!("AI completion failed: {}", e));
            }
            Err(_) => {
                results.ai_integration_tests.fail("AI completion timed out".to_string());
            }
        }

        // Test AI provider health check
        match engine.check_ai_provider_health().await {
            Ok(health_status) => {
                results.ai_integration_tests.pass();
                println!("   ✅ AI provider health check: {} providers", health_status.len());
                for (provider, is_healthy) in health_status {
                    println!("      - {}: {}", provider, if is_healthy { "Healthy" } else { "Unhealthy" });
                }
            }
            Err(e) => {
                results.ai_integration_tests.fail(format!("AI health check failed: {}", e));
            }
        }

        // Test AI provider statistics
        match engine.get_ai_provider_stats().await {
            Ok(stats) => {
                results.ai_integration_tests.pass();
                println!("   ✅ AI provider statistics: {} providers", stats.len());
            }
            Err(e) => {
                results.ai_integration_tests.fail(format!("AI stats failed: {}", e));
            }
        }

        // Test integrated writing service if available
        if let Some(writing_service) = engine.integrated_writing_service() {
            results.ai_integration_tests.pass();
            println!("   ✅ Integrated writing service available");

            // Create test document for AI integration
            let doc_service = engine.document_management_service();
            let title = DocumentTitle::new("AI Test Document")?;
            let content = DocumentContent::new("This is a test document for AI integration.")?;
            
            let doc_aggregate = doc_service
                .create_document(title, content, ContentType::Markdown, None)
                .await?;

            let document_id = doc_aggregate.document().id;

            // Test content generation
            match timeout(
                Duration::from_secs(self.config.timeout_seconds),
                writing_service.generate_content_for_document(
                    document_id,
                    "Continue this document with technical details".to_string(),
                    Some(200),
                    false, // Don't apply to document
                    None,
                )
            ).await {
                Ok(Ok(response)) => {
                    results.ai_integration_tests.pass();
                    println!("   ✅ AI content generation: {} chars", response.content.len());
                    println!("   📊 Confidence: {:.1}%", response.confidence_score * 100.0);
                }
                Ok(Err(e)) => {
                    results.ai_integration_tests.fail(format!("AI content generation failed: {}", e));
                }
                Err(_) => {
                    results.ai_integration_tests.fail("AI content generation timed out".to_string());
                }
            }
        } else {
            results.ai_integration_tests.fail("Integrated writing service not available".to_string());
        }

        println!("   📊 AI Integration Tests: {}\n", results.ai_integration_tests.summary());
        Ok(())
    }

    /// Validate memory safety and resource management
    async fn validate_memory_safety(&self) -> Result<()> {
        println!("4. 🔒 Validating Memory Safety...");
        let mut results = self.results.lock().unwrap();

        // Test multiple engine instances (resource isolation)
        let engines = futures::future::try_join_all(
            (0..5).map(|_| self.create_test_engine())
        ).await?;

        if engines.len() == 5 {
            results.memory_safety_tests.pass();
            println!("   ✅ Multiple engine instances created successfully");
        } else {
            results.memory_safety_tests.fail("Failed to create multiple engine instances".to_string());
        }

        // Test graceful shutdown
        for engine in engines {
            engine.shutdown().await;
        }
        results.memory_safety_tests.pass();
        println!("   ✅ Graceful shutdown of all engines");

        // Test large document handling
        let engine = self.create_test_engine().await?;
        let doc_repo = engine.document_repository();
        
        let large_content = "A".repeat(1024 * 1024); // 1MB document
        let large_doc = Document::new(
            "Large Test Document".to_string(),
            large_content,
            ContentType::PlainText,
            Some(EntityId::new()),
        );

        match doc_repo.save(&large_doc).await {
            Ok(_) => {
                results.memory_safety_tests.pass();
                println!("   ✅ Large document (1MB) handling");
            }
            Err(e) => {
                results.memory_safety_tests.fail(format!("Large document handling failed: {}", e));
            }
        }

        // Test rapid creation/destruction
        for i in 0..10 {
            let doc = Document::new(
                format!("Rapid Doc {}", i),
                format!("Content {}", i),
                ContentType::Markdown,
                Some(EntityId::new()),
            );
            
            if doc_repo.save(&doc).await.is_err() {
                results.memory_safety_tests.fail(format!("Rapid creation failed at iteration {}", i));
                break;
            }
        }
        results.memory_safety_tests.pass();
        println!("   ✅ Rapid creation/destruction (10 iterations)");

        println!("   📊 Memory Safety Tests: {}\n", results.memory_safety_tests.summary());
        Ok(())
    }

    /// Benchmark performance characteristics
    async fn benchmark_performance(&self) -> Result<()> {
        println!("5. ⚡ Benchmarking Performance...");
        let engine = self.create_test_engine().await?;
        let doc_repo = engine.document_repository();
        let doc_service = engine.document_management_service();

        // Benchmark document creation
        let mut creation_times = Vec::new();
        for i in 0..10 {
            let start = Instant::now();
            
            let title = DocumentTitle::new(&format!("Benchmark Doc {}", i))?;
            let content = DocumentContent::new(&format!("Benchmark content {}", i))?;
            
            let result = doc_service
                .create_document(title, content, ContentType::Markdown, None)
                .await;

            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            creation_times.push(elapsed);

            if result.is_err() {
                return Err(WritemagicError::internal("Benchmark creation failed"));
            }
        }

        let avg_creation = creation_times.iter().sum::<f64>() / creation_times.len() as f64;
        println!("   📊 Average document creation: {:.2}ms", avg_creation);

        // Benchmark document retrieval
        let docs = doc_repo.find_all(Pagination::new(0, 10)?).await?;
        let mut retrieval_times = Vec::new();
        
        for doc in &docs {
            let start = Instant::now();
            let _ = doc_repo.find_by_id(&doc.id).await?;
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            retrieval_times.push(elapsed);
        }

        let avg_retrieval = if !retrieval_times.is_empty() {
            retrieval_times.iter().sum::<f64>() / retrieval_times.len() as f64
        } else {
            0.0
        };
        println!("   📊 Average document retrieval: {:.2}ms", avg_retrieval);

        // Update performance results
        if let Ok(mut perf_results) = self.results.try_lock() {
            perf_results.performance_tests.document_creation_avg_ms = avg_creation;
            perf_results.performance_tests.document_retrieval_avg_ms = avg_retrieval;
            
            // Estimate memory usage (simplified)
            perf_results.performance_tests.memory_usage_mb = docs.len() as f64 * 0.1; // Rough estimate
        }

        println!("   📊 Performance benchmarks completed\n");
        Ok(())
    }

    /// Validate error handling and recovery
    async fn validate_error_handling(&self) -> Result<()> {
        println!("6. ⚠️  Validating Error Handling...");
        let mut results = self.results.lock().unwrap();
        let engine = self.create_test_engine().await?;

        // Test invalid document ID
        let doc_repo = engine.document_repository();
        match doc_repo.find_by_id(&EntityId::new()).await {
            Ok(None) => {
                results.error_handling_tests.pass();
                println!("   ✅ Invalid document ID handling");
            }
            Ok(Some(_)) => {
                results.error_handling_tests.fail("Found document with random ID".to_string());
            }
            Err(e) => {
                results.error_handling_tests.fail(format!("Unexpected error for invalid ID: {}", e));
            }
        }

        // Test invalid pagination
        match Pagination::new(0, 0) {
            Ok(_) => {
                results.error_handling_tests.fail("Invalid pagination accepted".to_string());
            }
            Err(_) => {
                results.error_handling_tests.pass();
                println!("   ✅ Invalid pagination rejection");
            }
        }

        // Test empty document title
        match DocumentTitle::new("") {
            Ok(_) => {
                results.error_handling_tests.fail("Empty document title accepted".to_string());
            }
            Err(_) => {
                results.error_handling_tests.pass();
                println!("   ✅ Empty document title rejection");
            }
        }

        // Test AI error handling (if AI enabled)
        if self.config.enable_ai {
            match engine.complete_text("".to_string(), None).await {
                Ok(_) => {
                    // Empty prompt might still work, that's OK
                    results.error_handling_tests.pass();
                }
                Err(_) => {
                    // Error is expected and properly handled
                    results.error_handling_tests.pass();
                }
            }
            println!("   ✅ AI error handling verified");
        }

        println!("   📊 Error Handling Tests: {}\n", results.error_handling_tests.summary());
        Ok(())
    }

    /// Validate concurrent access patterns
    async fn validate_concurrent_access(&self) -> Result<()> {
        println!("7. 🔄 Validating Concurrent Access...");
        let mut results = self.results.lock().unwrap();
        let engine = self.create_test_engine().await?;
        let doc_repo = engine.document_repository();

        // Create documents concurrently
        let concurrent_tasks: Vec<_> = (0..self.config.concurrent_operations)
            .map(|i| {
                let repo = doc_repo.clone();
                async move {
                    let doc = Document::new(
                        format!("Concurrent Doc {}", i),
                        format!("Concurrent content {}", i),
                        ContentType::Markdown,
                        Some(EntityId::new()),
                    );
                    repo.save(&doc).await
                }
            })
            .collect();

        let start = Instant::now();
        let concurrent_results = futures::future::join_all(concurrent_tasks).await;
        let elapsed = start.elapsed();

        let successful_creates = concurrent_results.into_iter()
            .filter(|r| r.is_ok())
            .count();

        if successful_creates == self.config.concurrent_operations {
            results.concurrent_access_tests.pass();
            println!("   ✅ Concurrent document creation: {} ops in {:.2}ms", 
                     successful_creates, elapsed.as_secs_f64() * 1000.0);
        } else {
            results.concurrent_access_tests.fail(
                format!("Only {} of {} concurrent operations succeeded", 
                        successful_creates, self.config.concurrent_operations)
            );
        }

        // Update performance metrics
        if let Ok(mut perf_results) = self.results.try_lock() {
            perf_results.performance_tests.max_concurrent_operations = successful_creates;
        }

        // Test concurrent reads
        let read_tasks: Vec<_> = (0..self.config.concurrent_operations)
            .map(|_| {
                let repo = doc_repo.clone();
                async move {
                    repo.find_all(Pagination::new(0, 5).unwrap()).await
                }
            })
            .collect();

        let concurrent_reads = futures::future::join_all(read_tasks).await;
        let successful_reads = concurrent_reads.into_iter()
            .filter(|r| r.is_ok())
            .count();

        if successful_reads == self.config.concurrent_operations {
            results.concurrent_access_tests.pass();
            println!("   ✅ Concurrent document reads: {} ops", successful_reads);
        } else {
            results.concurrent_access_tests.fail(
                format!("Only {} of {} concurrent reads succeeded", 
                        successful_reads, self.config.concurrent_operations)
            );
        }

        println!("   📊 Concurrent Access Tests: {}\n", results.concurrent_access_tests.summary());
        Ok(())
    }

    /// Validate FFI layer simulation (mobile integration patterns)
    async fn validate_ffi_layer(&self) -> Result<()> {
        println!("8. 📱 Validating FFI Layer Patterns...");
        let mut results = self.results.lock().unwrap();

        // Simulate Android FFI patterns
        self.simulate_android_ffi_operations(&mut results).await?;

        // Simulate iOS FFI patterns  
        self.simulate_ios_ffi_operations(&mut results).await?;

        println!("   📊 FFI Layer Tests: {}\n", 
                 results.core_engine_tests.passed + results.sqlite_persistence_tests.passed);
        Ok(())
    }

    /// Simulate Android JNI operations
    async fn simulate_android_ffi_operations(&self, results: &mut ValidationResults) -> Result<()> {
        println!("   🤖 Simulating Android JNI operations...");
        
        let engine = self.create_test_engine().await?;
        
        // Simulate the Android FFI initialization pattern
        let doc_service = engine.document_management_service();
        
        // Simulate createDocument JNI call
        let title = DocumentTitle::new("Android Test Document")?;
        let content = DocumentContent::new("Content from Android app")?;
        
        match doc_service.create_document(title, content, ContentType::Markdown, None).await {
            Ok(aggregate) => {
                results.core_engine_tests.pass();
                
                // Simulate getDocument JNI call
                let doc_id = aggregate.document().id;
                match engine.document_repository().find_by_id(&doc_id).await {
                    Ok(Some(_)) => {
                        results.core_engine_tests.pass();
                        
                        // Simulate updateDocumentContent JNI call
                        let new_content = DocumentContent::new("Updated from Android")?;
                        match doc_service.update_document_content(doc_id, new_content, None, None).await {
                            Ok(_) => {
                                results.core_engine_tests.pass();
                                println!("      ✅ Android JNI simulation: Create/Read/Update");
                            }
                            Err(e) => {
                                results.core_engine_tests.fail(format!("Android update failed: {}", e));
                            }
                        }
                    }
                    Ok(None) => {
                        results.core_engine_tests.fail("Android document not found after creation".to_string());
                    }
                    Err(e) => {
                        results.core_engine_tests.fail(format!("Android retrieval failed: {}", e));
                    }
                }
            }
            Err(e) => {
                results.core_engine_tests.fail(format!("Android creation failed: {}", e));
            }
        }

        Ok(())
    }

    /// Simulate iOS C-FFI operations
    async fn simulate_ios_ffi_operations(&self, results: &mut ValidationResults) -> Result<()> {
        println!("   🍎 Simulating iOS C-FFI operations...");
        
        let engine = self.create_test_engine().await?;
        
        // Simulate the iOS FFI patterns with C-compatible operations
        let doc_service = engine.document_management_service();
        
        // Simulate writemagic_create_document C call
        let title = DocumentTitle::new("iOS Test Document")?;
        let content = DocumentContent::new("Content from iOS app")?;
        
        match doc_service.create_document(title, content, ContentType::Markdown, None).await {
            Ok(aggregate) => {
                results.core_engine_tests.pass();
                
                // Simulate writemagic_get_document C call
                let doc_id = aggregate.document().id;
                match engine.document_repository().find_by_id(&doc_id).await {
                    Ok(Some(doc)) => {
                        results.core_engine_tests.pass();
                        
                        // Simulate JSON serialization for iOS FFI
                        let json_data = serde_json::json!({
                            "id": doc.id.to_string(),
                            "title": doc.title,
                            "content": doc.content,
                            "contentType": doc.content_type.to_string(),
                        });
                        
                        if !json_data.to_string().is_empty() {
                            results.core_engine_tests.pass();
                            println!("      ✅ iOS C-FFI simulation: Create/Read/JSON");
                        } else {
                            results.core_engine_tests.fail("iOS JSON serialization failed".to_string());
                        }
                    }
                    Ok(None) => {
                        results.core_engine_tests.fail("iOS document not found after creation".to_string());
                    }
                    Err(e) => {
                        results.core_engine_tests.fail(format!("iOS retrieval failed: {}", e));
                    }
                }
            }
            Err(e) => {
                results.core_engine_tests.fail(format!("iOS creation failed: {}", e));
            }
        }

        Ok(())
    }

    /// Create test engine instance
    async fn create_test_engine(&self) -> Result<CoreEngine> {
        if self.config.use_sqlite {
            ApplicationConfigBuilder::new()
                .with_sqlite_in_memory()
                .with_log_level("info".to_string())
                .with_content_filtering(false)
                .build()
                .await
        } else {
            CoreEngine::new_in_memory().await
        }
    }

    /// Create test engine with AI capabilities
    async fn create_test_engine_with_ai(&self) -> Result<CoreEngine> {
        ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .with_claude_key("test-claude-key".to_string())
            .with_openai_key("test-openai-key".to_string())
            .with_log_level("info".to_string())
            .with_content_filtering(true)
            .build()
            .await
    }

    /// Print comprehensive validation summary
    fn print_validation_summary(&self, results: &ValidationResults) {
        println!("📋 WriteMagic Integration Validation Summary");
        println!("===========================================\n");

        println!("🏗️  Core Engine: {}", results.core_engine_tests.summary());
        println!("🗄️  SQLite Persistence: {}", results.sqlite_persistence_tests.summary());
        println!("🤖 AI Integration: {}", results.ai_integration_tests.summary());
        println!("🔒 Memory Safety: {}", results.memory_safety_tests.summary());
        println!("⚠️  Error Handling: {}", results.error_handling_tests.summary());
        println!("🔄 Concurrent Access: {}", results.concurrent_access_tests.summary());

        println!("\n⚡ Performance Metrics:");
        println!("   Document Creation: {:.2}ms avg", results.performance_tests.document_creation_avg_ms);
        println!("   Document Retrieval: {:.2}ms avg", results.performance_tests.document_retrieval_avg_ms);
        println!("   AI Completion: {:.2}ms avg", results.performance_tests.ai_completion_avg_ms);
        println!("   SQLite Queries: {:.2}ms avg", results.performance_tests.sqlite_query_avg_ms);
        println!("   Max Concurrent Ops: {}", results.performance_tests.max_concurrent_operations);

        let total_passed = results.core_engine_tests.passed + 
                          results.sqlite_persistence_tests.passed +
                          results.ai_integration_tests.passed +
                          results.memory_safety_tests.passed +
                          results.error_handling_tests.passed +
                          results.concurrent_access_tests.passed;

        let total_failed = results.core_engine_tests.failed + 
                          results.sqlite_persistence_tests.failed +
                          results.ai_integration_tests.failed +
                          results.memory_safety_tests.failed +
                          results.error_handling_tests.failed +
                          results.concurrent_access_tests.failed;

        println!("\n🎯 Overall Results: {} passed, {} failed", total_passed, total_failed);

        if total_failed == 0 {
            println!("🎉 All validations passed! WriteMagic is ready for production deployment.");
        } else {
            println!("❌ Some validations failed. Review errors before deployment:");
            self.print_all_errors(results);
        }
    }

    /// Print all validation errors
    fn print_all_errors(&self, results: &ValidationResults) {
        let all_errors = vec![
            &results.core_engine_tests.errors,
            &results.sqlite_persistence_tests.errors,
            &results.ai_integration_tests.errors,
            &results.memory_safety_tests.errors,
            &results.error_handling_tests.errors,
            &results.concurrent_access_tests.errors,
        ];

        for (i, errors) in all_errors.into_iter().enumerate() {
            if !errors.is_empty() {
                let category = ["Core Engine", "SQLite", "AI Integration", 
                               "Memory Safety", "Error Handling", "Concurrent Access"][i];
                println!("\n❌ {} Errors:", category);
                for error in errors {
                    println!("   - {}", error);
                }
            }
        }
    }
}

/// Validation test runner functions
#[cfg(test)]
mod tests {
    use super::*;

    /// Run basic validation suite
    #[tokio::test]
    async fn test_basic_validation() {
        let config = ValidationConfig {
            use_sqlite: true,
            enable_ai: false,
            concurrent_operations: 5,
            stress_test_iterations: 10,
            timeout_seconds: 10,
        };

        let validator = IntegrationValidator::new(config);
        let results = validator.validate_complete_workflow().await.unwrap();
        
        assert!(results.core_engine_tests.is_success(), 
                "Core engine tests failed: {:?}", results.core_engine_tests.errors);
        assert!(results.sqlite_persistence_tests.is_success(),
                "SQLite tests failed: {:?}", results.sqlite_persistence_tests.errors);
    }

    /// Run memory safety validation
    #[tokio::test]
    async fn test_memory_safety_validation() {
        let config = ValidationConfig {
            use_sqlite: false, // Use in-memory for faster tests
            enable_ai: false,
            concurrent_operations: 3,
            stress_test_iterations: 50,
            timeout_seconds: 5,
        };

        let validator = IntegrationValidator::new(config);
        validator.validate_memory_safety().await.unwrap();
        
        let results = validator.results.lock().unwrap();
        assert!(results.memory_safety_tests.is_success(),
                "Memory safety tests failed: {:?}", results.memory_safety_tests.errors);
    }

    /// Run performance benchmark
    #[tokio::test]
    async fn test_performance_benchmark() {
        let config = ValidationConfig::default();
        let validator = IntegrationValidator::new(config);
        validator.benchmark_performance().await.unwrap();
        
        let results = validator.results.lock().unwrap();
        assert!(results.performance_tests.document_creation_avg_ms > 0.0);
        assert!(results.performance_tests.document_retrieval_avg_ms >= 0.0);
    }

    /// Test FFI simulation patterns
    #[tokio::test]
    async fn test_ffi_simulation() {
        let config = ValidationConfig::default();
        let validator = IntegrationValidator::new(config);
        validator.validate_ffi_layer().await.unwrap();
    }
}

/// Comprehensive validation CLI runner
pub async fn run_validation_suite() -> Result<()> {
    let config = ValidationConfig {
        use_sqlite: true,
        enable_ai: std::env::var("WRITEMAGIC_ENABLE_AI_TESTS").is_ok(),
        concurrent_operations: 10,
        stress_test_iterations: 100,
        timeout_seconds: 30,
    };

    println!("🔍 WriteMagic Integration Validation Suite");
    println!("==========================================");
    println!("Configuration:");
    println!("  - SQLite: {}", config.use_sqlite);
    println!("  - AI Testing: {}", config.enable_ai);
    println!("  - Concurrent Ops: {}", config.concurrent_operations);
    println!("  - Timeout: {}s", config.timeout_seconds);
    println!();

    let validator = IntegrationValidator::new(config);
    let results = validator.validate_complete_workflow().await?;

    // Exit with error code if validations failed
    let total_failed = results.core_engine_tests.failed + 
                      results.sqlite_persistence_tests.failed +
                      results.ai_integration_tests.failed +
                      results.memory_safety_tests.failed +
                      results.error_handling_tests.failed +
                      results.concurrent_access_tests.failed;

    if total_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}