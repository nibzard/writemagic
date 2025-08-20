# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the WriteMagic codebase.

## Project Overview

WriteMagic is an AI-powered writing application with a **production-ready foundation** built on Domain-Driven Design principles. The project focuses on Android mobile and web platforms with a shared Rust core engine.

## 📋 Essential Reference Documents

Before starting work, always review these key documents:

- **`README.md`** - Project overview, setup instructions, and getting started guide
- **`todo.md`** - Current development priorities, task management, and project roadmap
- **`SPECS.md`** - Detailed technical specifications and architectural decisions
- **`CONTRIBUTING.md`** - Development workflow, coding standards, and contribution guidelines
- **`SECURITY.md`** - Security policies, vulnerability reporting, and compliance requirements

## 🎯 Current Development Status

**Status**: MVP Development Phase - Android + Web Focus  
**Architecture**: Solid foundation established, ready for feature completion  
**Quality**: Production-ready core with comprehensive test coverage

### Current Priorities
1. Complete Android native application with FFI integration
2. Finish Progressive Web App with WASM integration  
3. Code quality refinement (reduce compilation warnings)
4. AI service reliability and performance enhancements

## 🤖 Available Specialized Subagents

Use these specialized subagents for complex tasks that match their expertise:

### Core Development
- **`rust-core-engineer`** - High-performance Rust engine, FFI bindings, domain-driven design, memory safety
- **`ai-integration-specialist`** - LLM provider abstraction, AI feature implementation, fallback strategies, cost optimization

### Platform Development  
- **`mobile-architect`** - iOS/Android development, cross-platform architecture, native performance with Rust FFI
- **`ux-writing-specialist`** - Writing-focused UX design, multi-pane workflows, accessibility, creative flow optimization

### Infrastructure & Operations
- **`devops-platform-engineer`** - Infrastructure, CI/CD, deployment automation, monitoring, security, scalable cloud infrastructure
- **`project-manager`** - Project coordination, task management, git workflow, milestone tracking, development standards

### When to Use Subagents
- **Complex multi-step tasks** requiring specialized domain knowledge
- **Cross-platform integration** issues needing architectural decisions
- **Performance optimization** and security implementation
- **Project planning** and workflow coordination

## 💻 Development Workflow

### Task Management
- All tasks are managed in `todo.md` with structured priorities
- Always update task status when starting, progressing, or completing work
- Follow conventional commit standards (see `CONTRIBUTING.md`)

### Git Workflow
```bash
# Setup (run once)
./scripts/setup-git.sh

# Standard workflow
git feat scope "description"  # New features
git fix scope "description"   # Bug fixes
git docs "description"        # Documentation updates
```

### Build Commands
```bash
# Build entire workspace
cargo build --workspace

# Run all tests  
cargo test --workspace

# Build Android app
cd android && ./gradlew assembleDebug

# Build WASM for web
./scripts/build-wasm.sh
```

## 🏗️ Architecture Principles

### Core Values
- **Functional over Mock**: Always implement working functionality, never placeholder/mock code
- **Domain-Driven Design**: Business logic organized in clear domain boundaries
- **Cross-Platform**: Shared Rust core with platform-specific UI layers
- **Performance First**: Optimize for memory, battery, and responsiveness

### Technology Stack
- **Core**: Rust with Domain-Driven Design architecture
- **Android**: Kotlin/Jetpack Compose with FFI integration
- **Web**: Progressive Web App with WASM integration
- **AI**: Provider-agnostic integration (Claude, GPT-4, local models)
- **Data**: SQLite (local), IndexedDB (web)

## 🔧 Development Context

### Current Focus: MVP Completion
The project has a **solid architectural foundation** and is focused on completing the Minimum Viable Product across Android and web platforms.

### Key Success Metrics
- Functional Android app with native UI and Rust FFI integration
- Deployed Progressive Web App with offline WASM functionality
- Professional code quality (<10 compilation warnings)
- Robust AI service integration with intelligent fallback
- Comprehensive documentation reflecting current state

### Out of Scope (Post-MVP)
- iOS application (planned for Phase 3)
- Desktop applications
- Advanced Git integration features
- Enterprise scaling features

## 📝 Development Guidelines

### Always Do
- Read `todo.md` for current priorities before starting work
- Implement functional, working code (never mocks or placeholders)
- Update task status and documentation as you progress
- Use appropriate subagents for specialized work
- Follow domain-driven design patterns established in the codebase

### Never Do
- Create mock implementations or placeholder code
- Make architectural changes without reviewing `SPECS.md`
- Work on out-of-scope features (iOS, enterprise features, etc.)
- Skip updating `todo.md` when completing tasks
- Implement security-sensitive features without consulting security policies

## 🎯 Getting Started

1. **Read Documentation**: Review `README.md`, `todo.md`, and `SPECS.md`
2. **Understand Current State**: Check the latest development wave priorities in `todo.md`
3. **Choose Appropriate Tools**: Use specialized subagents for complex tasks
4. **Follow Workflow**: Use conventional commits and update task management
5. **Focus on MVP**: Prioritize Android + Web completion over new features

---

**Remember**: WriteMagic has a solid, production-ready foundation. Focus on completing the MVP applications rather than architectural changes. The codebase is ready for confident, accelerated development velocity.