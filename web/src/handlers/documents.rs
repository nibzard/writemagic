use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use garde::Validate;
use serde::Deserialize;

use crate::error::{AppError, Result as AppResult};
use crate::extractors::{AuthenticatedUser, Pagination, ValidatedJson};
use crate::state::AppState;
use writemagic_writing::{
    DocumentDto, CreateDocumentDto, UpdateDocumentDto, TypeConverter, 
    PaginationConverter, ListResponse
};

/// Web-specific document creation request (keeping for validation)
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    
    #[garde(skip)]
    pub content: Option<String>,
    
    #[garde(skip)]
    pub content_type: Option<String>,
}

/// Web-specific document update request (keeping for validation)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[garde(length(min = 1, max = 255))]
    pub title: Option<String>,
    
    #[garde(skip)]
    pub content: Option<String>,
}

/// Create a new document
pub async fn create_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(request): ValidatedJson<CreateDocumentRequest>,
) -> AppResult<(StatusCode, Json<DocumentDto>)> {
    tracing::info!("Creating document for user {}: {}", user.user_id, request.title);

    // Parse user ID
    let user_entity_id = TypeConverter::string_to_entity_id(&user.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Convert web DTO to domain DTO
    let create_dto = CreateDocumentDto {
        title: request.title,
        content: request.content,
        content_type: request.content_type,
    };

    // Convert to domain types
    let (title, content, content_type) = TypeConverter::create_document_dto_to_domain(&create_dto, Some(user_entity_id))
        .map_err(|e| AppError::BadRequest(format!("Invalid document data: {}", e)))?;

    // Access the core engine's writing service
    let writing_service = state.core_engine.document_management_service();

    // Create the document using the writing service
    let document_aggregate = writing_service
        .create_document(title, content, content_type, Some(user_entity_id))
        .await
        .map_err(AppError::Database)?;

    // Convert to DTO for response
    let response = DocumentDto::from_aggregate(&document_aggregate);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a document by ID
pub async fn get_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
) -> AppResult<Json<DocumentDto>> {
    tracing::debug!("Getting document {} for user {}", document_id, user.user_id);

    // Parse document ID
    let doc_id = TypeConverter::string_to_entity_id(&document_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid document ID: {}", e)))?;

    let writing_service = state.core_engine.document_management_service();

    // Get the document
    let document_aggregate = writing_service
        .get_document(&doc_id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // TODO: Add proper ownership/permission checking
    // For now, we'll return the document without ownership verification

    // Convert to DTO for response
    let response = DocumentDto::from_aggregate(&document_aggregate);

    Ok(Json(response))
}

/// Update a document
pub async fn update_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
    ValidatedJson(request): ValidatedJson<UpdateDocumentRequest>,
) -> AppResult<Json<DocumentDto>> {
    tracing::info!("Updating document {} for user {}", document_id, user.user_id);

    // Parse IDs
    let doc_id = TypeConverter::string_to_entity_id(&document_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid document ID: {}", e)))?;
    let user_entity_id = TypeConverter::string_to_entity_id(&user.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Convert web DTO to domain DTO
    let update_dto = UpdateDocumentDto {
        title: request.title,
        content: request.content,
    };

    // Convert to domain types
    let (title, content) = TypeConverter::update_document_dto_to_domain(&update_dto)
        .map_err(|e| AppError::BadRequest(format!("Invalid update data: {}", e)))?;

    let writing_service = state.core_engine.document_management_service();

    // TODO: Add proper ownership/permission checking

    // Update the document
    let updated_aggregate = writing_service
        .update_document(doc_id, title, content, Some(user_entity_id))
        .await
        .map_err(AppError::Database)?;

    // Convert to DTO for response
    let response = DocumentDto::from_aggregate(&updated_aggregate);

    Ok(Json(response))
}

/// Delete a document
pub async fn delete_document(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(document_id): Path<String>,
) -> AppResult<StatusCode> {
    tracing::info!("Deleting document {} for user {}", document_id, user.user_id);

    // Parse IDs
    let doc_id = TypeConverter::string_to_entity_id(&document_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid document ID: {}", e)))?;
    let user_entity_id = TypeConverter::string_to_entity_id(&user.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let writing_service = state.core_engine.document_management_service();

    // TODO: Add proper ownership/permission checking

    writing_service
        .delete_document(doc_id, Some(user_entity_id))
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// List user's documents with pagination
pub async fn list_documents(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
) -> AppResult<Json<ListResponse<DocumentDto>>> {
    tracing::debug!(
        "Listing documents for user {} (page {}, per_page {})",
        user.user_id,
        pagination.page,
        pagination.per_page
    );

    // Parse user ID
    let user_entity_id = TypeConverter::string_to_entity_id(&user.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Convert web pagination to domain pagination
    let domain_pagination = PaginationConverter::from_web_params(pagination.page, pagination.per_page)
        .map_err(|e| AppError::BadRequest(format!("Invalid pagination: {}", e)))?;

    let writing_service = state.core_engine.document_management_service();

    // Get user's documents with pagination
    let document_aggregates = writing_service
        .list_documents_by_creator(&user_entity_id, domain_pagination)
        .await
        .map_err(AppError::Database)?;

    // Convert to DTOs
    let document_dtos: Vec<DocumentDto> = document_aggregates
        .into_iter()
        .map(|aggregate| DocumentDto::from_aggregate(&aggregate))
        .collect();

    // TODO: Get actual total count from database
    // For now, approximate based on returned results
    let total = document_dtos.len() as u64;

    let response = ListResponse::new(document_dtos, total, pagination.page, pagination.per_page);

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_validation() {
        let valid_request = CreateDocumentRequest {
            title: "Test Document".to_string(),
            content: Some("Document content".to_string()),
            content_type: Some("markdown".to_string()),
        };
        assert!(valid_request.validate(&()).is_ok());

        // Title too short
        let invalid_request = CreateDocumentRequest {
            title: "".to_string(),
            content: None,
            content_type: None,
        };
        assert!(invalid_request.validate(&()).is_err());
    }

    #[test]
    fn test_update_document_request_validation() {
        let valid_request = UpdateDocumentRequest {
            title: Some("Updated Title".to_string()),
            content: Some("Updated content".to_string()),
        };
        assert!(valid_request.validate(&()).is_ok());

        // Empty title should fail
        let invalid_request = UpdateDocumentRequest {
            title: Some("".to_string()),
            content: None,
        };
        assert!(invalid_request.validate(&()).is_err());
    }
}