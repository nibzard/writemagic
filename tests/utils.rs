//! Test Utilities and Helpers
//! 
//! Common utilities for testing across all WriteMagic test suites

use anyhow::Result;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

/// Test platform enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TestPlatform {
    Rust,
    Android,
    Web,
    Wasm,
    CrossPlatform,
}

/// Test execution status
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Pending,
}

/// Individual test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub platform: TestPlatform,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub metrics: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Collection of test results
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestSuiteResults {
    pub suite_name: String,
    pub results: Vec<TestResult>,
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total_duration_ms: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl TestSuiteResults {
    pub fn new(suite_name: String) -> Self {
        Self {
            suite_name,
            results: Vec::new(),
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            total_duration_ms: 0,
            start_time: chrono::Utc::now(),
            end_time: None,
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Skipped => self.skipped += 1,
            TestStatus::Pending => {}
        }
        self.total_tests += 1;
        self.total_duration_ms += result.duration_ms;
        self.results.push(result);
    }

    pub fn complete(&mut self) {
        self.end_time = Some(chrono::Utc::now());
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total_tests as f64) * 100.0
        }
    }
}

/// Test helper functions
pub mod test_helpers {
    use super::*;
    use sqlx::SqlitePool;

    /// Create a temporary test database
    pub async fn create_test_db() -> Result<(tempfile::NamedTempFile, String)> {
        let temp_file = tempfile::NamedTempFile::new()?;
        let db_url = format!("sqlite:{}", temp_file.path().to_string_lossy());
        
        // Initialize the database
        let pool = SqlitePool::connect(&db_url).await?;
        
        // Create test schema
        sqlx::query(r#"
            CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text/plain',
                project_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                settings TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await?;

        pool.close().await;

        Ok((temp_file, db_url))
    }

    /// Create a temporary test workspace
    pub fn create_test_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;
        
        // Create common test directories
        std::fs::create_dir_all(temp_dir.path().join("documents"))?;
        std::fs::create_dir_all(temp_dir.path().join("projects"))?;
        std::fs::create_dir_all(temp_dir.path().join("temp"))?;
        
        Ok(temp_dir)
    }

    /// Create a unique test identifier
    pub fn create_test_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Test timeout helper
    pub async fn with_timeout<F, T>(future: F, timeout_secs: u64) -> anyhow::Result<T>
    where
        F: std::future::Future<Output = T>,
    {
        use std::time::Duration;
        match tokio::time::timeout(Duration::from_secs(timeout_secs), future).await {
            Ok(result) => Ok(result),
            Err(_) => anyhow::bail!("Operation timed out after {} seconds", timeout_secs),
        }
    }
}