#!/bin/bash
set -e

echo "🚀 Setting up WriteMagic development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
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

# Check if we're in a Git repository
if [ ! -d ".git" ]; then
    print_warning "Not in a Git repository. Initializing..."
    git init
    git config --global init.defaultBranch main
fi

# Set up Git configuration if not already set
if [ -z "$(git config --global user.name)" ]; then
    print_status "Setting up Git configuration..."
    git config --global user.name "Developer"
    git config --global user.email "developer@writemagic.dev"
fi

# Set up Git hooks directory
print_status "Setting up Git hooks..."
git config core.hooksPath .githooks
mkdir -p .githooks

# Install pre-commit hooks
print_status "Installing pre-commit hooks..."
if command -v pre-commit >/dev/null 2>&1; then
    pre-commit install --install-hooks
    pre-commit install --hook-type commit-msg
    print_success "Pre-commit hooks installed"
else
    print_warning "pre-commit not found, skipping hook installation"
fi

# Create secrets baseline for detect-secrets
print_status "Setting up secrets detection baseline..."
if command -v detect-secrets >/dev/null 2>&1; then
    if [ ! -f ".secrets.baseline" ]; then
        detect-secrets scan --baseline .secrets.baseline
        print_success "Secrets baseline created"
    fi
else
    print_warning "detect-secrets not found, creating empty baseline"
    echo '{}' > .secrets.baseline
fi

# Set up Rust environment
print_status "Setting up Rust environment..."
rustup update stable
rustup component add rustfmt clippy

# Check if all required Rust targets are installed
REQUIRED_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "i686-linux-android"
    "x86_64-linux-android"
    "aarch64-apple-ios"
    "x86_64-apple-ios"
    "aarch64-apple-ios-sim"
)

for target in "${REQUIRED_TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        print_status "Installing Rust target: $target"
        rustup target add "$target"
    fi
done

# Install additional Rust tools if not present
RUST_TOOLS=(
    "cargo-edit"
    "cargo-audit"
    "cargo-deny"
    "cargo-outdated"
    "cargo-watch"
    "cargo-expand"
)

for tool in "${RUST_TOOLS[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        print_status "Installing $tool..."
        cargo install "$tool" --locked
    fi
done

# Set up Android environment
if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
    print_status "Android SDK found at $ANDROID_HOME"
    
    # Create AVD if it doesn't exist
    if ! $ANDROID_HOME/cmdline-tools/latest/bin/avdmanager list avd | grep -q "WritemagicTest"; then
        print_status "Creating Android Virtual Device..."
        echo "no" | $ANDROID_HOME/cmdline-tools/latest/bin/avdmanager create avd \
            -n WritemagicTest \
            -k "system-images;android-29;google_apis;x86_64" \
            --force
        print_success "Android AVD created"
    fi
else
    print_warning "Android SDK not found. Mobile development features may not work."
fi

# Set up project-specific configurations
print_status "Setting up project configurations..."

# Create .env file for development
if [ ! -f ".env" ]; then
    cat > .env << EOF
# Development environment variables
RUST_LOG=debug
RUST_BACKTRACE=1

# Android development
ANDROID_HOME=$ANDROID_HOME
ANDROID_NDK_HOME=$ANDROID_NDK_HOME

# Java
JAVA_HOME=$JAVA_HOME

# Development flags
WRITEMAGIC_DEV_MODE=true
WRITEMAGIC_LOG_LEVEL=debug
EOF
    print_success "Development .env file created"
fi

# Create development configuration files
mkdir -p .vscode

# Create VS Code settings if not present
if [ ! -f ".vscode/settings.json" ]; then
    cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.allTargets": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.procMacro.enable": true,
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true,
        "source.organizeImports": true
    },
    "files.associations": {
        "*.rs": "rust",
        "*.kt": "kotlin",
        "*.kts": "kotlin",
        "*.swift": "swift"
    },
    "search.exclude": {
        "**/target": true,
        "**/node_modules": true,
        "**/build": true
    },
    "files.watcherExclude": {
        "**/target/**": true,
        "**/node_modules/**": true,
        "**/build/**": true
    }
}
EOF
    print_success "VS Code settings created"
fi

# Create launch configuration for debugging
if [ ! -f ".vscode/launch.json" ]; then
    cat > .vscode/launch.json << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Rust Tests",
            "cargo": {
                "args": [
                    "test",
                    "--no-run",
                    "--bin=writemagic"
                ],
                "filter": {
                    "name": "writemagic",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Rust Core",
            "cargo": {
                "args": [
                    "build",
                    "--bin=writemagic"
                ],
                "filter": {
                    "name": "writemagic",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
EOF
    print_success "VS Code launch configuration created"
fi

# Set up cargo configuration for cross-compilation
mkdir -p .cargo
if [ ! -f ".cargo/config.toml" ]; then
    cat > .cargo/config.toml << 'EOF'
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android21-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi21-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android21-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android21-clang"

[build]
jobs = 4

[profile.dev]
debug = true
opt-level = 0

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
EOF
    print_success "Cargo configuration created"
fi

# Create development scripts
mkdir -p scripts/dev

cat > scripts/dev/build-all.sh << 'EOF'
#!/bin/bash
set -e

echo "🔧 Building all WriteMagic components..."

# Build Rust core
echo "Building Rust core..."
cargo build --workspace

# Build Android
if [ -d "android" ]; then
    echo "Building Android app..."
    cd android
    ./gradlew assembleDebug
    cd ..
fi

# Build iOS (on macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [ -d "ios" ]; then
    echo "Building iOS app..."
    cd ios
    xcodebuild -project WriteMagic.xcodeproj -scheme WriteMagic -configuration Debug build
    cd ..
fi

echo "✅ Build completed!"
EOF

cat > scripts/dev/test-all.sh << 'EOF'
#!/bin/bash
set -e

echo "🧪 Running all WriteMagic tests..."

# Test Rust core
echo "Testing Rust core..."
cargo test --workspace

# Test Android
if [ -d "android" ]; then
    echo "Testing Android app..."
    cd android
    ./gradlew testDebugUnitTest
    cd ..
fi

# Test iOS (on macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [ -d "ios" ]; then
    echo "Testing iOS app..."
    cd ios
    xcodebuild -project WriteMagic.xcodeproj -scheme WriteMagic -configuration Debug test -destination 'platform=iOS Simulator,name=iPhone 15,OS=latest'
    cd ..
fi

echo "✅ All tests completed!"
EOF

chmod +x scripts/dev/*.sh
print_success "Development scripts created"

# Final setup steps
print_status "Running initial checks..."

# Check Rust installation
if cargo version >/dev/null 2>&1; then
    print_success "Rust toolchain: $(rustc --version)"
else
    print_error "Rust toolchain not properly installed"
fi

# Check Java installation
if java -version >/dev/null 2>&1; then
    print_success "Java: $(java -version 2>&1 | head -n1)"
else
    print_error "Java not properly installed"
fi

# Check Android SDK
if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
    print_success "Android SDK found at $ANDROID_HOME"
else
    print_warning "Android SDK not found or not configured"
fi

print_success "WriteMagic development environment setup completed!"
print_status "Next steps:"
echo "  1. Run 'scripts/dev/build-all.sh' to build all components"
echo "  2. Run 'scripts/dev/test-all.sh' to run all tests"
echo "  3. Start coding! 🚀"