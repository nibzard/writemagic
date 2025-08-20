//! Unit tests for error handling and result types

use crate::error::{WritemagicError, Result, ErrorCode, ErrorResponse};
use std::error::Error;

#[cfg(test)]
mod writemagic_error_tests {
    use super::*;

    #[test]
    fn test_error_creation_validation() {
        let error = WritemagicError::validation("Invalid input");
        assert!(matches!(error, WritemagicError::Validation { .. }));
        assert_eq!(error.message(), "Invalid input");
    }

    #[test]
    fn test_error_creation_not_found() {
        let error = WritemagicError::not_found("Entity with ID 123");
        assert!(matches!(error, WritemagicError::NotFound { .. }));
        assert!(error.message().contains("Entity with ID 123"));
    }

    #[test]
    fn test_error_creation_internal() {
        let error = WritemagicError::internal("System error");
        assert!(matches!(error, WritemagicError::Internal { .. }));
        assert_eq!(error.message(), "System error");
    }

    #[test]
    fn test_error_creation_database() {
        let error = WritemagicError::database("Connection failed");
        assert!(matches!(error, WritemagicError::Database { .. }));
        assert_eq!(error.message(), "Connection failed");
    }

    #[test]
    fn test_error_creation_ai_provider() {
        let error = WritemagicError::ai_provider("Provider unavailable");
        assert!(matches!(error, WritemagicError::AiProvider { .. }));
        assert_eq!(error.message(), "Provider unavailable");
    }

    #[test]
    fn test_error_creation_timeout() {
        let error = WritemagicError::timeout(5000);
        assert!(matches!(error, WritemagicError::Timeout { .. }));
        assert!(error.message().contains("5000"));
    }

    #[test]
    fn test_error_creation_rate_limit() {
        let error = WritemagicError::rate_limited(10, 60);
        assert!(matches!(error, WritemagicError::RateLimited { .. }));
        assert!(error.message().contains("10"));
        assert!(error.message().contains("60"));
    }

    #[test]
    fn test_error_creation_authentication() {
        let error = WritemagicError::authentication("Invalid token");
        assert!(matches!(error, WritemagicError::Authentication { .. }));
        assert_eq!(error.message(), "Invalid token");
    }

    #[test]
    fn test_error_creation_version_conflict() {
        let error = WritemagicError::version_conflict("Version mismatch");
        assert!(matches!(error, WritemagicError::VersionConflict { .. }));
        assert_eq!(error.message(), "Version mismatch");
    }

    #[test]
    fn test_error_display() {
        let error = WritemagicError::validation("Test error");
        let display_str = format!("{}", error);
        assert!(display_str.contains("Test error"));
    }

    #[test]
    fn test_error_debug() {
        let error = WritemagicError::validation("Test error");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Validation"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_error_source() {
        let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = WritemagicError::internal_with_source("Wrapper error", source_error);
        
        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_message() {
        let error = WritemagicError::validation("Input error");
        
        assert!(error.message().contains("Input error"));
        // Test debug representation is available
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Validation"));
        assert!(debug_str.contains("Input error"));
    }

    #[test]
    fn test_error_to_response() {
        let error = WritemagicError::validation("Test validation error");
        let response = error.to_error_response(Some("req-123".to_string()));
        
        assert_eq!(response.code, ErrorCode::ValidationFailed);
        assert!(response.message.contains("Test validation error"));
        assert_eq!(response.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_error_display_format() {
        let timeout_error = WritemagicError::timeout(5000);
        let rate_limit_error = WritemagicError::rate_limited(10, 60);
        let ai_error = WritemagicError::ai_provider("Temporary failure");
        
        assert!(timeout_error.to_string().contains("5000"));
        assert!(rate_limit_error.to_string().contains("10"));
        assert!(rate_limit_error.to_string().contains("60"));
        assert!(ai_error.to_string().contains("Temporary failure"));
    }
}

#[cfg(test)]
mod error_response_tests {
    use super::*;

    #[test]
    fn test_error_response_from_error() {
        let error = WritemagicError::validation("Invalid field");
        let response = error.to_error_response(None);
        
        assert_eq!(response.code, ErrorCode::ValidationFailed);
        assert!(response.message.contains("Invalid field"));
        assert!(response.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let rate_limit_error = WritemagicError::rate_limited(10, 60);
        let response = rate_limit_error.to_error_response(None);
        
        assert_eq!(response.code, ErrorCode::RateLimited);
        assert!(response.details.is_some());
        
        let details = response.details.as_ref().unwrap();
        assert!(details["limit"] == 10);
        assert!(details["window_seconds"] == 60);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = WritemagicError::not_found("Entity not found");
        let response = error.to_error_response(Some("req-456".to_string()));
        
        let serialized = serde_json::to_string(&response).expect("Serialize ErrorResponse");
        
        // Basic check that it serializes without error
        assert!(serialized.contains("NOT_FOUND"));
        assert!(serialized.contains("Entity not found"));
        assert!(serialized.contains("req-456"));
    }
}

#[cfg(test)]
mod result_tests {
    use super::*;

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(WritemagicError::validation("Test error"));
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error.message(), "Test error");
    }

    #[test]
    fn test_result_map() {
        let result: Result<i32> = Ok(21);
        let mapped = result.map(|x| x * 2);
        
        assert!(mapped.is_ok());
        assert_eq!(mapped.unwrap(), 42);
    }

    #[test]
    fn test_result_map_err() {
        let result: Result<i32> = Err(WritemagicError::validation("Original error"));
        let mapped = result.map_err(|_| WritemagicError::internal("Mapped error"));
        
        assert!(mapped.is_err());
        assert_eq!(mapped.unwrap_err().message(), "Mapped error");
    }

    #[test]
    fn test_result_and_then() {
        let result: Result<i32> = Ok(21);
        let chained = result.and_then(|x| {
            if x > 20 {
                Ok(x * 2)
            } else {
                Err(WritemagicError::validation("Too small"))
            }
        });
        
        assert!(chained.is_ok());
        assert_eq!(chained.unwrap(), 42);
    }

    #[test]
    fn test_result_or_else() {
        let result: Result<i32> = Err(WritemagicError::validation("Test error"));
        let recovered: Result<i32> = result.or_else(|_| Ok(42));
        
        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), 42);
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    fn operation_that_fails() -> Result<String> {
        Err(WritemagicError::validation("Operation failed"))
    }

    fn operation_that_succeeds() -> Result<String> {
        Ok("Success".to_string())
    }

    #[test]
    fn test_question_mark_operator() {
        fn calling_function() -> Result<String> {
            let result = operation_that_fails()?;
            Ok(format!("Got: {}", result))
        }
        
        let result = calling_function();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message(), "Operation failed");
    }

    #[test]
    fn test_successful_propagation() {
        fn calling_function() -> Result<String> {
            let result = operation_that_succeeds()?;
            Ok(format!("Got: {}", result))
        }
        
        let result = calling_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Got: Success");
    }

    #[tokio::test]
    async fn test_async_error_propagation() {
        async fn async_operation() -> Result<String> {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            Err(WritemagicError::timeout(5000))
        }
        
        async fn calling_async_function() -> Result<String> {
            let result = async_operation().await?;
            Ok(format!("Got: {}", result))
        }
        
        let result = calling_async_function().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WritemagicError::Timeout { .. }));
    }
}