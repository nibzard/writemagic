//! Integration test for mobile FFI persistence
//! 
//! Tests that documents created through mobile FFI calls survive 
//! across engine restarts and properly persist to SQLite.

use std::sync::{Arc, Mutex, Once};
use writemagic_shared::{EntityId, ContentType};
use writemagic_writing::{CoreEngine, ApplicationConfigBuilder};

static INIT: Once = Once::new();
static mut CORE_ENGINE: Option<Arc<Mutex<CoreEngine>>> = None;

/// Initialize core engine with SQLite (similar to mobile FFI)
async fn get_or_create_core_engine() -> Result<Arc<Mutex<CoreEngine>>, String> {
    unsafe {
        if CORE_ENGINE.is_none() {
            let engine = ApplicationConfigBuilder::new()
                .with_sqlite()  // Use persistent SQLite
                .with_claude_key("".to_string())
                .with_openai_key("".to_string())
                .with_log_level("info".to_string())
                .with_content_filtering(true)
                .build()
                .await
                .map_err(|e| format!("Failed to create CoreEngine: {}", e))?;
                
            CORE_ENGINE = Some(Arc::new(Mutex::new(engine)));
        }
        Ok(CORE_ENGINE.as_ref().unwrap().clone())
    }
}

/// Reset core engine (simulates app restart)
fn reset_core_engine() {
    unsafe {
        CORE_ENGINE = None;
    }
}

#[tokio::test]
async fn test_mobile_ffi_document_persistence() {
    INIT.call_once(|| {
        env_logger::init();
    });

    println!("Testing mobile FFI document persistence...");

    // Step 1: Initialize core engine and create a document
    let document_id = {
        let engine = get_or_create_core_engine().await.expect("Failed to create core engine");
        let engine_guard = engine.lock().unwrap();
        
        // Create document using domain service (like mobile FFI does)
        let document_title = writemagic_writing::value_objects::DocumentTitle::new("Test Mobile Document")
            .expect("Failed to create document title");
        let document_content = writemagic_writing::value_objects::DocumentContent::new("This is a test document created through mobile FFI simulation.")
            .expect("Failed to create document content");
        
        let result = engine_guard.runtime().block_on(async {
            engine_guard.document_service().create_document(
                document_title,
                document_content,
                ContentType::Markdown,
                None, // created_by
            ).await
        }).expect("Failed to create document");
        
        let document = result.document();
        let doc_id = document.id.clone();
        
        println!("Created document with ID: {}", doc_id);
        
        // Verify document was created
        assert_eq!(document.title, "Test Mobile Document");
        assert_eq!(document.content, "This is a test document created through mobile FFI simulation.");
        assert_eq!(document.content_type, ContentType::Markdown);
        
        doc_id
    };
    
    // Step 2: Reset core engine (simulate app restart)
    println!("Simulating app restart...");
    reset_core_engine();
    
    // Step 3: Initialize core engine again and verify document persists
    {
        let engine = get_or_create_core_engine().await.expect("Failed to recreate core engine");
        let engine_guard = engine.lock().unwrap();
        
        // Retrieve document using repository (like mobile FFI does)
        let result = engine_guard.runtime().block_on(async {
            engine_guard.document_repository().find_by_id(&document_id).await
        }).expect("Failed to retrieve document");
        
        let document = result.expect("Document should exist after restart");
        
        println!("Retrieved document after restart: {} - {}", document.id, document.title);
        
        // Verify document data survived restart
        assert_eq!(document.id, document_id);
        assert_eq!(document.title, "Test Mobile Document");
        assert_eq!(document.content, "This is a test document created through mobile FFI simulation.");
        assert_eq!(document.content_type, ContentType::Markdown);
        assert!(!document.is_deleted);
    }
    
    // Step 4: Test document update through mobile FFI pattern
    {
        let engine = get_or_create_core_engine().await.expect("Failed to get core engine");
        let engine_guard = engine.lock().unwrap();
        
        let new_content = writemagic_writing::value_objects::DocumentContent::new("Updated content through mobile FFI.")
            .expect("Failed to create updated content");
        
        // Update document using domain service
        engine_guard.runtime().block_on(async {
            engine_guard.document_service().update_document_content(
                document_id.clone(),
                new_content,
                None, // text selection
                None, // updated_by
            ).await
        }).expect("Failed to update document");
        
        println!("Updated document content");
    }
    
    // Step 5: Reset again and verify update persisted
    println!("Simulating second app restart...");
    reset_core_engine();
    
    {
        let engine = get_or_create_core_engine().await.expect("Failed to recreate core engine again");
        let engine_guard = engine.lock().unwrap();
        
        let result = engine_guard.runtime().block_on(async {
            engine_guard.document_repository().find_by_id(&document_id).await
        }).expect("Failed to retrieve updated document");
        
        let document = result.expect("Updated document should exist after restart");
        
        println!("Retrieved updated document: {}", document.content);
        
        // Verify update persisted
        assert_eq!(document.content, "Updated content through mobile FFI.");
        assert!(document.version > 1, "Document version should have incremented");
    }
    
    println!("✓ Mobile FFI persistence test passed!");
}

#[tokio::test]
async fn test_mobile_ffi_document_listing() {
    println!("Testing mobile FFI document listing...");
    
    let engine = get_or_create_core_engine().await.expect("Failed to create core engine");
    let engine_guard = engine.lock().unwrap();
    
    // Create multiple documents
    let mut document_ids = Vec::new();
    
    for i in 1..=3 {
        let document_title = writemagic_writing::value_objects::DocumentTitle::new(&format!("Mobile Test Doc {}", i))
            .expect("Failed to create document title");
        let document_content = writemagic_writing::value_objects::DocumentContent::new(&format!("Content for document {}", i))
            .expect("Failed to create document content");
        
        let result = engine_guard.runtime().block_on(async {
            engine_guard.document_service().create_document(
                document_title,
                document_content,
                ContentType::PlainText,
                None,
            ).await
        }).expect("Failed to create document");
        
        document_ids.push(result.document().id.clone());
        println!("Created document {} with ID: {}", i, document_ids.last().unwrap());
    }
    
    // List documents with pagination (like mobile FFI does)
    let pagination = writemagic_shared::Pagination::new(0, 10).expect("Failed to create pagination");
    
    let documents = engine_guard.runtime().block_on(async {
        engine_guard.document_repository().find_all(pagination).await
    }).expect("Failed to list documents");
    
    println!("Listed {} documents", documents.len());
    
    // Verify our test documents are in the list
    assert!(documents.len() >= 3, "Should have at least 3 documents");
    
    let our_docs: Vec<_> = documents.iter()
        .filter(|doc| document_ids.contains(&doc.id))
        .collect();
    
    assert_eq!(our_docs.len(), 3, "All our test documents should be listed");
    
    println!("✓ Mobile FFI document listing test passed!");
}

#[tokio::test] 
async fn test_mobile_ffi_ai_integration() {
    println!("Testing mobile FFI AI integration...");
    
    let engine = get_or_create_core_engine().await.expect("Failed to create core engine");
    let engine_guard = engine.lock().unwrap();
    
    // Test AI completion (should gracefully handle missing API keys)
    let result = engine_guard.runtime().block_on(async {
        engine_guard.complete_text("Write a short greeting.".to_string(), None).await
    });
    
    match result {
        Ok(completion) => {
            println!("AI completion successful: {}", completion);
            assert!(!completion.is_empty(), "Completion should not be empty");
        }
        Err(e) => {
            println!("AI completion failed (expected without API keys): {}", e);
            // This is expected without proper API keys
            assert!(e.to_string().contains("No AI providers") || 
                    e.to_string().contains("API key") ||
                    e.to_string().contains("authentication"),
                    "Error should indicate missing API keys: {}", e);
        }
    }
    
    println!("✓ Mobile FFI AI integration test passed!");
}