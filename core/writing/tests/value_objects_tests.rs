//! Unit tests for writing domain value objects

use writemagic_writing::value_objects::*;
use writemagic_shared::{Result, WritemagicError};

#[cfg(test)]
mod document_title_tests {
    use super::*;

    #[test]
    fn test_document_title_creation() -> Result<()> {
        let title = DocumentTitle::new("Valid Document Title".to_string())?;
        assert_eq!(title.value, "Valid Document Title");
        Ok(())
    }

    #[test]
    fn test_document_title_empty_invalid() {
        let result = DocumentTitle::new("".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_document_title_whitespace_only_invalid() {
        let result = DocumentTitle::new("   \t\n  ".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_document_title_too_long_invalid() {
        let long_title = "a".repeat(501); // Assuming max length is 500
        let result = DocumentTitle::new(long_title);
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("too long"));
    }

    #[test]
    fn test_document_title_trimming() -> Result<()> {
        let title = DocumentTitle::new("  Trimmed Title  ".to_string())?;
        assert_eq!(title.value, "Trimmed Title");
        Ok(())
    }

    #[test]
    fn test_document_title_equality() -> Result<()> {
        let title1 = DocumentTitle::new("Same Title".to_string())?;
        let title2 = DocumentTitle::new("Same Title".to_string())?;
        let title3 = DocumentTitle::new("Different Title".to_string())?;
        
        assert_eq!(title1, title2);
        assert_ne!(title1, title3);
        Ok(())
    }

    #[test]
    fn test_document_title_serialization() -> Result<()> {
        let title = DocumentTitle::new("Serialization Test".to_string())?;
        
        let serialized = serde_json::to_string(&title).expect("Serialize title");
        let deserialized: DocumentTitle = serde_json::from_str(&serialized).expect("Deserialize title");
        
        assert_eq!(title, deserialized);
        Ok(())
    }
}

#[cfg(test)]
mod document_content_tests {
    use super::*;

    #[test]
    fn test_document_content_creation() -> Result<()> {
        let content = DocumentContent::new("This is valid document content.".to_string())?;
        assert_eq!(content.value, "This is valid document content.");
        Ok(())
    }

    #[test]
    fn test_document_content_empty_valid() -> Result<()> {
        let content = DocumentContent::new("".to_string())?;
        assert_eq!(content.value, "");
        Ok(())
    }

    #[test]
    fn test_document_content_large_valid() -> Result<()> {
        let large_content = "a".repeat(1_000_000); // 1MB of content
        let content = DocumentContent::new(large_content.clone())?;
        assert_eq!(content.value, large_content);
        Ok(())
    }

    #[test]
    fn test_document_content_word_count() -> Result<()> {
        let content = DocumentContent::new("This has exactly five words.".to_string())?;
        assert_eq!(content.word_count(), 5);
        
        let empty_content = DocumentContent::new("".to_string())?;
        assert_eq!(empty_content.word_count(), 0);
        
        let whitespace_content = DocumentContent::new("   \t\n  ".to_string())?;
        assert_eq!(whitespace_content.word_count(), 0);
        Ok(())
    }

    #[test]
    fn test_document_content_character_count() -> Result<()> {
        let content = DocumentContent::new("Hello".to_string())?;
        assert_eq!(content.character_count(), 5);
        
        let content_with_spaces = DocumentContent::new("Hello World".to_string())?;
        assert_eq!(content_with_spaces.character_count(), 11);
        Ok(())
    }

    #[test]
    fn test_document_content_equality() -> Result<()> {
        let content1 = DocumentContent::new("Same content".to_string())?;
        let content2 = DocumentContent::new("Same content".to_string())?;
        let content3 = DocumentContent::new("Different content".to_string())?;
        
        assert_eq!(content1, content2);
        assert_ne!(content1, content3);
        Ok(())
    }

    #[test]
    fn test_document_content_serialization() -> Result<()> {
        let content = DocumentContent::new("Content for serialization test.".to_string())?;
        
        let serialized = serde_json::to_string(&content).expect("Serialize content");
        let deserialized: DocumentContent = serde_json::from_str(&serialized).expect("Deserialize content");
        
        assert_eq!(content, deserialized);
        Ok(())
    }
}

#[cfg(test)]
mod project_name_tests {
    use super::*;

    #[test]
    fn test_project_name_creation() -> Result<()> {
        let name = ProjectName::new("Valid Project Name".to_string())?;
        assert_eq!(name.value, "Valid Project Name");
        Ok(())
    }

    #[test]
    fn test_project_name_empty_invalid() {
        let result = ProjectName::new("".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_project_name_whitespace_only_invalid() {
        let result = ProjectName::new("   \t  ".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("empty"));
    }

    #[test]
    fn test_project_name_too_long_invalid() {
        let long_name = "a".repeat(201); // Assuming max length is 200
        let result = ProjectName::new(long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("too long"));
    }

    #[test]
    fn test_project_name_trimming() -> Result<()> {
        let name = ProjectName::new("  Trimmed Name  ".to_string())?;
        assert_eq!(name.value, "Trimmed Name");
        Ok(())
    }

    #[test]
    fn test_project_name_equality() -> Result<()> {
        let name1 = ProjectName::new("Same Name".to_string())?;
        let name2 = ProjectName::new("Same Name".to_string())?;
        let name3 = ProjectName::new("Different Name".to_string())?;
        
        assert_eq!(name1, name2);
        assert_ne!(name1, name3);
        Ok(())
    }

    #[test]
    fn test_project_name_serialization() -> Result<()> {
        let name = ProjectName::new("Serialization Test Project".to_string())?;
        
        let serialized = serde_json::to_string(&name).expect("Serialize name");
        let deserialized: ProjectName = serde_json::from_str(&serialized).expect("Deserialize name");
        
        assert_eq!(name, deserialized);
        Ok(())
    }
}

#[cfg(test)]
mod text_selection_tests {
    use super::*;

    #[test]
    fn test_text_selection_creation() -> Result<()> {
        let selection = TextSelection::new(10, 25)?;
        assert_eq!(selection.start, 10);
        assert_eq!(selection.end, 25);
        assert_eq!(selection.length(), 15);
        Ok(())
    }

    #[test]
    fn test_text_selection_zero_length() -> Result<()> {
        let selection = TextSelection::new(15, 15)?;
        assert_eq!(selection.start, 15);
        assert_eq!(selection.end, 15);
        assert_eq!(selection.length(), 0);
        Ok(())
    }

    #[test]
    fn test_text_selection_invalid_range() {
        let result = TextSelection::new(25, 10); // End before start
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("invalid"));
    }

    #[test]
    fn test_text_selection_contains_position() -> Result<()> {
        let selection = TextSelection::new(10, 20)?;
        
        assert!(!selection.contains_position(5));
        assert!(selection.contains_position(10)); // Start inclusive
        assert!(selection.contains_position(15)); // Middle
        assert!(!selection.contains_position(20)); // End exclusive
        assert!(!selection.contains_position(25));
        Ok(())
    }

    #[test]
    fn test_text_selection_overlaps() -> Result<()> {
        let selection1 = TextSelection::new(10, 20)?;
        let selection2 = TextSelection::new(15, 25)?; // Overlaps
        let selection3 = TextSelection::new(25, 30)?; // No overlap
        let selection4 = TextSelection::new(5, 15)?;  // Overlaps
        
        assert!(selection1.overlaps(&selection2));
        assert!(selection2.overlaps(&selection1)); // Symmetric
        assert!(!selection1.overlaps(&selection3));
        assert!(selection1.overlaps(&selection4));
        Ok(())
    }

    #[test]
    fn test_text_selection_merge() -> Result<()> {
        let selection1 = TextSelection::new(10, 20)?;
        let selection2 = TextSelection::new(15, 25)?;
        
        let merged = selection1.merge(&selection2)?;
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 25);
        Ok(())
    }

    #[test]
    fn test_text_selection_merge_non_overlapping_fails() -> Result<()> {
        let selection1 = TextSelection::new(10, 15)?;
        let selection2 = TextSelection::new(20, 25)?;
        
        let result = selection1.merge(&selection2);
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("overlap"));
        Ok(())
    }

    #[test]
    fn test_text_selection_equality() -> Result<()> {
        let selection1 = TextSelection::new(10, 20)?;
        let selection2 = TextSelection::new(10, 20)?;
        let selection3 = TextSelection::new(10, 25)?;
        
        assert_eq!(selection1, selection2);
        assert_ne!(selection1, selection3);
        Ok(())
    }

    #[test]
    fn test_text_selection_serialization() -> Result<()> {
        let selection = TextSelection::new(42, 84)?;
        
        let serialized = serde_json::to_string(&selection).expect("Serialize selection");
        let deserialized: TextSelection = serde_json::from_str(&serialized).expect("Deserialize selection");
        
        assert_eq!(selection, deserialized);
        Ok(())
    }
}

#[cfg(test)]
mod document_metadata_tests {
    use super::*;
    use writemagic_shared::{EntityId, Timestamp};

    #[test]
    fn test_document_metadata_creation() {
        let document_id = EntityId::new();
        let title = "Test Document";
        
        let metadata = DocumentMetadata::new(document_id, title.to_string());
        
        assert_eq!(metadata.document_id, document_id);
        assert_eq!(metadata.title, title);
        assert!(!metadata.is_deleted);
        assert!(metadata.deleted_at.is_none());
        assert!(metadata.created_at.0 <= Timestamp::now().0);
    }

    #[test]
    fn test_document_metadata_update_title() {
        let mut metadata = DocumentMetadata::new(
            EntityId::new(),
            "Original Title".to_string(),
        );
        
        let original_updated_at = metadata.updated_at.clone();
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        metadata.update_title("New Title".to_string());
        
        assert_eq!(metadata.title, "New Title");
        assert!(metadata.updated_at.0 > original_updated_at.0);
    }

    #[test]
    fn test_document_metadata_mark_deleted() {
        let mut metadata = DocumentMetadata::new(
            EntityId::new(),
            "Delete Test".to_string(),
        );
        
        assert!(!metadata.is_deleted);
        assert!(metadata.deleted_at.is_none());
        
        metadata.mark_deleted();
        
        assert!(metadata.is_deleted);
        assert!(metadata.deleted_at.is_some());
    }

    #[test]
    fn test_document_metadata_restore() {
        let mut metadata = DocumentMetadata::new(
            EntityId::new(),
            "Restore Test".to_string(),
        );
        
        metadata.mark_deleted();
        assert!(metadata.is_deleted);
        
        metadata.restore();
        
        assert!(!metadata.is_deleted);
        assert!(metadata.deleted_at.is_none());
    }

    #[test]
    fn test_document_metadata_serialization() {
        let metadata = DocumentMetadata::new(
            EntityId::new(),
            "Serialization Test".to_string(),
        );
        
        let serialized = serde_json::to_string(&metadata).expect("Serialize metadata");
        let deserialized: DocumentMetadata = serde_json::from_str(&serialized).expect("Deserialize metadata");
        
        assert_eq!(metadata.document_id, deserialized.document_id);
        assert_eq!(metadata.title, deserialized.title);
        assert_eq!(metadata.is_deleted, deserialized.is_deleted);
    }
}

#[cfg(test)]
mod value_object_validation_tests {
    use super::*;

    #[test]
    fn test_document_title_validation_edge_cases() {
        // Test minimum length (assuming minimum is 1 after trimming)
        assert!(DocumentTitle::new("A".to_string()).is_ok());
        
        // Test maximum length (assuming maximum is 500)
        let max_title = "a".repeat(500);
        assert!(DocumentTitle::new(max_title).is_ok());
        
        // Test unicode characters
        assert!(DocumentTitle::new("文档标题".to_string()).is_ok());
        assert!(DocumentTitle::new("Document Título".to_string()).is_ok());
        assert!(DocumentTitle::new("Документ заголовок".to_string()).is_ok());
    }

    #[test]
    fn test_project_name_validation_edge_cases() {
        // Test minimum length
        assert!(ProjectName::new("P".to_string()).is_ok());
        
        // Test maximum length (assuming maximum is 200)
        let max_name = "a".repeat(200);
        assert!(ProjectName::new(max_name).is_ok());
        
        // Test special characters
        assert!(ProjectName::new("Project-Name_123".to_string()).is_ok());
        assert!(ProjectName::new("Project (Version 2)".to_string()).is_ok());
    }

    #[test]
    fn test_text_selection_edge_cases() -> Result<()> {
        // Test at start of document
        let selection = TextSelection::new(0, 5)?;
        assert_eq!(selection.start, 0);
        assert_eq!(selection.end, 5);
        
        // Test large positions
        let selection = TextSelection::new(1000000, 1000100)?;
        assert_eq!(selection.length(), 100);
        
        Ok(())
    }

    #[test]
    fn test_document_content_special_characters() -> Result<()> {
        // Test content with various special characters
        let content = DocumentContent::new("Content with émojis 🚀 and special chars: @#$%^&*()".to_string())?;
        assert!(content.character_count() > 0);
        
        // Test content with line breaks
        let multiline_content = DocumentContent::new("Line 1\nLine 2\r\nLine 3".to_string())?;
        assert!(multiline_content.word_count() >= 6);
        
        // Test content with tabs and spaces
        let whitespace_content = DocumentContent::new("Word1\t\tWord2    Word3".to_string())?;
        assert_eq!(whitespace_content.word_count(), 3);
        
        Ok(())
    }
}

#[cfg(test)]
mod value_object_integration_tests {
    use super::*;

    #[test]
    fn test_value_objects_in_document_context() -> Result<()> {
        // Create value objects
        let title = DocumentTitle::new("Integration Test Document".to_string())?;
        let content = DocumentContent::new("This is content for integration testing.".to_string())?;
        let selection = TextSelection::new(8, 15)?; // Select "content"
        
        // Verify they work together
        assert_eq!(title.value, "Integration Test Document");
        assert_eq!(content.word_count(), 7);
        assert_eq!(selection.length(), 7);
        
        // Test that selection is within content bounds
        let content_length = content.character_count();
        assert!(selection.end <= content_length);
        
        Ok(())
    }

    #[test]
    fn test_value_objects_in_project_context() -> Result<()> {
        let project_name = ProjectName::new("Integration Test Project".to_string())?;
        let document_title = DocumentTitle::new("Project Document".to_string())?;
        
        // These would be used together in a project context
        assert_ne!(project_name.value, document_title.value);
        assert!(!project_name.value.is_empty());
        assert!(!document_title.value.is_empty());
        
        Ok(())
    }
}