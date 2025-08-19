use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::extractors::{AuthenticatedUser, Pagination, ValidatedJson};
use crate::state::AppState;
use writemagic_shared::types::{DocumentId, DocumentMetadata};
use writemagic_writing::services::WritingService;

/// Document creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    
    #[garde(length(max = 1000))]
    pub description: Option<String>,
    
    pub content: Option<String>,
    
    #[garde(custom(validate_tags))]
    pub tags: Option<Vec<String>>,
}

/// Document update request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[garde(length(min = 1, max = 255))]
    pub title: Option<String>,
    
    #[garde(length(max = 1000))]
    pub description: Option<String>,
    
    pub content: Option<String>,
    
    #[garde(custom(validate_tags))]
    pub tags: Option<Vec<String>>,
}

/// Document response
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
}

/// Documents list response
#[derive(Debug, Serialize)]
pub struct DocumentsListResponse {
    pub documents: Vec<DocumentSummary>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Document summary for list views
#[derive(Debug, Serialize)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub word_count: u32,
}

/// Create a new document
pub async fn create_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(request): ValidatedJson<CreateDocumentRequest>,
) -> AppResult<(StatusCode, Json<DocumentResponse>)> {
    tracing::info!("Creating document for user {}: {}", user.user_id, request.title);

    // Access the core engine's writing service
    let writing_service = state.core_engine.writing_service();
    
    // Create document metadata
    let metadata = DocumentMetadata {
        title: request.title.clone(),
        description: request.description.clone(),
        tags: request.tags.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Create the document using the writing service
    let document_id = writing_service
        .create_document(&user.user_id, metadata, request.content.as_deref())
        .await
        .map_err(|e| AppError::Database(format!("Failed to create document: {}", e)))?;

    // Retrieve the created document to return it
    let document = writing_service
        .get_document(&document_id)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve created document: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Document not found after creation".to_string()))?;

    let response = DocumentResponse {
        id: document_id.to_string(),
        title: document.metadata.title,
        description: document.metadata.description,
        content: document.content,
        tags: document.metadata.tags,
        created_at: document.metadata.created_at,
        updated_at: document.metadata.updated_at,
        user_id: user.user_id,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a document by ID
pub async fn get_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::debug!("Getting document {} for user {}", document_id, user.user_id);

    let doc_id = DocumentId::from(document_id);
    let writing_service = state.core_engine.writing_service();

    let document = writing_service
        .get_document(&doc_id)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve document: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // TODO: Add proper ownership/permission checking
    // For now, we'll return the document without ownership verification

    let response = DocumentResponse {
        id: doc_id.to_string(),
        title: document.metadata.title,
        description: document.metadata.description,
        content: document.content,
        tags: document.metadata.tags,
        created_at: document.metadata.created_at,
        updated_at: document.metadata.updated_at,
        user_id: user.user_id, // This should come from the document in real implementation
    };

    Ok(Json(response))
}

/// Update a document
pub async fn update_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
    ValidatedJson(request): ValidatedJson<UpdateDocumentRequest>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::info!("Updating document {} for user {}", document_id, user.user_id);

    let doc_id = DocumentId::from(document_id);
    let writing_service = state.core_engine.writing_service();

    // Get existing document
    let mut document = writing_service
        .get_document(&doc_id)
        .await
        .map_err(|e| AppError::Database(format!("Failed to retrieve document: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // TODO: Add proper ownership/permission checking

    // Update metadata if provided
    if let Some(title) = request.title {
        document.metadata.title = title;
    }
    if let Some(description) = request.description {
        document.metadata.description = Some(description);
    }
    if let Some(tags) = request.tags {
        document.metadata.tags = tags;
    }
    if let Some(content) = request.content {
        document.content = Some(content);
    }

    document.metadata.updated_at = chrono::Utc::now();

    // Update the document
    writing_service
        .update_document(&doc_id, document.metadata.clone(), document.content.as_deref())
        .await
        .map_err(|e| AppError::Database(format!("Failed to update document: {}", e)))?;

    let response = DocumentResponse {
        id: doc_id.to_string(),
        title: document.metadata.title,
        description: document.metadata.description,
        content: document.content,
        tags: document.metadata.tags,
        created_at: document.metadata.created_at,
        updated_at: document.metadata.updated_at,
        user_id: user.user_id,
    };

    Ok(Json(response))
}

/// Delete a document
pub async fn delete_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
) -> AppResult<StatusCode> {
    tracing::info!("Deleting document {} for user {}", document_id, user.user_id);

    let doc_id = DocumentId::from(document_id);
    let writing_service = state.core_engine.writing_service();

    // TODO: Add proper ownership/permission checking

    writing_service
        .delete_document(&doc_id)
        .await
        .map_err(|e| AppError::Database(format!("Failed to delete document: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// List user's documents with pagination
pub async fn list_documents(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
) -> AppResult<Json<DocumentsListResponse>> {
    tracing::debug!(
        "Listing documents for user {} (page {}, per_page {})",
        user.user_id,
        pagination.page,
        pagination.per_page
    );

    let writing_service = state.core_engine.writing_service();

    // TODO: Implement proper user-scoped document listing
    // For now, this is a placeholder implementation
    
    // Get all documents (this should be filtered by user in real implementation)
    let documents = writing_service
        .list_documents(Some(pagination.per_page as usize), Some(pagination.offset as usize))
        .await
        .map_err(|e| AppError::Database(format!("Failed to list documents: {}", e)))?;

    let document_summaries: Vec<DocumentSummary> = documents
        .into_iter()
        .map(|doc| DocumentSummary {
            id: doc.id.to_string(),
            title: doc.metadata.title,
            description: doc.metadata.description,
            tags: doc.metadata.tags,
            created_at: doc.metadata.created_at,
            updated_at: doc.metadata.updated_at,
            word_count: doc.content
                .as_ref()
                .map(|content| content.split_whitespace().count() as u32)
                .unwrap_or(0),
        })
        .collect();

    // TODO: Get actual total count from database
    let total = document_summaries.len() as u64;
    let total_pages = ((total as f64) / (pagination.per_page as f64)).ceil() as u32;

    let response = DocumentsListResponse {
        documents: document_summaries,
        total,
        page: pagination.page,
        per_page: pagination.per_page,
        total_pages,
    };

    Ok(Json(response))
}

// Validation helpers
fn validate_tags(tags: &[String], _context: &()) -> garde::Result {
    if tags.len() > 10 {
        return Err(garde::Error::new("Cannot have more than 10 tags"));
    }
    
    for tag in tags {
        if tag.is_empty() || tag.len() > 50 {
            return Err(garde::Error::new("Tags must be between 1 and 50 characters"));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_validation() {
        let valid_request = CreateDocumentRequest {
            title: "Test Document".to_string(),
            description: Some("A test document".to_string()),
            content: Some("Document content".to_string()),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        };
        assert!(valid_request.validate(&()).is_ok());

        // Title too short
        let invalid_request = CreateDocumentRequest {
            title: "".to_string(),
            description: None,
            content: None,
            tags: None,
        };
        assert!(invalid_request.validate(&()).is_err());

        // Too many tags
        let invalid_request = CreateDocumentRequest {
            title: "Test".to_string(),
            description: None,
            content: None,
            tags: Some((0..15).map(|i| format!("tag{}", i)).collect()),
        };
        assert!(invalid_request.validate(&()).is_err());
    }

    #[test]
    fn test_validate_tags() {
        // Valid tags
        let valid_tags = vec!["test".to_string(), "example".to_string()];
        assert!(validate_tags(&valid_tags, &()).is_ok());

        // Too many tags
        let too_many_tags: Vec<String> = (0..15).map(|i| format!("tag{}", i)).collect();
        assert!(validate_tags(&too_many_tags, &()).is_err());

        // Empty tag
        let empty_tag = vec!["".to_string()];
        assert!(validate_tags(&empty_tag, &()).is_err());

        // Tag too long
        let long_tag = vec!["a".repeat(51)];
        assert!(validate_tags(&long_tag, &()).is_err());
    }
}