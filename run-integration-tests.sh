#!/bin/bash

# WriteMagic Cross-Platform Integration Test Runner
# This script coordinates test execution across all platforms

set -e

echo "=€ WriteMagic Cross-Platform Integration Test Suite"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RUST_TESTS=true
ANDROID_TESTS=true
WEB_TESTS=true
WASM_TESTS=true
INTEGRATION_TESTS=true
PERFORMANCE_TESTS=false
PARALLEL=true
TIMEOUT_MINUTES=30

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-rust)
            RUST_TESTS=false
            shift
            ;;
        --no-android)
            ANDROID_TESTS=false
            shift
            ;;
        --no-web)
            WEB_TESTS=false
            shift
            ;;
        --no-wasm)
            WASM_TESTS=false
            shift
            ;;
        --no-integration)
            INTEGRATION_TESTS=false
            shift
            ;;
        --performance)
            PERFORMANCE_TESTS=true
            shift
            ;;
        --sequential)
            PARALLEL=false
            shift
            ;;
        --timeout)
            TIMEOUT_MINUTES="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --no-rust         Skip Rust core tests"
            echo "  --no-android      Skip Android tests"
            echo "  --no-web          Skip web tests"
            echo "  --no-wasm         Skip WASM tests"
            echo "  --no-integration  Skip integration tests"
            echo "  --performance     Include performance tests"
            echo "  --sequential      Run tests sequentially"
            echo "  --timeout N       Set timeout in minutes (default: 30)"
            echo "  --help, -h        Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Create results directory
mkdir -p test-results

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

run_with_timeout() {
    local cmd="$1"
    local desc="$2"
    local timeout_sec=$((TIMEOUT_MINUTES * 60))
    
    log_info "Running: $desc"
    
    if timeout ${timeout_sec}s bash -c "$cmd"; then
        log_info " $desc - PASSED"
        return 0
    else
        log_error "L $desc - FAILED"
        return 1
    fi
}

# Function to run Rust tests
run_rust_tests() {
    log_info ">€ Running Rust core tests..."
    
    if run_with_timeout "cargo test --workspace --verbose" "Rust Core Tests"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to run Android tests
run_android_tests() {
    log_info "> Running Android tests..."
    
    # Check if Android environment is available
    if [ ! -d "android" ]; then
        log_warn "Android directory not found, skipping Android tests"
        return
    fi
    
    cd android
    
    # Build first
    if run_with_timeout "./gradlew assembleDebug" "Android Build"; then
        # Run unit tests
        if run_with_timeout "./gradlew test" "Android Unit Tests"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    cd ..
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to run Web tests
run_web_tests() {
    log_info "< Running Web tests..."
    
    # Check if web-app directory exists
    if [ ! -d "web-app/tests" ]; then
        log_warn "Web app tests directory not found, skipping web tests"
        return
    fi
    
    cd web-app/tests
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing web test dependencies..."
        npm install
    fi
    
    if run_with_timeout "npm run test:all" "Web Tests"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    cd ../..
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to run WASM tests
run_wasm_tests() {
    log_info "=x Running WASM tests..."
    
    # Check if wasm-pack is available
    if ! command -v wasm-pack &> /dev/null; then
        log_warn "wasm-pack not found, skipping WASM tests"
        return
    fi
    
    # Build WASM first
    if run_with_timeout "cargo build --package writemagic-wasm --target wasm32-unknown-unknown --profile wasm-dev" "WASM Build"; then
        # Run WASM tests
        if run_with_timeout "wasm-pack test --node core/wasm" "WASM Tests"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to run integration tests
run_integration_tests() {
    log_info "= Running cross-platform integration tests..."
    
    if run_with_timeout "cargo run --bin test-orchestrator" "Integration Tests"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to run performance tests
run_performance_tests() {
    log_info "¡ Running performance tests..."
    
    if run_with_timeout "cargo bench --package writemagic-integration-tests" "Performance Tests"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_info "Starting test execution..."
    log_info "Configuration:"
    log_info "  Rust tests: $RUST_TESTS"
    log_info "  Android tests: $ANDROID_TESTS" 
    log_info "  Web tests: $WEB_TESTS"
    log_info "  WASM tests: $WASM_TESTS"
    log_info "  Integration tests: $INTEGRATION_TESTS"
    log_info "  Performance tests: $PERFORMANCE_TESTS"
    log_info "  Parallel execution: $PARALLEL"
    log_info "  Timeout: $TIMEOUT_MINUTES minutes"
    echo ""
    
    # Set up test environment
    export RUST_LOG=info
    export TEST_LOG_LEVEL=info
    
    if [ "$PARALLEL" = true ]; then
        log_info "Running tests in parallel..."
        
        # Run platform tests in parallel
        pids=()
        
        if [ "$RUST_TESTS" = true ]; then
            run_rust_tests &
            pids+=($!)
        fi
        
        if [ "$ANDROID_TESTS" = true ]; then
            run_android_tests &
            pids+=($!)
        fi
        
        if [ "$WEB_TESTS" = true ]; then
            run_web_tests &
            pids+=($!)
        fi
        
        if [ "$WASM_TESTS" = true ]; then
            run_wasm_tests &
            pids+=($!)
        fi
        
        # Wait for all parallel tests to complete
        for pid in "${pids[@]}"; do
            wait $pid
        done
        
        # Run integration tests sequentially after platform tests
        if [ "$INTEGRATION_TESTS" = true ]; then
            run_integration_tests
        fi
        
        if [ "$PERFORMANCE_TESTS" = true ]; then
            run_performance_tests
        fi
        
    else
        log_info "Running tests sequentially..."
        
        [ "$RUST_TESTS" = true ] && run_rust_tests
        [ "$ANDROID_TESTS" = true ] && run_android_tests  
        [ "$WEB_TESTS" = true ] && run_web_tests
        [ "$WASM_TESTS" = true ] && run_wasm_tests
        [ "$INTEGRATION_TESTS" = true ] && run_integration_tests
        [ "$PERFORMANCE_TESTS" = true ] && run_performance_tests
    fi
    
    # Calculate execution time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))
    
    # Generate results
    echo ""
    echo "=================================================="
    echo "=Ê WriteMagic Test Results"
    echo "=================================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    
    if [ $TOTAL_TESTS -gt 0 ]; then
        local success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo "Success Rate: $success_rate%"
    fi
    
    echo "Execution Time: ${minutes}m ${seconds}s"
    echo ""
    
    # Save results to file
    cat > test-results/summary.json << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "success_rate": $([ $TOTAL_TESTS -gt 0 ] && echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc || echo "0"),
    "duration_seconds": $duration,
    "configuration": {
        "rust_tests": $RUST_TESTS,
        "android_tests": $ANDROID_TESTS,
        "web_tests": $WEB_TESTS,
        "wasm_tests": $WASM_TESTS,
        "integration_tests": $INTEGRATION_TESTS,
        "performance_tests": $PERFORMANCE_TESTS,
        "parallel": $PARALLEL
    }
}
EOF
    
    if [ $FAILED_TESTS -gt 0 ]; then
        log_error "Some tests failed! Check the output above for details."
        exit 1
    else
        log_info "<‰ All tests passed!"
        exit 0
    fi
}

# Run main function
main "$@"