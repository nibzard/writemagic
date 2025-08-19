#!/bin/bash
# Build script for WriteMagic WASM module
#
# Usage:
#   ./scripts/build-wasm.sh [dev|release|profiling]
#
# Examples:
#   ./scripts/build-wasm.sh          # Default development build
#   ./scripts/build-wasm.sh release  # Optimized release build
#   ./scripts/build-wasm.sh dev      # Development build with debug symbols

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
WASM_DIR="core/wasm"
BUILD_MODE="${1:-dev}"
OUTPUT_DIR="pkg"

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

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    print_error "wasm-pack is not installed. Please install it first:"
    echo "  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "$WASM_DIR" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_info "Building WriteMagic WASM module in $BUILD_MODE mode..."

# Change to WASM directory
cd "$WASM_DIR"

# Clean previous build
if [ -d "$OUTPUT_DIR" ]; then
    print_info "Cleaning previous build..."
    rm -rf "$OUTPUT_DIR"
fi

# Set build flags based on mode
case "$BUILD_MODE" in
    "dev")
        print_info "Building in development mode with debug symbols..."
        wasm-pack build \
            --target web \
            --out-dir "$OUTPUT_DIR" \
            --dev \
            -- --features "console_error_panic_hook"
        ;;
    "release")
        print_info "Building optimized release build..."
        wasm-pack build \
            --target web \
            --out-dir "$OUTPUT_DIR" \
            -- --features "console_error_panic_hook"
        ;;
    "profiling")
        print_info "Building profiling build with optimizations and debug info..."
        wasm-pack build \
            --target web \
            --out-dir "$OUTPUT_DIR" \
            -- --features "console_error_panic_hook" --profile release-dbg
        ;;
    *)
        print_error "Unknown build mode: $BUILD_MODE"
        print_info "Available modes: dev, release, profiling"
        exit 1
        ;;
esac

# Check if build was successful
if [ -d "$OUTPUT_DIR" ] && [ -f "$OUTPUT_DIR/writemagic_wasm.js" ]; then
    print_success "WASM build completed successfully!"
    
    # Show output files
    print_info "Generated files:"
    ls -la "$OUTPUT_DIR"
    
    # Show package size
    if [ -f "$OUTPUT_DIR/writemagic_wasm_bg.wasm" ]; then
        WASM_SIZE=$(du -h "$OUTPUT_DIR/writemagic_wasm_bg.wasm" | cut -f1)
        print_info "WASM module size: $WASM_SIZE"
    fi
    
    # Return to project root
    cd - > /dev/null
    
    print_success "Build completed! WASM module is ready for web integration."
    print_info "You can now import the module in your web application:"
    echo "  import init, { WriteMagicEngine } from './${WASM_DIR}/${OUTPUT_DIR}/writemagic_wasm.js';"
else
    print_error "Build failed! Please check the output above for errors."
    exit 1
fi