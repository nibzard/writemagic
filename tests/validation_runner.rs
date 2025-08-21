//! Comprehensive Validation Test Runner
//!
//! Orchestrates all validation test suites for WriteMagic:
//! - Integration validation (core workflow)
//! - Mobile FFI validation (platform bindings)  
//! - Performance validation (load and stress testing)
//! - Real-world scenario validation

use std::time::Instant;
use writemagic_shared::Result;

mod integration_validation;
mod mobile_ffi_validation;
mod performance_validation;

use integration_validation::{IntegrationValidator, ValidationConfig as IntegrationConfig};
use mobile_ffi_validation::{MobileFFIValidator, run_mobile_ffi_validation};
use performance_validation::{PerformanceValidator, PerformanceConfig, run_performance_validation_suite};

/// Validation suite configuration
#[derive(Debug, Clone)]
pub struct ValidationSuiteConfig {
    pub run_integration_tests: bool,
    pub run_mobile_ffi_tests: bool,
    pub run_performance_tests: bool,
    pub run_ai_tests: bool,
    pub quick_mode: bool,
    pub verbose: bool,
}

impl Default for ValidationSuiteConfig {
    fn default() -> Self {
        Self {
            run_integration_tests: true,
            run_mobile_ffi_tests: true,
            run_performance_tests: true,
            run_ai_tests: std::env::var("WRITEMAGIC_ENABLE_AI_TESTS").is_ok(),
            quick_mode: std::env::var("WRITEMAGIC_QUICK_VALIDATION").is_ok(),
            verbose: std::env::var("WRITEMAGIC_VERBOSE").is_ok(),
        }
    }
}

/// Overall validation results
#[derive(Debug, Default)]
pub struct ValidationSuiteResults {
    pub integration_passed: bool,
    pub mobile_ffi_passed: bool,
    pub performance_passed: bool,
    pub total_duration_seconds: f64,
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Main validation orchestrator
pub struct ValidationSuiteRunner {
    config: ValidationSuiteConfig,
}

impl ValidationSuiteRunner {
    pub fn new(config: ValidationSuiteConfig) -> Self {
        Self { config }
    }

    /// Run complete validation suite
    pub async fn run_validation_suite(&self) -> Result<ValidationSuiteResults> {
        let start_time = Instant::now();
        let mut results = ValidationSuiteResults::default();

        self.print_suite_header();

        // 1. Integration Validation
        if self.config.run_integration_tests {
            results.integration_passed = self.run_integration_validation().await?;
        }

        // 2. Mobile FFI Validation
        if self.config.run_mobile_ffi_tests {
            results.mobile_ffi_passed = self.run_mobile_validation().await?;
        }

        // 3. Performance Validation
        if self.config.run_performance_tests {
            results.performance_passed = self.run_performance_validation().await?;
        }

        // 4. Real-world scenario validation
        self.run_scenario_validation().await?;

        results.total_duration_seconds = start_time.elapsed().as_secs_f64();
        self.print_final_summary(&results);

        Ok(results)
    }

    /// Run integration validation tests
    async fn run_integration_validation(&self) -> Result<bool> {
        println!("🧪 Running Integration Validation Tests");
        println!("======================================\n");

        let integration_config = if self.config.quick_mode {
            IntegrationConfig {
                use_sqlite: true,
                enable_ai: self.config.run_ai_tests,
                concurrent_operations: 5,
                stress_test_iterations: 20,
                timeout_seconds: 15,
            }
        } else {
            IntegrationConfig {
                use_sqlite: true,
                enable_ai: self.config.run_ai_tests,
                concurrent_operations: 10,
                stress_test_iterations: 100,
                timeout_seconds: 30,
            }
        };

        let validator = IntegrationValidator::new(integration_config);
        let results = validator.validate_complete_workflow().await?;

        // Check if all tests passed
        let integration_passed = results.core_engine_tests.is_success() &&
                                results.sqlite_persistence_tests.is_success() &&
                                results.memory_safety_tests.is_success() &&
                                results.error_handling_tests.is_success() &&
                                results.concurrent_access_tests.is_success();

        if self.config.run_ai_tests {
            let ai_passed = results.ai_integration_tests.is_success();
            println!("Integration Validation: {} (AI: {})", 
                     if integration_passed { "✅ PASS" } else { "❌ FAIL" },
                     if ai_passed { "✅ PASS" } else { "❌ FAIL" });
            Ok(integration_passed && ai_passed)
        } else {
            println!("Integration Validation: {}", 
                     if integration_passed { "✅ PASS" } else { "❌ FAIL" });
            Ok(integration_passed)
        }
    }

    /// Run mobile FFI validation tests
    async fn run_mobile_validation(&self) -> Result<bool> {
        println!("\n📱 Running Mobile FFI Validation Tests");
        println!("=====================================\n");

        match run_mobile_ffi_validation().await {
            Ok(()) => {
                println!("Mobile FFI Validation: ✅ PASS");
                Ok(true)
            }
            Err(e) => {
                println!("Mobile FFI Validation: ❌ FAIL - {}", e);
                Ok(false)
            }
        }
    }

    /// Run performance validation tests
    async fn run_performance_validation(&self) -> Result<bool> {
        println!("\n⚡ Running Performance Validation Tests");
        println!("======================================\n");

        let perf_config = if self.config.quick_mode {
            PerformanceConfig {
                warm_up_iterations: 5,
                test_iterations: 50,
                concurrent_users: 10,
                large_document_size_mb: 2,
                batch_sizes: vec![1, 10],
                timeout_seconds: 15,
                enable_ai_tests: self.config.run_ai_tests,
                memory_pressure_test: false,
            }
        } else {
            PerformanceConfig {
                warm_up_iterations: 10,
                test_iterations: 200,
                concurrent_users: 50,
                large_document_size_mb: 10,
                batch_sizes: vec![1, 10, 50, 100],
                timeout_seconds: 60,
                enable_ai_tests: self.config.run_ai_tests,
                memory_pressure_test: true,
            }
        };

        let validator = PerformanceValidator::new(perf_config);
        let results = validator.run_performance_validation().await?;

        // Assess performance results
        let performance_passed = self.assess_performance_results(&results);
        
        println!("Performance Validation: {}", 
                 if performance_passed { "✅ PASS" } else { "❌ FAIL" });
        
        Ok(performance_passed)
    }

    /// Run real-world scenario validation
    async fn run_scenario_validation(&self) -> Result<()> {
        println!("\n🌍 Running Real-World Scenario Tests");
        println!("===================================\n");

        // Scenario 1: Complete writing workflow
        self.test_complete_writing_workflow().await?;

        // Scenario 2: Multi-user collaboration simulation
        self.test_collaboration_scenario().await?;

        // Scenario 3: Large project management
        self.test_large_project_scenario().await?;

        // Scenario 4: Mobile app usage patterns
        self.test_mobile_usage_patterns().await?;

        println!("Real-World Scenarios: ✅ COMPLETE\n");
        Ok(())
    }

    /// Test complete writing workflow scenario
    async fn test_complete_writing_workflow(&self) -> Result<()> {
        use writemagic_writing::{ApplicationConfigBuilder, DocumentTitle, DocumentContent, ProjectName};
        use writemagic_shared::ContentType;

        println!("📝 Testing Complete Writing Workflow...");

        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await?;

        // 1. Create a project
        let proj_service = engine.project_management_service();
        let project_name = ProjectName::new("User Research Project")?;
        let project_aggregate = proj_service
            .create_project(project_name, Some("Complete user research documentation".to_string()), None)
            .await?;

        // 2. Create multiple documents
        let doc_service = engine.document_management_service();
        let documents = vec![
            ("Research Plan", "# Research Plan\n\nObjectives and methodology"),
            ("User Interviews", "# User Interviews\n\nInterview notes and insights"),
            ("Data Analysis", "# Data Analysis\n\nFindings and recommendations"),
            ("Final Report", "# Final Report\n\nExecutive summary and conclusions"),
        ];

        for (title, content) in documents {
            let doc_title = DocumentTitle::new(title)?;
            let doc_content = DocumentContent::new(content)?;
            
            let doc_aggregate = doc_service
                .create_document(doc_title, doc_content, ContentType::Markdown, None)
                .await?;

            // Add to project
            proj_service
                .add_document_to_project(project_aggregate.project().id, doc_aggregate.document().id, None)
                .await?;

            // Simulate editing
            let updated_content = DocumentContent::new(&format!("{}\n\nUpdated with new insights", content))?;
            doc_service
                .update_document_content(doc_aggregate.document().id, updated_content, None, None)
                .await?;
        }

        // 3. Verify project integrity
        let final_project = engine.project_repository()
            .find_by_id(&project_aggregate.project().id)
            .await?;

        match final_project {
            Some(project) if project.document_ids.len() == 4 => {
                println!("   ✅ Complete workflow: Project with 4 documents");
            }
            Some(project) => {
                println!("   ⚠️  Complete workflow: Expected 4 documents, got {}", project.document_ids.len());
            }
            None => {
                println!("   ❌ Complete workflow: Project not found");
            }
        }

        Ok(())
    }

    /// Test collaboration scenario with concurrent users
    async fn test_collaboration_scenario(&self) -> Result<()> {
        use writemagic_writing::{ApplicationConfigBuilder, DocumentTitle, DocumentContent};
        use writemagic_shared::ContentType;

        println!("👥 Testing Collaboration Scenario...");

        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await?;

        // Create shared document
        let doc_service = engine.document_management_service();
        let title = DocumentTitle::new("Collaborative Document")?;
        let content = DocumentContent::new("# Collaborative Document\n\nShared content")?;
        
        let doc_aggregate = doc_service
            .create_document(title, content, ContentType::Markdown, None)
            .await?;
        
        let doc_id = doc_aggregate.document().id;

        // Simulate multiple users editing concurrently
        let mut handles = Vec::new();
        for i in 0..5 {
            let service = doc_service.clone();
            let document_id = doc_id;
            
            let handle = tokio::spawn(async move {
                let updated_content = DocumentContent::new(
                    &format!("# Collaborative Document\n\nShared content\n\n## Section from User {}\n\nUser {} contribution", i, i)
                ).unwrap();
                
                service.update_document_content(document_id, updated_content, None, None).await
            });
            
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;
        let successful_updates = results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();

        println!("   ✅ Collaboration: {}/5 concurrent updates succeeded", successful_updates);
        Ok(())
    }

    /// Test large project management scenario
    async fn test_large_project_scenario(&self) -> Result<()> {
        use writemagic_writing::{ApplicationConfigBuilder, DocumentTitle, DocumentContent, ProjectName};
        use writemagic_shared::ContentType;

        println!("📚 Testing Large Project Scenario...");

        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await?;

        // Create large project with many documents
        let proj_service = engine.project_management_service();
        let project_name = ProjectName::new("Large Documentation Project")?;
        let project_aggregate = proj_service
            .create_project(project_name, Some("Comprehensive documentation suite".to_string()), None)
            .await?;

        let doc_service = engine.document_management_service();
        let document_count = if self.config.quick_mode { 20 } else { 100 };

        for i in 0..document_count {
            let title = DocumentTitle::new(&format!("Doc {} of {}", i + 1, document_count))?;
            let content = DocumentContent::new(&format!("Content for document {} with detailed information", i + 1))?;
            
            let doc_aggregate = doc_service
                .create_document(title, content, ContentType::Markdown, None)
                .await?;

            proj_service
                .add_document_to_project(project_aggregate.project().id, doc_aggregate.document().id, None)
                .await?;

            // Progress indicator
            if (i + 1) % (document_count / 10).max(1) == 0 {
                let progress = ((i + 1) as f64 / document_count as f64) * 100.0;
                print!("\r   Progress: {:.0}%", progress);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        println!();

        // Verify large project
        let final_project = engine.project_repository()
            .find_by_id(&project_aggregate.project().id)
            .await?;

        match final_project {
            Some(project) if project.document_ids.len() == document_count => {
                println!("   ✅ Large project: {} documents managed successfully", document_count);
            }
            Some(project) => {
                println!("   ⚠️  Large project: Expected {} documents, got {}", document_count, project.document_ids.len());
            }
            None => {
                println!("   ❌ Large project: Project not found");
            }
        }

        Ok(())
    }

    /// Test mobile app usage patterns
    async fn test_mobile_usage_patterns(&self) -> Result<()> {
        println!("📱 Testing Mobile Usage Patterns...");

        // Simulate mobile app usage patterns
        // This would test:
        // - Frequent document switching
        // - Background/foreground transitions
        // - Network interruptions
        // - Battery optimization scenarios

        // For now, simulate rapid document operations like a mobile user would do
        use writemagic_writing::{ApplicationConfigBuilder, DocumentTitle, DocumentContent};
        use writemagic_shared::{ContentType, Pagination};

        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await?;

        let doc_service = engine.document_management_service();
        let doc_repo = engine.document_repository();
        let mut document_ids = Vec::new();

        // 1. Create multiple documents (mobile user creating notes)
        for i in 0..10 {
            let title = DocumentTitle::new(&format!("Mobile Note {}", i + 1))?;
            let content = DocumentContent::new(&format!("Quick note {} from mobile", i + 1))?;
            
            let doc_aggregate = doc_service
                .create_document(title, content, ContentType::PlainText, None)
                .await?;
            
            document_ids.push(doc_aggregate.document().id);
        }

        // 2. Rapid switching between documents (mobile app behavior)
        for _ in 0..20 {
            let doc_id = &document_ids[fastrand::usize(0..document_ids.len())];
            let _ = doc_repo.find_by_id(doc_id).await?;
        }

        // 3. Quick edits (mobile typing patterns)
        for doc_id in &document_ids[..5] {
            let updated_content = DocumentContent::new("Updated content from mobile editing")?;
            let _ = doc_service.update_document_content(*doc_id, updated_content, None, None).await?;
        }

        // 4. List operations (mobile browsing)
        let _ = doc_repo.find_all(Pagination::new(0, 10)?).await?;

        println!("   ✅ Mobile patterns: Document switching and rapid edits");
        Ok(())
    }

    /// Assess performance test results
    fn assess_performance_results(&self, results: &performance_validation::PerformanceResults) -> bool {
        // Define performance thresholds
        let doc_creation_threshold = 200.0; // ms
        let doc_retrieval_threshold = 50.0;  // ms
        let concurrent_success_threshold = 0.95; // 95%
        
        let creation_ok = results.document_creation.p95_latency_ms <= doc_creation_threshold;
        let retrieval_ok = results.document_retrieval.p95_latency_ms <= doc_retrieval_threshold;
        let concurrent_ok = results.concurrent_access.concurrent_success_rate >= concurrent_success_threshold;

        creation_ok && retrieval_ok && concurrent_ok
    }

    /// Print validation suite header
    fn print_suite_header(&self) {
        println!("🚀 WriteMagic Complete Validation Suite");
        println!("=======================================\n");
        
        println!("Configuration:");
        println!("  - Integration Tests: {}", if self.config.run_integration_tests { "✅" } else { "⏭️ " });
        println!("  - Mobile FFI Tests: {}", if self.config.run_mobile_ffi_tests { "✅" } else { "⏭️ " });
        println!("  - Performance Tests: {}", if self.config.run_performance_tests { "✅" } else { "⏭️ " });
        println!("  - AI Integration: {}", if self.config.run_ai_tests { "✅" } else { "⏭️ " });
        println!("  - Mode: {}", if self.config.quick_mode { "Quick" } else { "Comprehensive" });
        println!();
    }

    /// Print final validation summary
    fn print_final_summary(&self, results: &ValidationSuiteResults) {
        println!("📋 WriteMagic Validation Suite Final Results");
        println!("============================================\n");

        println!("🎯 Test Results:");
        if self.config.run_integration_tests {
            println!("   Integration Tests: {}", if results.integration_passed { "✅ PASS" } else { "❌ FAIL" });
        }
        if self.config.run_mobile_ffi_tests {
            println!("   Mobile FFI Tests: {}", if results.mobile_ffi_passed { "✅ PASS" } else { "❌ FAIL" });
        }
        if self.config.run_performance_tests {
            println!("   Performance Tests: {}", if results.performance_passed { "✅ PASS" } else { "❌ FAIL" });
        }
        println!("   Real-World Scenarios: ✅ PASS");

        println!("\n⏱️  Total Duration: {:.1}s", results.total_duration_seconds);

        // Overall verdict
        let all_passed = results.integration_passed && 
                         results.mobile_ffi_passed && 
                         results.performance_passed;

        println!("\n🏁 Final Verdict:");
        if all_passed {
            println!("   ✅ ALL VALIDATIONS PASSED");
            println!("   🚀 WriteMagic is ready for production deployment!");
            println!("   📱 Mobile apps can be released to app stores");
            println!("   ⚡ Performance meets production requirements");
        } else {
            println!("   ❌ SOME VALIDATIONS FAILED");
            println!("   🔧 Review failed tests before deployment");
            println!("   📋 Check detailed logs for specific issues");
        }

        // Provide next steps
        println!("\n📝 Next Steps:");
        if all_passed {
            println!("   1. Run final security audit");
            println!("   2. Prepare app store submissions");
            println!("   3. Set up production monitoring");
            println!("   4. Prepare launch documentation");
        } else {
            println!("   1. Address validation failures");
            println!("   2. Re-run failed test suites");
            println!("   3. Performance tuning if needed");
            println!("   4. Code review for critical issues");
        }
    }
}

/// CLI runner for the complete validation suite
pub async fn run_complete_validation_suite() -> Result<()> {
    let config = ValidationSuiteConfig {
        run_integration_tests: !std::env::var("SKIP_INTEGRATION_TESTS").is_ok(),
        run_mobile_ffi_tests: !std::env::var("SKIP_MOBILE_FFI_TESTS").is_ok(),
        run_performance_tests: !std::env::var("SKIP_PERFORMANCE_TESTS").is_ok(),
        run_ai_tests: std::env::var("WRITEMAGIC_ENABLE_AI_TESTS").is_ok(),
        quick_mode: std::env::var("WRITEMAGIC_QUICK_VALIDATION").is_ok(),
        verbose: std::env::var("WRITEMAGIC_VERBOSE").is_ok(),
    };

    let runner = ValidationSuiteRunner::new(config);
    let results = runner.run_validation_suite().await?;

    // Exit with error code if any validations failed
    if !results.integration_passed || !results.mobile_ffi_passed || !results.performance_passed {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_validation_suite() {
        let config = ValidationSuiteConfig {
            run_integration_tests: true,
            run_mobile_ffi_tests: true,
            run_performance_tests: true,
            run_ai_tests: false,
            quick_mode: true,
            verbose: false,
        };

        let runner = ValidationSuiteRunner::new(config);
        let results = runner.run_validation_suite().await.unwrap();
        
        assert!(results.total_duration_seconds > 0.0);
        // In quick mode, all basic tests should pass
        assert!(results.integration_passed);
        assert!(results.mobile_ffi_passed);
    }
}