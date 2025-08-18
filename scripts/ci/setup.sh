#!/bin/bash
set -euo pipefail

# WriteMagic CI Setup Script
# This script sets up the environment for CI/CD operations

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[CI-SETUP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[CI-SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[CI-WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[CI-ERROR]${NC} $1"
}

# Check if running in CI environment
if [ -z "${CI:-}" ]; then
    print_warning "Not running in CI environment"
fi

print_status "Setting up WriteMagic CI environment..."

# Set up directories
mkdir -p artifacts/{rust,android,ios,coverage,security}
mkdir -p logs
mkdir -p test-results

# Install Rust if not present
if ! command -v rustc >/dev/null 2>&1; then
    print_status "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    print_success "Rust toolchain installed"
fi

# Update Rust toolchain
print_status "Updating Rust toolchain..."
rustup update stable
rustup component add rustfmt clippy

# Install cross-compilation targets
print_status "Installing cross-compilation targets..."
TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi" 
    "i686-linux-android"
    "x86_64-linux-android"
    "aarch64-apple-ios"
    "x86_64-apple-ios"
    "aarch64-apple-ios-sim"
)

for target in "${TARGETS[@]}"; do
    print_status "Installing target: $target"
    rustup target add "$target"
done

# Install Rust tools
print_status "Installing Rust development tools..."
RUST_TOOLS=(
    "cargo-audit"
    "cargo-deny"
    "cargo-outdated"
    "cargo-tarpaulin"
    "cargo-edit"
)

for tool in "${RUST_TOOLS[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        print_status "Installing $tool..."
        cargo install "$tool" --locked
    else
        print_status "$tool already installed"
    fi
done

# Set up Android environment if on Linux
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_status "Setting up Android environment..."
    
    # Check if Android SDK is already installed
    if [ -z "${ANDROID_HOME:-}" ] || [ ! -d "${ANDROID_HOME:-}" ]; then
        print_warning "Android SDK not found. Mobile builds may fail."
    else
        print_success "Android SDK found at $ANDROID_HOME"
        
        # Verify NDK installation
        if [ -z "${ANDROID_NDK_HOME:-}" ] || [ ! -d "${ANDROID_NDK_HOME:-}" ]; then
            print_warning "Android NDK not found at expected location"
        else
            print_success "Android NDK found at $ANDROID_NDK_HOME"
        fi
    fi
fi

# Set up iOS environment if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_status "Setting up iOS environment..."
    
    # Check for Xcode
    if ! command -v xcodebuild >/dev/null 2>&1; then
        print_error "Xcode not found. iOS builds will fail."
        exit 1
    else
        print_success "Xcode found: $(xcodebuild -version | head -n1)"
    fi
    
    # Install iOS simulator if needed
    xcrun simctl list runtimes | grep -q "iOS" || {
        print_warning "iOS simulator runtime not found"
    }
fi

# Cache directories setup
print_status "Setting up cache directories..."
mkdir -p ~/.cargo/registry
mkdir -p ~/.cargo/git
mkdir -p target

# Set up environment variables
print_status "Setting up environment variables..."
cat > ci-env.sh << 'EOF'
#!/bin/bash
# CI Environment Variables for WriteMagic

export RUST_BACKTRACE=1
export RUST_LOG=info
export CARGO_TERM_COLOR=always
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Dwarnings"

# Android variables (if available)
if [ -n "${ANDROID_HOME:-}" ]; then
    export PATH="$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools"
fi

# Rust cache optimization
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"

# Test configuration
export WRITEMAGIC_TEST_MODE=true
export WRITEMAGIC_LOG_LEVEL=debug

echo "WriteMagic CI environment loaded"
EOF

chmod +x ci-env.sh
print_success "CI environment script created"

# Create test configuration
print_status "Creating test configuration..."
cat > test-config.toml << 'EOF'
[test]
timeout = "300s"
parallel = true
verbose = true

[coverage]
format = "lcov"
output_dir = "coverage"
include_tests = true

[android]
emulator_timeout = "300s"
test_timeout = "600s"

[ios] 
simulator_timeout = "300s"
test_timeout = "600s"
EOF

print_success "Test configuration created"

# Verify setup
print_status "Verifying CI setup..."

# Check Rust
if cargo version >/dev/null 2>&1; then
    print_success "Rust: $(rustc --version)"
else
    print_error "Rust verification failed"
    exit 1
fi

# Check clippy
if cargo clippy --version >/dev/null 2>&1; then
    print_success "Clippy: $(cargo clippy --version)"
else
    print_error "Clippy verification failed"
    exit 1
fi

# Check rustfmt
if cargo fmt --version >/dev/null 2>&1; then
    print_success "Rustfmt: $(cargo fmt --version)"
else
    print_error "Rustfmt verification failed"
    exit 1
fi

# Check Android (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]] && [ -n "${ANDROID_HOME:-}" ]; then
    if command -v adb >/dev/null 2>&1; then
        print_success "Android tools: $(adb --version | head -n1)"
    else
        print_warning "Android tools not found in PATH"
    fi
fi

# Check iOS (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if command -v xcodebuild >/dev/null 2>&1; then
        print_success "Xcode: $(xcodebuild -version | head -n1)"
    else
        print_error "Xcode not found"
    fi
fi

print_success "WriteMagic CI setup completed successfully!"
print_status "To load the environment, run: source ci-env.sh"