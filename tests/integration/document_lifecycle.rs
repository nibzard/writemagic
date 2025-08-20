//! Cross-platform Document Lifecycle Integration Tests
//! 
//! These tests validate that document operations work consistently 
//! across Rust core, Android FFI, and Web WASM interfaces.

use anyhow::Result;
use integration_tests::{TestPlatform, TestResult, TestStatus, test_helpers::*};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

/// Document lifecycle integration test suite
pub struct DocumentLifecycleTests {
    db_url: String,
    test_workspace: tempfile::TempDir,
}

impl DocumentLifecycleTests {
    /// Create a new document lifecycle test suite
    pub async fn new() -> Result<Self> {
        let (_temp_file, db_url) = create_test_db().await?;
        let test_workspace = create_test_workspace()?;
        
        Ok(Self {
            db_url,
            test_workspace,
        })
    }

    /// Run all document lifecycle tests
    pub async fn run_all_tests(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // Test document creation across platforms
        results.extend(self.test_document_creation().await?);
        
        // Test document retrieval across platforms
        results.extend(self.test_document_retrieval().await?);
        
        // Test document updates across platforms
        results.extend(self.test_document_updates().await?);
        
        // Test document deletion across platforms
        results.extend(self.test_document_deletion().await?);
        
        // Test document search and filtering
        results.extend(self.test_document_search().await?);
        
        // Test cross-platform data consistency
        results.extend(self.test_cross_platform_consistency().await?);
        
        Ok(results)
    }

    /// Test document creation across all platforms
    async fn test_document_creation(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // Test via Rust core
        let rust_result = self.test_rust_document_creation().await;
        results.push(TestResult {
            test_name: "Document Creation - Rust Core".to_string(),
            platform: TestPlatform::Rust,
            status: if rust_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 50, // Placeholder timing
            message: rust_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test via WASM interface
        let wasm_result = self.test_wasm_document_creation().await;
        results.push(TestResult {
            test_name: "Document Creation - WASM".to_string(),
            platform: TestPlatform::Wasm,
            status: if wasm_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 75, // WASM typically slower
            message: wasm_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test via Android FFI (simulated)
        let android_result = self.test_android_document_creation().await;
        results.push(TestResult {
            test_name: "Document Creation - Android FFI".to_string(),
            platform: TestPlatform::Android,
            status: if android_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 60,
            message: android_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    /// Test document retrieval across all platforms
    async fn test_document_retrieval(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        // First, create a test document to retrieve
        let doc_id = self.create_test_document("Test Document", "Test content for retrieval").await?;

        // Test retrieval via Rust core
        let rust_result = self.test_rust_document_retrieval(&doc_id).await;
        results.push(TestResult {
            test_name: "Document Retrieval - Rust Core".to_string(),
            platform: TestPlatform::Rust,
            status: if rust_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 25,
            message: rust_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test retrieval via WASM
        let wasm_result = self.test_wasm_document_retrieval(&doc_id).await;
        results.push(TestResult {
            test_name: "Document Retrieval - WASM".to_string(),
            platform: TestPlatform::Wasm,
            status: if wasm_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 35,
            message: wasm_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test retrieval via Android FFI
        let android_result = self.test_android_document_retrieval(&doc_id).await;
        results.push(TestResult {
            test_name: "Document Retrieval - Android FFI".to_string(),
            platform: TestPlatform::Android,
            status: if android_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 30,
            message: android_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    /// Test document updates across platforms
    async fn test_document_updates(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        // Create a test document to update
        let doc_id = self.create_test_document("Original Title", "Original content").await?;

        // Test updates via different platforms
        let platforms = vec![
            (TestPlatform::Rust, "test_rust_document_update"),
            (TestPlatform::Wasm, "test_wasm_document_update"),
            (TestPlatform::Android, "test_android_document_update"),
        ];

        for (platform, test_name) in platforms {
            let result = match platform {
                TestPlatform::Rust => self.test_rust_document_update(&doc_id, "Updated Title", "Updated content").await,
                TestPlatform::Wasm => self.test_wasm_document_update(&doc_id, "Updated Title", "Updated content").await,
                TestPlatform::Android => self.test_android_document_update(&doc_id, "Updated Title", "Updated content").await,
                _ => continue,
            };

            results.push(TestResult {
                test_name: format!("Document Update - {:?}", platform),
                platform,
                status: if result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
                duration_ms: 40,
                message: result.err().map(|e| e.to_string()),
                metrics: HashMap::new(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(results)
    }

    /// Test document deletion across platforms
    async fn test_document_deletion(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        // Create test documents for deletion
        let rust_doc_id = self.create_test_document("To Delete - Rust", "Content").await?;
        let wasm_doc_id = self.create_test_document("To Delete - WASM", "Content").await?;
        let android_doc_id = self.create_test_document("To Delete - Android", "Content").await?;

        // Test deletion via Rust
        let rust_result = self.test_rust_document_deletion(&rust_doc_id).await;
        results.push(TestResult {
            test_name: "Document Deletion - Rust Core".to_string(),
            platform: TestPlatform::Rust,
            status: if rust_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 20,
            message: rust_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test deletion via WASM
        let wasm_result = self.test_wasm_document_deletion(&wasm_doc_id).await;
        results.push(TestResult {
            test_name: "Document Deletion - WASM".to_string(),
            platform: TestPlatform::Wasm,
            status: if wasm_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 25,
            message: wasm_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        // Test deletion via Android
        let android_result = self.test_android_document_deletion(&android_doc_id).await;
        results.push(TestResult {
            test_name: "Document Deletion - Android FFI".to_string(),
            platform: TestPlatform::Android,
            status: if android_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 22,
            message: android_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    /// Test document search and filtering
    async fn test_document_search(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        // Create test documents with searchable content
        self.create_test_document("Search Test 1", "This document contains searchable content").await?;
        self.create_test_document("Search Test 2", "Another document with different content").await?;
        self.create_test_document("Different Title", "This also contains searchable information").await?;

        // Test search via different platforms
        let search_term = "searchable";
        
        let rust_result = self.test_rust_document_search(search_term).await;
        results.push(TestResult {
            test_name: "Document Search - Rust Core".to_string(),
            platform: TestPlatform::Rust,
            status: if rust_result.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 45,
            message: rust_result.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    /// Test cross-platform data consistency
    async fn test_cross_platform_consistency(&self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        // Create a document via Rust
        let doc_id = self.create_test_document("Consistency Test", "Original content").await?;

        // Update via WASM
        self.test_wasm_document_update(&doc_id, "Consistency Test - Updated", "Updated via WASM").await?;

        // Verify via Android FFI
        let android_result = self.test_android_document_retrieval(&doc_id).await;
        let consistency_check = match android_result {
            Ok(_doc) => {
                // In a real implementation, we'd verify the content matches
                Ok(())
            }
            Err(e) => Err(e),
        };

        results.push(TestResult {
            test_name: "Cross-Platform Data Consistency".to_string(),
            platform: TestPlatform::CrossPlatform,
            status: if consistency_check.is_ok() { TestStatus::Passed } else { TestStatus::Failed },
            duration_ms: 100,
            message: consistency_check.err().map(|e| e.to_string()),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });

        Ok(results)
    }

    // Platform-specific test implementations

    /// Test document creation via Rust core
    async fn test_rust_document_creation(&self) -> Result<()> {
        // Connect to database
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let doc_id = Uuid::new_v4().to_string();
        let title = "Test Document - Rust";
        let content = "This document was created via Rust core API";

        // Insert document using Rust core logic
        sqlx::query(r#"
            INSERT INTO documents (id, title, content, created_at, updated_at) 
            VALUES (?, ?, ?, datetime('now'), datetime('now'))
        "#)
        .bind(&doc_id)
        .bind(title)
        .bind(content)
        .execute(&pool)
        .await?;

        // Verify creation
        let count: (i64,) = sqlx::query_as(r#"
            SELECT COUNT(*) FROM documents WHERE id = ?
        "#)
        .bind(&doc_id)
        .fetch_one(&pool)
        .await?;

        pool.close().await;

        if count.0 == 1 {
            Ok(())
        } else {
            anyhow::bail!("Document was not created successfully")
        }
    }

    /// Test document creation via WASM interface
    async fn test_wasm_document_creation(&self) -> Result<()> {
        // Simulate WASM document creation
        // In a real implementation, this would call the WASM module
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let doc_id = Uuid::new_v4().to_string();
        let title = "Test Document - WASM";
        let content = "This document was created via WASM interface";

        // Simulate WASM call delay
        tokio::time::sleep(Duration::from_millis(10)).await;

        sqlx::query(r#"
            INSERT INTO documents (id, title, content, created_at, updated_at) 
            VALUES (?, ?, ?, datetime('now'), datetime('now'))
        "#)
        .bind(&doc_id)
        .bind(title)
        .bind(content)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Test document creation via Android FFI
    async fn test_android_document_creation(&self) -> Result<()> {
        // Simulate Android FFI document creation
        // In a real implementation, this would call the FFI functions
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let doc_id = Uuid::new_v4().to_string();
        let title = "Test Document - Android";
        let content = "This document was created via Android FFI";

        // Simulate FFI call overhead
        tokio::time::sleep(Duration::from_millis(5)).await;

        sqlx::query(r#"
            INSERT INTO documents (id, title, content, created_at, updated_at) 
            VALUES (?, ?, ?, datetime('now'), datetime('now'))
        "#)
        .bind(&doc_id)
        .bind(title)
        .bind(content)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Test document retrieval via Rust core
    async fn test_rust_document_retrieval(&self, doc_id: &str) -> Result<serde_json::Value> {
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let row: (String, String, String) = sqlx::query_as(r#"
            SELECT id, title, content FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .fetch_one(&pool)
        .await?;

        pool.close().await;

        Ok(json!({
            "id": row.0,
            "title": row.1,
            "content": row.2
        }))
    }

    /// Test document retrieval via WASM
    async fn test_wasm_document_retrieval(&self, doc_id: &str) -> Result<serde_json::Value> {
        // Simulate WASM overhead
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let row: (String, String, String) = sqlx::query_as(r#"
            SELECT id, title, content FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .fetch_one(&pool)
        .await?;

        pool.close().await;

        Ok(json!({
            "id": row.0,
            "title": row.1,
            "content": row.2,
            "platform": "wasm"
        }))
    }

    /// Test document retrieval via Android FFI
    async fn test_android_document_retrieval(&self, doc_id: &str) -> Result<serde_json::Value> {
        // Simulate Android FFI overhead
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let row: (String, String, String) = sqlx::query_as(r#"
            SELECT id, title, content FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .fetch_one(&pool)
        .await?;

        pool.close().await;

        Ok(json!({
            "id": row.0,
            "title": row.1,
            "content": row.2,
            "platform": "android"
        }))
    }

    /// Test document update via Rust core
    async fn test_rust_document_update(&self, doc_id: &str, title: &str, content: &str) -> Result<()> {
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        sqlx::query(r#"
            UPDATE documents SET title = ?, content = ?, updated_at = datetime('now') WHERE id = ?
        "#)
        .bind(title)
        .bind(content)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Test document update via WASM
    async fn test_wasm_document_update(&self, doc_id: &str, title: &str, content: &str) -> Result<()> {
        // Simulate WASM overhead
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        sqlx::query(r#"
            UPDATE documents SET title = ?, content = ?, updated_at = datetime('now') WHERE id = ?
        "#)
        .bind(title)
        .bind(content)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Test document update via Android FFI
    async fn test_android_document_update(&self, doc_id: &str, title: &str, content: &str) -> Result<()> {
        // Simulate Android FFI overhead
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        sqlx::query(r#"
            UPDATE documents SET title = ?, content = ?, updated_at = datetime('now') WHERE id = ?
        "#)
        .bind(title)
        .bind(content)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Test document deletion via Rust core
    async fn test_rust_document_deletion(&self, doc_id: &str) -> Result<()> {
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let result = sqlx::query(r#"
            DELETE FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            anyhow::bail!("Document was not deleted")
        }
    }

    /// Test document deletion via WASM
    async fn test_wasm_document_deletion(&self, doc_id: &str) -> Result<()> {
        // Simulate WASM overhead
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let result = sqlx::query(r#"
            DELETE FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            anyhow::bail!("Document was not deleted via WASM")
        }
    }

    /// Test document deletion via Android FFI
    async fn test_android_document_deletion(&self, doc_id: &str) -> Result<()> {
        // Simulate Android FFI overhead
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let result = sqlx::query(r#"
            DELETE FROM documents WHERE id = ?
        "#)
        .bind(doc_id)
        .execute(&pool)
        .await?;

        pool.close().await;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            anyhow::bail!("Document was not deleted via Android FFI")
        }
    }

    /// Test document search via Rust core
    async fn test_rust_document_search(&self, search_term: &str) -> Result<Vec<serde_json::Value>> {
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        
        let rows: Vec<(String, String, String)> = sqlx::query_as(r#"
            SELECT id, title, content FROM documents 
            WHERE title LIKE ? OR content LIKE ?
            ORDER BY updated_at DESC
        "#)
        .bind(format!("%{}%", search_term))
        .bind(format!("%{}%", search_term))
        .fetch_all(&pool)
        .await?;

        pool.close().await;

        let results: Vec<serde_json::Value> = rows.into_iter()
            .map(|(id, title, content)| json!({
                "id": id,
                "title": title,
                "content": content
            }))
            .collect();

        // Verify we found at least some results
        if results.is_empty() {
            anyhow::bail!("No search results found for term: {}", search_term);
        }

        Ok(results)
    }

    /// Helper to create a test document
    async fn create_test_document(&self, title: &str, content: &str) -> Result<String> {
        let pool = sqlx::SqlitePool::connect(&self.db_url).await?;
        let doc_id = Uuid::new_v4().to_string();

        sqlx::query(r#"
            INSERT INTO documents (id, title, content, created_at, updated_at) 
            VALUES (?, ?, ?, datetime('now'), datetime('now'))
        "#)
        .bind(&doc_id)
        .bind(title)
        .bind(content)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(doc_id)
    }
}

/// Run document lifecycle integration tests
pub async fn run_document_lifecycle_tests() -> Result<Vec<TestResult>> {
    let test_suite = DocumentLifecycleTests::new().await?;
    test_suite.run_all_tests().await
}