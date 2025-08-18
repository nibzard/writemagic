# Contributing to WriteMagic

Thank you for your interest in contributing to WriteMagic! This guide will help you get started with our development workflow and sub-agent coordination system.

## 🚀 Quick Start

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Set up** the development environment
4. **Create** a feature branch following our naming conventions
5. **Make** your changes following our coding standards
6. **Test** your changes thoroughly
7. **Submit** a pull request

## 🏗️ Development Setup

### Prerequisites

- **Rust** 1.75+ with cross-compilation targets
- **Node.js** 18+ (for tooling)
- **Android Studio** with NDK (for Android development)
- **Xcode** 15+ (for iOS development, macOS only)
- **Docker** (optional, for containerized development)

### Environment Setup

#### Option 1: DevContainer (Recommended)
```bash
# Open in VS Code with Dev Containers extension
code .
# Select "Reopen in Container"
```

#### Option 2: Local Setup
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/writemagic.git
cd writemagic

# Set up git hooks and configuration
./scripts/setup-git.sh

# Install pre-commit hooks
pre-commit install

# Install Rust targets
rustup target add aarch64-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

# Install development tools
cargo install cargo-audit cargo-deny cargo-udeps
```

## 🤖 Sub-Agent Coordination System

WriteMagic uses a specialized sub-agent coordination system. When contributing, please work with the appropriate agent for your changes:

### Available Sub-Agents

| Agent | Scope | Contact Pattern |
|-------|-------|----------------|
| **@project-manager** | Task coordination, project structure | `[PROJECT-MANAGER]` |
| **@mobile-architect** | iOS/Android development | `[MOBILE-ARCHITECT]` |
| **@ai-integration-specialist** | AI/LLM features | `[AI-INTEGRATION-SPECIALIST]` |
| **@rust-core-engineer** | Rust core engine | `[RUST-CORE-ENGINEER]` |
| **@ux-writing-specialist** | UX/Writing experience | `[UX-WRITING-SPECIALIST]` |
| **@devops-platform-engineer** | CI/CD, infrastructure | `[DEVOPS-PLATFORM-ENGINEER]` |

### Agent Coordination Workflow

1. **Identify** the appropriate agent for your contribution
2. **Check** `todo.md` for existing related tasks
3. **Create** a branch following the pattern: `feature/[agent]-[task-name]`
4. **Add** your task to `todo.md` if it's not already present
5. **Update** task status as you progress

Example:
```bash
# Mobile feature
git checkout -b feature/mobile-architect-gesture-navigation

# AI enhancement  
git checkout -b feature/ai-integration-specialist-context-memory

# Infrastructure improvement
git checkout -b fix/devops-platform-engineer-ci-optimization
```

## 📝 Coding Standards

### Rust Code Style

```rust
// Use descriptive names and comprehensive documentation
/// Manages document lifecycle and content operations
pub struct DocumentManager {
    repository: Arc<dyn DocumentRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl DocumentManager {
    /// Creates a new document with validation and event publishing
    pub async fn create_document(
        &self,
        title: DocumentTitle,
        content: DocumentContent,
    ) -> Result<Document> {
        // Implementation with proper error handling
    }
}
```

**Requirements:**
- Follow `rustfmt` formatting (automatic with pre-commit)
- Use `clippy` recommendations (enforced in CI)
- Comprehensive documentation for public APIs
- Async/await patterns for I/O operations
- Proper error handling with custom error types

### Mobile Code Style

#### Android (Kotlin + Jetpack Compose)
```kotlin
@Composable
fun WritingScreen(
    viewModel: WritingViewModel = hiltViewModel(),
    onNavigateToAI: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Implementation
    }
}
```

#### iOS (Swift + SwiftUI)
```swift
struct WritingView: View {
    @StateObject private var viewModel = WritingViewModel()
    @State private var showingAIAssistant = false
    
    var body: some View {
        NavigationView {
            VStack(spacing: 16) {
                // Implementation
            }
        }
    }
}
```

**Requirements:**
- Material Design 3 (Android) / Human Interface Guidelines (iOS)
- ktlint (Android) / SwiftLint (iOS) compliance
- Accessibility support
- Proper state management
- FFI integration patterns

## 🔄 Git Workflow

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) with the following format:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

#### Types
- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks
- **perf**: Performance improvements
- **ci**: CI/CD changes

#### Scopes
- **mobile**: iOS/Android specific changes
- **core**: Rust core engine
- **ai**: AI integration features
- **ux**: User experience improvements
- **infra**: Infrastructure and deployment
- **docs**: Documentation
- **agents**: Sub-agent system

#### Examples
```bash
feat(mobile): add gesture navigation for multi-pane interface
fix(core): resolve memory leak in FFI string handling
docs(agents): update sub-agent coordination guidelines
```

### Branch Naming

```bash
# Feature branches
feature/[agent]-[brief-description]
feature/mobile-architect-gesture-nav
feature/ai-integration-specialist-claude-provider

# Bug fixes
fix/[agent]-[brief-description]  
fix/rust-core-engineer-memory-leak
fix/mobile-architect-ios-crash

# Documentation
docs/[area]-[brief-description]
docs/agents-coordination-guide
docs/mobile-architecture-update
```

### Pull Request Process

1. **Create** a descriptive PR title using conventional commit format
2. **Fill out** the PR template completely
3. **Ensure** all CI checks pass
4. **Request review** from relevant agent maintainers
5. **Address** feedback promptly
6. **Squash** commits before merging (if requested)

#### PR Template Checklist
- [ ] Tests added/updated and passing
- [ ] Documentation updated (if applicable)
- [ ] Conventional commit format used
- [ ] Agent coordination patterns followed
- [ ] Security considerations addressed
- [ ] Performance impact assessed
- [ ] Mobile platforms tested (if applicable)

## 🧪 Testing Guidelines

### Rust Tests

```bash
# Run all tests
cargo test --workspace

# Run specific domain tests
cargo test -p writemagic-writing
cargo test -p writemagic-ai

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage/
```

### Mobile Tests

#### Android
```bash
cd android
./gradlew test                    # Unit tests
./gradlew connectedAndroidTest    # Instrumented tests
```

#### iOS
```bash
cd ios
xcodebuild test -scheme WriteMagic -destination 'platform=iOS Simulator,name=iPhone 15'
```

### Integration Tests

```bash
# Cross-platform FFI tests
cargo test --test integration_tests

# End-to-end mobile tests
./scripts/test/e2e-mobile.sh
```

## 📋 Task Management

All project tasks are tracked in [`todo.md`](todo.md) with structured format:

```markdown
- [ ] [AGENT] Task description with clear acceptance criteria
  - Estimated effort: [S/M/L/XL]  
  - Dependencies: [List dependencies]
  - Acceptance criteria: [Clear definition of done]
```

### Adding New Tasks

1. **Identify** the appropriate agent
2. **Define** clear acceptance criteria
3. **Estimate** effort (S=1-2 days, M=3-5 days, L=1-2 weeks, XL=2+ weeks)
4. **List** dependencies on other tasks
5. **Update** `todo.md` with your task

### Task Status Updates

- **Pending**: `[ ]` - Not started
- **In Progress**: `[~]` - Currently being worked on  
- **Completed**: `[x]` - Finished and verified
- **Blocked**: `[!]` - Waiting on dependencies

## 🔒 Security Guidelines

### Code Security
- **Never** commit secrets, API keys, or passwords
- **Use** secure coding practices for FFI boundaries
- **Validate** all user inputs
- **Follow** OWASP guidelines for mobile security

### AI Safety
- **Filter** sensitive content before AI processing
- **Implement** rate limiting and cost controls
- **Respect** user privacy and data sovereignty
- **Handle** AI provider failures gracefully

### Dependency Management
- **Audit** new dependencies with `cargo audit`
- **Keep** dependencies updated via Dependabot
- **Review** licenses for compatibility
- **Minimize** external dependencies

## 🚀 Performance Guidelines

### Rust Core
- **Profile** performance-critical paths
- **Use** appropriate data structures
- **Minimize** allocations in hot paths
- **Leverage** Rust's zero-cost abstractions

### Mobile Apps
- **Optimize** for battery life
- **Minimize** memory usage
- **Use** native platform optimizations
- **Profile** with platform tools

### AI Integration
- **Cache** AI responses when appropriate
- **Implement** request deduplication
- **Monitor** token usage and costs
- **Provide** offline fallbacks

## 📚 Documentation

### Code Documentation
- **Document** all public APIs with examples
- **Include** safety notes for unsafe code
- **Explain** complex algorithms and data structures
- **Update** docs with code changes

### Architecture Documentation
- **Update** [`MOBILE_ARCHITECTURE.md`](MOBILE_ARCHITECTURE.md) for mobile changes
- **Update** [`SPECS_IMPROVED.md`](SPECS_IMPROVED.md) for major architecture changes
- **Document** agent coordination patterns
- **Include** decision records for major changes

## 🆘 Getting Help

### Questions and Discussions
- **GitHub Discussions**: For general questions and feature discussions
- **GitHub Issues**: For bug reports and feature requests
- **PR Comments**: For code-specific questions

### Agent-Specific Help
- **@project-manager**: Project coordination and task management
- **@mobile-architect**: Mobile development questions
- **@ai-integration-specialist**: AI/LLM integration help
- **@rust-core-engineer**: Rust core engine assistance
- **@ux-writing-specialist**: UX and writing experience guidance  
- **@devops-platform-engineer**: CI/CD and infrastructure support

### Resources
- [Rust Documentation](https://doc.rust-lang.org/)
- [Android Developers](https://developer.android.com/)
- [iOS Developer Documentation](https://developer.apple.com/documentation/)
- [Conventional Commits](https://www.conventionalcommits.org/)

## 🏆 Recognition

Contributors are recognized in several ways:
- **README credits**: Major contributors listed in README
- **Release notes**: Contributions highlighted in release announcements
- **Agent badges**: Special recognition for agent-specific expertise
- **Community showcase**: Outstanding contributions featured in discussions

Thank you for contributing to WriteMagic! Together, we're building the future of AI-powered writing. 🎉