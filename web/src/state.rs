use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use writemagic_writing::core_engine::CoreEngine;
use migration;
use crate::config::Config;
use crate::error::Result;
use crate::middleware::RateLimitState;
use crate::utils::crypto::JwtKeys;
use crate::websocket::ConnectionManager;

/// Application state that holds all shared resources
/// This follows the single, cloneable state pattern recommended by the best practices guide
#[derive(Clone)]
pub struct AppState {
    /// Core engine from the writing domain
    pub core_engine: Arc<CoreEngine>,
    /// SeaORM database connection
    pub db: DatabaseConnection,
    /// Application configuration
    pub config: Arc<Config>,
    /// HTTP client for external requests
    pub http_client: reqwest::Client,
    /// In-memory cache for frequently accessed data
    pub cache: Arc<DashMap<String, CachedValue>>,
    /// JWT keys for authentication
    pub jwt_keys: Arc<JwtKeys>,
    /// Rate limiting state
    pub rate_limiter: RateLimitState,
    /// WebSocket connection manager
    pub connection_manager: ConnectionManager,
}

/// Cached value with expiration
#[derive(Debug, Clone)]
pub struct CachedValue {
    pub data: serde_json::Value,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}


impl AppState {
    /// Create a new application state instance
    pub async fn new(config: Config) -> Result<Self> {
        tracing::info!("Initializing application state");
        
        // Initialize SeaORM database connection
        let db = Database::connect(&config.database.url)
            .await
            .map_err(|e| crate::error::AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to connect to database: {}", e))))?;
        
        // Run migrations
        migration::Migrator::up(&db, None)
            .await
            .map_err(|e| crate::error::AppError::Database(writemagic_shared::WritemagicError::database(format!("Failed to run migrations: {}", e))))?;
        
        // Initialize core engine with database connection
        let core_engine = Arc::new(
            CoreEngine::initialize()
                .await
                .map_err(|e| crate::error::AppError::Internal(e.into()))?
        );
        
        // Create HTTP client with connection pooling
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(32)
            .timeout(config.server.request_timeout())
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("writemagic-web/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| crate::error::AppError::Internal(e.into()))?;
        
        // Initialize JWT keys
        let (jwt_keys, _) = JwtKeys::generate();
        let jwt_keys = Arc::new(jwt_keys);
        
        // Create cache with reasonable capacity
        let cache = Arc::new(DashMap::with_capacity(10_000));
        
        // Initialize rate limiter (100 requests per minute)
        let rate_limiter = RateLimitState::new(100, 60);
        
        // Initialize WebSocket connection manager
        let connection_manager = ConnectionManager::new();
        
        tracing::info!("Application state initialized successfully");
        
        Ok(Self {
            core_engine,
            db,
            config: Arc::new(config),
            http_client,
            cache,
            jwt_keys,
            rate_limiter,
            connection_manager,
        })
    }
    
    /// Get a cached value if it exists and hasn't expired
    pub fn get_cached<T>(&self, key: &str) -> Option<T> 
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.cache.get(key).and_then(|entry| {
            if entry.expires_at > chrono::Utc::now() {
                serde_json::from_value(entry.data.clone()).ok()
            } else {
                // Remove expired entry
                self.cache.remove(key);
                None
            }
        })
    }
    
    /// Set a cached value with expiration
    pub fn set_cached<T>(&self, key: String, value: T, ttl_secs: i64)
    where
        T: serde::Serialize,
    {
        if let Ok(data) = serde_json::to_value(value) {
            let cached_value = CachedValue {
                data,
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(ttl_secs),
            };
            self.cache.insert(key, cached_value);
        }
    }
    
    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        let now = chrono::Utc::now();
        self.cache.retain(|_k, v| v.expires_at > now);
    }
    
    /// Graceful shutdown cleanup
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down application state");
        
        // Clear cache
        self.cache.clear();
        
        // Any other cleanup can be added here
        tracing::info!("Application state shutdown complete");
    }
}

