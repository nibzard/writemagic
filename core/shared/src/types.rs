//! Shared value objects and common types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use validator::Validate;

/// Unique identifier for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

/// Timestamp value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S UTC"))
    }
}

/// Content hash for change detection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn new(content: &str) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Self(format!("{:x}", result))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File path value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct FilePath {
    #[validate(length(min = 1, max = 4096))]
    pub path: String,
}

impl FilePath {
    pub fn new(path: impl Into<String>) -> crate::Result<Self> {
        let path = path.into();
        let file_path = Self { path };
        file_path.validate().map_err(|e| {
            crate::WritemagicError::validation(format!("Invalid file path: {}", e))
        })?;
        Ok(file_path)
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// Content type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Markdown,
    PlainText,
    Html,
    Json,
    Yaml,
    Code { language: String },
}

impl ContentType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Self::Markdown,
            "txt" => Self::PlainText,
            "html" | "htm" => Self::Html,
            "json" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "rs" => Self::Code {
                language: "rust".to_string(),
            },
            "js" | "ts" => Self::Code {
                language: "javascript".to_string(),
            },
            "py" => Self::Code {
                language: "python".to_string(),
            },
            _ => Self::PlainText,
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::PlainText => write!(f, "plain_text"),
            Self::Html => write!(f, "html"),
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Code { language } => write!(f, "code:{}", language),
        }
    }
}

/// Version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = 0, max = 10000))]
    pub offset: u32,
    #[validate(range(min = 1, max = 1000))]
    pub limit: u32,
}

impl Pagination {
    pub fn new(offset: u32, limit: u32) -> crate::Result<Self> {
        let pagination = Self { offset, limit };
        pagination.validate().map_err(|e| {
            crate::WritemagicError::validation(format!("Invalid pagination: {}", e))
        })?;
        Ok(pagination)
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}