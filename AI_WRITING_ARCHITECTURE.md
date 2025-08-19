# AI Writing Service Architecture

This document describes the AI orchestration service that bridges AI providers with WriteMagic's writing domain, providing contextual, document-aware writing assistance.

## Overview

The AI Writing Service creates a comprehensive bridge between generic AI providers (Claude, GPT-4, local models) and WriteMagic's writing domain. Instead of providing raw AI API access, it offers writing-specific functionality that understands documents, projects, and writing workflows.

## Architecture Components

### 1. Core AI Layer (`core/ai/`)

#### AIOrchestrationService
- **Purpose**: Provider-agnostic LLM integration with intelligent fallback
- **Features**:
  - Automatic provider switching based on availability and performance
  - Health monitoring and response time tracking  
  - Rate limiting and cost optimization
  - Response caching with TTL

#### AIWritingService  
- **Purpose**: Writing-specific AI functionality
- **Features**:
  - Content generation and completion
  - Document summarization
  - Writing improvement and style suggestions
  - Grammar checking and tone adjustment
  - Content analysis with readability metrics
  - Multi-turn conversation management

#### Key Value Objects
- `WritingContext`: Document and project context for AI requests
- `WritingAssistanceType`: Enumeration of writing assistance types
- `WritingPreferences`: User preferences for tone, audience, formality
- `ConversationSession`: Manages multi-turn interactions per document

### 2. Writing Domain Integration (`core/writing/`)

#### IntegratedWritingService
- **Purpose**: Bridges AI services with document management
- **Features**:
  - Document-aware AI assistance with automatic content application
  - Project context integration for related document awareness
  - Conversation session management per document
  - Support for text selections and partial document editing

#### CoreEngine Enhancement
- **AI Integration**: Automatic initialization of AI services when API keys are provided
- **Service Orchestration**: Manages lifecycle of all AI and writing services
- **Configuration**: Comprehensive config for AI providers, context limits, and filtering

### 3. Provider Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Claude API    │    │   OpenAI API    │    │  Local Models   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ AI Orchestration│
                    │    Service      │
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │  AI Writing     │
                    │    Service      │
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Integrated     │
                    │ Writing Service │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Document      │    │    Project      │    │   Content       │
│  Management     │    │  Management     │    │   Analysis      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Writing Assistance Types

### Content Operations
- **ContentGeneration**: Create new content based on prompts
- **ContentCompletion**: Continue existing content naturally
- **Summarization**: Create concise summaries with configurable length
- **Brainstorming**: Generate ideas and explore concepts

### Quality Enhancement
- **Improvement**: Enhance clarity, flow, and impact
- **GrammarCheck**: Identify and correct grammatical errors
- **StyleSuggestions**: Improve word choice and sentence structure
- **Rewrite**: Restructure content while preserving meaning

### Content Manipulation  
- **Expand**: Add details, examples, and supporting information
- **Condense**: Reduce length while preserving key information
- **Tone(adjustment)**: Adjust writing tone (Professional, Casual, Academic, etc.)
- **Outline**: Create structured outlines from content

## Context Management

### Writing Context Assembly
```rust
WritingContext {
    document_id: EntityId,
    document_title: String,
    document_content: String,
    content_type: ContentType,
    selection: Option<TextSelection>,
    project_context: Option<ProjectContext>,
    conversation_history: Vec<ConversationEntry>,
    user_preferences: WritingPreferences,
}
```

### Project Context Integration
- Automatically includes related documents for context
- Provides project description and goals
- Maintains awareness of document relationships

### Conversation Continuity
- Per-document conversation sessions
- Context-aware multi-turn interactions
- Conversation history management with token limits

## Safety and Quality Features

### Content Filtering
- PII detection and removal
- Sensitive information filtering
- Configurable content safety rules

### Quality Assurance
- Response validation and quality scoring
- Confidence metrics for AI suggestions
- A/B testing framework for response quality

### Cost Optimization
- Token usage monitoring and reporting
- Response caching to reduce API calls
- Provider cost comparison and optimization

## Usage Examples

### Basic Document Enhancement
```rust
// Initialize engine with AI capabilities
let engine = ApplicationConfigBuilder::new()
    .with_claude_key("your-claude-key".to_string())
    .with_openai_key("your-openai-key".to_string())
    .build()
    .await?;

// Get integrated writing service
let writing_service = engine.integrated_writing_service().unwrap();

// Generate content for document
let response = writing_service.generate_content_for_document(
    document_id,
    "Write a comprehensive introduction to microservices".to_string(),
    Some(500), // Target token count
    true,      // Apply to document
    None,
).await?;
```

### Advanced Workflow
```rust
// Create custom assistance request
let response = writing_service.provide_custom_assistance(
    document_id,
    WritingAssistanceType::Tone(ToneAdjustment::Academic),
    Some("Adjust for graduate-level academic audience".to_string()),
    Some(text_selection),
    Some(custom_preferences),
    true, // Apply changes
    None,
).await?;

// Analyze content quality  
let analysis = writing_service.analyze_document_content(
    document_id, 
    None
).await?;

println!("Readability: {}", analysis.readability.reading_level_description());
println!("Primary tone: {:?}", analysis.tone_analysis.primary_tone);
```

## Mobile Platform Integration

### FFI Layer Support
The AI writing services are designed to work seamlessly through the FFI layer:

- **iOS**: Swift can call Rust functions for AI assistance
- **Android**: Kotlin can access AI features via JNI
- **Async Operations**: Background AI processing with progress callbacks
- **Offline Graceful Degradation**: Local analysis when AI unavailable

### Platform-Specific Optimizations
- **Battery-Aware**: Reduces API calls when on battery power
- **Network-Aware**: Caches responses for offline usage
- **Memory-Efficient**: Streaming responses for large content generation

## Configuration and Deployment

### API Key Management
```rust
let config = ApplicationConfig {
    ai: AIConfig {
        claude_api_key: Some(env::var("CLAUDE_API_KEY")?),
        openai_api_key: Some(env::var("OPENAI_API_KEY")?),
        default_model: "claude-3-5-sonnet-20241022".to_string(),
        max_context_length: 8000,
        enable_content_filtering: true,
        cache_ttl_seconds: 600,
    },
    // ... other config
};
```

### Provider Fallback Configuration
```rust
// Set custom fallback order
ai_service.set_fallback_order(vec![
    "claude".to_string(),
    "openai".to_string(),
    "local".to_string(),
]);
```

### Health Monitoring
```rust
// Check provider health
let health_status = engine.check_ai_provider_health().await?;
for (provider, is_healthy) in health_status {
    println!("{}: {}", provider, if is_healthy { "✅" } else { "❌" });
}

// Get detailed statistics
let stats = engine.get_ai_provider_stats().await?;
```

## Benefits

### For Writers
- **Contextual Assistance**: AI understands document and project context
- **Quality Enhancement**: Automated grammar, style, and tone improvements  
- **Productivity**: Faster content creation and editing
- **Consistency**: Maintains writing style across documents

### For Developers  
- **Provider Agnostic**: Easy switching between AI providers
- **Robust Fallback**: Automatic failover ensures reliability
- **Cost Optimization**: Intelligent caching and provider selection
- **Easy Integration**: Simple API for complex AI operations

### For Mobile Apps
- **Native Performance**: Rust core ensures optimal performance
- **Battery Efficient**: Smart caching reduces network usage
- **Offline Capable**: Local analysis when connectivity limited
- **Platform Integration**: Native UI with Rust AI backend

## Future Enhancements

### Planned Features
- **Local Model Support**: Integration with Ollama and other local models
- **Voice Integration**: Speech-to-text and text-to-speech capabilities
- **Collaborative Writing**: Multi-user AI-assisted editing
- **Template System**: AI-powered document templates
- **Advanced Analytics**: Writing pattern analysis and suggestions

### Scalability Improvements
- **Distributed Processing**: Multi-node AI request processing
- **Model Fine-tuning**: Custom models for specific writing domains
- **Edge Computing**: On-device processing for privacy-sensitive content
- **Advanced Caching**: Semantic similarity-based response caching

## Conclusion

The AI Writing Service architecture provides a robust, scalable, and user-friendly bridge between cutting-edge AI capabilities and practical writing workflows. By focusing on document-aware, contextual assistance rather than generic AI access, it delivers genuine value to writers while maintaining technical excellence for developers and optimal performance for mobile platforms.

The modular design allows for easy extension and customization while the provider-agnostic approach ensures long-term flexibility as the AI landscape evolves. This architecture positions WriteMagic as a leader in AI-assisted writing applications.