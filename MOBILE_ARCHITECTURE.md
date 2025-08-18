# WriteMagic Mobile Architecture

This document outlines the mobile architecture for WriteMagic's native iOS and Android applications.

## Overview

WriteMagic uses a shared Rust core with native mobile UIs to provide optimal performance and platform-specific user experiences while maintaining consistent business logic across platforms.

## Architecture Layers

### 1. Native UI Layer
- **iOS**: SwiftUI with native iOS design patterns
- **Android**: Jetpack Compose with Material Design 3

### 2. FFI Bridge Layer
- **iOS**: Swift-C FFI bridge (`RustFFI.swift`)
- **Android**: JNI bridge (`MainActivity.kt` with native methods)

### 3. Rust Core Engine
- Shared business logic across platforms
- Domain-driven design with specialized modules
- Cross-platform compilation targeting mobile architectures

## Directory Structure

```
mobile/
├── android/                    # Android application
│   ├── app/
│   │   ├── build.gradle.kts   # Android build configuration
│   │   ├── src/main/
│   │   │   ├── AndroidManifest.xml
│   │   │   ├── java/com/writemagic/
│   │   │   │   ├── MainActivity.kt      # Main activity with JNI setup
│   │   │   │   └── ui/                  # Compose UI components
│   │   │   │       ├── WriteMagicApp.kt
│   │   │   │       ├── screens/         # Feature screens
│   │   │   │       └── theme/           # Material Design theme
│   │   │   └── res/                     # Android resources
│   │   └── libs/                        # Compiled Rust libraries
│   ├── gradle/
│   ├── build.gradle.kts               # Project-level build config
│   ├── settings.gradle.kts
│   └── gradle.properties
│
└── ios/                        # iOS application
    ├── WriteMagic.xcodeproj/   # Xcode project configuration
    ├── WriteMagic/
    │   ├── WriteMagicApp.swift          # App entry point
    │   ├── ContentView.swift            # Main view controller
    │   ├── Views/                       # SwiftUI views
    │   │   ├── WritingView.swift
    │   │   ├── ProjectsView.swift
    │   │   ├── AIView.swift
    │   │   ├── TimelineView.swift
    │   │   └── SettingsView.swift
    │   ├── Models/                      # Data models
    │   │   ├── Document.swift
    │   │   ├── Project.swift
    │   │   └── GitCommit.swift
    │   ├── Services/
    │   │   └── RustFFI.swift           # Rust FFI bridge
    │   ├── Assets.xcassets
    │   └── Info.plist
    └── libs/                           # Compiled Rust libraries
```

## Core Features

### 1. Multi-Pane Writing Interface
- **iOS**: Split view with drag-and-drop support
- **Android**: Adaptive layouts with gesture navigation
- Real-time synchronization between panes

### 2. AI Integration
- Provider-agnostic AI assistance (Claude, GPT-4, Local models)
- Context-aware suggestions
- Seamless integration with writing workflow

### 3. Project Management
- Hierarchical document organization
- Cross-device synchronization
- Git-based version control

### 4. Timeline Visualization
- Beautiful git history visualization
- Branch-based workflow
- Visual diff and merge capabilities

### 5. Native Platform Integration
- **iOS**: Document browser, Files app integration, Handoff support
- **Android**: Intent handling, file sharing, background sync

## FFI Integration

### iOS (Swift-C Bridge)

```swift
// RustFFI.swift - Example method
func createDocument(title: String) -> String {
    // Converts Swift String to C string
    // Calls Rust function via C FFI
    // Converts C string result back to Swift String
}
```

### Android (JNI Bridge)

```kotlin
// MainActivity.kt - Example JNI declaration
external fun createDocument(title: String): String

companion object {
    init {
        System.loadLibrary("writemagic_android")
    }
}
```

## Build Process

### Android Build Steps
1. Compile Rust core for Android targets (ARM64, ARM, x86, x86_64)
2. Copy compiled libraries to `android/libs/`
3. Build Android app with Gradle
4. Generate APK/AAB with embedded native libraries

### iOS Build Steps
1. Compile Rust core for iOS targets (ARM64, x86_64, Simulator)
2. Create universal static library with `lipo`
3. Copy library to `ios/libs/`
4. Build iOS app with Xcode
5. Generate IPA with embedded static library

## Performance Considerations

### Memory Management
- Rust handles core business logic memory efficiently
- Mobile UIs use platform-native memory management
- Careful FFI boundary management to prevent leaks

### Battery Optimization
- Background processing delegated to Rust core
- Native platform background task management
- Efficient AI request batching and caching

### Storage Strategy
- SQLite for local document storage
- Git for version control and sync
- Platform-native file system integration

## Security

### Data Protection
- Encryption at rest using platform keychains
- Secure API key storage
- PII detection before AI processing

### Platform Security
- iOS: App Transport Security, keychain services
- Android: Android Keystore, encrypted preferences

## Development Workflow

### Prerequisites
```bash
# Rust toolchain
rustup target add aarch64-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

# Android development
# Install Android Studio and SDK

# iOS development  
# Install Xcode and iOS SDK
```

### Build Commands
```bash
# Build Android
cd android && ./gradlew assembleDebug

# Build iOS
cd ios && xcodebuild -scheme WriteMagic build

# Build Rust core for all platforms
cargo build --workspace --release
```

## Testing Strategy

### Unit Tests
- Rust core: `cargo test`
- iOS: XCTest framework
- Android: JUnit + Espresso

### Integration Tests
- FFI boundary testing
- Cross-platform feature parity
- Performance benchmarking

### UI Tests
- Platform-specific UI automation
- Accessibility testing
- Multi-device testing

## Future Enhancements

### Planned Features
- Collaborative editing
- Advanced AI integrations
- Plugin system
- Advanced git workflows

### Technical Debt
- Comprehensive error handling at FFI boundaries
- Performance profiling and optimization
- Comprehensive accessibility support
- Advanced offline capabilities