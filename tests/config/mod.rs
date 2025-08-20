//! Test configuration and environment setup
//! 
//! Provides configuration management and environment setup for integration tests

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Test environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub database: DatabaseConfig,
    pub ai_providers: AIProvidersConfig,
    pub platforms: PlatformConfig,
    pub timeouts: TimeoutConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub test_db_url: String,
    pub reset_between_tests: bool,
    pub pool_max_connections: u32,
    pub migration_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfig {
    pub mock_enabled: bool,
    pub mock_server_port: u16,
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub response_delay_ms: u64,
    pub failure_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub android: AndroidConfig,
    pub web: WebConfig,
    pub wasm: WasmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidConfig {
    pub enabled: bool,
    pub gradle_timeout_minutes: u64,
    pub device_filter: Option<String>,
    pub emulator_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub browsers: Vec<String>,
    pub headless: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub enabled: bool,
    pub node_version: String,
    pub build_timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub test_timeout_minutes: u64,
    pub setup_timeout_minutes: u64,
    pub teardown_timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub output_file: Option<String>,
    pub json_format: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                test_db_url: "sqlite:///tmp/writemagic-test.db".to_string(),
                reset_between_tests: true,
                pool_max_connections: 10,
                migration_timeout_seconds: 30,
            },
            ai_providers: AIProvidersConfig {
                mock_enabled: true,
                mock_server_port: 8999,
                claude_api_key: std::env::var("CLAUDE_API_KEY").ok(),
                openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
                response_delay_ms: 100,
                failure_rate_percent: 0.0, // No failures by default
            },
            platforms: PlatformConfig {
                android: AndroidConfig {
                    enabled: true,
                    gradle_timeout_minutes: 10,
                    device_filter: None,
                    emulator_required: false,
                },
                web: WebConfig {
                    enabled: true,
                    browsers: vec!["chromium".to_string()],
                    headless: true,
                    viewport_width: 1280,
                    viewport_height: 720,
                },
                wasm: WasmConfig {
                    enabled: true,
                    node_version: "18".to_string(),
                    build_timeout_minutes: 5,
                },
            },
            timeouts: TimeoutConfig {
                test_timeout_minutes: 30,
                setup_timeout_minutes: 5,
                teardown_timeout_minutes: 2,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                output_file: Some("/tmp/writemagic-test.log".to_string()),
                json_format: false,
            },
        }
    }
}

impl TestConfig {
    /// Load configuration from file or environment
    pub fn load() -> Result<Self> {
        // Try to load from config file first
        if let Ok(config_path) = std::env::var("WRITEMAGIC_TEST_CONFIG") {
            return Self::load_from_file(&config_path);
        }

        // Check for standard config locations
        let possible_paths = vec![
            "./test-config.toml",
            "./tests/config.toml",
            "~/.writemagic/test-config.toml",
        ];

        for path in possible_paths {
            if std::path::Path::new(path).exists() {
                return Self::load_from_file(path);
            }
        }

        // Fall back to default configuration with environment variable overrides
        let mut config = Self::default();
        config.apply_environment_overrides();
        
        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: TestConfig = toml::from_str(&content)?;
        config.apply_environment_overrides();
        Ok(config)
    }

    /// Apply environment variable overrides
    pub fn apply_environment_overrides(&mut self) {
        // Database overrides
        if let Ok(db_url) = std::env::var("TEST_DATABASE_URL") {
            self.database.test_db_url = db_url;
        }

        // AI provider overrides
        if let Ok(api_key) = std::env::var("CLAUDE_API_KEY") {
            self.ai_providers.claude_api_key = Some(api_key);
        }
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            self.ai_providers.openai_api_key = Some(api_key);
        }
        if let Ok(mock_enabled) = std::env::var("AI_MOCK_ENABLED") {
            self.ai_providers.mock_enabled = mock_enabled.parse().unwrap_or(true);
        }

        // Platform overrides
        if let Ok(android_enabled) = std::env::var("ANDROID_TESTS_ENABLED") {
            self.platforms.android.enabled = android_enabled.parse().unwrap_or(true);
        }
        if let Ok(web_enabled) = std::env::var("WEB_TESTS_ENABLED") {
            self.platforms.web.enabled = web_enabled.parse().unwrap_or(true);
        }
        if let Ok(wasm_enabled) = std::env::var("WASM_TESTS_ENABLED") {
            self.platforms.wasm.enabled = wasm_enabled.parse().unwrap_or(true);
        }

        // Timeout overrides
        if let Ok(timeout) = std::env::var("TEST_TIMEOUT_MINUTES") {
            if let Ok(timeout_val) = timeout.parse() {
                self.timeouts.test_timeout_minutes = timeout_val;
            }
        }

        // Logging overrides
        if let Ok(level) = std::env::var("TEST_LOG_LEVEL") {
            self.logging.level = level;
        }
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate database URL
        if self.database.test_db_url.is_empty() {
            anyhow::bail!("Database URL cannot be empty");
        }

        // Validate timeouts
        if self.timeouts.test_timeout_minutes == 0 {
            anyhow::bail!("Test timeout must be greater than 0");
        }

        // Validate AI provider configuration
        if !self.ai_providers.mock_enabled {
            if self.ai_providers.claude_api_key.is_none() && self.ai_providers.openai_api_key.is_none() {
                anyhow::bail!("At least one AI provider API key must be configured when mocks are disabled");
            }
        }

        // Validate platform configuration
        if !self.platforms.android.enabled && !self.platforms.web.enabled && !self.platforms.wasm.enabled {
            anyhow::bail!("At least one platform must be enabled");
        }

        Ok(())
    }

    /// Get effective configuration for CI environments
    pub fn for_ci() -> Self {
        let mut config = Self::default();
        
        // CI-specific overrides
        config.platforms.android.emulator_required = true;
        config.platforms.web.headless = true;
        config.ai_providers.mock_enabled = true; // Always use mocks in CI
        config.timeouts.test_timeout_minutes = 45; // Longer timeout for CI
        config.logging.json_format = true; // Structured logs for CI
        
        config.apply_environment_overrides();
        config
    }

    /// Get configuration for local development
    pub fn for_local() -> Self {
        let mut config = Self::default();
        
        // Local development overrides
        config.platforms.android.emulator_required = false;
        config.platforms.web.headless = false; // Show browser in development
        config.ai_providers.mock_enabled = false; // Use real APIs if available
        config.logging.json_format = false; // Human-readable logs
        
        config.apply_environment_overrides();
        config
    }
}

/// Test environment manager
pub struct TestEnvironment {
    config: TestConfig,
    temp_dirs: Vec<PathBuf>,
}

impl TestEnvironment {
    /// Create a new test environment
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            temp_dirs: Vec::new(),
        }
    }

    /// Setup the test environment
    pub async fn setup(&mut self) -> Result<()> {
        self.config.validate()?;

        // Create temporary directories
        self.create_temp_directories().await?;

        // Setup logging
        self.setup_logging()?;

        // Setup database
        self.setup_database().await?;

        Ok(())
    }

    /// Teardown the test environment
    pub async fn teardown(&mut self) -> Result<()> {
        // Cleanup temporary directories
        for temp_dir in &self.temp_dirs {
            if temp_dir.exists() {
                tokio::fs::remove_dir_all(temp_dir).await?;
            }
        }
        self.temp_dirs.clear();

        Ok(())
    }

    /// Create necessary temporary directories
    async fn create_temp_directories(&mut self) -> Result<()> {
        let base_temp = PathBuf::from("/tmp/writemagic-tests");
        tokio::fs::create_dir_all(&base_temp).await?;
        self.temp_dirs.push(base_temp.clone());

        // Create subdirectories
        let subdirs = vec!["data", "logs", "cache", "artifacts"];
        for subdir in subdirs {
            let path = base_temp.join(subdir);
            tokio::fs::create_dir_all(&path).await?;
        }

        Ok(())
    }

    /// Setup logging configuration
    fn setup_logging(&self) -> Result<()> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .or_else(|_| tracing_subscriber::EnvFilter::try_new(&self.config.logging.level))
            .unwrap();

        let subscriber = tracing_subscriber::registry()
            .with(env_filter);

        if let Some(log_file) = &self.config.logging.output_file {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(log_file)?;

            if self.config.logging.json_format {
                let layer = tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(file);
                subscriber.with(layer).init();
            } else {
                let layer = tracing_subscriber::fmt::layer()
                    .with_writer(file);
                subscriber.with(layer).init();
            }
        } else {
            if self.config.logging.json_format {
                let layer = tracing_subscriber::fmt::layer()
                    .json();
                subscriber.with(layer).init();
            } else {
                let layer = tracing_subscriber::fmt::layer();
                subscriber.with(layer).init();
            }
        }

        Ok(())
    }

    /// Setup test database
    async fn setup_database(&self) -> Result<()> {
        let pool = sqlx::SqlitePool::connect(&self.config.database.test_db_url).await?;
        
        // Run database migrations or setup
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS test_documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                project_id TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS test_projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                settings TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await?;

        pool.close().await;
        Ok(())
    }

    /// Get configuration reference
    pub fn config(&self) -> &TestConfig {
        &self.config
    }

    /// Get temporary directory path
    pub fn temp_dir(&self) -> Option<&PathBuf> {
        self.temp_dirs.first()
    }
}