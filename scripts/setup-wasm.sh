#!/bin/bash
# Setup script for WriteMagic WASM development environment
#
# This script installs and configures the necessary tools for WASM development

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

# Check if Rust is installed
check_rust() {
    print_info "Checking Rust installation..."
    
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust is installed: $RUST_VERSION"
        return 0
    else
        print_error "Rust is not installed!"
        print_info "Please install Rust from: https://rustup.rs/"
        print_info "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        return 1
    fi
}

# Check if wasm32 target is installed
check_wasm_target() {
    print_info "Checking wasm32-unknown-unknown target..."
    
    if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        print_success "wasm32-unknown-unknown target is installed"
        return 0
    else
        print_warning "wasm32-unknown-unknown target is not installed"
        print_info "Installing wasm32-unknown-unknown target..."
        if rustup target add wasm32-unknown-unknown; then
            print_success "Successfully installed wasm32-unknown-unknown target"
            return 0
        else
            print_error "Failed to install wasm32-unknown-unknown target"
            return 1
        fi
    fi
}

# Check if wasm-pack is installed
check_wasm_pack() {
    print_info "Checking wasm-pack installation..."
    
    if command -v wasm-pack &> /dev/null; then
        WASM_PACK_VERSION=$(wasm-pack --version)
        print_success "wasm-pack is installed: $WASM_PACK_VERSION"
        return 0
    else
        print_warning "wasm-pack is not installed"
        print_info "Installing wasm-pack..."
        if curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh; then
            print_success "Successfully installed wasm-pack"
            return 0
        else
            print_error "Failed to install wasm-pack"
            return 1
        fi
    fi
}

# Check if wasm-bindgen-cli is installed
check_wasm_bindgen_cli() {
    print_info "Checking wasm-bindgen-cli installation..."
    
    if command -v wasm-bindgen &> /dev/null; then
        WASM_BINDGEN_VERSION=$(wasm-bindgen --version)
        print_success "wasm-bindgen-cli is installed: $WASM_BINDGEN_VERSION"
        return 0
    else
        print_warning "wasm-bindgen-cli is not installed"
        print_info "Installing wasm-bindgen-cli..."
        if cargo install wasm-bindgen-cli; then
            print_success "Successfully installed wasm-bindgen-cli"
            return 0
        else
            print_error "Failed to install wasm-bindgen-cli"
            return 1
        fi
    fi
}

# Check if basic-http-server is installed (optional)
check_basic_http_server() {
    print_info "Checking basic-http-server for local testing..."
    
    if command -v basic-http-server &> /dev/null; then
        print_success "basic-http-server is available"
        return 0
    elif command -v python3 &> /dev/null; then
        print_success "Python3 is available (can use 'python3 -m http.server')"
        return 0
    else
        print_warning "No local HTTP server found"
        print_info "Consider installing basic-http-server: cargo install basic-http-server"
        print_info "Or use Python: python3 -m http.server"
        return 0
    fi
}

# Test basic WASM compilation
test_wasm_compilation() {
    print_info "Testing basic WASM compilation..."
    
    # Create a temporary test project
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Create a minimal Cargo.toml
    cat > Cargo.toml << EOF
[package]
name = "wasm-test"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
EOF

    # Create a minimal lib.rs
    mkdir -p src
    cat > src/lib.rs << 'EOF'
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
EOF

    # Try to compile
    if cargo check --target wasm32-unknown-unknown; then
        print_success "Basic WASM compilation test passed!"
        cd - > /dev/null
        rm -rf "$TEMP_DIR"
        return 0
    else
        print_error "Basic WASM compilation test failed!"
        cd - > /dev/null
        rm -rf "$TEMP_DIR"
        return 1
    fi
}

# Main setup function
main() {
    print_info "Setting up WriteMagic WASM development environment..."
    echo
    
    # Check all dependencies
    local all_good=true
    
    if ! check_rust; then
        all_good=false
    fi
    
    if [[ "$all_good" == "true" ]]; then
        if ! check_wasm_target; then
            all_good=false
        fi
    fi
    
    if [[ "$all_good" == "true" ]]; then
        if ! check_wasm_pack; then
            all_good=false
        fi
    fi
    
    if [[ "$all_good" == "true" ]]; then
        if ! check_wasm_bindgen_cli; then
            all_good=false
        fi
    fi
    
    check_basic_http_server
    
    if [[ "$all_good" == "true" ]]; then
        if ! test_wasm_compilation; then
            all_good=false
        fi
    fi
    
    echo
    if [[ "$all_good" == "true" ]]; then
        print_success "✓ WASM development environment is ready!"
        print_info "You can now build the WriteMagic WASM module with:"
        echo "  ./scripts/build-wasm.sh dev"
        echo "  ./scripts/build-wasm.sh release"
    else
        print_error "✗ WASM development environment setup failed!"
        print_info "Please resolve the issues above and run this script again."
        exit 1
    fi
}

# Run main function
main "$@"