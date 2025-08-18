# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WriteMagic is a cross-platform AI-powered writing application built with a Rust core engine and native mobile UIs. The architecture follows Domain-Driven Design principles with specialized sub-agents coordinating development across different expertise areas.

## Core Architecture

### Multi-Layer Architecture
- **Mobile UIs**: Native iOS (SwiftUI) and Android (Jetpack Compose) applications
- **Rust Core**: Shared business logic engine with FFI bindings for mobile platforms
- **AI Integration**: Provider-agnostic LLM integration with Claude, GPT-4, and local model support
- **Data Layer**: SQLite for local storage, Git for version control, cloud sync for cross-device

### Domain Structure
The Rust core is organized into bounded contexts:
- **Writing Domain**: Document management, content editing, project organization
- **AI Domain**: LLM integration, context management, response processing
- **Project Domain**: Multi-pane workspaces, session management
- **Version Control Domain**: Git integration with beautiful timeline visualization
- **Agent Domain**: File-based YAML agents for background processing

## Development Workflow

### Git Workflow and Conventional Commits
This project uses strict conventional commit standards. Set up git configuration:
```bash
./scripts/setup-git.sh
```

Commit format: `type(scope): description`
- **Types**: feat, fix, docs, style, refactor, test, chore, perf, ci
- **Scopes**: mobile, core, ai, ux, infra, docs, agents

Quick commit aliases:
```bash
git feat mobile "add gesture navigation"
git fix core "resolve memory leak in FFI"
git docs agents "update sub-agent documentation"
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

## Sub-Agent Coordination System

The project uses 6 specialized sub-agents located in `.claude/agents/`:

### 1. Project Manager (`@project-manager`)
- Maintains todo.md and coordinates all agents
- Enforces git workflow and conventional commits
- Tracks milestones and manages project risks

### 2. Mobile Architect (`@mobile-architect`)
- iOS/Android development and cross-platform architecture
- Platform-specific optimizations and native integrations
- Mobile performance and UX implementation

### 3. AI Integration Specialist (`@ai-integration-specialist`)
- LLM provider abstractions and fallback strategies
- Context management and response processing
- AI safety, cost optimization, and quality assurance

### 4. Rust Core Engineer (`@rust-core-engineer`)
- High-performance Rust engine with FFI bindings
- Memory management, async operations, cross-platform compilation
- Domain-driven design implementation

### 5. UX Writing Specialist (`@ux-writing-specialist`)
- Writing-focused user experience and workflow design
- Multi-pane interfaces, gesture interactions, accessibility
- Creative flow optimization

### 6. DevOps Platform Engineer (`@devops-platform-engineer`)
- Infrastructure, CI/CD, monitoring, and security
- Kubernetes deployment and scalable cloud architecture

### Agent Coordination Patterns
```
# Full feature development
@project-manager Break down feature into tasks
@ux-writing-specialist Design the interface
@mobile-architect Implement mobile components
@rust-core-engineer Build core logic
@ai-integration-specialist Add AI features
@devops-platform-engineer Deploy and monitor
@project-manager Track progress and update todo.md
```

## Technical Specifications

### Cross-Platform FFI Strategy
- Rust core exposes C-compatible interfaces
- Android: JNI bindings with Kotlin
- iOS: Swift-C++ interop with Rust
- Shared business logic, platform-specific UI

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

### Multi-Pane Writing System
- Each pane corresponds to a Git branch
- Content can be moved between panes via drag/drop
- AI can suggest alternative approaches in new panes
- Timeline visualizes writing journey across branches

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

# Build iOS app
cd ios && xcodebuild -scheme WriteMagic build

# Run linting
cargo clippy -- -D warnings
```

## Key Implementation Notes

### Mobile Performance Priorities
- Native UI components over cross-platform frameworks
- Optimize for memory usage and battery life
- Implement proper background task management
- Use platform-specific optimizations (iOS: Metal, Android: Vulkan)

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

Refer to `todo.md` for current sprint tasks and priorities. The project is in initial setup phase focusing on:
1. Rust core foundation with workspace structure
2. Mobile project setup with FFI integration
3. AI provider abstraction design
4. Basic CI/CD pipeline establishment

For detailed technical specifications, see `SPECS_IMPROVED.md`.