# Android Native Libraries

This directory contains the compiled Rust libraries for Android integration.

## Expected Files

- `libwritemagic_android.so` - Main Rust core library compiled for Android
- Architecture-specific builds:
  - `arm64-v8a/libwritemagic_android.so`
  - `armeabi-v7a/libwritemagic_android.so`
  - `x86/libwritemagic_android.so`
  - `x86_64/libwritemagic_android.so`

## Build Instructions

From the project root, build the Android Rust library:

```bash
# Build for all Android architectures
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release
cargo build --target x86_64-linux-android --release

# Copy to libs directory
cp ../target/aarch64-linux-android/release/libwritemagic_android.so libs/arm64-v8a/
cp ../target/armv7-linux-androideabi/release/libwritemagic_android.so libs/armeabi-v7a/
cp ../target/i686-linux-android/release/libwritemagic_android.so libs/x86/
cp ../target/x86_64-linux-android/release/libwritemagic_android.so libs/x86_64/
```

## JNI Integration

The `MainActivity.kt` file loads these libraries and provides JNI method declarations for:

- `initializeCore()` - Initialize the Rust core engine
- `createDocument()` - Create new documents
- `saveDocument()` - Save document content
- `loadDocument()` - Load document content
- `processAIRequest()` - Handle AI processing requests