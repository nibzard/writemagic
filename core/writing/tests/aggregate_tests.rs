//! Unit tests for aggregate operations
//! 
//! These tests ensure proper aggregate loading, reloading, and version conflict resolution

use writemagic_writing::{
    aggregates::{DocumentAggregate, ProjectAggregate},
    entities::{Document, Project},
    value_objects::{DocumentTitle, DocumentContent, ProjectName, TextSelection},
};
use writemagic_shared::{EntityId, ContentType, Timestamp, Result, WritemagicError};

#[tokio::test]
async fn test_document_aggregate_version_conflict_detection() -> Result<()> {
    let title = DocumentTitle::new("Test Document".to_string())?;
    let content = DocumentContent::new("Test content".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = DocumentAggregate::new(
        title,
        content,
        ContentType::Markdown,
        created_by,
    );
    
    // Check version conflict with expected version 1 (current)
    assert!(aggregate.check_version_conflict(1).is_ok());
    
    // Check version conflict with expected version 2 (should fail)
    assert!(aggregate.check_version_conflict(2).is_err());
    
    // Update content to increment version
    let new_content = DocumentContent::new("Updated content".to_string())?;
    aggregate.update_content(new_content, None, created_by)?;
    
    // Now version should be 2
    assert!(aggregate.check_version_conflict(2).is_ok());
    assert!(aggregate.check_version_conflict(1).is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_document_aggregate_reload_preserves_events() -> Result<()> {
    let title = DocumentTitle::new("Test Document".to_string())?;
    let content = DocumentContent::new("Test content".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = DocumentAggregate::new(
        title.clone(),
        content.clone(),
        ContentType::Markdown,
        created_by,
    );
    
    // Make changes to generate events
    let new_content = DocumentContent::new("Updated content".to_string())?;
    aggregate.update_content(new_content, None, created_by)?;
    
    // Should have uncommitted events
    assert!(!aggregate.uncommitted_events().is_empty());
    let events_count = aggregate.uncommitted_events().len();
    
    // Create a fresh document (simulating repo save)
    let mut fresh_document = Document::new(
        title.value.clone(),
        "Updated content".to_string(),
        ContentType::Markdown,
        created_by,
    );
    fresh_document.id = *aggregate.document().id(); // Match the ID
    fresh_document.version = 2; // Updated version
    
    // Reload should preserve uncommitted events
    aggregate.reload_from_repository(fresh_document)?;
    
    // Events should still be there
    assert_eq!(aggregate.uncommitted_events().len(), events_count);
    assert_eq!(aggregate.document().version, 2);
    
    Ok(())
}

#[tokio::test]
async fn test_project_aggregate_version_conflict_detection() -> Result<()> {
    let name = ProjectName::new("Test Project".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = ProjectAggregate::new(
        name,
        Some("Test description".to_string()),
        created_by,
    );
    
    // Check version conflict with expected version 1 (current)
    assert!(aggregate.check_version_conflict(1).is_ok());
    
    // Check version conflict with expected version 2 (should fail)
    assert!(aggregate.check_version_conflict(2).is_err());
    
    // Update name to increment version
    let new_name = ProjectName::new("Updated Project".to_string())?;
    aggregate.update_name(new_name, created_by)?;
    
    // Now version should be 2
    assert!(aggregate.check_version_conflict(2).is_ok());
    assert!(aggregate.check_version_conflict(1).is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_project_aggregate_reload_preserves_metadata() -> Result<()> {
    let name = ProjectName::new("Test Project".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = ProjectAggregate::new(
        name.clone(),
        Some("Test description".to_string()),
        created_by,
    );
    
    // Add document to generate metadata
    let document_id = EntityId::new();
    aggregate.add_document(document_id, "Test Doc".to_string(), created_by)?;
    
    // Should have document metadata
    assert!(!aggregate.document_metadata().is_empty());
    assert!(!aggregate.uncommitted_events().is_empty());
    
    let metadata_count = aggregate.document_metadata().len();
    let events_count = aggregate.uncommitted_events().len();
    
    // Create fresh project (simulating repo save)
    let mut fresh_project = Project::new(
        name.value.clone(),
        Some("Test description".to_string()),
        created_by,
    );
    fresh_project.id = *aggregate.project().id(); // Match the ID
    fresh_project.version = 2; // Updated version
    fresh_project.add_document(document_id, created_by);
    
    // Reload should preserve metadata and events
    aggregate.reload_from_repository(fresh_project)?;
    
    // Metadata and events should still be there
    assert_eq!(aggregate.document_metadata().len(), metadata_count);
    assert_eq!(aggregate.uncommitted_events().len(), events_count);
    assert_eq!(aggregate.project().version, 2);
    
    Ok(())
}

#[tokio::test]
async fn test_aggregate_reload_id_mismatch_error() -> Result<()> {
    let title = DocumentTitle::new("Test Document".to_string())?;
    let content = DocumentContent::new("Test content".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = DocumentAggregate::new(
        title.clone(),
        content.clone(),
        ContentType::Markdown,
        created_by,
    );
    
    // Create fresh document with different ID
    let fresh_document = Document::new(
        title.value.clone(),
        content.value.clone(),
        ContentType::Markdown,
        created_by,
    );
    // Don't match the ID - this should cause error
    
    // Reload should fail with ID mismatch
    let result = aggregate.reload_from_repository(fresh_document);
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("ID mismatch"));
    
    Ok(())
}

#[tokio::test] 
async fn test_document_aggregate_delete_without_timestamp_error() -> Result<()> {
    let title = DocumentTitle::new("Test Document".to_string())?;
    let content = DocumentContent::new("Test content".to_string())?;
    let created_by = Some(EntityId::new());
    
    let mut aggregate = DocumentAggregate::new(
        title,
        content,
        ContentType::Markdown,
        created_by,
    );
    
    // Manually corrupt the document state (should never happen in practice)
    // but this tests our error handling
    let mut corrupt_doc = aggregate.document().clone();
    corrupt_doc.is_deleted = true;
    corrupt_doc.deleted_at = None; // Missing timestamp!
    
    let mut corrupt_aggregate = DocumentAggregate::load_from_document(corrupt_doc);
    
    // Try to create delete event - this should handle the missing timestamp gracefully
    let result = corrupt_aggregate.delete(created_by);
    
    // Should fail because document is already marked deleted
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("already deleted"));
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_document_updates() -> Result<()> {
    let title = DocumentTitle::new("Concurrent Test".to_string())?;
    let content = DocumentContent::new("Initial content".to_string())?;
    let created_by = Some(EntityId::new());
    
    // Create initial aggregate
    let aggregate = DocumentAggregate::new(
        title,
        content,
        ContentType::Markdown,
        created_by,
    );
    
    // Simulate concurrent updates by creating multiple aggregates from same document
    let mut aggregate1 = DocumentAggregate::load_from_document(aggregate.document().clone());
    let mut aggregate2 = DocumentAggregate::load_from_document(aggregate.document().clone());
    
    // Both start at version 1
    assert!(aggregate1.check_version_conflict(1).is_ok());
    assert!(aggregate2.check_version_conflict(1).is_ok());
    
    // Update aggregate1
    let content1 = DocumentContent::new("Update from aggregate1".to_string())?;
    aggregate1.update_content(content1, None, created_by)?;
    
    // Now aggregate1 should be version 2
    assert!(aggregate1.check_version_conflict(2).is_ok());
    
    // aggregate2 should still think it's version 1, but actually it's stale
    // This would be detected when saving to repository
    assert!(aggregate2.check_version_conflict(1).is_ok()); // Local view
    
    // After repository reload, version conflicts would be detected
    // This simulates what happens in the service layer
    
    Ok(())
}