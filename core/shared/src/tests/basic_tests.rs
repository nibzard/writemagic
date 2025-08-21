//! Basic functionality tests for the shared library

use crate::{EntityId, Timestamp, ContentHash, FilePath, ContentType};

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_entity_id_creation() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();
        
        // IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id1.as_uuid(), id2.as_uuid());
    }

    #[test]
    fn test_entity_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = EntityId::from_string(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_timestamp_creation() {
        let ts1 = Timestamp::now();
        let ts2 = Timestamp::now();
        
        // Timestamps should be close but potentially different
        assert!(ts2.as_datetime() >= ts1.as_datetime());
    }

    #[test]
    fn test_timestamp_from_datetime() {
        use chrono::{DateTime, Utc};
        let dt: DateTime<Utc> = "2023-01-01T12:00:00Z".parse().unwrap();
        let ts = Timestamp::from_datetime(dt);
        assert_eq!(ts.as_datetime(), dt);
    }

    #[test]
    fn test_content_hash_creation() {
        let hash1 = ContentHash::new("test content");
        let hash2 = ContentHash::new("test content");
        let hash3 = ContentHash::new("different content");
        
        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_file_path_creation() {
        let path = FilePath::new("/path/to/file.txt").unwrap();
        assert_eq!(path.as_str(), "/path/to/file.txt");
        
        // Test file name and extension using std::path::Path
        let std_path = std::path::Path::new(path.as_str());
        let file_name = std_path.file_name().unwrap().to_str().unwrap();
        assert_eq!(file_name, "file.txt");
        
        let extension = std_path.extension().unwrap().to_str().unwrap();
        assert_eq!(extension, "txt");
    }

    #[test]
    fn test_content_type_creation() {
        let ct1 = ContentType::from_string("plain_text").unwrap();
        assert_eq!(ct1.to_string(), "plain_text");
        
        let ct2 = ContentType::from_string("json").unwrap();
        assert_eq!(ct2.to_string(), "json");
        
        let ct3 = ContentType::from_string("html").unwrap();
        assert_eq!(ct3.to_string(), "html");
    }

    #[test]
    fn test_content_type_markdown() {
        let ct = ContentType::Markdown;
        assert_eq!(ct.to_string(), "markdown");
    }

    #[test]
    fn test_entity_id_default() {
        let id = EntityId::default();
        // Default should create a new ID
        assert_ne!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }
}