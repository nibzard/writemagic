//! Validation utilities and custom validators

use crate::{Result, WritemagicError};
use regex::Regex;
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors};

/// Validation context for domain-specific validation
pub struct ValidationContext {
    pub user_id: Option<crate::EntityId>,
    pub organization_id: Option<crate::EntityId>,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            user_id: None,
            organization_id: None,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_user(mut self, user_id: crate::EntityId) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain validator trait
pub trait DomainValidator<T>: Send + Sync {
    fn validate(&self, value: &T, context: &ValidationContext) -> Result<()>;
}

/// Content validation utilities
pub struct ContentValidator;

impl ContentValidator {
    /// Validate content is not empty or just whitespace
    pub fn validate_not_empty(content: &str) -> std::result::Result<(), ValidationError> {
        if content.trim().is_empty() {
            return Err(ValidationError::new("content_empty"));
        }
        Ok(())
    }
    
    /// Validate content length
    pub fn validate_length(content: &str, min: usize, max: usize) -> std::result::Result<(), ValidationError> {
        let len = content.len();
        if len < min {
            let mut error = ValidationError::new("content_too_short");
            error.add_param(std::borrow::Cow::from("min"), &min);
            error.add_param(std::borrow::Cow::from("actual"), &len);
            return Err(error);
        }
        if len > max {
            let mut error = ValidationError::new("content_too_long");
            error.add_param(std::borrow::Cow::from("max"), &max);
            error.add_param(std::borrow::Cow::from("actual"), &len);
            return Err(error);
        }
        Ok(())
    }
    
    /// Validate no prohibited content
    pub fn validate_no_prohibited_content(content: &str) -> std::result::Result<(), ValidationError> {
        // Check for common patterns that should be filtered
        let prohibited_patterns = [
            r"<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>", // Script tags
            r"javascript:", // JavaScript protocols
            r"data:text/html", // Data URLs with HTML
        ];
        
        for pattern in &prohibited_patterns {
            let regex = Regex::new(pattern).map_err(|_| ValidationError::new("regex_error"))?;
            if regex.is_match(content) {
                return Err(ValidationError::new("prohibited_content"));
            }
        }
        
        Ok(())
    }
}

/// File path validation utilities
pub struct FilePathValidator;

impl FilePathValidator {
    /// Validate file path is safe (no directory traversal)
    pub fn validate_safe_path(path: &str) -> std::result::Result<(), ValidationError> {
        if path.contains("..") || path.contains("~") {
            return Err(ValidationError::new("unsafe_path"));
        }
        
        // Check for absolute paths on Unix systems
        if path.starts_with('/') {
            return Err(ValidationError::new("absolute_path_not_allowed"));
        }
        
        // Check for Windows drive letters
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            return Err(ValidationError::new("windows_drive_not_allowed"));
        }
        
        Ok(())
    }
    
    /// Validate file extension is allowed
    pub fn validate_allowed_extension(path: &str, allowed: &[&str]) -> std::result::Result<(), ValidationError> {
        if let Some(extension) = std::path::Path::new(path).extension() {
            if let Some(ext_str) = extension.to_str() {
                if allowed.contains(&ext_str.to_lowercase().as_str()) {
                    return Ok(());
                }
            }
        }
        
        let mut error = ValidationError::new("invalid_file_extension");
        error.add_param(std::borrow::Cow::from("allowed"), &allowed.join(", "));
        Err(error)
    }
}

/// Convert validation errors to WriteMagic errors
pub fn validation_errors_to_writemagic_error(errors: ValidationErrors) -> WritemagicError {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match error.code.as_ref() {
                "length" => format!("Field '{}' length is invalid", field),
                "range" => format!("Field '{}' value is out of range", field),
                "email" => format!("Field '{}' must be a valid email address", field),
                "url" => format!("Field '{}' must be a valid URL", field),
                "required" => format!("Field '{}' is required", field),
                "content_empty" => format!("Field '{}' cannot be empty", field),
                "content_too_short" => format!("Field '{}' is too short", field),
                "content_too_long" => format!("Field '{}' is too long", field),
                "prohibited_content" => format!("Field '{}' contains prohibited content", field),
                "unsafe_path" => format!("Field '{}' contains unsafe path", field),
                "absolute_path_not_allowed" => format!("Field '{}' absolute paths not allowed", field),
                "windows_drive_not_allowed" => format!("Field '{}' Windows drive letters not allowed", field),
                "invalid_file_extension" => format!("Field '{}' has invalid file extension", field),
                _ => format!("Field '{}' validation failed: {}", field, error.code),
            };
            messages.push(message);
        }
    }
    
    WritemagicError::validation(messages.join("; "))
}

/// Validate with context
pub fn validate_with_context<T: Validate>(value: &T) -> Result<()> {
    value.validate().map_err(validation_errors_to_writemagic_error)
}