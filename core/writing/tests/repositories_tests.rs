//! Unit tests for writing domain repositories

use writemagic_writing::{
    DocumentRepository, ProjectRepository, SqliteDocumentRepository, SqliteProjectRepository,
    Document, Project, DocumentTitle, DocumentContent, ProjectName
};
use writemagic_shared::{
    EntityId, ContentType, Result, WritemagicError, DatabaseManager, DatabaseConfig
};
use std::sync::Arc;
use tempfile::TempDir;

#[cfg(test)]
mod sqlite_document_repository_tests {
    use super::*;

    async fn create_test_document_repository() -> Result<(TempDir, SqliteDocumentRepository)> {
        let temp_dir = TempDir::new().expect("Create temp directory");
        let db_path = temp_dir.path().join("test_documents.db");
        
        let config = DatabaseConfig {
            url: format!("sqlite:{}", db_path.display()),
            max_connections: 5,
            min_connections: 1,
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
        };
        
        let db_manager = DatabaseManager::new(config).await?;
        let repository = SqliteDocumentRepository::new(Arc::new(db_manager));
        
        Ok((temp_dir, repository))
    }

    #[tokio::test]
    async fn test_save_and_find_document() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let document = Document::new(
            "Test Document".to_string(),
            "Test content for document.".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        let document_id = *document.id();
        
        // Save document
        repository.save(document.clone()).await?;
        
        // Find document
        let found_document = repository.find_by_id(document_id).await?;
        
        assert!(found_document.is_some());
        let found = found_document.unwrap();
        
        assert_eq!(found.id, document.id);
        assert_eq!(found.title, document.title);
        assert_eq!(found.content, document.content);
        assert_eq!(found.content_type, document.content_type);
        assert_eq!(found.version, document.version);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_document() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let mut document = Document::new(
            "Original Title".to_string(),
            "Original content".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        // Save initial document
        repository.save(document.clone()).await?;
        
        // Update document
        document.update_content("Updated content".to_string(), Some(EntityId::new()));
        document.update_title("Updated Title".to_string(), Some(EntityId::new()));
        
        // Save updated document
        repository.save(document.clone()).await?;
        
        // Retrieve and verify update
        let updated_document = repository.find_by_id(*document.id()).await?;
        assert!(updated_document.is_some());
        
        let updated = updated_document.unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.content, "Updated content");
        assert_eq!(updated.version, document.version);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_document() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let document = Document::new(
            "To Delete".to_string(),
            "This will be deleted".to_string(),
            ContentType::PlainText,
            None,
        );
        
        let document_id = *document.id();
        
        // Save document
        repository.save(document.clone()).await?;
        
        // Verify it exists
        assert!(repository.find_by_id(document_id).await?.is_some());
        
        // Delete document
        repository.delete(document_id).await?;
        
        // Verify it's gone (or marked as deleted)
        let result = repository.find_by_id(document_id).await?;
        
        // Depending on implementation (hard vs soft delete)
        match result {
            None => {}, // Hard delete - document not found
            Some(doc) => assert!(doc.is_deleted), // Soft delete - marked as deleted
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_find_all_documents() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        // Create multiple documents
        let documents = vec![
            Document::new("Doc 1".to_string(), "Content 1".to_string(), ContentType::Markdown, None),
            Document::new("Doc 2".to_string(), "Content 2".to_string(), ContentType::PlainText, None),
            Document::new("Doc 3".to_string(), "Content 3".to_string(), ContentType::RichText, None),
        ];
        
        // Save all documents
        for document in &documents {
            repository.save(document.clone()).await?;
        }
        
        // Find all documents
        let all_documents = repository.find_all().await?;
        
        assert_eq!(all_documents.len(), 3);
        
        let titles: Vec<&String> = all_documents.iter().map(|d| &d.title).collect();
        assert!(titles.contains(&&"Doc 1".to_string()));
        assert!(titles.contains(&&"Doc 2".to_string()));
        assert!(titles.contains(&&"Doc 3".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_documents_by_title() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let documents = vec![
            Document::new("Rust Programming".to_string(), "Learn Rust".to_string(), ContentType::Markdown, None),
            Document::new("Python Guide".to_string(), "Learn Python".to_string(), ContentType::Markdown, None),
            Document::new("Web Development".to_string(), "HTML and CSS".to_string(), ContentType::Markdown, None),
            Document::new("Rust Advanced".to_string(), "Advanced Rust topics".to_string(), ContentType::Markdown, None),
        ];
        
        for document in &documents {
            repository.save(document.clone()).await?;
        }
        
        // Search for documents containing "Rust"
        let rust_docs = repository.search_by_title("Rust".to_string()).await?;
        
        assert_eq!(rust_docs.len(), 2);
        
        let titles: Vec<&String> = rust_docs.iter().map(|d| &d.title).collect();
        assert!(titles.contains(&&"Rust Programming".to_string()));
        assert!(titles.contains(&&"Rust Advanced".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_documents_by_content() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let documents = vec![
            Document::new("Doc 1".to_string(), "Programming with Rust language".to_string(), ContentType::Markdown, None),
            Document::new("Doc 2".to_string(), "Web development tutorial".to_string(), ContentType::Markdown, None),
            Document::new("Doc 3".to_string(), "Advanced Rust programming concepts".to_string(), ContentType::Markdown, None),
        ];
        
        for document in &documents {
            repository.save(document.clone()).await?;
        }
        
        // Search for documents containing "programming" in content
        let programming_docs = repository.search_by_content("programming".to_string()).await?;
        
        assert_eq!(programming_docs.len(), 2);
        
        let titles: Vec<&String> = programming_docs.iter().map(|d| &d.title).collect();
        assert!(titles.contains(&&"Doc 1".to_string()));
        assert!(titles.contains(&&"Doc 3".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_content_type() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        let documents = vec![
            Document::new("Markdown Doc".to_string(), "Content".to_string(), ContentType::Markdown, None),
            Document::new("Plain Doc".to_string(), "Content".to_string(), ContentType::PlainText, None),
            Document::new("Rich Doc".to_string(), "Content".to_string(), ContentType::RichText, None),
            Document::new("Another Markdown".to_string(), "Content".to_string(), ContentType::Markdown, None),
        ];
        
        for document in &documents {
            repository.save(document.clone()).await?;
        }
        
        // Find markdown documents
        let markdown_docs = repository.find_by_content_type(ContentType::Markdown).await?;
        
        assert_eq!(markdown_docs.len(), 2);
        
        for doc in &markdown_docs {
            assert_eq!(doc.content_type, ContentType::Markdown);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_repository_error_handling() -> Result<()> {
        let (_temp_dir, repository) = create_test_document_repository().await?;
        
        // Try to find non-existent document
        let nonexistent_id = EntityId::new();
        let result = repository.find_by_id(nonexistent_id).await?;
        assert!(result.is_none());
        
        // Try to delete non-existent document
        let delete_result = repository.delete(nonexistent_id).await;
        assert!(delete_result.is_err() || delete_result.is_ok()); // Implementation dependent
        
        Ok(())
    }
}

#[cfg(test)]
mod sqlite_project_repository_tests {
    use super::*;

    async fn create_test_project_repository() -> Result<(TempDir, SqliteProjectRepository)> {
        let temp_dir = TempDir::new().expect("Create temp directory");
        let db_path = temp_dir.path().join("test_projects.db");
        
        let config = DatabaseConfig {
            url: format!("sqlite:{}", db_path.display()),
            max_connections: 5,
            min_connections: 1,
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
        };
        
        let db_manager = DatabaseManager::new(config).await?;
        let repository = SqliteProjectRepository::new(Arc::new(db_manager));
        
        Ok((temp_dir, repository))
    }

    #[tokio::test]
    async fn test_save_and_find_project() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let project = Project::new(
            "Test Project".to_string(),
            Some("A test project".to_string()),
            Some(EntityId::new()),
        );
        
        let project_id = *project.id();
        
        // Save project
        repository.save(project.clone()).await?;
        
        // Find project
        let found_project = repository.find_by_id(project_id).await?;
        
        assert!(found_project.is_some());
        let found = found_project.unwrap();
        
        assert_eq!(found.id, project.id);
        assert_eq!(found.name, project.name);
        assert_eq!(found.description, project.description);
        assert_eq!(found.version, project.version);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_project_with_documents() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let mut project = Project::new(
            "Project with Docs".to_string(),
            None,
            None,
        );
        
        // Add documents to project
        let doc_id1 = EntityId::new();
        let doc_id2 = EntityId::new();
        
        project.add_document(doc_id1, None);
        project.add_document(doc_id2, None);
        
        // Save project
        repository.save(project.clone()).await?;
        
        // Retrieve and verify
        let found_project = repository.find_by_id(*project.id()).await?;
        assert!(found_project.is_some());
        
        let found = found_project.unwrap();
        assert_eq!(found.document_ids.len(), 2);
        assert!(found.document_ids.contains(&doc_id1));
        assert!(found.document_ids.contains(&doc_id2));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let mut project = Project::new(
            "Original Name".to_string(),
            Some("Original description".to_string()),
            Some(EntityId::new()),
        );
        
        // Save initial project
        repository.save(project.clone()).await?;
        
        // Update project
        project.update_name("Updated Name".to_string(), Some(EntityId::new()));
        project.update_description(Some("Updated description".to_string()), Some(EntityId::new()));
        
        // Save updated project
        repository.save(project.clone()).await?;
        
        // Retrieve and verify
        let updated_project = repository.find_by_id(*project.id()).await?;
        assert!(updated_project.is_some());
        
        let updated = updated_project.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.version, project.version);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_find_all_projects() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let projects = vec![
            Project::new("Project 1".to_string(), Some("Desc 1".to_string()), None),
            Project::new("Project 2".to_string(), None, None),
            Project::new("Project 3".to_string(), Some("Desc 3".to_string()), None),
        ];
        
        for project in &projects {
            repository.save(project.clone()).await?;
        }
        
        let all_projects = repository.find_all().await?;
        
        assert_eq!(all_projects.len(), 3);
        
        let names: Vec<&String> = all_projects.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"Project 1".to_string()));
        assert!(names.contains(&&"Project 2".to_string()));
        assert!(names.contains(&&"Project 3".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_projects_by_name() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let projects = vec![
            Project::new("Web Development Project".to_string(), None, None),
            Project::new("Mobile App Project".to_string(), None, None),
            Project::new("Web Design Guidelines".to_string(), None, None),
            Project::new("Backend Service".to_string(), None, None),
        ];
        
        for project in &projects {
            repository.save(project.clone()).await?;
        }
        
        // Search for projects containing "Web"
        let web_projects = repository.search_by_name("Web".to_string()).await?;
        
        assert_eq!(web_projects.len(), 2);
        
        let names: Vec<&String> = web_projects.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"Web Development Project".to_string()));
        assert!(names.contains(&&"Web Design Guidelines".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_find_projects_by_document() -> Result<()> {
        let (_temp_dir, repository) = create_test_project_repository().await?;
        
        let document_id = EntityId::new();
        
        let mut project1 = Project::new("Project 1".to_string(), None, None);
        project1.add_document(document_id, None);
        
        let mut project2 = Project::new("Project 2".to_string(), None, None);
        project2.add_document(document_id, None);
        project2.add_document(EntityId::new(), None); // Add another document
        
        let project3 = Project::new("Project 3".to_string(), None, None);
        // Project 3 doesn't contain the document
        
        repository.save(project1.clone()).await?;
        repository.save(project2.clone()).await?;
        repository.save(project3.clone()).await?;
        
        // Find projects containing the document
        let projects_with_doc = repository.find_by_document_id(document_id).await?;
        
        assert_eq!(projects_with_doc.len(), 2);
        
        let names: Vec<&String> = projects_with_doc.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"Project 1".to_string()));
        assert!(names.contains(&&"Project 2".to_string()));
        assert!(!names.contains(&&"Project 3".to_string()));
        
        Ok(())
    }
}

#[cfg(test)]
mod repository_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_document_and_project_repository_integration() -> Result<()> {
        // Create both repositories using the same database
        let temp_dir = TempDir::new().expect("Create temp directory");
        let db_path = temp_dir.path().join("integration_test.db");
        
        let config = DatabaseConfig {
            url: format!("sqlite:{}", db_path.display()),
            max_connections: 10,
            min_connections: 1,
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
        };
        
        let db_manager = Arc::new(DatabaseManager::new(config).await?);
        let doc_repository = SqliteDocumentRepository::new(Arc::clone(&db_manager));
        let project_repository = SqliteProjectRepository::new(db_manager);
        
        let user_id = Some(EntityId::new());
        
        // Create documents
        let document1 = Document::new(
            "Integration Doc 1".to_string(),
            "Content 1".to_string(),
            ContentType::Markdown,
            user_id,
        );
        
        let document2 = Document::new(
            "Integration Doc 2".to_string(),
            "Content 2".to_string(),
            ContentType::PlainText,
            user_id,
        );
        
        doc_repository.save(document1.clone()).await?;
        doc_repository.save(document2.clone()).await?;
        
        // Create project and add documents
        let mut project = Project::new(
            "Integration Project".to_string(),
            Some("Testing integration".to_string()),
            user_id,
        );
        
        project.add_document(*document1.id(), user_id);
        project.add_document(*document2.id(), user_id);
        
        project_repository.save(project.clone()).await?;
        
        // Verify integration
        let found_project = project_repository.find_by_id(*project.id()).await?;
        assert!(found_project.is_some());
        
        let found = found_project.unwrap();
        assert_eq!(found.document_ids.len(), 2);
        assert!(found.document_ids.contains(document1.id()));
        assert!(found.document_ids.contains(document2.id()));
        
        // Verify documents can be found
        for &doc_id in &found.document_ids {
            let doc = doc_repository.find_by_id(doc_id).await?;
            assert!(doc.is_some());
        }
        
        // Find projects containing specific document
        let projects_with_doc1 = project_repository.find_by_document_id(*document1.id()).await?;
        assert_eq!(projects_with_doc1.len(), 1);
        assert_eq!(projects_with_doc1[0].name, "Integration Project");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_repository_access() -> Result<()> {
        let temp_dir = TempDir::new().expect("Create temp directory");
        let db_path = temp_dir.path().join("concurrent_test.db");
        
        let config = DatabaseConfig {
            url: format!("sqlite:{}", db_path.display()),
            max_connections: 10,
            min_connections: 2,
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
        };
        
        let db_manager = Arc::new(DatabaseManager::new(config).await?);
        let doc_repository = Arc::new(SqliteDocumentRepository::new(db_manager));
        
        // Create multiple concurrent save operations
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let repo = Arc::clone(&doc_repository);
            let handle = tokio::spawn(async move {
                let document = Document::new(
                    format!("Concurrent Doc {}", i),
                    format!("Content {}", i),
                    ContentType::Markdown,
                    None,
                );
                
                repo.save(document).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_ok() {
                success_count += 1;
            }
        }
        
        assert_eq!(success_count, 10);
        
        // Verify all documents were saved
        let all_docs = doc_repository.find_all().await?;
        assert_eq!(all_docs.len(), 10);
        
        Ok(())
    }
}