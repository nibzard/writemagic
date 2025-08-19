//! WriteMagic Validation Test Suite Main Runner
//!
//! Command-line interface for running comprehensive validation tests.

use clap::{Arg, Command, ArgAction};
use std::process;
use writemagic_validation_tests::{
    run_complete_validation_suite,
    generate_validation_report,
    export_validation_report_json,
    export_validation_report_html,
    ValidationSuiteRunner,
    ValidationSuiteConfig,
};

#[tokio::main]
async fn main() {
    let matches = Command::new("WriteMagic Validation Suite")
        .version(env!("CARGO_PKG_VERSION"))
        .author("WriteMagic Team")
        .about("Comprehensive validation testing for WriteMagic mobile and core systems")
        .arg(Arg::new("quick")
            .long("quick")
            .short('q')
            .help("Run in quick mode with reduced test iterations")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("skip-integration")
            .long("skip-integration")
            .help("Skip integration tests")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("skip-mobile-ffi")
            .long("skip-mobile-ffi")
            .help("Skip mobile FFI tests")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("skip-performance")
            .long("skip-performance")
            .help("Skip performance tests")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("enable-ai")
            .long("enable-ai")
            .help("Enable AI integration tests (requires API keys)")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("report-json")
            .long("report-json")
            .value_name("FILE")
            .help("Export validation report as JSON"))
        .arg(Arg::new("report-html")
            .long("report-html")
            .value_name("FILE")
            .help("Export validation report as HTML"))
        .arg(Arg::new("verbose")
            .long("verbose")
            .short('v')
            .help("Enable verbose output")
            .action(ArgAction::SetTrue))
        .get_matches();

    // Configure logging
    if matches.get_flag("verbose") {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Print banner
    print_banner();

    // Configure validation suite
    let config = ValidationSuiteConfig {
        run_integration_tests: !matches.get_flag("skip-integration"),
        run_mobile_ffi_tests: !matches.get_flag("skip-mobile-ffi"),
        run_performance_tests: !matches.get_flag("skip-performance"),
        run_ai_tests: matches.get_flag("enable-ai"),
        quick_mode: matches.get_flag("quick"),
        verbose: matches.get_flag("verbose"),
    };

    println!("🔧 Configuration:");
    println!("   Integration Tests: {}", if config.run_integration_tests { "✅" } else { "⏭️ Skipped" });
    println!("   Mobile FFI Tests: {}", if config.run_mobile_ffi_tests { "✅" } else { "⏭️ Skipped" });
    println!("   Performance Tests: {}", if config.run_performance_tests { "✅" } else { "⏭️ Skipped" });
    println!("   AI Integration: {}", if config.run_ai_tests { "✅" } else { "⏭️ Disabled" });
    println!("   Mode: {}", if config.quick_mode { "⚡ Quick" } else { "🔍 Comprehensive" });
    println!();

    // Run validation suite
    let runner = ValidationSuiteRunner::new(config);
    let validation_start = std::time::Instant::now();

    match runner.run_validation_suite().await {
        Ok(results) => {
            let duration = validation_start.elapsed();
            println!("\n✅ Validation suite completed in {:.1}s", duration.as_secs_f64());

            // Generate and export reports if requested
            if matches.contains_id("report-json") || matches.contains_id("report-html") {
                println!("📊 Generating validation report...");
                
                match generate_validation_report().await {
                    Ok(report) => {
                        // Export JSON report
                        if let Some(json_file) = matches.get_one::<String>("report-json") {
                            match export_validation_report_json(&report, json_file).await {
                                Ok(()) => println!("   ✅ JSON report exported to: {}", json_file),
                                Err(e) => eprintln!("   ❌ Failed to export JSON report: {}", e),
                            }
                        }

                        // Export HTML report
                        if let Some(html_file) = matches.get_one::<String>("report-html") {
                            match export_validation_report_html(&report, html_file).await {
                                Ok(()) => println!("   ✅ HTML report exported to: {}", html_file),
                                Err(e) => eprintln!("   ❌ Failed to export HTML report: {}", e),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to generate validation report: {}", e);
                    }
                }
            }

            // Exit with appropriate code
            let success = results.integration_passed && results.mobile_ffi_passed && results.performance_passed;
            if success {
                println!("\n🎉 All validations passed! WriteMagic is ready for production.");
                process::exit(0);
            } else {
                println!("\n⚠️  Some validations failed. Review results before deployment.");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Validation suite failed: {}", e);
            process::exit(1);
        }
    }
}

fn print_banner() {
    println!(r#"
╭─────────────────────────────────────────────────────────────╮
│                                                             │
│  🚀 WriteMagic Comprehensive Validation Suite              │
│                                                             │
│  Testing complete mobile-to-core-to-AI workflow            │
│  • Integration Testing                                      │
│  • Mobile FFI Validation                                   │
│  • Performance Benchmarking                                │
│  • AI Provider Testing                                     │
│  • Memory Safety Verification                              │
│  • Concurrent Access Validation                            │
│                                                             │
╰─────────────────────────────────────────────────────────────╯
"#);
    
    println!("📋 Test Categories:");
    println!("   🏗️  Core Engine Integration");
    println!("   🗄️  SQLite Persistence Layer");
    println!("   📱 Mobile FFI Bindings (Android JNI, iOS C-FFI)");
    println!("   🤖 AI Provider Orchestration & Fallback");
    println!("   ⚡ Performance & Load Testing");
    println!("   🔒 Memory Safety & Resource Management");
    println!("   🔄 Concurrent Access Patterns");
    println!("   🌍 Real-World Usage Scenarios");
    println!();
}

/// Run individual test suites (for CI/CD integration)
#[allow(dead_code)]
async fn run_individual_test_suite(suite_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match suite_name {
        "integration" => {
            use writemagic_validation_tests::{IntegrationValidator, integration_validation::ValidationConfig};
            
            let config = ValidationConfig::default();
            let validator = IntegrationValidator::new(config);
            let _results = validator.validate_complete_workflow().await?;
            Ok(())
        }
        "mobile-ffi" => {
            use writemagic_validation_tests::run_mobile_ffi_validation;
            run_mobile_ffi_validation().await.map_err(|e| e.into())
        }
        "performance" => {
            use writemagic_validation_tests::run_performance_validation_suite;
            run_performance_validation_suite().await.map_err(|e| e.into())
        }
        _ => {
            eprintln!("Unknown test suite: {}", suite_name);
            Err("Invalid test suite".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_validation_mode() {
        let config = ValidationSuiteConfig {
            run_integration_tests: true,
            run_mobile_ffi_tests: true,
            run_performance_tests: false, // Skip perf for quick test
            run_ai_tests: false,
            quick_mode: true,
            verbose: false,
        };

        let runner = ValidationSuiteRunner::new(config);
        let results = runner.run_validation_suite().await;
        
        // Should complete without errors in quick mode
        assert!(results.is_ok());
    }

    #[test]
    fn test_banner_display() {
        // Just verify the banner function doesn't panic
        print_banner();
    }
}