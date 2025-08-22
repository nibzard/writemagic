#!/bin/bash
# Advanced Android NDK cross-compilation for WriteMagic FFI

set -e

echo "🔧 Building WriteMagic Android FFI Library..."

# Configuration
ANDROID_TARGETS=("aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android")
FFI_PACKAGE="writemagic-android-ffi"
ANDROID_API=24

# Check if targets are installed
echo "📋 Checking Rust Android targets..."
for target in "${ANDROID_TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "🔽 Installing target: $target"
        rustup target add "$target"
    else
        echo "✅ Target already installed: $target"
    fi
done

# Create output directories
echo "📁 Setting up Android library directories..."
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86,x86_64}

# Function to build for a specific target
build_for_target() {
    local target=$1
    local output_dir=$2
    local lib_name="libwritemagic_android.so"
    
    echo "🔨 Building for $target..."
    
    # Set environment variables for cross-compilation
    export CC_aarch64_linux_android="aarch64-linux-android${ANDROID_API}-clang"
    export CC_armv7_linux_androideabi="armv7a-linux-androideabi${ANDROID_API}-clang" 
    export CC_i686_linux_android="i686-linux-android${ANDROID_API}-clang"
    export CC_x86_64_linux_android="x86_64-linux-android${ANDROID_API}-clang"
    
    export AR_aarch64_linux_android="llvm-ar"
    export AR_armv7_linux_androideabi="llvm-ar"
    export AR_i686_linux_android="llvm-ar"
    export AR_x86_64_linux_android="llvm-ar"
    
    # Try building with cargo
    if cargo build --package "$FFI_PACKAGE" --target "$target" --release; then
        # Copy the built library
        local source_lib="target/$target/release/$lib_name"
        if [ -f "$source_lib" ]; then
            cp "$source_lib" "$output_dir/"
            echo "✅ Successfully built and copied $lib_name for $target"
            return 0
        else
            echo "⚠️  Library file not found: $source_lib"
            return 1
        fi
    else
        echo "⚠️  Build failed for $target, skipping..."
        return 1
    fi
}

# Build for each architecture
echo "🚀 Starting cross-compilation..."

# aarch64 (arm64-v8a) - most common modern Android devices
if build_for_target "aarch64-linux-android" "android/app/src/main/jniLibs/arm64-v8a"; then
    ARM64_SUCCESS=true
else
    ARM64_SUCCESS=false
fi

# armv7 (armeabi-v7a) - older Android devices  
if build_for_target "armv7-linux-androideabi" "android/app/src/main/jniLibs/armeabi-v7a"; then
    ARM32_SUCCESS=true
else
    ARM32_SUCCESS=false
fi

# x86_64 - Android emulators and x86 devices
if build_for_target "x86_64-linux-android" "android/app/src/main/jniLibs/x86_64"; then
    X86_64_SUCCESS=true
else
    X86_64_SUCCESS=false
fi

# i686 (x86) - older Android emulators
if build_for_target "i686-linux-android" "android/app/src/main/jniLibs/x86"; then
    X86_SUCCESS=true
else
    X86_SUCCESS=false
fi

# Summary
echo ""
echo "📊 Build Summary:"
echo "=================="
if [ "$ARM64_SUCCESS" = true ]; then
    echo "✅ ARM64 (arm64-v8a): SUCCESS"
else
    echo "❌ ARM64 (arm64-v8a): FAILED"
fi

if [ "$ARM32_SUCCESS" = true ]; then
    echo "✅ ARM32 (armeabi-v7a): SUCCESS"
else
    echo "❌ ARM32 (armeabi-v7a): FAILED"
fi

if [ "$X86_64_SUCCESS" = true ]; then
    echo "✅ x86_64: SUCCESS"
else
    echo "❌ x86_64: FAILED"
fi

if [ "$X86_SUCCESS" = true ]; then
    echo "✅ x86: SUCCESS"
else
    echo "❌ x86: FAILED"
fi

# Check if we have at least one successful build
if [ "$ARM64_SUCCESS" = true ] || [ "$ARM32_SUCCESS" = true ] || [ "$X86_64_SUCCESS" = true ] || [ "$X86_SUCCESS" = true ]; then
    echo ""
    echo "🎉 Android FFI build completed with at least one architecture!"
    echo "📱 Ready to build Android APK"
    
    # List the built libraries
    echo ""
    echo "📦 Built libraries:"
    find android/app/src/main/jniLibs -name "*.so" -type f 2>/dev/null || echo "No .so files found"
    
    exit 0
else
    echo ""
    echo "💥 All Android builds failed!"
    echo "This might be due to:"
    echo "  - Missing Android NDK"
    echo "  - Incorrect environment setup"
    echo "  - Missing system dependencies"
    echo ""
    echo "💡 The app will still compile but will crash at runtime without the native library."
    exit 1
fi