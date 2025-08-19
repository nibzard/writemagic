//! Example demonstrating the enhanced CoreEngine with comprehensive dependency injection
//! 
//! This example shows how to initialize and use the WriteMagic CoreEngine with:
//! - Database configuration (SQLite/In-Memory)
//! - AI provider integration (Claude, OpenAI)
//! - Configuration management
//! - Graceful shutdown
//! - Error handling

use writemagic_writing::{
    ApplicationConfigBuilder, CoreEngine, ApplicationConfig, AIConfig,
    entities::Document, 
    CompletionRequest, Message
};
use writemagic_shared::{ContentType, EntityId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 WriteMagic Enhanced CoreEngine Example");
    
    // Example 1: Basic engine with in-memory storage
    println!("\n📝 Example 1: Basic Engine (In-Memory)");
    let basic_engine = CoreEngine::new_in_memory().await?;
    println!("✅ Basic engine initialized");
    
    // Test document operations
    let doc = Document::new(
        "My First Document".to_string(),
        "This is the content of my first document.".to_string(),
        ContentType::Markdown,
        None,
    );
    
    let repo = basic_engine.document_repository();
    let saved_doc = repo.save(&doc).await?;
    println!("✅ Document created: {}", saved_doc.id);
    
    basic_engine.shutdown().await;
    
    // Example 2: Enhanced engine with AI integration
    println!("\n🤖 Example 2: Enhanced Engine with AI");
    
    // Note: In a real application, you'd get these from environment variables or config files
    let claude_key = std::env::var("CLAUDE_API_KEY").ok();
    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    
    let enhanced_engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key(claude_key.unwrap_or_else(|| "demo-key".to_string()))
        .with_openai_key(openai_key.unwrap_or_else(|| "demo-key".to_string()))
        .with_default_model("claude-3-haiku-20240307".to_string())
        .with_max_context_length(8000)
        .with_content_filtering(true)
        .with_log_level("info".to_string())
        .with_encryption_at_rest(true)
        .build()
        .await?;
    
    println!("✅ Enhanced engine initialized with AI integration");
    
    // Validate configuration
    let issues = enhanced_engine.validate_config();
    if !issues.is_empty() {
        println!("⚠️  Configuration issues:");
        for issue in issues {
            println!("   - {}", issue);
        }
    }
    
    // Check AI provider health
    println!("\n🏥 Checking AI Provider Health");
    let health_status = enhanced_engine.check_ai_provider_health().await?;
    for (provider, is_healthy) in health_status {
        println!("   {} -> {}", provider, if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
    }
    
    // Get AI provider statistics
    println!("\n📊 AI Provider Statistics");
    let stats = enhanced_engine.get_ai_provider_stats().await?;
    for (provider, stat) in stats {
        println!("   {}: {}", provider, stat);
    }
    
    // Test AI completion (will work if valid API keys are provided)
    println!("\n💬 Testing AI Completion");
    match enhanced_engine.complete_text("Write a haiku about programming".to_string(), None).await {
        Ok(completion) => {
            println!("✅ AI Completion successful:");
            println!("   {}", completion);
        }
        Err(e) => {
            println!("⚠️  AI Completion failed (expected without valid API keys): {}", e);
        }
    }
    
    // Test content filtering
    if let Some(filter) = enhanced_engine.content_filtering_service() {
        println!("\n🔒 Testing Content Filtering");
        let sensitive_content = "My password is secret123";
        match filter.filter_content(sensitive_content) {
            Ok(filtered) => println!("   Content passed filtering: {}", filtered),
            Err(e) => println!("   Content blocked by filter: {}", e),
        }
    }
    
    // Example 3: Custom configuration
    println!("\n⚙️  Example 3: Custom Configuration");
    
    let custom_config = ApplicationConfig {
        database: writemagic_shared::DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 5,
            min_connections: 1,
            enable_wal: false,
            enable_foreign_keys: true,
        },
        ai: AIConfig {
            claude_api_key: None,
            openai_api_key: None,
            default_model: "gpt-3.5-turbo".to_string(),
            max_context_length: 4000,
            enable_content_filtering: false,
            cache_ttl_seconds: 300,
        },
        logging: writemagic_writing::LoggingConfig {
            level: "debug".to_string(),
            enable_tracing: true,
        },
        security: writemagic_writing::SecurityConfig {
            encrypt_at_rest: false,
            api_rate_limit_per_hour: 500,
        },
    };
    
    let custom_engine = CoreEngine::new_with_config(custom_config).await?;
    println!("✅ Custom configured engine initialized");
    
    // Show configuration details
    let config = custom_engine.config();
    println!("   Database: {}", config.database.database_url);
    println!("   Default AI Model: {}", config.ai.default_model);
    println!("   Max Context Length: {}", config.ai.max_context_length);
    println!("   Content Filtering: {}", config.ai.enable_content_filtering);
    println!("   Log Level: {}", config.logging.level);
    println!("   Encrypt at Rest: {}", config.security.encrypt_at_rest);
    
    custom_engine.shutdown().await;
    
    // Example 4: Database operations with enhanced engine
    println!("\n🗄️  Example 4: Database Operations");
    
    let db_engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .build()
        .await?;
    
    println!("✅ Database engine initialized");
    
    // Create multiple documents
    let doc_repo = db_engine.document_repository();
    let mut document_ids = Vec::new();
    
    for i in 1..=5 {
        let doc = Document::new(
            format!("Document {}", i),
            format!("Content for document {}", i),
            ContentType::Markdown,
            None,
        );
        let saved = doc_repo.save(&doc).await?;
        document_ids.push(saved.id);
        println!("   Created document {}: {}", i, saved.id);
    }
    
    // Count documents
    let count = doc_repo.count().await?;
    println!("✅ Total documents in database: {}", count);
    
    // Get migration status
    if let Some(migrations) = db_engine.get_migration_status().await? {
        println!("📋 Database Migration Status:");
        for migration in migrations {
            println!("   {}: {}", 
                migration.name, 
                if migration.applied { "✅ Applied" } else { "❌ Pending" }
            );
        }
    }
    
    db_engine.shutdown().await;
    
    println!("\n🎉 All examples completed successfully!");
    
    Ok(())
}