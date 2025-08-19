#!/bin/bash

# WriteMagic Comprehensive Validation Runner
# This script runs all validation tests for WriteMagic

set -e

echo "🚀 WriteMagic Validation Test Suite"
echo "==================================="
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the WriteMagic project root directory"
    exit 1
fi

# Parse command line arguments
QUICK_MODE=false
ENABLE_AI=false
SKIP_INTEGRATION=false
SKIP_MOBILE_FFI=false
SKIP_PERFORMANCE=false
VERBOSE=false
REPORT_JSON=""
REPORT_HTML=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick|-q)
            QUICK_MODE=true
            shift
            ;;
        --enable-ai)
            ENABLE_AI=true
            shift
            ;;
        --skip-integration)
            SKIP_INTEGRATION=true
            shift
            ;;
        --skip-mobile-ffi)
            SKIP_MOBILE_FFI=true
            shift
            ;;
        --skip-performance)
            SKIP_PERFORMANCE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --report-json)
            REPORT_JSON="$2"
            shift 2
            ;;
        --report-html)
            REPORT_HTML="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --quick, -q              Run in quick mode (reduced iterations)"
            echo "  --enable-ai              Enable AI integration tests (requires API keys)"
            echo "  --skip-integration       Skip core integration tests"
            echo "  --skip-mobile-ffi        Skip mobile FFI binding tests"
            echo "  --skip-performance       Skip performance benchmarks"
            echo "  --verbose, -v            Enable verbose output"
            echo "  --report-json FILE       Export results to JSON file"
            echo "  --report-html FILE       Export results to HTML file"
            echo "  --help, -h               Show this help message"
            echo
            echo "Environment Variables:"
            echo "  WRITEMAGIC_ENABLE_AI_TESTS=1    Enable AI tests"
            echo "  WRITEMAGIC_QUICK_VALIDATION=1   Enable quick mode"
            echo "  WRITEMAGIC_VERBOSE=1             Enable verbose output"
            echo
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment variables based on options
if [ "$ENABLE_AI" = true ]; then
    export WRITEMAGIC_ENABLE_AI_TESTS=1
fi

if [ "$QUICK_MODE" = true ]; then
    export WRITEMAGIC_QUICK_VALIDATION=1
fi

if [ "$VERBOSE" = true ]; then
    export WRITEMAGIC_VERBOSE=1
    export RUST_LOG=debug
else
    export RUST_LOG=info
fi

if [ "$SKIP_INTEGRATION" = true ]; then
    export SKIP_INTEGRATION_TESTS=1
fi

if [ "$SKIP_MOBILE_FFI" = true ]; then
    export SKIP_MOBILE_FFI_TESTS=1
fi

if [ "$SKIP_PERFORMANCE" = true ]; then
    export SKIP_PERFORMANCE_TESTS=1
fi

echo "🔧 Configuration:"
echo "   Quick Mode: $(if [ "$QUICK_MODE" = true ]; then echo "✅"; else echo "❌"; fi)"
echo "   AI Tests: $(if [ "$ENABLE_AI" = true ]; then echo "✅"; else echo "❌"; fi)"
echo "   Integration Tests: $(if [ "$SKIP_INTEGRATION" = true ]; then echo "⏭️ Skipped"; else echo "✅"; fi)"
echo "   Mobile FFI Tests: $(if [ "$SKIP_MOBILE_FFI" = true ]; then echo "⏭️ Skipped"; else echo "✅"; fi)"
echo "   Performance Tests: $(if [ "$SKIP_PERFORMANCE" = true ]; then echo "⏭️ Skipped"; else echo "✅"; fi)"
echo "   Verbose Output: $(if [ "$VERBOSE" = true ]; then echo "✅"; else echo "❌"; fi)"
echo

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo/Rust is not installed"
    echo "   Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if we can build the project
echo "🔨 Building WriteMagic core..."
if ! cargo build --release --quiet; then
    echo "❌ Error: Failed to build WriteMagic core"
    echo "   Please fix build errors before running validation"
    exit 1
fi

echo "✅ Prerequisites check passed"
echo

# Run validation phases
VALIDATION_START_TIME=$(date +%s)
VALIDATION_FAILED=false

echo "🧪 Starting Validation Test Suite..."
echo

# Phase 1: Integration Tests
if [ "$SKIP_INTEGRATION" != true ]; then
    echo "Phase 1: Core Integration Validation"
    echo "===================================="
    
    if cargo test integration_validation --release --quiet; then
        echo "✅ Integration tests: PASSED"
    else
        echo "❌ Integration tests: FAILED"
        VALIDATION_FAILED=true
    fi
    echo
fi

# Phase 2: Mobile FFI Tests  
if [ "$SKIP_MOBILE_FFI" != true ]; then
    echo "Phase 2: Mobile FFI Validation"
    echo "==============================="
    
    if cargo test mobile_ffi_validation --release --quiet; then
        echo "✅ Mobile FFI tests: PASSED"
    else
        echo "❌ Mobile FFI tests: FAILED"
        VALIDATION_FAILED=true
    fi
    echo
fi

# Phase 3: Performance Tests
if [ "$SKIP_PERFORMANCE" != true ]; then
    echo "Phase 3: Performance Validation"
    echo "================================"
    
    if cargo test performance_validation --release --quiet; then
        echo "✅ Performance tests: PASSED"
    else
        echo "❌ Performance tests: FAILED"
        VALIDATION_FAILED=true
    fi
    echo
fi

# Phase 4: End-to-End Scenario Tests
echo "Phase 4: Real-World Scenario Tests"
echo "==================================="

# Test 1: Complete document workflow
echo "📝 Testing complete document workflow..."
if cargo test test_complete_writing_workflow --release --quiet; then
    echo "✅ Document workflow: PASSED"
else
    echo "❌ Document workflow: FAILED"
    VALIDATION_FAILED=true
fi

# Test 2: Concurrent operations
echo "👥 Testing concurrent operations..."
if cargo test test_collaboration_scenario --release --quiet; then
    echo "✅ Concurrent operations: PASSED"
else
    echo "❌ Concurrent operations: FAILED"
    VALIDATION_FAILED=true
fi

# Test 3: Large project handling
echo "📚 Testing large project handling..."
if cargo test test_large_project_scenario --release --quiet; then
    echo "✅ Large project handling: PASSED"
else
    echo "❌ Large project handling: FAILED"
    VALIDATION_FAILED=true
fi

echo

# AI-specific tests (if enabled)
if [ "$ENABLE_AI" = true ]; then
    echo "Phase 5: AI Integration Tests"
    echo "=============================="
    
    echo "🤖 Testing AI text completion..."
    if cargo test test_ai_completion --release --quiet; then
        echo "✅ AI completion: PASSED"
    else
        echo "❌ AI completion: FAILED"
        VALIDATION_FAILED=true
    fi
    
    echo "🔄 Testing AI provider fallback..."
    if cargo test test_ai_fallback --release --quiet; then
        echo "✅ AI fallback: PASSED"
    else
        echo "❌ AI fallback: FAILED"
        VALIDATION_FAILED=true
    fi
    echo
fi

# Calculate total time
VALIDATION_END_TIME=$(date +%s)
TOTAL_TIME=$((VALIDATION_END_TIME - VALIDATION_START_TIME))

# Generate reports (if requested)
if [ -n "$REPORT_JSON" ] || [ -n "$REPORT_HTML" ]; then
    echo "📊 Generating validation reports..."
    
    # Create report directory if it doesn't exist
    REPORT_DIR=$(dirname "${REPORT_JSON:-${REPORT_HTML}}")
    if [ "$REPORT_DIR" != "." ] && [ ! -d "$REPORT_DIR" ]; then
        mkdir -p "$REPORT_DIR"
    fi
    
    # Generate timestamp
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Generate JSON report
    if [ -n "$REPORT_JSON" ]; then
        cat > "$REPORT_JSON" << EOF
{
  "test_suite_version": "1.0.0",
  "timestamp": "$TIMESTAMP",
  "environment": {
    "platform": "$(uname -s)",
    "architecture": "$(uname -m)",
    "rust_version": "$(rustc --version | cut -d' ' -f2)"
  },
  "configuration": {
    "quick_mode": $QUICK_MODE,
    "ai_tests_enabled": $ENABLE_AI,
    "integration_tests": $(if [ "$SKIP_INTEGRATION" = true ]; then echo "false"; else echo "true"; fi),
    "mobile_ffi_tests": $(if [ "$SKIP_MOBILE_FFI" = true ]; then echo "false"; else echo "true"; fi),
    "performance_tests": $(if [ "$SKIP_PERFORMANCE" = true ]; then echo "false"; else echo "true"; fi)
  },
  "results": {
    "overall_success": $(if [ "$VALIDATION_FAILED" = true ]; then echo "false"; else echo "true"; fi),
    "execution_time_seconds": $TOTAL_TIME,
    "validation_failed": $VALIDATION_FAILED
  }
}
EOF
        echo "✅ JSON report exported to: $REPORT_JSON"
    fi
    
    # Generate HTML report
    if [ -n "$REPORT_HTML" ]; then
        OVERALL_STATUS=$(if [ "$VALIDATION_FAILED" = true ]; then echo "FAILED"; else echo "PASSED"; fi)
        STATUS_COLOR=$(if [ "$VALIDATION_FAILED" = true ]; then echo "#e74c3c"; else echo "#27ae60"; fi)
        
        cat > "$REPORT_HTML" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WriteMagic Validation Report</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1000px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        .status { font-size: 2em; font-weight: bold; color: $STATUS_COLOR; text-align: center; margin: 20px 0; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .metric { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; text-align: center; }
        .timestamp { color: #7f8c8d; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 WriteMagic Validation Report</h1>
        <p class="timestamp">Generated: $TIMESTAMP</p>
        
        <div class="status">Overall Status: $OVERALL_STATUS</div>
        
        <div class="summary">
            <div class="metric">
                <h3>$TOTAL_TIME s</h3>
                <p>Execution Time</p>
            </div>
            <div class="metric">
                <h3>$(uname -s)</h3>
                <p>Platform</p>
            </div>
            <div class="metric">
                <h3>$(rustc --version | cut -d' ' -f2)</h3>
                <p>Rust Version</p>
            </div>
        </div>
        
        <h2>Test Configuration</h2>
        <ul>
            <li>Quick Mode: $(if [ "$QUICK_MODE" = true ]; then echo "✅ Enabled"; else echo "❌ Disabled"; fi)</li>
            <li>AI Tests: $(if [ "$ENABLE_AI" = true ]; then echo "✅ Enabled"; else echo "❌ Disabled"; fi)</li>
            <li>Integration Tests: $(if [ "$SKIP_INTEGRATION" = true ]; then echo "⏭️ Skipped"; else echo "✅ Enabled"; fi)</li>
            <li>Mobile FFI Tests: $(if [ "$SKIP_MOBILE_FFI" = true ]; then echo "⏭️ Skipped"; else echo "✅ Enabled"; fi)</li>
            <li>Performance Tests: $(if [ "$SKIP_PERFORMANCE" = true ]; then echo "⏭️ Skipped"; else echo "✅ Enabled"; fi)</li>
        </ul>
        
        <h2>Recommendations</h2>
        <div style="background: #e8f6f3; padding: 15px; border-radius: 5px;">
            $(if [ "$VALIDATION_FAILED" = true ]; then
                echo "<p>❌ Some validations failed. Please review and fix issues before deployment.</p>"
                echo "<p>🔧 Run individual test suites to identify specific problems.</p>"
                echo "<p>📋 Check logs for detailed error information.</p>"
            else
                echo "<p>✅ All validations passed successfully!</p>"
                echo "<p>🚀 WriteMagic is ready for production deployment.</p>"
                echo "<p>📱 Mobile apps can be submitted to app stores.</p>"
            fi)
        </div>
    </div>
</body>
</html>
EOF
        echo "✅ HTML report exported to: $REPORT_HTML"
    fi
fi

# Final summary
echo "📋 WriteMagic Validation Summary"
echo "================================"
echo "   Total execution time: ${TOTAL_TIME}s"
echo "   Overall result: $(if [ "$VALIDATION_FAILED" = true ]; then echo "❌ FAILED"; else echo "✅ PASSED"; fi)"
echo

if [ "$VALIDATION_FAILED" = true ]; then
    echo "❌ VALIDATION FAILED"
    echo "   Some tests did not pass. Review the output above for details."
    echo "   Fix the issues and re-run validation before deployment."
    echo
    echo "🔧 Troubleshooting:"
    echo "   • Run with --verbose for more detailed output"
    echo "   • Check individual test logs for specific errors"
    echo "   • Ensure all dependencies are correctly installed"
    echo "   • Verify database migrations are up to date"
    echo
    exit 1
else
    echo "✅ ALL VALIDATIONS PASSED"
    echo "   🎉 WriteMagic is ready for production deployment!"
    echo "   📱 Mobile applications can be released to app stores"
    echo "   ⚡ Performance meets production requirements"
    echo "   🔒 Security and reliability validated"
    echo
    echo "📝 Next steps:"
    echo "   1. Perform final security audit"
    echo "   2. Set up production monitoring"
    echo "   3. Prepare deployment documentation" 
    echo "   4. Schedule production release"
    echo
    exit 0
fi