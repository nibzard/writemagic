# WriteMagic

> **Cross-platform AI-powered writing application with Rust core and native mobile UIs**

[![CI/CD](https://github.com/nibzard/writemagic/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/nibzard/writemagic/actions)
[![Security](https://github.com/nibzard/writemagic/actions/workflows/security.yml/badge.svg)](https://github.com/nibzard/writemagic/actions)
[![Mobile CI](https://github.com/nibzard/writemagic/actions/workflows/mobile-ci.yml/badge.svg)](https://github.com/nibzard/writemagic/actions)

WriteMagic is a revolutionary writing application that combines the power of AI with intuitive cross-platform design. Built with a high-performance Rust core and native mobile UIs, it offers a seamless writing experience across iOS and Android devices.

## ✨ Key Features

- **🎨 Multi-Pane Writing**: Each pane corresponds to a Git branch, enabling parallel writing exploration
- **🤖 AI Integration**: Provider-agnostic support for Claude, GPT-4, and local models with intelligent fallbacks
- **📱 Native Mobile**: SwiftUI (iOS) and Jetpack Compose (Android) for optimal platform experiences
- **🌊 Git Timeline**: Beautiful visualization of your writing journey across branches
- **⚡ High Performance**: Rust core engine with memory-safe FFI bindings
- **🔧 Agent Automation**: YAML-based background agents for workflow optimization

## 🏗️ Architecture

WriteMagic follows a **Domain-Driven Design** architecture with specialized sub-agents coordinating development:

```
┌─────────────────┬─────────────────┐
│   iOS (Swift)   │ Android (Kotlin)│  ← Native UIs
├─────────────────┼─────────────────┤
│        FFI Bindings (C/JNI)       │  ← Cross-platform layer
├─────────────────────────────────────┤
│           Rust Core Engine          │  ← Business logic
│  ┌─────────┬─────────┬─────────┐   │
│  │Writing  │   AI    │ Project │   │  ← Domain modules
│  │ Domain  │ Domain  │ Domain  │   │
│  └─────────┴─────────┴─────────┘   │
└─────────────────────────────────────┘
```

### 🎯 Core Domains

- **Writing Domain**: Document management, content editing, project organization
- **AI Domain**: LLM integration, context management, response processing
- **Project Domain**: Multi-pane workspaces, session management
- **Version Control Domain**: Git integration with timeline visualization
- **Agent Domain**: File-based YAML agents for background processing

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ with cross-compilation targets
- **Android Studio** with NDK for Android development
- **Xcode** 15+ for iOS development (macOS only)
- **Git** with conventional commit setup

### 1. Clone and Setup

```bash
git clone https://github.com/nibzard/writemagic.git
cd writemagic

# Set up git hooks and configuration
./scripts/setup-git.sh

# Install pre-commit hooks
pre-commit install
```

### 2. Development Environment

#### Option A: DevContainer (Recommended)
```bash
# Open in VS Code with Dev Containers extension
code .
# Select "Reopen in Container"
```

#### Option B: Local Setup
```bash
# Install Rust targets
rustup target add aarch64-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

# Install additional tools
cargo install cargo-audit cargo-deny cargo-udeps
```

### 3. Build and Test

```bash
# Build Rust workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Build Android app
cd android && ./gradlew assembleDebug

# Build iOS app (macOS only)
cd ios && xcodebuild -scheme WriteMagic build
```

## 📚 Development Workflow

WriteMagic uses a **sub-agent coordination system** for organized development:

### 🤖 Sub-Agents

- **@project-manager**: Coordinates tasks and maintains project structure
- **@mobile-architect**: iOS/Android development and cross-platform architecture  
- **@ai-integration-specialist**: LLM provider abstractions and AI features
- **@rust-core-engineer**: High-performance Rust engine with FFI bindings
- **@ux-writing-specialist**: Writing-focused user experience design
- **@devops-platform-engineer**: Infrastructure, CI/CD, and deployment

### 🔄 Git Workflow

We use **conventional commits** with structured formatting:

```bash
# Feature development
git feat mobile "add gesture navigation"
git fix core "resolve memory leak in FFI"

# Agent-specific branches
git checkout -b feature/mobile-architect-gesture-nav
git checkout -b fix/rust-core-memory-leak
```

### 📋 Task Management

All tasks are tracked in [`todo.md`](todo.md) with structured format:

```markdown
- [ ] [AGENT] Task description with clear acceptance criteria
  - Estimated effort: [S/M/L/XL]
  - Dependencies: [List dependencies]
  - Acceptance criteria: [Clear definition of done]
```

## 🛠️ Technology Stack

### Core Technologies
- **Backend**: Rust with async/await, Domain-Driven Design
- **Mobile**: SwiftUI (iOS), Jetpack Compose (Android)
- **AI**: Provider-agnostic with Claude, OpenAI, local model support
- **Data**: SQLite with Git for version control

### Development Tools
- **CI/CD**: GitHub Actions with multi-platform builds
- **Quality**: Clippy, rustfmt, ktlint, SwiftLint
- **Security**: cargo-audit, dependency scanning, secret detection
- **Monitoring**: Prometheus, Grafana, distributed tracing

## 📱 Platform-Specific Setup

### Android Development

```bash
cd android

# Install dependencies and build
./gradlew assembleDebug

# Run on device/emulator
./gradlew installDebug
```

**Requirements:**
- Android Studio Arctic Fox+
- NDK 25+
- Minimum SDK: API 24 (Android 7.0)
- Target SDK: API 34 (Android 14)

### iOS Development

```bash
cd ios

# Build for simulator
xcodebuild -scheme WriteMagic -destination 'platform=iOS Simulator,name=iPhone 15'

# Build for device
xcodebuild -scheme WriteMagic -destination 'generic/platform=iOS'
```

**Requirements:**
- Xcode 15+
- iOS 17+ deployment target
- Apple Developer account for device testing

## 🚢 Deployment

### Development
```bash
# Start development environment
docker-compose -f docker-compose.yml --profile development up

# Access at http://localhost:3000
```

### Production
```bash
# Deploy to staging
./scripts/deploy/staging.sh

# Production deployment (CI/CD)
git tag v1.0.0
git push origin v1.0.0  # Triggers release pipeline
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Steps

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Follow** our sub-agent coordination patterns
4. **Test** your changes thoroughly
5. **Submit** a pull request with conventional commit format

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔒 Security

For security concerns, please see [SECURITY.md](SECURITY.md) for our security policy and reporting procedures.

## 📖 Documentation

- **[Architecture Overview](MOBILE_ARCHITECTURE.md)**: Detailed system design
- **[Technical Specifications](SPECS_IMPROVED.md)**: Complete technical specifications
- **[CI/CD Setup](docs/CI_CD_SETUP.md)**: Development and deployment workflows
- **[Agent Coordination](.claude/agents/README.md)**: Sub-agent system documentation

## 🌟 Roadmap

- [ ] **Phase 1**: Core writing engine with basic AI integration
- [ ] **Phase 2**: Advanced multi-pane workflows and Git visualization
- [ ] **Phase 3**: Cloud sync and collaborative features
- [ ] **Phase 4**: Plugin system and community extensions

## 💬 Community

- **Issues**: [GitHub Issues](https://github.com/nibzard/writemagic/issues)
- **Discussions**: [GitHub Discussions](https://github.com/nibzard/writemagic/discussions)
- **Releases**: [GitHub Releases](https://github.com/nibzard/writemagic/releases)

---

<div align="center">

**Built with ❤️ using Rust, SwiftUI, and Jetpack Compose**

[Report Bug](https://github.com/nibzard/writemagic/issues) • [Request Feature](https://github.com/nibzard/writemagic/issues) • [View Documentation](https://github.com/nibzard/writemagic/wiki)

</div>