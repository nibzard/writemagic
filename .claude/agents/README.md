# WriteMagic Sub-Agents Team

This directory contains specialized sub-agents designed to work collaboratively on the WriteMagic project. Each agent has specific expertise and responsibilities that complement the others.

## Agent Team Structure

### 1. Project Manager Agent
**Focus**: Task coordination, git workflow, and team synchronization
- Maintains centralized todo.md with all project tasks
- Enforces conventional commit standards and git workflow
- Coordinates work between all sub-agents
- Tracks progress, milestones, and manages risks

### 2. Mobile Architect Agent
**Focus**: Cross-platform mobile development and native performance
- Designs mobile application architecture
- Implements iOS and Android specific features
- Optimizes for mobile performance and user experience
- Handles platform-specific integrations

### 3. AI Integration Specialist Agent
**Focus**: LLM integration and intelligent writing assistance
- Designs AI provider abstractions and fallback strategies
- Implements context management and memory systems
- Optimizes AI usage costs and response quality
- Ensures AI safety and content filtering

### 4. Rust Core Engineer Agent
**Focus**: High-performance core library and FFI bindings
- Builds the Rust core engine with optimal performance
- Creates safe FFI bindings for mobile platforms
- Implements async operations and memory management
- Ensures cross-platform compatibility

### 5. UX Writing Specialist Agent
**Focus**: Writing-focused user experience and workflow optimization
- Designs intuitive writing workflows and interfaces
- Creates multi-pane navigation and gesture interactions
- Ensures accessibility and inclusive design
- Optimizes for creative flow and productivity

### 6. DevOps Platform Engineer Agent
**Focus**: Infrastructure, deployment, and operational excellence
- Designs scalable cloud infrastructure
- Implements CI/CD pipelines and automation
- Sets up monitoring and observability systems
- Ensures security and compliance

## Collaboration Patterns

### Primary Workflows

1. **Feature Development**
   ```
   Project Manager (plans) → UX Writing Specialist → Mobile Architect → Rust Core Engineer
   ↓
   AI Integration Specialist → DevOps Platform Engineer → Project Manager (tracks)
   ```

2. **Architecture Decisions**
   ```
   Project Manager (coordinates) → Mobile Architect ↔ Rust Core Engineer ↔ AI Integration Specialist
   ↓
   UX Writing Specialist ↔ DevOps Platform Engineer → Project Manager (documents)
   ```

3. **Deployment Pipeline**
   ```
   Project Manager (schedules) → Rust Core Engineer → Mobile Architect → DevOps Platform Engineer
   ↓
   AI Integration Specialist → UX Writing Specialist (testing) → Project Manager (releases)
   ```

### Cross-Agent Dependencies

| Agent | Dependencies | Provides To |
|-------|--------------|-------------|
| **Project Manager** | All agents (status updates) | All agents (task coordination, git management) |
| **Mobile Architect** | Rust Core (FFI), UX (designs), PM (tasks) | All agents (platform constraints), PM (progress) |
| **AI Integration** | Rust Core (business logic), PM (tasks) | Mobile Architect (AI features), PM (progress) |
| **Rust Core** | DevOps (build tools), PM (tasks) | Mobile Architect, AI Integration, PM (progress) |
| **UX Writing** | PM (tasks) | Mobile Architect, AI Integration, PM (designs) |
| **DevOps Platform** | All agents (deployment artifacts), PM (tasks) | All agents (infrastructure), PM (deployment status) |

## Usage Guidelines

### When to Invoke Specific Agents

**Project Manager**:
- Task planning and assignment
- Progress tracking and coordination
- Git workflow and conventional commits
- Sprint planning and milestone tracking
- Risk identification and mitigation
- Cross-agent communication issues

**Mobile Architect**: 
- Native mobile development questions
- Platform-specific implementation
- Cross-platform architecture decisions
- Mobile performance optimization

**AI Integration Specialist**:
- LLM provider integration
- AI response processing
- Context management
- AI safety and filtering

**Rust Core Engineer**:
- Core business logic implementation
- FFI binding design
- Performance optimization
- Memory management

**UX Writing Specialist**:
- User interface design
- Writing workflow optimization
- Accessibility requirements
- User experience research

**DevOps Platform Engineer**:
- Infrastructure design
- Deployment automation
- Monitoring and observability
- Security and compliance

### Collaboration Commands

Use these patterns to coordinate multiple agents:

```
# Project planning and coordination
@project-manager Plan the next sprint and assign tasks
@project-manager Update todo.md with current progress
@project-manager Review git history and ensure conventional commits

# Architecture review with multiple perspectives
@project-manager Coordinate architecture review
@mobile-architect @rust-core-engineer Review the FFI interface design

# Full feature development workflow
@project-manager Break down feature into tasks
@ux-writing-specialist Design the multi-pane interface
@mobile-architect Implement the mobile components  
@rust-core-engineer Build the core logic
@ai-integration-specialist Add AI features
@devops-platform-engineer Deploy and monitor
@project-manager Track progress and update todo.md

# Performance optimization
@project-manager Identify performance bottlenecks
@rust-core-engineer @mobile-architect Optimize memory usage
@devops-platform-engineer Monitor production performance
@project-manager Document optimizations and results

# Release coordination
@project-manager Plan release timeline
@devops-platform-engineer Prepare deployment pipeline
@project-manager Coordinate testing and QA
@project-manager Execute release with conventional commits
```

## Quality Assurance

Each agent is responsible for:
- **Code Reviews**: Review implementations in their expertise area
- **Testing**: Ensure comprehensive testing coverage
- **Documentation**: Maintain clear documentation
- **Best Practices**: Enforce domain-specific best practices
- **Knowledge Sharing**: Share insights across the team

## Evolution and Maintenance

Agents should be updated when:
- New technologies are adopted
- Best practices change
- Project requirements evolve
- Performance bottlenecks are identified
- User feedback indicates UX improvements needed

Regular agent reviews should occur:
- After major feature releases
- When adopting new technologies
- When performance issues arise
- Based on user feedback and analytics