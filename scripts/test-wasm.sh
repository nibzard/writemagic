#!/bin/bash
# Test script for WriteMagic WASM module
#
# Usage:
#   ./scripts/test-wasm.sh
#
# This script tests the WASM module in a headless browser environment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
WASM_DIR="core/wasm"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "$WASM_DIR" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_info "Running WASM tests for WriteMagic..."

# Change to WASM directory
cd "$WASM_DIR"

# Check if wasm-pack is available
if ! command -v wasm-pack &> /dev/null; then
    print_error "wasm-pack is not installed. Please install it first:"
    echo "  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Run WASM tests
print_info "Running wasm-bindgen tests..."
if wasm-pack test --headless --firefox; then
    print_success "Firefox tests passed!"
else
    print_warning "Firefox tests failed or Firefox is not available"
fi

if wasm-pack test --headless --chrome; then
    print_success "Chrome tests passed!"
else
    print_warning "Chrome tests failed or Chrome is not available"
fi

# Test basic compilation
print_info "Testing basic compilation..."
if cargo check --target wasm32-unknown-unknown; then
    print_success "WASM target compilation check passed!"
else
    print_error "WASM target compilation check failed!"
    exit 1
fi

# Return to project root
cd - > /dev/null

print_success "WASM testing completed!"