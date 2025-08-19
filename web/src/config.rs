use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub body_limit_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_duration_secs: i64,
    pub refresh_token_duration_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub max_age_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Set default configuration
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(port) = std::env::var("PORT") {
            config.server.port = port.parse()?;
        }
        
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database.url = database_url;
        }
        
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        
        Ok(config)
    }
    
    pub fn test_default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Random port for testing
                request_timeout_secs: 30,
                body_limit_bytes: 10 * 1024 * 1024, // 10MB
            },
            database: DatabaseConfig {
                url: ":memory:".to_string(), // SQLite in-memory for tests
                max_connections: 10,
                min_connections: 1,
                connection_timeout_secs: 5,
                idle_timeout_secs: 600,
                max_lifetime_secs: 1800,
            },
            auth: AuthConfig {
                jwt_secret: "test_secret_key".to_string(),
                access_token_duration_secs: 3600, // 1 hour
                refresh_token_duration_secs: 86400 * 7, // 7 days
            },
            cors: CorsConfig {
                allowed_origins: vec!["http://localhost:3000".to_string()],
                max_age_secs: 3600,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                request_timeout_secs: 30,
                body_limit_bytes: 10 * 1024 * 1024, // 10MB
            },
            database: DatabaseConfig {
                url: "sqlite:writemagic.db".to_string(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout_secs: 5,
                idle_timeout_secs: 600,
                max_lifetime_secs: 1800,
            },
            auth: AuthConfig {
                jwt_secret: "default_secret_change_in_production".to_string(),
                access_token_duration_secs: 3600, // 1 hour
                refresh_token_duration_secs: 86400 * 7, // 7 days
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                ],
                max_age_secs: 3600,
            },
        }
    }
}

impl ServerConfig {
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }
}

impl DatabaseConfig {
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }
    
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }
    
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }
}

impl AuthConfig {
    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.access_token_duration_secs as u64)
    }
    
    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.refresh_token_duration_secs as u64)
    }
}