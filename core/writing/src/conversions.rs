//! Type conversion utilities for web DTOs and domain types

use crate::value_objects::{DocumentTitle, DocumentContent, ProjectName};
use crate::entities::{Document, Project};
use crate::aggregates::{DocumentAggregate, ProjectAggregate};
use writemagic_shared::{EntityId, Result, WritemagicError, ContentType};
use serde::{Serialize, Deserialize};

/// Document DTO for web API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub word_count: u32,
    pub character_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: u64,
    pub is_deleted: bool,
}

/// Project DTO for web API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub document_ids: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: u64,
    pub is_deleted: bool,
}

/// Create document request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentDto {
    pub title: String,
    pub content: Option<String>,
    pub content_type: Option<String>,
}

/// Update document request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentDto {
    pub title: Option<String>,
    pub content: Option<String>,
}

/// Create project request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectDto {
    pub name: String,
    pub description: Option<String>,
}

/// Conversion functions for Document types
impl DocumentDto {
    /// Convert from Document entity
    pub fn from_document(document: &Document) -> Self {
        Self {
            id: document.id.to_string(),
            title: document.title.clone(),
            content: document.content.clone(),
            content_type: document.content_type.to_string(),
            word_count: document.word_count,
            character_count: document.character_count,
            created_at: document.created_at.as_datetime(),
            updated_at: document.updated_at.as_datetime(),
            created_by: document.created_by.map(|id| id.to_string()),
            updated_by: document.updated_by.map(|id| id.to_string()),
            version: document.version,
            is_deleted: document.is_deleted,
        }
    }

    /// Convert from DocumentAggregate
    pub fn from_aggregate(aggregate: &DocumentAggregate) -> Self {
        Self::from_document(aggregate.document())
    }
}

/// Conversion functions for Project types
impl ProjectDto {
    /// Convert from Project entity
    pub fn from_project(project: &Project) -> Self {
        Self {
            id: project.id.to_string(),
            name: project.name.clone(),
            description: project.description.clone(),
            document_ids: project.document_ids.iter().map(|id| id.to_string()).collect(),
            created_at: project.created_at.as_datetime(),
            updated_at: project.updated_at.as_datetime(),
            created_by: project.created_by.map(|id| id.to_string()),
            updated_by: project.updated_by.map(|id| id.to_string()),
            version: project.version,
            is_deleted: project.is_deleted,
        }
    }

    /// Convert from ProjectAggregate
    pub fn from_aggregate(aggregate: &ProjectAggregate) -> Self {
        Self::from_project(aggregate.project())
    }
}

/// Type conversion utilities
pub struct TypeConverter;

impl TypeConverter {
    /// Convert string to EntityId
    pub fn string_to_entity_id(id_str: &str) -> Result<EntityId> {
        EntityId::from_string(id_str)
            .map_err(|e| WritemagicError::validation(&format!("Invalid entity ID: {}", e)))
    }

    /// Convert string to DocumentTitle
    pub fn string_to_document_title(title: &str) -> Result<DocumentTitle> {
        DocumentTitle::new(title)
    }

    /// Convert string to DocumentContent
    pub fn string_to_document_content(content: &str) -> Result<DocumentContent> {
        DocumentContent::new(content)
    }

    /// Convert string to ProjectName
    pub fn string_to_project_name(name: &str) -> Result<ProjectName> {
        ProjectName::new(name)
    }

    /// Convert string to ContentType
    pub fn string_to_content_type(content_type_str: &str) -> Result<ContentType> {
        match content_type_str.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ContentType::Markdown),
            "plaintext" | "text" | "txt" => Ok(ContentType::PlainText),
            "html" => Ok(ContentType::Html),
            "json" => Ok(ContentType::Json),
            "yaml" | "yml" => Ok(ContentType::Yaml),
            "xml" => Ok(ContentType::Code { language: "xml".to_string() }),
            lang if lang.starts_with("code:") => {
                let language = lang.strip_prefix("code:").unwrap_or(lang).to_string();
                Ok(ContentType::Code { language })
            },
            _ => Err(WritemagicError::validation(&format!("Unsupported content type: {}", content_type_str)))
        }
    }

    /// Convert CreateDocumentDto to domain types
    pub fn create_document_dto_to_domain(
        dto: &CreateDocumentDto,
        _created_by: Option<EntityId>,
    ) -> Result<(DocumentTitle, DocumentContent, ContentType)> {
        let title = Self::string_to_document_title(&dto.title)?;
        let content = Self::string_to_document_content(&dto.content.as_deref().unwrap_or(""))?;
        let content_type = match &dto.content_type {
            Some(ct) => Self::string_to_content_type(ct)?,
            None => ContentType::Markdown, // Default to Markdown
        };
        Ok((title, content, content_type))
    }

    /// Convert UpdateDocumentDto to domain types
    pub fn update_document_dto_to_domain(
        dto: &UpdateDocumentDto,
    ) -> Result<(Option<DocumentTitle>, Option<DocumentContent>)> {
        let title = match &dto.title {
            Some(t) => Some(Self::string_to_document_title(t)?),
            None => None,
        };
        let content = match &dto.content {
            Some(c) => Some(Self::string_to_document_content(c)?),
            None => None,
        };
        Ok((title, content))
    }

    /// Convert CreateProjectDto to domain types
    pub fn create_project_dto_to_domain(
        dto: &CreateProjectDto,
    ) -> Result<ProjectName> {
        Self::string_to_project_name(&dto.name)
    }
}

/// Pagination conversion utilities
pub struct PaginationConverter;

impl PaginationConverter {
    /// Convert web pagination parameters to domain Pagination
    pub fn from_web_params(page: u32, per_page: u32) -> Result<writemagic_shared::Pagination> {
        let limit = per_page.clamp(1, 100); // Limit to reasonable bounds
        let offset = page.saturating_sub(1) * limit; // Convert 1-based page to 0-based offset
        writemagic_shared::Pagination::new(offset, limit)
    }

    /// Calculate pagination metadata for responses
    pub fn calculate_metadata(total: u64, page: u32, per_page: u32) -> PaginationMetadata {
        let total_pages = if per_page > 0 {
            ((total as f64) / (per_page as f64)).ceil() as u32
        } else {
            0
        };

        PaginationMetadata {
            total,
            page,
            per_page,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Pagination metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadata {
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// List response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMetadata,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let pagination = PaginationConverter::calculate_metadata(total, page, per_page);
        Self { items, pagination }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writemagic_shared::EntityId;

    #[test]
    fn test_string_to_entity_id() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = TypeConverter::string_to_entity_id(uuid_str);
        assert!(result.is_ok());

        let invalid_uuid = "invalid-uuid";
        let result = TypeConverter::string_to_entity_id(invalid_uuid);
        assert!(result.is_err());
    }

    #[test]
    fn test_document_title_conversion() {
        let title = "Test Document";
        let result = TypeConverter::string_to_document_title(title);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), title);

        let empty_title = "";
        let result = TypeConverter::string_to_document_title(empty_title);
        assert!(result.is_err());
    }

    #[test]
    fn test_content_type_conversion() {
        assert!(matches!(
            TypeConverter::string_to_content_type("markdown").unwrap(),
            ContentType::Markdown
        ));
        assert!(matches!(
            TypeConverter::string_to_content_type("text").unwrap(),
            ContentType::PlainText
        ));
        assert!(TypeConverter::string_to_content_type("invalid").is_err());
    }

    #[test]
    fn test_pagination_conversion() {
        let pagination = PaginationConverter::from_web_params(1, 20).unwrap();
        assert_eq!(pagination.offset, 0);
        assert_eq!(pagination.limit, 20);

        let pagination = PaginationConverter::from_web_params(3, 10).unwrap();
        assert_eq!(pagination.offset, 20);
        assert_eq!(pagination.limit, 10);
    }

    #[test]
    fn test_pagination_metadata() {
        let metadata = PaginationConverter::calculate_metadata(100, 2, 20);
        assert_eq!(metadata.total, 100);
        assert_eq!(metadata.page, 2);
        assert_eq!(metadata.per_page, 20);
        assert_eq!(metadata.total_pages, 5);
        assert!(metadata.has_next);
        assert!(metadata.has_prev);
    }
}