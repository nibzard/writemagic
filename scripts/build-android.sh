#!/bin/bash
# Build script for Android FFI library

set -e

echo "Building WriteMagic Android FFI library..."

# Build the Rust FFI library for development (host architecture)
echo "Building Rust FFI library for host architecture..."
cargo build --package writemagic-android-ffi --release

# For actual Android cross-compilation, you would need:
# 1. Android NDK installed
# 2. Rust targets added:
#    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
# 3. Then build for each target:
#    cargo build --package writemagic-android-ffi --target aarch64-linux-android --release
#    cargo build --package writemagic-android-ffi --target armv7-linux-androideabi --release
#    cargo build --package writemagic-android-ffi --target i686-linux-android --release
#    cargo build --package writemagic-android-ffi --target x86_64-linux-android --release

echo "FFI library build completed!"

# Create libs directory structure for Android
echo "Setting up Android libs directory..."
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86,x86_64}

echo "✅ Android build setup completed!"
echo ""
echo "Note: To complete the Android build:"
echo "1. Install Android Studio and NDK"
echo "2. Add Rust Android targets: rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android"
echo "3. Run this script again for cross-compilation"
echo "4. Copy .so files to android/app/src/main/jniLibs/"
echo "5. Build Android APK: cd android && ./gradlew assembleDebug"