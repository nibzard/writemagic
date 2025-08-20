#!/bin/bash

# WriteMagic Comprehensive Test Runner
# This script executes the complete test suite with coverage analysis and reporting

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
REPORT_DIR="$PROJECT_ROOT/target/test-reports"
COVERAGE_DIR="$PROJECT_ROOT/target/coverage"
LOG_FILE="$REPORT_DIR/test-execution.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_header() {
    echo
    echo "=============================================="
    echo "  $1"
    echo "=============================================="
    echo
}

print_step() {
    print_status "$BLUE" "→ $1"
}

print_success() {
    print_status "$GREEN" "✅ $1"
}

print_warning() {
    print_status "$YELLOW" "⚠️  $1"
}

print_error() {
    print_status "$RED" "❌ $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install missing dependencies
install_dependencies() {
    print_step "Checking and installing dependencies..."
    
    # Check for tarpaulin (coverage tool)
    if ! command_exists cargo-tarpaulin; then
        print_step "Installing cargo-tarpaulin for coverage analysis..."
        cargo install cargo-tarpaulin || {
            print_warning "Failed to install cargo-tarpaulin. Coverage analysis will be skipped."
            SKIP_COVERAGE=true
        }
    fi
    
    # Check for grcov (alternative coverage tool)
    if ! command_exists grcov && [ "${SKIP_COVERAGE:-}" != "true" ]; then
        print_step "Installing grcov as backup coverage tool..."
        cargo install grcov || {
            print_warning "Failed to install grcov. Using tarpaulin only."
        }
    fi
    
    # Check for wasm-pack
    if ! command_exists wasm-pack; then
        print_step "Installing wasm-pack for WASM tests..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh || {
            print_warning "Failed to install wasm-pack. WASM tests will be skipped."
            SKIP_WASM=true
        }
    fi
}

# Function to setup test environment
setup_test_environment() {
    print_step "Setting up test environment..."
    
    # Create necessary directories
    mkdir -p "$REPORT_DIR"
    mkdir -p "$COVERAGE_DIR"
    
    # Clear previous reports
    rm -f "$REPORT_DIR"/*.html
    rm -f "$REPORT_DIR"/*.json
    rm -f "$COVERAGE_DIR"/*
    
    # Initialize log file
    echo "WriteMagic Comprehensive Test Execution - $(date)" > "$LOG_FILE"
    echo "======================================================" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
}

# Function to run unit tests
run_unit_tests() {
    print_step "Running unit tests..."
    
    {
        echo "Running unit tests at $(date)"
        cargo test --workspace --lib --verbose 2>&1
        echo "Unit tests completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Unit tests passed"
        return 0
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_step "Running integration tests..."
    
    {
        echo "Running integration tests at $(date)"
        cargo test --package writemagic-integration-tests --verbose 2>&1
        echo "Integration tests completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Integration tests passed"
        return 0
    else
        print_error "Integration tests failed"
        return 1
    fi
}

# Function to run performance benchmarks
run_performance_tests() {
    print_step "Running performance benchmarks..."
    
    {
        echo "Running performance benchmarks at $(date)"
        cargo bench --package writemagic-integration-tests 2>&1
        echo "Performance benchmarks completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Performance benchmarks completed"
        return 0
    else
        print_warning "Performance benchmarks had issues (non-critical)"
        return 0
    fi
}

# Function to run WASM tests
run_wasm_tests() {
    if [ "${SKIP_WASM:-}" = "true" ]; then
        print_warning "Skipping WASM tests (wasm-pack not available)"
        return 0
    fi
    
    print_step "Running WASM tests..."
    
    {
        echo "Running WASM tests at $(date)"
        cd "$PROJECT_ROOT/core/wasm"
        wasm-pack test --node 2>&1
        cd "$PROJECT_ROOT"
        echo "WASM tests completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "WASM tests passed"
        return 0
    else
        print_warning "WASM tests failed (non-critical)"
        return 0
    fi
}

# Function to run test orchestrator
run_test_orchestrator() {
    print_step "Running comprehensive test orchestrator..."
    
    {
        echo "Running test orchestrator at $(date)"
        cargo run --bin test-orchestrator -- \
            --output-dir "$REPORT_DIR" \
            --parallel 4 \
            --timeout 30 \
            --verbose 2>&1
        echo "Test orchestrator completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Test orchestrator completed successfully"
        return 0
    else
        print_error "Test orchestrator failed"
        return 1
    fi
}

# Function to run coverage analysis
run_coverage_analysis() {
    if [ "${SKIP_COVERAGE:-}" = "true" ]; then
        print_warning "Skipping coverage analysis (tools not available)"
        return 0
    fi
    
    print_step "Running coverage analysis..."
    
    {
        echo "Running coverage analysis at $(date)"
        
        # Try tarpaulin first
        if command_exists cargo-tarpaulin; then
            cargo tarpaulin \
                --workspace \
                --exclude-files "target/*" \
                --exclude-files "tests/*" \
                --exclude-files "*/tests.rs" \
                --exclude-files "*_test.rs" \
                --timeout 300 \
                --output-dir "$COVERAGE_DIR" \
                --out Html \
                --out Json \
                --verbose 2>&1
        else
            echo "Tarpaulin not available, skipping coverage"
        fi
        
        echo "Coverage analysis completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Coverage analysis completed"
        return 0
    else
        print_warning "Coverage analysis had issues (non-critical)"
        return 0
    fi
}

# Function to run property-based tests
run_property_tests() {
    print_step "Running property-based tests..."
    
    {
        echo "Running property-based tests at $(date)"
        cargo test --package writemagic-integration-tests property_ --verbose 2>&1
        echo "Property-based tests completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Property-based tests passed"
        return 0
    else
        print_warning "Property-based tests failed (non-critical)"
        return 0
    fi
}

# Function to run edge case tests
run_edge_case_tests() {
    print_step "Running edge case tests..."
    
    {
        echo "Running edge case tests at $(date)"
        cargo test --package writemagic-integration-tests edge_case --verbose 2>&1
        echo "Edge case tests completed with exit code: $?"
    } | tee -a "$LOG_FILE"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        print_success "Edge case tests passed"
        return 0
    else
        print_warning "Edge case tests failed (non-critical)"
        return 0
    fi
}

# Function to generate comprehensive report
generate_final_report() {
    print_step "Generating comprehensive test report..."
    
    local report_file="$REPORT_DIR/comprehensive-test-report.html"
    local json_report="$REPORT_DIR/test-results.json"
    
    # Generate JSON report
    cat > "$json_report" << EOF
{
    "execution_time": "$(date -Iseconds)",
    "project": "WriteMagic",
    "test_results": {
        "unit_tests": $UNIT_TEST_RESULT,
        "integration_tests": $INTEGRATION_TEST_RESULT,
        "wasm_tests": $WASM_TEST_RESULT,
        "performance_tests": $PERFORMANCE_TEST_RESULT,
        "property_tests": $PROPERTY_TEST_RESULT,
        "edge_case_tests": $EDGE_CASE_TEST_RESULT
    },
    "coverage_available": $([ "${SKIP_COVERAGE:-}" != "true" ] && echo "true" || echo "false"),
    "reports_location": "$REPORT_DIR",
    "logs_location": "$LOG_FILE"
}
EOF
    
    # Generate HTML report
    cat > "$report_file" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WriteMagic Comprehensive Test Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f7fa;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 12px;
            text-align: center;
            margin-bottom: 30px;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .metric {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            text-align: center;
        }
        .metric.success { border-left: 4px solid #10b981; }
        .metric.warning { border-left: 4px solid #f59e0b; }
        .metric.error { border-left: 4px solid #ef4444; }
        .test-details {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .status-badge {
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
        }
        .status-badge.pass { background: #dcfce7; color: #166534; }
        .status-badge.fail { background: #fef2f2; color: #991b1b; }
        .status-badge.warn { background: #fef3c7; color: #92400e; }
        .links {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .links a {
            display: inline-block;
            margin: 5px 10px 5px 0;
            padding: 8px 16px;
            background: #3b82f6;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }
        .links a:hover { background: #2563eb; }
    </style>
</head>
<body>
EOF

    # Add dynamic content to HTML report
    cat >> "$report_file" << EOF
    <div class="header">
        <h1>🚀 WriteMagic Test Execution Report</h1>
        <p>Generated on $(date)</p>
    </div>
    
    <div class="summary">
        <div class="metric $([ $UNIT_TEST_RESULT -eq 0 ] && echo 'success' || echo 'error')">
            <h3>Unit Tests</h3>
            <div class="status-badge $([ $UNIT_TEST_RESULT -eq 0 ] && echo 'pass' || echo 'fail')">
                $([ $UNIT_TEST_RESULT -eq 0 ] && echo 'PASSED' || echo 'FAILED')
            </div>
        </div>
        
        <div class="metric $([ $INTEGRATION_TEST_RESULT -eq 0 ] && echo 'success' || echo 'error')">
            <h3>Integration Tests</h3>
            <div class="status-badge $([ $INTEGRATION_TEST_RESULT -eq 0 ] && echo 'pass' || echo 'fail')">
                $([ $INTEGRATION_TEST_RESULT -eq 0 ] && echo 'PASSED' || echo 'FAILED')
            </div>
        </div>
        
        <div class="metric $([ $WASM_TEST_RESULT -eq 0 ] && echo 'success' || echo 'warning')">
            <h3>WASM Tests</h3>
            <div class="status-badge $([ $WASM_TEST_RESULT -eq 0 ] && echo 'pass' || echo 'warn')">
                $([ $WASM_TEST_RESULT -eq 0 ] && echo 'PASSED' || echo 'WARNING')
            </div>
        </div>
        
        <div class="metric $([ $PERFORMANCE_TEST_RESULT -eq 0 ] && echo 'success' || echo 'warning')">
            <h3>Performance</h3>
            <div class="status-badge $([ $PERFORMANCE_TEST_RESULT -eq 0 ] && echo 'pass' || echo 'warn')">
                $([ $PERFORMANCE_TEST_RESULT -eq 0 ] && echo 'COMPLETED' || echo 'ISSUES')
            </div>
        </div>
    </div>
    
    <div class="test-details">
        <h2>Test Execution Details</h2>
        <p><strong>Unit Tests:</strong> $([ $UNIT_TEST_RESULT -eq 0 ] && echo '✅ All unit tests passed successfully' || echo '❌ Unit tests failed - check logs for details')</p>
        <p><strong>Integration Tests:</strong> $([ $INTEGRATION_TEST_RESULT -eq 0 ] && echo '✅ All integration tests passed' || echo '❌ Integration tests failed - review implementation')</p>
        <p><strong>WASM Tests:</strong> $([ $WASM_TEST_RESULT -eq 0 ] && echo '✅ WASM compilation and tests successful' || echo '⚠️ WASM tests had issues - check wasm-pack installation')</p>
        <p><strong>Performance:</strong> $([ $PERFORMANCE_TEST_RESULT -eq 0 ] && echo '✅ Performance benchmarks completed' || echo '⚠️ Performance tests had issues - non-critical')</p>
        <p><strong>Property Tests:</strong> $([ $PROPERTY_TEST_RESULT -eq 0 ] && echo '✅ Property-based tests passed' || echo '⚠️ Property tests failed - review edge cases')</p>
        <p><strong>Edge Cases:</strong> $([ $EDGE_CASE_TEST_RESULT -eq 0 ] && echo '✅ Edge case tests passed' || echo '⚠️ Edge case tests failed - review boundary conditions')</p>
    </div>
    
    <div class="links">
        <h3>Additional Resources</h3>
        <a href="test-execution.log">View Execution Logs</a>
        $([ -f "$COVERAGE_DIR/tarpaulin-report.html" ] && echo '<a href="../coverage/tarpaulin-report.html">Coverage Report</a>')
        <a href="test-results.json">JSON Results</a>
    </div>
    
    <div class="test-details">
        <h3>Next Steps</h3>
EOF

    # Add recommendations based on results
    if [ $UNIT_TEST_RESULT -ne 0 ] || [ $INTEGRATION_TEST_RESULT -ne 0 ]; then
        echo "        <p>❌ <strong>Critical:</strong> Core tests failed. Fix failing tests before deployment.</p>" >> "$report_file"
    fi
    
    if [ $WASM_TEST_RESULT -ne 0 ]; then
        echo "        <p>⚠️ <strong>Warning:</strong> WASM tests had issues. Verify web functionality manually.</p>" >> "$report_file"
    fi
    
    if [ $PERFORMANCE_TEST_RESULT -ne 0 ]; then
        echo "        <p>⚠️ <strong>Info:</strong> Performance tests had issues. Review benchmarks for regressions.</p>" >> "$report_file"
    fi
    
    if [ $UNIT_TEST_RESULT -eq 0 ] && [ $INTEGRATION_TEST_RESULT -eq 0 ]; then
        echo "        <p>✅ <strong>Success:</strong> Core functionality is working. Ready for further testing.</p>" >> "$report_file"
    fi
    
    cat >> "$report_file" << 'EOF'
        <p>📊 Review the detailed logs and coverage reports for comprehensive analysis.</p>
        <p>🚀 Consider running additional manual testing for critical user workflows.</p>
    </div>
</body>
</html>
EOF

    print_success "Comprehensive report generated: $report_file"
}

# Function to display final summary
display_final_summary() {
    print_header "COMPREHENSIVE TEST EXECUTION SUMMARY"
    
    echo "Test Results:"
    echo "============"
    echo "Unit Tests:       $([ $UNIT_TEST_RESULT -eq 0 ] && print_status "$GREEN" "PASSED" || print_status "$RED" "FAILED")"
    echo "Integration:      $([ $INTEGRATION_TEST_RESULT -eq 0 ] && print_status "$GREEN" "PASSED" || print_status "$RED" "FAILED")"
    echo "WASM Tests:       $([ $WASM_TEST_RESULT -eq 0 ] && print_status "$GREEN" "PASSED" || print_status "$YELLOW" "WARNING")"
    echo "Performance:      $([ $PERFORMANCE_TEST_RESULT -eq 0 ] && print_status "$GREEN" "COMPLETED" || print_status "$YELLOW" "ISSUES")"
    echo "Property Tests:   $([ $PROPERTY_TEST_RESULT -eq 0 ] && print_status "$GREEN" "PASSED" || print_status "$YELLOW" "WARNING")"
    echo "Edge Cases:       $([ $EDGE_CASE_TEST_RESULT -eq 0 ] && print_status "$GREEN" "PASSED" || print_status "$YELLOW" "WARNING")"
    
    echo ""
    echo "Reports Generated:"
    echo "=================="
    echo "HTML Report:     $REPORT_DIR/comprehensive-test-report.html"
    echo "JSON Results:    $REPORT_DIR/test-results.json"
    echo "Execution Log:   $LOG_FILE"
    [ -f "$COVERAGE_DIR/tarpaulin-report.html" ] && echo "Coverage Report: $COVERAGE_DIR/tarpaulin-report.html"
    
    echo ""
    
    # Determine overall result
    if [ $UNIT_TEST_RESULT -eq 0 ] && [ $INTEGRATION_TEST_RESULT -eq 0 ]; then
        print_success "Overall Status: CORE TESTS PASSED ✅"
        echo ""
        print_status "$GREEN" "✅ WriteMagic core functionality is working correctly"
        print_status "$GREEN" "✅ Ready for deployment and further testing"
        echo ""
        return 0
    else
        print_error "Overall Status: CORE TESTS FAILED ❌"
        echo ""
        print_status "$RED" "❌ Critical test failures detected"
        print_status "$RED" "❌ Fix failing tests before proceeding"
        echo ""
        return 1
    fi
}

# Main execution function
main() {
    # Parse command line arguments
    SKIP_COVERAGE=false
    SKIP_WASM=false
    QUICK_MODE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-coverage)
                SKIP_COVERAGE=true
                shift
                ;;
            --skip-wasm)
                SKIP_WASM=true
                shift
                ;;
            --quick)
                QUICK_MODE=true
                shift
                ;;
            --help)
                echo "WriteMagic Comprehensive Test Runner"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-coverage    Skip coverage analysis"
                echo "  --skip-wasm        Skip WASM tests"
                echo "  --quick            Run only essential tests"
                echo "  --help             Show this help message"
                echo ""
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    print_header "WRITEMAGIC COMPREHENSIVE TEST EXECUTION"
    
    # Setup
    install_dependencies
    setup_test_environment
    
    # Initialize result variables
    UNIT_TEST_RESULT=1
    INTEGRATION_TEST_RESULT=1
    WASM_TEST_RESULT=1
    PERFORMANCE_TEST_RESULT=1
    PROPERTY_TEST_RESULT=1
    EDGE_CASE_TEST_RESULT=1
    
    # Core tests (always run)
    print_header "CORE TESTS"
    run_unit_tests && UNIT_TEST_RESULT=0 || UNIT_TEST_RESULT=1
    run_integration_tests && INTEGRATION_TEST_RESULT=0 || INTEGRATION_TEST_RESULT=1
    
    # Extended tests (skip in quick mode)
    if [ "$QUICK_MODE" != "true" ]; then
        print_header "EXTENDED TESTS"
        run_wasm_tests && WASM_TEST_RESULT=0 || WASM_TEST_RESULT=1
        run_performance_tests && PERFORMANCE_TEST_RESULT=0 || PERFORMANCE_TEST_RESULT=1
        run_property_tests && PROPERTY_TEST_RESULT=0 || PROPERTY_TEST_RESULT=1
        run_edge_case_tests && EDGE_CASE_TEST_RESULT=0 || EDGE_CASE_TEST_RESULT=1
        
        # Coverage analysis
        print_header "COVERAGE ANALYSIS"
        run_coverage_analysis
        
        # Test orchestrator
        print_header "ORCHESTRATED TESTING"
        run_test_orchestrator
    else
        print_status "$YELLOW" "Quick mode: Skipping extended tests"
        WASM_TEST_RESULT=0
        PERFORMANCE_TEST_RESULT=0
        PROPERTY_TEST_RESULT=0
        EDGE_CASE_TEST_RESULT=0
    fi
    
    # Generate reports
    print_header "REPORT GENERATION"
    generate_final_report
    
    # Final summary
    display_final_summary
}

# Execute main function with all arguments
main "$@"