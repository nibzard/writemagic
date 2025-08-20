//! Unit tests for writing domain entities

use writemagic_writing::{
    Document, Project, entities::*
};
use writemagic_shared::{
    EntityId, ContentType, Timestamp, FilePath, ContentHash, Result
};

#[cfg(test)]
mod document_tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let created_by = Some(EntityId::new());
        let document = Document::new(
            "Test Document".to_string(),
            "This is test content.".to_string(),
            ContentType::Markdown,
            created_by,
        );
        
        assert_eq!(document.title, "Test Document");
        assert_eq!(document.content, "This is test content.");
        assert_eq!(document.content_type, ContentType::Markdown);
        assert_eq!(document.created_by, created_by);
        assert_eq!(document.updated_by, created_by);
        assert_eq!(document.version, 1);
        assert!(!document.is_deleted);
        assert!(document.deleted_at.is_none());
    }

    #[test]
    fn test_document_word_count() {
        let document = Document::new(
            "Word Count Test".to_string(),
            "This is a test with exactly eight words here.".to_string(),
            ContentType::PlainText,
            None,
        );
        
        assert_eq!(document.word_count, 9); // "This is a test with exactly eight words here."
        assert_eq!(document.character_count, 45);
    }

    #[test]
    fn test_document_content_hash() {
        let content = "Content for hashing";
        let document1 = Document::new(
            "Test 1".to_string(),
            content.to_string(),
            ContentType::Markdown,
            None,
        );
        let document2 = Document::new(
            "Test 2".to_string(),
            content.to_string(),
            ContentType::PlainText,
            None,
        );
        
        // Same content should produce same hash regardless of title or content type
        assert_eq!(document1.content_hash, document2.content_hash);
        
        // Different content should produce different hash
        let document3 = Document::new(
            "Test 3".to_string(),
            "Different content".to_string(),
            ContentType::Markdown,
            None,
        );
        
        assert_ne!(document1.content_hash, document3.content_hash);
    }

    #[test]
    fn test_document_update_content() {
        let mut document = Document::new(
            "Test Document".to_string(),
            "Original content".to_string(),
            ContentType::Markdown,
            None,
        );
        
        let original_hash = document.content_hash.clone();
        let original_updated_at = document.updated_at.clone();
        
        // Update content
        std::thread::sleep(std::time::Duration::from_millis(1)); // Ensure timestamp difference
        document.update_content("Updated content".to_string(), Some(EntityId::new()));
        
        assert_eq!(document.content, "Updated content");
        assert_ne!(document.content_hash, original_hash);
        assert!(document.updated_at.0 > original_updated_at.0);
        assert_eq!(document.version, 2);
        assert_eq!(document.word_count, 2);
        assert_eq!(document.character_count, 15);
    }

    #[test]
    fn test_document_update_title() {
        let mut document = Document::new(
            "Original Title".to_string(),
            "Content".to_string(),
            ContentType::Markdown,
            None,
        );
        
        let original_updated_at = document.updated_at.clone();
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        document.update_title("New Title".to_string(), Some(EntityId::new()));
        
        assert_eq!(document.title, "New Title");
        assert_eq!(document.version, 2);
        assert!(document.updated_at.0 > original_updated_at.0);
    }

    #[test]
    fn test_document_set_file_path() -> Result<()> {
        let mut document = Document::new(
            "File Document".to_string(),
            "File content".to_string(),
            ContentType::Markdown,
            None,
        );
        
        assert!(document.file_path.is_none());
        
        let file_path = FilePath::new("/path/to/document.md")?;
        document.set_file_path(Some(file_path.clone()));
        
        assert!(document.file_path.is_some());
        assert_eq!(document.file_path.unwrap().as_str(), "/path/to/document.md");
        
        Ok(())
    }

    #[test]
    fn test_document_mark_deleted() {
        let mut document = Document::new(
            "Delete Test".to_string(),
            "To be deleted".to_string(),
            ContentType::Markdown,
            None,
        );
        
        assert!(!document.is_deleted);
        assert!(document.deleted_at.is_none());
        
        document.mark_deleted(Some(EntityId::new()));
        
        assert!(document.is_deleted);
        assert!(document.deleted_at.is_some());
    }

    #[test]
    fn test_document_restore() {
        let mut document = Document::new(
            "Restore Test".to_string(),
            "To be restored".to_string(),
            ContentType::Markdown,
            None,
        );
        
        // First mark as deleted
        document.mark_deleted(Some(EntityId::new()));
        assert!(document.is_deleted);
        
        // Then restore
        document.restore();
        
        assert!(!document.is_deleted);
        assert!(document.deleted_at.is_none());
    }

    #[test]
    fn test_document_serialization() {
        let document = Document::new(
            "Serialization Test".to_string(),
            "Content to serialize".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        let serialized = serde_json::to_string(&document).expect("Serialize document");
        let deserialized: Document = serde_json::from_str(&serialized).expect("Deserialize document");
        
        assert_eq!(document.id, deserialized.id);
        assert_eq!(document.title, deserialized.title);
        assert_eq!(document.content, deserialized.content);
        assert_eq!(document.content_type, deserialized.content_type);
        assert_eq!(document.version, deserialized.version);
    }

    #[test]
    fn test_document_empty_content() {
        let document = Document::new(
            "Empty Content Test".to_string(),
            "".to_string(),
            ContentType::PlainText,
            None,
        );
        
        assert_eq!(document.word_count, 0);
        assert_eq!(document.character_count, 0);
        assert_eq!(document.content_hash, ContentHash::new(""));
    }

    #[test]
    fn test_document_whitespace_content() {
        let document = Document::new(
            "Whitespace Test".to_string(),
            "   \t\n  ".to_string(),
            ContentType::PlainText,
            None,
        );
        
        assert_eq!(document.word_count, 0); // Only whitespace
        assert_eq!(document.character_count, 7); // Including whitespace characters
    }
}

#[cfg(test)]
mod project_tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let created_by = Some(EntityId::new());
        let project = Project::new(
            "Test Project".to_string(),
            Some("A test project description".to_string()),
            created_by,
        );
        
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("A test project description".to_string()));
        assert_eq!(project.created_by, created_by);
        assert_eq!(project.updated_by, created_by);
        assert_eq!(project.version, 1);
        assert!(project.document_ids.is_empty());
        assert!(!project.is_deleted);
    }

    #[test]
    fn test_project_without_description() {
        let project = Project::new(
            "Simple Project".to_string(),
            None,
            None,
        );
        
        assert_eq!(project.name, "Simple Project");
        assert!(project.description.is_none());
    }

    #[test]
    fn test_project_add_document() {
        let mut project = Project::new(
            "Document Project".to_string(),
            None,
            None,
        );
        
        let document_id = EntityId::new();
        let user_id = Some(EntityId::new());
        
        project.add_document(document_id, user_id);
        
        assert_eq!(project.document_ids.len(), 1);
        assert!(project.document_ids.contains(&document_id));
        assert_eq!(project.version, 2);
        assert_eq!(project.updated_by, user_id);
    }

    #[test]
    fn test_project_add_duplicate_document() {
        let mut project = Project::new(
            "Duplicate Test".to_string(),
            None,
            None,
        );
        
        let document_id = EntityId::new();
        
        project.add_document(document_id, None);
        project.add_document(document_id, None); // Try to add same document again
        
        // Should only have one instance
        assert_eq!(project.document_ids.len(), 1);
    }

    #[test]
    fn test_project_remove_document() {
        let mut project = Project::new(
            "Remove Test".to_string(),
            None,
            None,
        );
        
        let document_id1 = EntityId::new();
        let document_id2 = EntityId::new();
        let user_id = Some(EntityId::new());
        
        project.add_document(document_id1, user_id);
        project.add_document(document_id2, user_id);
        
        assert_eq!(project.document_ids.len(), 2);
        
        project.remove_document(document_id1, user_id);
        
        assert_eq!(project.document_ids.len(), 1);
        assert!(!project.document_ids.contains(&document_id1));
        assert!(project.document_ids.contains(&document_id2));
    }

    #[test]
    fn test_project_remove_nonexistent_document() {
        let mut project = Project::new(
            "Remove Nonexistent".to_string(),
            None,
            None,
        );
        
        let document_id = EntityId::new();
        let nonexistent_id = EntityId::new();
        
        project.add_document(document_id, None);
        let initial_version = project.version;
        
        project.remove_document(nonexistent_id, None);
        
        // Should not change anything
        assert_eq!(project.document_ids.len(), 1);
        assert_eq!(project.version, initial_version);
    }

    #[test]
    fn test_project_update_name() {
        let mut project = Project::new(
            "Original Name".to_string(),
            None,
            None,
        );
        
        let original_updated_at = project.updated_at.clone();
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        project.update_name("New Name".to_string(), Some(EntityId::new()));
        
        assert_eq!(project.name, "New Name");
        assert_eq!(project.version, 2);
        assert!(project.updated_at.0 > original_updated_at.0);
    }

    #[test]
    fn test_project_update_description() {
        let mut project = Project::new(
            "Description Test".to_string(),
            None,
            None,
        );
        
        project.update_description(Some("New description".to_string()), Some(EntityId::new()));
        
        assert_eq!(project.description, Some("New description".to_string()));
        assert_eq!(project.version, 2);
        
        // Remove description
        project.update_description(None, Some(EntityId::new()));
        
        assert!(project.description.is_none());
        assert_eq!(project.version, 3);
    }

    #[test]
    fn test_project_mark_deleted() {
        let mut project = Project::new(
            "Delete Test".to_string(),
            None,
            None,
        );
        
        assert!(!project.is_deleted);
        assert!(project.deleted_at.is_none());
        
        project.mark_deleted(Some(EntityId::new()));
        
        assert!(project.is_deleted);
        assert!(project.deleted_at.is_some());
    }

    #[test]
    fn test_project_serialization() {
        let mut project = Project::new(
            "Serialization Test".to_string(),
            Some("Test description".to_string()),
            Some(EntityId::new()),
        );
        
        project.add_document(EntityId::new(), None);
        
        let serialized = serde_json::to_string(&project).expect("Serialize project");
        let deserialized: Project = serde_json::from_str(&serialized).expect("Deserialize project");
        
        assert_eq!(project.id, deserialized.id);
        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(project.document_ids, deserialized.document_ids);
        assert_eq!(project.version, deserialized.version);
    }
}

#[cfg(test)]
mod entity_trait_tests {
    use super::*;
    use writemagic_shared::{Entity, AggregateRoot, Auditable, Versioned};

    #[test]
    fn test_document_entity_trait() {
        let document = Document::new(
            "Entity Test".to_string(),
            "Test content".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        // Test Entity trait
        assert_ne!(document.id(), &EntityId::new());
        
        // Test Auditable trait
        assert!(document.created_at().0 <= Timestamp::now().0);
        assert_eq!(document.created_at(), document.updated_at());
        
        // Test Versioned trait
        assert_eq!(document.version(), 1);
    }

    #[test]
    fn test_project_entity_trait() {
        let project = Project::new(
            "Entity Test Project".to_string(),
            None,
            Some(EntityId::new()),
        );
        
        // Test Entity trait
        assert_ne!(project.id(), &EntityId::new());
        
        // Test Auditable trait
        assert!(project.created_at().0 <= Timestamp::now().0);
        assert_eq!(project.created_at(), project.updated_at());
        
        // Test Versioned trait
        assert_eq!(project.version(), 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_document_project_integration() {
        let user_id = Some(EntityId::new());
        
        // Create a project
        let mut project = Project::new(
            "Integration Test Project".to_string(),
            Some("Testing document-project integration".to_string()),
            user_id,
        );
        
        // Create some documents
        let document1 = Document::new(
            "First Document".to_string(),
            "Content of first document".to_string(),
            ContentType::Markdown,
            user_id,
        );
        
        let document2 = Document::new(
            "Second Document".to_string(),
            "Content of second document".to_string(),
            ContentType::PlainText,
            user_id,
        );
        
        // Add documents to project
        project.add_document(*document1.id(), user_id);
        project.add_document(*document2.id(), user_id);
        
        assert_eq!(project.document_ids.len(), 2);
        assert!(project.document_ids.contains(document1.id()));
        assert!(project.document_ids.contains(document2.id()));
        assert_eq!(project.version, 3); // Initial + 2 document additions
    }

    #[test]
    fn test_document_lifecycle() {
        let user_id = Some(EntityId::new());
        
        // Create document
        let mut document = Document::new(
            "Lifecycle Test".to_string(),
            "Initial content".to_string(),
            ContentType::Markdown,
            user_id,
        );
        
        let initial_version = document.version;
        
        // Update content
        document.update_content("Updated content".to_string(), user_id);
        assert_eq!(document.version, initial_version + 1);
        
        // Update title
        document.update_title("Updated Title".to_string(), user_id);
        assert_eq!(document.version, initial_version + 2);
        
        // Mark deleted
        document.mark_deleted(user_id);
        assert!(document.is_deleted);
        
        // Restore
        document.restore();
        assert!(!document.is_deleted);
        
        assert_eq!(document.title, "Updated Title");
        assert_eq!(document.content, "Updated content");
    }
}