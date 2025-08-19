# Enhanced CoreEngine - Comprehensive Dependency Injection

The Enhanced CoreEngine is the central dependency injection container for WriteMagic, orchestrating all domains, services, and integrations across the entire application stack.

## Architecture Overview

The CoreEngine follows Domain-Driven Design principles and provides a unified entry point for:

- **Repository Layer**: Document and Project repositories with SQLite/In-Memory implementations
- **AI Integration**: Claude, OpenAI providers with intelligent fallback and health monitoring
- **Configuration Management**: Centralized configuration for database, AI, logging, and security
- **Service Orchestration**: Context management, content filtering, and domain services
- **Cross-Platform Support**: Native FFI bindings for iOS and Android

## Key Features

### 1. Comprehensive Configuration Management

```rust
use writemagic_writing::ApplicationConfigBuilder;

let engine = ApplicationConfigBuilder::new()
    .with_sqlite("./app.db")
    .with_claude_key("your-claude-key")
    .with_openai_key("your-openai-key")
    .with_default_model("claude-3-haiku-20240307")
    .with_max_context_length(8000)
    .with_content_filtering(true)
    .with_log_level("info")
    .with_encryption_at_rest(true)
    .build()
    .await?;
```

### 2. AI Provider Integration with Fallback

The CoreEngine automatically configures AI providers and handles failover:

```rust
// AI completion with automatic provider fallback
let completion = engine.complete_text(
    "Write a summary of this document".to_string(),
    Some("claude-3-haiku-20240307".to_string())
).await?;

// Health monitoring
let health = engine.check_ai_provider_health().await?;
let stats = engine.get_ai_provider_stats().await?;
```

### 3. Repository Access

```rust
// Document operations
let doc_repo = engine.document_repository();
let document = Document::new("Title", "Content", ContentType::Markdown, None);
let saved_doc = doc_repo.save(&document).await?;

// Project operations  
let project_repo = engine.project_repository();
let project = Project::new("My Project", Some("Description"), None);
let saved_project = project_repo.save(&project).await?;
```

### 4. Configuration Types

#### ApplicationConfig
- **DatabaseConfig**: SQLite connection settings, pooling, WAL mode
- **AIConfig**: API keys, model preferences, context limits, filtering
- **LoggingConfig**: Log levels, tracing configuration  
- **SecurityConfig**: Encryption, rate limiting, security policies

#### Builder Pattern
The `ApplicationConfigBuilder` provides a fluent interface for configuration:

```rust
let config = ApplicationConfigBuilder::new()
    .with_sqlite_in_memory()          // Database configuration
    .with_claude_key(api_key)         // AI provider setup
    .with_content_filtering(true)     // Security features
    .with_log_level("debug")          // Logging configuration
    .build()
    .await?;
```

## Mobile Integration

### Android FFI

```java
// Initialize with AI providers
WriteMagicCore.initialize(claudeKey, openaiKey);

// Create document
String docJson = WriteMagicCore.createDocument("Title", "Content", "markdown");

// AI completion
String completion = WriteMagicCore.completeText("Prompt", "claude-3-haiku");
```

### iOS FFI

```c
// Initialize with AI providers
writemagic_initialize_with_ai(1, claude_key, openai_key);

// Create document  
char* doc_id = writemagic_create_document("Title", "Content", "markdown");

// AI completion
char* completion = writemagic_complete_text("Prompt", "claude-3-haiku");
writemagic_free_string(completion);
```

## Service Access

### AI Services
- **AIOrchestrationService**: Provider management with intelligent fallback
- **ContextManagementService**: Context window management and optimization
- **ContentFilteringService**: Sensitive content detection and filtering

### Database Services
- **DatabaseManager**: Connection pooling, migrations, health monitoring
- **Repository Pattern**: Clean data access abstractions

### Configuration Services
- **Config Validation**: Comprehensive configuration validation
- **Health Monitoring**: Service health checks and monitoring

## Error Handling

The CoreEngine uses comprehensive error handling with specific error types:

```rust
match engine.complete_text(prompt, None).await {
    Ok(completion) => println!("Success: {}", completion),
    Err(WritemagicError::Configuration(msg)) => println!("Config error: {}", msg),
    Err(WritemagicError::AIProvider(msg)) => println!("AI error: {}", msg),
    Err(WritemagicError::Database(msg)) => println!("Database error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Testing Support

The CoreEngine provides comprehensive testing utilities:

```rust
// In-memory testing
let engine = CoreEngine::new_in_memory().await?;

// SQLite in-memory for integration tests
let engine = CoreEngine::new_sqlite_in_memory().await?;

// Mock AI providers for testing
let engine = ApplicationConfigBuilder::new()
    .with_sqlite_in_memory()
    .with_claude_key("mock-key")
    .build()
    .await?;
```

## Performance Optimizations

- **Connection Pooling**: Efficient database connection management
- **AI Response Caching**: Intelligent caching with TTL support
- **Rate Limiting**: Built-in rate limiting for AI providers
- **Memory Management**: Efficient memory usage with Arc/Mutex patterns
- **Async/Await**: Full async support throughout the stack

## Migration from Legacy CoreEngine

The enhanced CoreEngine maintains backwards compatibility:

```rust
// Legacy usage still works
let engine = CoreEngine::new_default().await?;
let engine = CoreEngine::new_in_memory().await?;

// Enhanced usage with new features
let engine = ApplicationConfigBuilder::new()
    .with_sqlite()
    .with_claude_key(key)
    .build()
    .await?;
```

## Configuration Examples

### Development Configuration
```rust
let dev_config = ApplicationConfigBuilder::new()
    .with_sqlite_in_memory()
    .with_log_level("debug")
    .with_content_filtering(false)  // Relaxed for development
    .build()
    .await?;
```

### Production Configuration  
```rust
let prod_config = ApplicationConfigBuilder::new()
    .with_sqlite("./production.db")
    .with_claude_key(std::env::var("CLAUDE_API_KEY")?)
    .with_openai_key(std::env::var("OPENAI_API_KEY")?)
    .with_content_filtering(true)
    .with_encryption_at_rest(true)
    .with_api_rate_limit(1000)
    .with_log_level("info")
    .build()
    .await?;
```

### Mobile Configuration
```rust
let mobile_config = ApplicationConfigBuilder::new()
    .with_sqlite_in_memory()  // Mobile apps often use in-memory
    .with_claude_key(api_key)
    .with_max_context_length(4000)  // Reduced for mobile
    .with_content_filtering(true)
    .build()
    .await?;
```

## Shutdown and Resource Management

```rust
// Graceful shutdown
engine.shutdown().await;  // Closes DB connections, cleans up resources
```

## Thread Safety

The CoreEngine is fully thread-safe and can be shared across threads:

```rust
let engine = Arc::new(engine);
let engine_clone = Arc::clone(&engine);

tokio::spawn(async move {
    let result = engine_clone.complete_text("Prompt".to_string(), None).await;
    // Handle result
});
```

## Dependencies

The enhanced CoreEngine integrates the following WriteMagic components:

- `writemagic-shared`: Common types, errors, database management
- `writemagic-ai`: AI provider abstractions and implementations  
- `writemagic-writing`: Document and project domain logic
- Platform-specific logging (Android/iOS)

## Future Enhancements

Planned improvements for the CoreEngine:

1. **Plugin System**: Dynamic loading of custom providers and services
2. **Metrics Collection**: Built-in metrics and monitoring
3. **Configuration Hot-Reload**: Dynamic configuration updates
4. **Advanced Caching**: Multi-layer caching with persistence
5. **Event Sourcing**: Domain event processing and replay
6. **Backup/Restore**: Automatic backup and restore capabilities

## Example Usage

See `/examples/enhanced_core_engine.rs` for a comprehensive example demonstrating all features of the enhanced CoreEngine.