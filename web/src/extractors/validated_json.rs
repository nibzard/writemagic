use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;
use serde::de::DeserializeOwned;
use validator::Validate as ValidatorValidate;


/// JSON extractor with validation using `garde`
/// This extractor deserializes JSON and validates it using garde validation rules
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    T::Context: Default,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidationError::JsonExtraction)?;

        value.validate().map_err(ValidationError::Validation)?;

        Ok(ValidatedJson(value))
    }
}

/// JSON extractor with validation using `validator` crate
/// Alternative to ValidatedJson for those preferring the validator crate
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatorJson<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequest<S> for ValidatorJson<T>
where
    T: DeserializeOwned + ValidatorValidate,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidationError::JsonExtraction)?;

        value.validate().map_err(ValidationError::ValidatorValidation)?;

        Ok(ValidatorJson(value))
    }
}

/// Validation error types
#[derive(Debug)]
pub enum ValidationError {
    JsonExtraction(JsonRejection),
    Validation(garde::Report),
    ValidatorValidation(validator::ValidationErrors),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = match self {
            ValidationError::JsonExtraction(rejection) => {
                let message = match rejection {
                    JsonRejection::JsonDataError(_) => "Invalid JSON data",
                    JsonRejection::JsonSyntaxError(_) => "Invalid JSON syntax",
                    JsonRejection::MissingJsonContentType(_) => "Missing JSON content type",
                    _ => "JSON parsing error",
                };
                (
                    StatusCode::BAD_REQUEST,
                    "JSON_PARSE_ERROR",
                    message,
                    Some(rejection.to_string()),
                )
            }
            ValidationError::Validation(report) => {
                let errors: Vec<String> = report
                    .iter()
                    .map(|(path, error)| format!("{}: {}", path, error.message()))
                    .collect();
                
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    "Request validation failed",
                    Some(errors.join(", ")),
                )
            }
            ValidationError::ValidatorValidation(errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!("{}: {}", field, error.message.as_ref().unwrap_or(&"validation failed".into()))
                        })
                    })
                    .collect();

                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    "Request validation failed",
                    Some(error_messages.join(", ")),
                )
            }
        };

        let body = serde_json::json!({
            "error": error_code,
            "message": message,
            "details": details,
            "status": status.as_u16()
        });

        (status, Json(body)).into_response()
    }
}

/// Pagination extractor
/// Extracts pagination parameters from query string
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            offset: 0,
        }
    }
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        let page = page.max(1);
        let per_page = per_page.min(100).max(1); // Limit max per_page to 100
        let offset = (page - 1) * per_page;
        
        Self {
            page,
            per_page,
            offset,
        }
    }
}

#[axum::async_trait]
impl<S> FromRequest<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let query = req.uri().query().unwrap_or("");
        
        let mut page = 1u32;
        let mut per_page = 20u32;
        
        // Parse query parameters
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                "page" => {
                    page = value.parse().unwrap_or(1);
                }
                "per_page" | "limit" => {
                    per_page = value.parse().unwrap_or(20);
                }
                _ => {}
            }
        }
        
        Ok(Pagination::new(page, per_page))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use garde::Validate;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Validate)]
    struct TestRequest {
        #[garde(length(min = 3, max = 50))]
        name: String,
        #[garde(range(min = 18, max = 120))]
        age: u32,
        #[garde(email)]
        email: String,
    }

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination::new(2, 10);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.offset, 10);
        
        // Test limits
        let pagination = Pagination::new(0, 150);
        assert_eq!(pagination.page, 1); // Minimum 1
        assert_eq!(pagination.per_page, 100); // Maximum 100
        assert_eq!(pagination.offset, 0);
    }

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 20);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn test_validation_error_response() {
        let json_error = ValidationError::JsonExtraction(
            JsonRejection::JsonSyntaxError(serde_json::Error::syntax(
                serde_json::error::ErrorCode::EofWhileParsingString,
                1,
                1,
            ))
        );
        
        let response = json_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}