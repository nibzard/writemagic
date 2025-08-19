# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WriteMagic is an AI-powered writing application focused on **Android mobile** and **web** platforms, built with a Rust core engine. The architecture follows Domain-Driven Design principles with functional implementations prioritized over mock code.

## Core Architecture

### Multi-Layer Architecture (MVP Scope)
- **Android App**: Native Kotlin/Jetpack Compose application with FFI integration
- **Web Application**: Progressive Web App with WASM integration - **PROMOTED TO MVP**
- **Rust Core**: Shared business logic engine with FFI and WASM bindings
- **AI Integration**: Provider-agnostic LLM integration with Claude, GPT-4, and local model support
- **Data Layer**: SQLite for local storage, IndexedDB for web

### Domain Structure (MVP Focus)
The Rust core is organized into bounded contexts:
- **Writing Domain**: Document management, content editing, project organization
- **AI Domain**: LLM integration, context management, response processing
- **Project Domain**: Multi-pane workspaces, session management

### Out of MVP Scope
The following features are planned for post-MVP phases:
- **Version Control Domain**: Git integration with timeline visualization
- **Agent Domain**: File-based YAML agents for background processing
- **iOS Application**: Native SwiftUI implementation - **DEMOTED TO PHASE 3**
- **CI/CD Pipeline**: Automated deployment and infrastructure
- **Cloud Infrastructure**: Kubernetes deployment and monitoring

## Development Workflow

### Git Workflow and Conventional Commits
This project uses strict conventional commit standards. Set up git configuration:
```bash
./scripts/setup-git.sh
```

Commit format: `type(scope): description`
- **Types**: feat, fix, docs, style, refactor, test, chore, perf
- **Scopes**: android, web, core, ai, ux, docs

Quick commit aliases:
```bash
git feat android "add gesture navigation"
git fix core "resolve memory leak in FFI"
git docs "update technical documentation"
```

Branch naming: `feature/[agent]-[task-name]`, `fix/[agent]-[issue]`

### Task Management
All project tasks are centrally managed in `todo.md` with structured format:
```
- [ ] [AGENT] Task description with clear acceptance criteria
  - Estimated effort: [S/M/L/XL]
  - Dependencies: [List dependencies]
  - Acceptance criteria: [Clear definition of done]
```

**Always update todo.md when starting, progressing, or completing tasks.**

## Development Team Focus Areas

The MVP development focuses on core functionality with streamlined team responsibilities:

### Core MVP Team
1. **Rust Core Engineer** - High-performance engine with Android FFI and WASM bindings
2. **Android Developer** - Native Kotlin/Compose implementation
3. **Web Developer** - Progressive Web App with WASM integration - **PROMOTED TO MVP**
4. **AI Integration Specialist** - LLM provider abstractions for mobile and web
5. **UX Writing Specialist** - Writing-focused user experience across platforms

### Post-MVP Team Expansion
Future phases will add:
- **iOS Developer** - Native SwiftUI implementation (Phase 3)
- **DevOps Engineer** - CI/CD and infrastructure automation (Phase 4)
- **Project Manager** - Coordination and workflow management (Phase 4)

### MVP Development Flow
```
1. Rust Core Engineer - Build functional domain logic with WASM support
2. AI Integration Specialist - Implement provider integration for mobile and web
3. Android Developer - Create native mobile interface with FFI
4. Web Developer - Build Progressive Web App with WASM integration
5. UX Writing Specialist - Optimize writing workflows across platforms
6. All - Cross-platform integration testing and refinement
```

## Technical Specifications

### Cross-Platform FFI Strategy (MVP Focus)
- **Android FFI**: Rust core exposes C-compatible interfaces for Android JNI bindings
- **Web FFI**: WASM compilation with wasm-bindgen for JavaScript integration
- **Shared business logic**: Same Rust domains across Android and web platforms
- **Platform-specific UI**: Android-native Kotlin/Compose, Web-native HTML/CSS/JS

### AI Provider Abstraction
```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;
    fn capabilities(&self) -> ModelCapabilities;
}
```

### Domain-Driven Design Structure
Each domain has:
- **Entities**: Core business objects with identity
- **Value Objects**: Immutable objects without identity
- **Aggregates**: Consistency boundaries with aggregate roots
- **Services**: Domain logic that doesn't belong to entities
- **Repositories**: Data access abstractions

### Multi-Pane Writing System (MVP)
- Multiple document editing panes for comparison
- Content organization and project management
- AI suggestions for alternative approaches
- Future: Git integration and timeline visualization

## Development Commands

### Project Setup
```bash
# Set up git configuration
./scripts/setup-git.sh

# Future commands (once implemented):
# Build Rust core for all platforms
cargo build --workspace

# Run Rust tests
cargo test --workspace

# Build Android app
cd android && ./gradlew assembleDebug

# Run linting
cargo clippy -- -D warnings
```

## Key Implementation Notes

### Android Performance Priorities
- Native Kotlin/Compose UI components
- Optimize for memory usage and battery life
- Implement proper background task management
- Android-specific optimizations (Vulkan, hardware acceleration)

### AI Integration Best Practices
- Always implement provider fallbacks
- Cache responses when appropriate
- Filter sensitive content before AI processing
- Monitor token usage and implement rate limiting
- Maintain conversation context across sessions

### Security Considerations
- Encrypt sensitive data at rest (AES-256-GCM)
- Use secure keystores for API keys
- Implement PII detection before AI processing
- Follow platform security guidelines

## Current Development Status

**🚧 MVP IN DEVELOPMENT - ANDROID + WEB FOCUS**

WriteMagic is actively developing the MVP with a focused scope on Android mobile and web applications. 

### 🎯 MVP Scope (Current Focus)
1. **Rust Core Foundation**: Domain-driven architecture with SQLite/IndexedDB persistence
2. **AI Integration**: Multi-provider orchestration (Claude/GPT-4) with intelligent fallback  
3. **Android Application**: Native Kotlin/Compose app with FFI integration
4. **Web Application**: Progressive Web App with WASM integration - **PROMOTED TO MVP**
5. **Core Writing Features**: Document management, AI-assisted writing, multi-pane editing

### 🔄 In Development
- **Android FFI Integration**: Connecting Rust core to Android app
- **Web WASM Integration**: Compiling Rust core to WebAssembly - **NEW MVP PRIORITY**
- **Native Library Compilation**: Building .so files for Android and WASM for web
- **Cross-platform UI**: Refining Android and web interfaces for writing workflows
- **AI Service Integration**: Implementing provider fallback and caching across platforms

### 📋 Post-MVP Roadmap
Future development phases will include:
- **iOS Application**: Native SwiftUI implementation - **DEMOTED TO PHASE 3**
- **CI/CD Pipeline**: Automated deployment and infrastructure
- **Advanced Features**: Git integration, collaborative editing, custom AI agents
- **Cloud Infrastructure**: Kubernetes deployment and monitoring

### ❌ Explicitly Out of MVP Scope
- iOS application development - **DEMOTED TO PHASE 3**
- CI/CD and infrastructure automation
- Cloud deployment and monitoring
- Advanced Git integration features

For current development tasks and progress, see `todo.md`.
- Primary directive: NEVER DO MOCK IMPLEMENTATIONS, ALWAYS FOCUS ON FUNCTIONAL IMPLEMENTATIONS.