//! Test script to verify SQLite persistence functionality

use std::path::Path;

// Since we have compilation issues, let's create a simple test that verifies
// the configuration defaults without needing to run the full application

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🧪 Testing WriteMagic SQLite Persistence...\n");
    
    // Remove test database if it exists
    let db_path = "test_writemagic.db";
    if Path::new(db_path).exists() {
        std::fs::remove_file(db_path)?;
        println!("🗑️  Removed existing test database");
    }
    if Path::new(&format!("{}-wal", db_path)).exists() {
        std::fs::remove_file(format!("{}-wal", db_path))?;
    }
    if Path::new(&format!("{}-shm", db_path)).exists() {
        std::fs::remove_file(format!("{}-shm", db_path))?;
    }
    
    // Test 1: Create engine with custom database path
    println!("📝 Test 1: Creating engine with custom SQLite database...");
    let config = writemagic_shared::DatabaseConfig {
        database_url: format!("sqlite://{}", db_path),
        max_connections: 10,
        min_connections: 1,
        enable_wal: true,
        enable_foreign_keys: true,
    };
    
    let app_config = writemagic_writing::ApplicationConfig {
        database: config,
        ai: writemagic_writing::AIConfig::default(),
        logging: writemagic_writing::LoggingConfig::default(),
        security: writemagic_writing::SecurityConfig::default(),
    };
    
    let engine1 = CoreEngine::new_with_config(app_config).await?;
    
    // Verify it's not using in-memory storage
    assert!(!engine1.is_in_memory(), "❌ Engine should not be using in-memory storage");
    assert!(engine1.database_manager().is_some(), "❌ Engine should have a database manager");
    println!("✅ Engine created with persistent storage");
    
    // Test 2: Create and save a document
    println!("\n📄 Test 2: Creating and saving a document...");
    let doc = Document::new(
        "Persistence Test Document".to_string(),
        "This document tests SQLite persistence across engine restarts.".to_string(),
        ContentType::Markdown,
        Some(EntityId::new()),
    );
    let doc_id = doc.id;
    
    let repo1 = engine1.document_repository();
    let saved_doc = repo1.save(&doc).await?;
    println!("✅ Document saved with ID: {}", saved_doc.id);
    
    // Verify document exists
    let found_doc = repo1.find_by_id(&doc_id).await?;
    assert!(found_doc.is_some(), "❌ Document should exist after save");
    println!("✅ Document found after save");
    
    // Check the database file was created
    assert!(Path::new(db_path).exists(), "❌ Database file should exist");
    println!("✅ Database file exists at: {}", db_path);
    
    // Test 3: Shutdown engine and verify file persists
    println!("\n🔄 Test 3: Shutting down engine...");
    engine1.shutdown().await;
    println!("✅ Engine shutdown completed");
    
    // Verify database file still exists
    assert!(Path::new(db_path).exists(), "❌ Database file should persist after shutdown");
    println!("✅ Database file persists after shutdown");
    
    // Test 4: Create new engine and verify data persistence
    println!("\n🔄 Test 4: Creating new engine instance...");
    let config2 = writemagic_shared::DatabaseConfig {
        database_url: format!("sqlite://{}", db_path),
        max_connections: 10,
        min_connections: 1,
        enable_wal: true,
        enable_foreign_keys: true,
    };
    
    let app_config2 = writemagic_writing::ApplicationConfig {
        database: config2,
        ai: writemagic_writing::AIConfig::default(),
        logging: writemagic_writing::LoggingConfig::default(),
        security: writemagic_writing::SecurityConfig::default(),
    };
    
    let engine2 = CoreEngine::new_with_config(app_config2).await?;
    println!("✅ Second engine instance created");
    
    // Test 5: Verify document persists across restart
    println!("\n📖 Test 5: Verifying document persistence...");
    let repo2 = engine2.document_repository();
    let persisted_doc = repo2.find_by_id(&doc_id).await?;
    
    match persisted_doc {
        Some(doc) => {
            println!("✅ Document found after engine restart!");
            println!("   Title: {}", doc.title);
            println!("   Content: {}", doc.content);
            println!("   ID: {}", doc.id);
            assert_eq!(doc.title, "Persistence Test Document");
            assert_eq!(doc.content, "This document tests SQLite persistence across engine restarts.");
        }
        None => {
            panic!("❌ Document not found after engine restart - persistence failed!");
        }
    }
    
    // Test 6: Test new_default() method
    println!("\n🏭 Test 6: Testing CoreEngine::new_default()...");
    engine2.shutdown().await;
    
    // Remove test database
    if Path::new(db_path).exists() {
        std::fs::remove_file(db_path)?;
    }
    if Path::new(&format!("{}-wal", db_path)).exists() {
        std::fs::remove_file(format!("{}-wal", db_path))?;
    }
    if Path::new(&format!("{}-shm", db_path)).exists() {
        std::fs::remove_file(format!("{}-shm", db_path))?;
    }
    
    let default_engine = CoreEngine::new_default().await?;
    assert!(!default_engine.is_in_memory(), "❌ new_default() should create persistent storage");
    assert!(default_engine.database_manager().is_some(), "❌ new_default() should have database manager");
    println!("✅ CoreEngine::new_default() creates persistent storage");
    
    // Check that writemagic.db is created
    assert!(Path::new("writemagic.db").exists(), "❌ Default database file should be created");
    println!("✅ Default database file 'writemagic.db' created");
    
    // Test persistence with default database
    let doc2 = Document::new(
        "Default Engine Test".to_string(),
        "Testing default engine persistence.".to_string(),
        ContentType::Markdown,
        Some(EntityId::new()),
    );
    let doc2_id = doc2.id;
    
    let default_repo = default_engine.document_repository();
    default_repo.save(&doc2).await?;
    println!("✅ Document saved using default engine");
    
    default_engine.shutdown().await;
    
    // Create another default engine and verify persistence
    let default_engine2 = CoreEngine::new_default().await?;
    let default_repo2 = default_engine2.document_repository();
    let persisted_doc2 = default_repo2.find_by_id(&doc2_id).await?;
    
    match persisted_doc2 {
        Some(doc) => {
            println!("✅ Document persisted with default engine!");
            println!("   Title: {}", doc.title);
        }
        None => {
            panic!("❌ Document not found with default engine - persistence failed!");
        }
    }
    
    default_engine2.shutdown().await;
    
    println!("\n🎉 ALL TESTS PASSED!");
    println!("✅ SQLite persistence is working correctly");
    println!("✅ CoreEngine::new_default() creates persistent storage");
    println!("✅ Data persists across engine restarts");
    println!("✅ Database files are created correctly");
    
    // Clean up
    if Path::new("writemagic.db").exists() {
        std::fs::remove_file("writemagic.db")?;
        println!("🗑️  Cleaned up default database file");
    }
    if Path::new("writemagic.db-wal").exists() {
        std::fs::remove_file("writemagic.db-wal")?;
    }
    if Path::new("writemagic.db-shm").exists() {
        std::fs::remove_file("writemagic.db-shm")?;
    }
    
    Ok(())
}