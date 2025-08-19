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
        print_info "Building optimized release build with streaming compilation..."
        wasm-pack build \
            --target web \
            --out-dir "$OUTPUT_DIR" \
            --no-typescript \
            --mode force \
            -- --features "console_error_panic_hook" --release
        
        # Apply additional optimizations
        print_info "Applying advanced optimizations..."
        optimize_wasm_bundle "$OUTPUT_DIR"
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

# Add optimization function
optimize_wasm_bundle() {
    local output_dir="$1"
    
    print_info "Optimizing WASM bundle for streaming compilation..."
    
    # Check if wasm-opt is available for size optimization
    if command -v wasm-opt &> /dev/null; then
        print_info "Applying wasm-opt optimizations..."
        
        local wasm_file="$output_dir/writemagic_wasm_bg.wasm"
        if [ -f "$wasm_file" ]; then
            # Create backup
            cp "$wasm_file" "$wasm_file.backup"
            
            # Apply aggressive optimizations
            wasm-opt -Os --enable-mutable-globals --enable-bulk-memory \
                --enable-sign-ext --enable-nontrapping-float-to-int \
                --enable-multi-value "$wasm_file.backup" -o "$wasm_file"
            
            # Show size improvement
            local old_size=$(du -h "$wasm_file.backup" | cut -f1)
            local new_size=$(du -h "$wasm_file" | cut -f1)
            print_success "WASM optimization complete: $old_size → $new_size"
            
            # Remove backup
            rm "$wasm_file.backup"
        fi
    else
        print_warning "wasm-opt not found, skipping advanced optimizations"
    fi
    
    # Generate feature-split modules if needed
    if [ "$BUILD_MODE" = "release" ]; then
        generate_feature_modules "$output_dir"
    fi
    
    # Generate streaming compilation helper
    generate_streaming_loader "$output_dir"
    
    # Generate integrity checksums
    generate_integrity_checksums "$output_dir"
}

# Generate feature-split modules for progressive loading
generate_feature_modules() {
    local output_dir="$1"
    
    print_info "Generating feature-split modules..."
    
    # This would involve more complex build logic to split WASM by features
    # For now, we'll create module metadata for the loader
    
    cat > "$output_dir/module_manifest.json" << EOF
{
    "version": "1.0.0",
    "modules": {
        "core": {
            "file": "writemagic_wasm_bg.wasm",
            "features": ["document", "project", "ai_basic"],
            "priority": "high",
            "required": true
        }
    },
    "buildTime": "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)",
    "optimizations": {
        "wasmOpt": $(command -v wasm-opt &> /dev/null && echo "true" || echo "false"),
        "streaming": true,
        "compression": true
    }
}
EOF
    
    print_success "Module manifest generated"
}

# Generate streaming compilation helper
generate_streaming_loader() {
    local output_dir="$1"
    
    print_info "Generating streaming compilation helper..."
    
    cat > "$output_dir/streaming_loader.js" << 'EOF'
/**
 * Streaming WASM Loader for WriteMagic
 * Provides optimized WebAssembly loading with streaming compilation
 */

export class StreamingWasmLoader {
    constructor(wasmUrl, jsBindingsUrl) {
        this.wasmUrl = wasmUrl;
        this.jsBindingsUrl = jsBindingsUrl;
        this.compiledModule = null;
        this.jsBindings = null;
    }
    
    async load() {
        const startTime = performance.now();
        
        try {
            // Load JS bindings and WASM in parallel
            const [jsBindings, wasmModule] = await Promise.all([
                this.loadJSBindings(),
                this.loadWasmWithStreaming()
            ]);
            
            this.jsBindings = jsBindings;
            this.compiledModule = wasmModule;
            
            const loadTime = performance.now() - startTime;
            console.log(`[StreamingWasmLoader] Loaded in ${loadTime.toFixed(2)}ms`);
            
            return {
                wasmModule,
                jsBindings,
                loadTime
            };
            
        } catch (error) {
            console.error('[StreamingWasmLoader] Loading failed:', error);
            throw error;
        }
    }
    
    async loadJSBindings() {
        const response = await fetch(this.jsBindingsUrl);
        if (!response.ok) {
            throw new Error(`Failed to load JS bindings: ${response.statusText}`);
        }
        
        // Dynamic import of the JS bindings
        const module = await import(this.jsBindingsUrl);
        return module;
    }
    
    async loadWasmWithStreaming() {
        if (typeof WebAssembly.compileStreaming === 'function') {
            try {
                return await WebAssembly.compileStreaming(fetch(this.wasmUrl));
            } catch (error) {
                console.warn('[StreamingWasmLoader] Streaming compilation failed, falling back:', error);
            }
        }
        
        // Fallback to traditional loading
        const response = await fetch(this.wasmUrl);
        if (!response.ok) {
            throw new Error(`Failed to fetch WASM: ${response.statusText}`);
        }
        
        const wasmBytes = await response.arrayBuffer();
        return await WebAssembly.compile(wasmBytes);
    }
    
    async instantiate() {
        if (!this.compiledModule) {
            throw new Error('WASM module not loaded. Call load() first.');
        }
        
        const instance = await WebAssembly.instantiate(this.compiledModule);
        
        // Initialize JS bindings with the instance
        if (this.jsBindings && this.jsBindings.default) {
            await this.jsBindings.default(instance);
        }
        
        return {
            instance,
            exports: instance.exports,
            jsBindings: this.jsBindings
        };
    }
}

// Default export for convenience
export default StreamingWasmLoader;
EOF
    
    print_success "Streaming loader helper generated"
}

# Generate integrity checksums for security
generate_integrity_checksums() {
    local output_dir="$1"
    
    print_info "Generating integrity checksums..."
    
    local checksums_file="$output_dir/integrity.json"
    echo "{" > "$checksums_file"
    echo '  "version": "1.0.0",' >> "$checksums_file"
    echo '  "generated": "'$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)'",' >> "$checksums_file"
    echo '  "checksums": {' >> "$checksums_file"
    
    local first=true
    for file in "$output_dir"/*.wasm "$output_dir"/*.js; do
        if [ -f "$file" ]; then
            local filename=$(basename "$file")
            local checksum=$(shasum -a 256 "$file" | cut -d' ' -f1)
            local size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null)
            
            if [ "$first" = true ]; then
                first=false
            else
                echo "," >> "$checksums_file"
            fi
            
            echo -n "    \"$filename\": {" >> "$checksums_file"
            echo -n "\"sha256\": \"$checksum\", " >> "$checksums_file"
            echo -n "\"size\": $size" >> "$checksums_file"
            echo -n "}" >> "$checksums_file"
        fi
    done
    
    echo "" >> "$checksums_file"
    echo "  }" >> "$checksums_file"
    echo "}" >> "$checksums_file"
    
    print_success "Integrity checksums generated"
}

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