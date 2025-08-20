//! Shared error types and handling

use thiserror::Error;
use serde::Serialize;
use std::backtrace::Backtrace;

/// Structured error response for APIs
#[derive(Debug, Serialize, Clone)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub timestamp: i64,
}

/// Standard error codes for API responses
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    ValidationFailed,
    RateLimited,
    InternalError,
    ServiceUnavailable,
    BadGateway,
}

impl ErrorCode {
    /// Get HTTP status code for this error code
    pub const fn status_code(self) -> u16 {
        match self {
            Self::InvalidRequest | Self::ValidationFailed => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::RateLimited => 429,
            Self::InternalError => 500,
            Self::BadGateway => 502,
            Self::ServiceUnavailable => 503,
        }
    }
}

/// Main error type for WriteMagic operations
#[derive(Error, Debug)]
pub enum WritemagicError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Repository error: {message}")]
    Repository { message: String },

    #[error("AI provider error: {message}")]
    AiProvider { message: String },

    #[error("Git operation error: {message}")]
    Git { message: String },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Security error: {message}")]
    Security { message: String },

    #[error("Internal error: {message}")]
    Internal { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Backtrace,
    },

    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Rate limit exceeded: {limit} requests per {window_seconds}s")]
    RateLimited { limit: u32, window_seconds: u32 },

    #[error("Version conflict: {message}")]
    VersionConflict { message: String },

    #[error("Feature not implemented: {message}")]
    NotImplemented { message: String },
}

/// Result type alias for WriteMagic operations
pub type Result<T> = std::result::Result<T, WritemagicError>;

impl WritemagicError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn repository(message: impl Into<String>) -> Self {
        Self::Repository {
            message: message.into(),
        }
    }

    pub fn ai_provider(message: impl Into<String>) -> Self {
        Self::AiProvider {
            message: message.into(),
        }
    }

    pub fn git(message: impl Into<String>) -> Self {
        Self::Git {
            message: message.into(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    pub fn security(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn internal_with_source<E>(message: impl Into<String>, source: E) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    pub fn rate_limited(limit: u32, window_seconds: u32) -> Self {
        Self::RateLimited { limit, window_seconds }
    }

    pub fn version_conflict(message: impl Into<String>) -> Self {
        Self::VersionConflict {
            message: message.into(),
        }
    }

    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::NotImplemented {
            message: message.into(),
        }
    }

    /// Get error message for debugging and testing
    pub fn message(&self) -> String {
        match self {
            Self::Validation { message } => message.clone(),
            Self::Repository { message } => message.clone(),
            Self::AiProvider { message } => message.clone(),
            Self::Git { message } => message.clone(),
            Self::Database { message } => message.clone(),
            Self::Authentication { message } => message.clone(),
            Self::Configuration { message } => message.clone(),
            Self::Network { message } => message.clone(),
            Self::Security { message } => message.clone(),
            Self::Internal { message, .. } => message.clone(),
            Self::NotFound { resource } => resource.clone(),
            Self::VersionConflict { message } => message.clone(),
            Self::NotImplemented { message } => message.clone(),
            Self::Io { source } => source.to_string(),
            Self::Serialization { source } => source.to_string(),
            Self::Timeout { timeout_ms } => format!("Request timeout after {}ms", timeout_ms),
            Self::Cancelled => "Operation cancelled".to_string(),
            Self::RateLimited { limit, window_seconds } => {
                format!("Rate limit exceeded: {} requests per {}s", limit, window_seconds)
            },
        }
    }

    /// Convert to structured error response
    pub fn to_error_response(&self, request_id: Option<String>) -> ErrorResponse {
        let (code, details) = match self {
            Self::Validation { .. } => (ErrorCode::ValidationFailed, None),
            Self::Authentication { .. } => (ErrorCode::Unauthorized, None),
            Self::Security { .. } => (ErrorCode::Forbidden, None),
            Self::NotFound { resource } => (
                ErrorCode::NotFound, 
                Some(serde_json::json!({ "resource": resource }))
            ),
            Self::RateLimited { limit, window_seconds } => (
                ErrorCode::RateLimited,
                Some(serde_json::json!({
                    "limit": limit,
                    "window_seconds": window_seconds
                }))
            ),
            Self::Network { .. } | Self::AiProvider { .. } => (
                ErrorCode::ServiceUnavailable, 
                None
            ),
            Self::VersionConflict { .. } => (ErrorCode::Conflict, None),
            Self::NotImplemented { .. } => (ErrorCode::ServiceUnavailable, None),
            _ => (ErrorCode::InternalError, None),
        };

        ErrorResponse {
            code,
            message: self.to_string(),
            details,
            request_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}