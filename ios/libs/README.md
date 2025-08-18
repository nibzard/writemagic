# iOS Native Libraries

This directory contains the compiled Rust libraries for iOS integration.

## Expected Files

- `libwritemagic_ios.a` - Main Rust core library compiled for iOS
- Architecture-specific builds:
  - `libwritemagic_ios_arm64.a` - iOS devices (ARM64)
  - `libwritemagic_ios_x86_64.a` - iOS Simulator (x86_64)
  - `libwritemagic_ios_aarch64.a` - iOS Simulator on Apple Silicon

## Build Instructions

From the project root, build the iOS Rust library:

```bash
# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

# Build for iOS architectures
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release

# Copy to libs directory
cp ../target/aarch64-apple-ios/release/libwritemagic_ios.a libs/libwritemagic_ios_arm64.a
cp ../target/x86_64-apple-ios/release/libwritemagic_ios.a libs/libwritemagic_ios_x86_64.a
cp ../target/aarch64-apple-ios-sim/release/libwritemagic_ios.a libs/libwritemagic_ios_aarch64.a

# Create universal library
lipo -create libs/libwritemagic_ios_*.a -output libs/libwritemagic_ios.a
```

## Swift FFI Integration

The `RustFFI.swift` file provides a Swift interface to the Rust core with methods for:

- Core initialization and management
- Document creation, saving, and loading
- AI request processing
- Project management
- Git integration and timeline visualization

## Xcode Configuration

The Xcode project is configured to:

- Link against the Rust static library
- Search for libraries in the `libs/` directory
- Support both device and simulator architectures