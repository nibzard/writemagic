//! Shared error types and handling

use thiserror::Error;

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

    #[error("Internal error: {message}")]
    Internal { message: String },
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

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}