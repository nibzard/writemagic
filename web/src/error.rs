use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Web-specific error type with rich context and proper HTTP mapping
/// This follows the unified error type pattern from the best practices guide
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] writemagic_shared::WritemagicError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication required")]
    Unauthorized,
    
    #[error("Insufficient permissions")]
    Forbidden,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Rate limit exceeded")]
    TooManyRequests,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP client error")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

/// Structured error response for API clients
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

#[derive(serde::Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: String,
    request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message, details) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required".to_string(),
                None,
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Insufficient permissions".to_string(),
                None,
            ),
            AppError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("Resource not found: {}", resource),
                None,
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg.clone(),
                None,
            ),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Rate limit exceeded".to_string(),
                None,
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg.clone(),
                None,
            ),
            AppError::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNPROCESSABLE_ENTITY",
                msg.clone(),
                None,
            ),
            AppError::ExternalService(service) => {
                tracing::error!("External service error: {}", service);
                (
                    StatusCode::BAD_GATEWAY,
                    "EXTERNAL_SERVICE_ERROR",
                    "External service unavailable".to_string(),
                    Some(json!({"service": service})),
                )
            }
            AppError::Jwt(e) => {
                tracing::warn!("JWT error: {}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    "INVALID_TOKEN",
                    "Invalid or expired token".to_string(),
                    None,
                )
            }
            AppError::Serialization(e) => {
                tracing::error!("Serialization error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SERIALIZATION_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
            AppError::HttpClient(e) => {
                tracing::error!("HTTP client error: {}", e);
                (
                    StatusCode::BAD_GATEWAY,
                    "HTTP_CLIENT_ERROR",
                    "External service error".to_string(),
                    None,
                )
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: error_code,
                message: error_message,
                request_id: get_request_id(),
                details,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

/// Validation error extension for garde validation
impl From<garde::Report> for AppError {
    fn from(report: garde::Report) -> Self {
        let message = report
            .iter()
            .map(|(path, error)| format!("{}: {}", path, error.message()))
            .collect::<Vec<_>>()
            .join(", ");
        AppError::Validation(message)
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    fn with_context(self, msg: &'static str) -> Result<T>;
    fn not_found(self, resource: &str) -> Result<T>;
    fn conflict(self, msg: &str) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E> 
where
    E: Into<AppError>,
{
    fn with_context(self, msg: &'static str) -> Result<T> {
        self.map_err(|e| {
            let inner_error = e.into();
            tracing::error!("Error with context '{}': {:?}", msg, inner_error);
            inner_error
        })
    }
    
    fn not_found(self, resource: &str) -> Result<T> {
        self.map_err(|_| AppError::NotFound(resource.to_string()))
    }
    
    fn conflict(self, msg: &str) -> Result<T> {
        self.map_err(|_| AppError::Conflict(msg.to_string()))
    }
}

/// Get request ID from tracing context if available
fn get_request_id() -> Option<String> {
    // In a real application, you would extract this from the request headers
    // or generate one in middleware. For now, we'll generate a random one.
    Some(Uuid::new_v4().to_string())
}

/// Convenience type alias following the best practices guide
pub type Result<T> = std::result::Result<T, AppError>;