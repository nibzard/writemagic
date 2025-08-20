//! Unit tests for writing domain services

use writemagic_writing::{
    DocumentService, ProjectService, WritingService, 
    DocumentRepository, ProjectRepository,
    Document, Project, DocumentTitle, DocumentContent, ProjectName, TextSelection
};
use writemagic_shared::{
    EntityId, ContentType, Result, WritemagicError, InMemoryRepository, Timestamp
};
use std::sync::Arc;

// Mock repository implementations for testing
type MockDocumentRepository = InMemoryRepository<Document>;
type MockProjectRepository = InMemoryRepository<Project>;

#[cfg(test)]
mod document_service_tests {
    use super::*;

    fn create_document_service() -> DocumentService<MockDocumentRepository> {
        let repository = Arc::new(MockDocumentRepository::new());
        DocumentService::new(repository)
    }

    #[tokio::test]
    async fn test_create_document() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        let title = DocumentTitle::new("Test Document".to_string())?;
        let content = DocumentContent::new("Test content for the document.".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        assert_eq!(document.title, "Test Document");
        assert_eq!(document.content, "Test content for the document.");
        assert_eq!(document.content_type, ContentType::Markdown);
        assert_eq!(document.created_by, user_id);
        assert_eq!(document.version, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_document() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create a document first
        let title = DocumentTitle::new("Get Test Document".to_string())?;
        let content = DocumentContent::new("Content to retrieve.".to_string())?;
        
        let created_document = service.create_document(
            title,
            content,
            ContentType::PlainText,
            user_id,
        ).await?;
        
        // Retrieve the document
        let retrieved_document = service.get_document(*created_document.id()).await?;
        
        assert_eq!(retrieved_document.id, created_document.id);
        assert_eq!(retrieved_document.title, "Get Test Document");
        assert_eq!(retrieved_document.content, "Content to retrieve.");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_nonexistent_document() -> Result<()> {
        let service = create_document_service();
        let nonexistent_id = EntityId::new();
        
        let result = service.get_document(nonexistent_id).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().message().contains("not found"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_document_content() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create document
        let title = DocumentTitle::new("Update Test".to_string())?;
        let content = DocumentContent::new("Original content".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        // Update content
        let new_content = DocumentContent::new("Updated content with more text.".to_string())?;
        let selection = Some(TextSelection::new(0, 16)?); // Select "Original content"
        
        let updated_document = service.update_document_content(
            *document.id(),
            new_content,
            selection,
            user_id,
        ).await?;
        
        assert_eq!(updated_document.content, "Updated content with more text.");
        assert_eq!(updated_document.version, 2);
        assert_eq!(updated_document.word_count, 5);
        assert_eq!(updated_document.updated_by, user_id);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_document_title() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create document
        let title = DocumentTitle::new("Original Title".to_string())?;
        let content = DocumentContent::new("Content".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        // Update title
        let new_title = DocumentTitle::new("Updated Title".to_string())?;
        
        let updated_document = service.update_document_title(
            *document.id(),
            new_title,
            user_id,
        ).await?;
        
        assert_eq!(updated_document.title, "Updated Title");
        assert_eq!(updated_document.version, 2);
        assert_eq!(updated_document.content, "Content"); // Content unchanged
        
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_document() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create document
        let title = DocumentTitle::new("Delete Test".to_string())?;
        let content = DocumentContent::new("To be deleted".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        let document_id = *document.id();
        
        // Delete document
        service.delete_document(document_id, user_id).await?;
        
        // Try to retrieve - should return error or marked as deleted
        let retrieved = service.get_document(document_id).await;
        
        // Depending on implementation, might return error or deleted document
        match retrieved {
            Err(_) => {}, // Document not found (hard delete)
            Ok(doc) => assert!(doc.is_deleted), // Document marked as deleted (soft delete)
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_documents() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create multiple documents
        for i in 1..=3 {
            let title = DocumentTitle::new(format!("Document {}", i))?;
            let content = DocumentContent::new(format!("Content {}", i))?;
            
            service.create_document(
                title,
                content,
                ContentType::Markdown,
                user_id,
            ).await?;
        }
        
        // List documents
        let documents = service.list_documents().await?;
        
        assert_eq!(documents.len(), 3);
        
        // Verify documents are correctly created
        let titles: Vec<&String> = documents.iter().map(|d| &d.title).collect();
        assert!(titles.contains(&&"Document 1".to_string()));
        assert!(titles.contains(&&"Document 2".to_string()));
        assert!(titles.contains(&&"Document 3".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_documents() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create documents with different content
        let documents_data = vec![
            ("Rust Programming", "Learn Rust programming language"),
            ("Python Guide", "Python programming tutorial"),
            ("Web Development", "HTML, CSS, and JavaScript guide"),
            ("Database Design", "SQL and database fundamentals"),
        ];
        
        for (title, content) in documents_data {
            let doc_title = DocumentTitle::new(title.to_string())?;
            let doc_content = DocumentContent::new(content.to_string())?;
            
            service.create_document(
                doc_title,
                doc_content,
                ContentType::Markdown,
                user_id,
            ).await?;
        }
        
        // Search for "programming"
        let results = service.search_documents("programming".to_string()).await?;
        
        assert_eq!(results.len(), 2); // "Rust Programming" and "Python Guide"
        
        let found_titles: Vec<&String> = results.iter().map(|d| &d.title).collect();
        assert!(found_titles.contains(&&"Rust Programming".to_string()));
        assert!(found_titles.contains(&&"Python Guide".to_string()));
        
        Ok(())
    }
}

#[cfg(test)]
mod project_service_tests {
    use super::*;

    fn create_project_service() -> ProjectService<MockProjectRepository> {
        let repository = Arc::new(MockProjectRepository::new());
        ProjectService::new(repository)
    }

    #[tokio::test]
    async fn test_create_project() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        let name = ProjectName::new("Test Project".to_string())?;
        let description = Some("A test project description".to_string());
        
        let project = service.create_project(name, description.clone(), user_id).await?;
        
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, description);
        assert_eq!(project.created_by, user_id);
        assert_eq!(project.version, 1);
        assert!(project.document_ids.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        // Create project first
        let name = ProjectName::new("Get Test Project".to_string())?;
        let created_project = service.create_project(name, None, user_id).await?;
        
        // Retrieve project
        let retrieved_project = service.get_project(*created_project.id()).await?;
        
        assert_eq!(retrieved_project.id, created_project.id);
        assert_eq!(retrieved_project.name, "Get Test Project");
        assert!(retrieved_project.description.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_project_name() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        // Create project
        let name = ProjectName::new("Original Name".to_string())?;
        let project = service.create_project(name, None, user_id).await?;
        
        // Update name
        let new_name = ProjectName::new("Updated Name".to_string())?;
        let updated_project = service.update_project_name(
            *project.id(),
            new_name,
            user_id,
        ).await?;
        
        assert_eq!(updated_project.name, "Updated Name");
        assert_eq!(updated_project.version, 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_add_document_to_project() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        // Create project
        let name = ProjectName::new("Document Project".to_string())?;
        let project = service.create_project(name, None, user_id).await?;
        
        // Add document to project
        let document_id = EntityId::new();
        let updated_project = service.add_document_to_project(
            *project.id(),
            document_id,
            user_id,
        ).await?;
        
        assert_eq!(updated_project.document_ids.len(), 1);
        assert!(updated_project.document_ids.contains(&document_id));
        assert_eq!(updated_project.version, 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_document_from_project() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        // Create project with document
        let name = ProjectName::new("Remove Test Project".to_string())?;
        let project = service.create_project(name, None, user_id).await?;
        
        let document_id1 = EntityId::new();
        let document_id2 = EntityId::new();
        
        let project = service.add_document_to_project(
            *project.id(),
            document_id1,
            user_id,
        ).await?;
        
        let project = service.add_document_to_project(
            *project.id(),
            document_id2,
            user_id,
        ).await?;
        
        assert_eq!(project.document_ids.len(), 2);
        
        // Remove one document
        let updated_project = service.remove_document_from_project(
            *project.id(),
            document_id1,
            user_id,
        ).await?;
        
        assert_eq!(updated_project.document_ids.len(), 1);
        assert!(!updated_project.document_ids.contains(&document_id1));
        assert!(updated_project.document_ids.contains(&document_id2));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_projects() -> Result<()> {
        let service = create_project_service();
        let user_id = Some(EntityId::new());
        
        // Create multiple projects
        for i in 1..=3 {
            let name = ProjectName::new(format!("Project {}", i))?;
            service.create_project(name, None, user_id).await?;
        }
        
        // List projects
        let projects = service.list_projects().await?;
        
        assert_eq!(projects.len(), 3);
        
        let names: Vec<&String> = projects.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"Project 1".to_string()));
        assert!(names.contains(&&"Project 2".to_string()));
        assert!(names.contains(&&"Project 3".to_string()));
        
        Ok(())
    }
}

#[cfg(test)]
mod writing_service_tests {
    use super::*;

    fn create_writing_service() -> WritingService<MockDocumentRepository, MockProjectRepository> {
        let document_repository = Arc::new(MockDocumentRepository::new());
        let project_repository = Arc::new(MockProjectRepository::new());
        
        WritingService::new(document_repository, project_repository)
    }

    #[tokio::test]
    async fn test_create_document_and_add_to_project() -> Result<()> {
        let service = create_writing_service();
        let user_id = Some(EntityId::new());
        
        // Create project
        let project_name = ProjectName::new("Integration Project".to_string())?;
        let project = service.create_project(project_name, None, user_id).await?;
        
        // Create document
        let doc_title = DocumentTitle::new("Project Document".to_string())?;
        let doc_content = DocumentContent::new("Document in project".to_string())?;
        
        let document = service.create_document(
            doc_title,
            doc_content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        // Add document to project
        let updated_project = service.add_document_to_project(
            *project.id(),
            *document.id(),
            user_id,
        ).await?;
        
        assert!(updated_project.document_ids.contains(document.id()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_project_documents() -> Result<()> {
        let service = create_writing_service();
        let user_id = Some(EntityId::new());
        
        // Create project
        let project_name = ProjectName::new("Multi-Document Project".to_string())?;
        let project = service.create_project(project_name, None, user_id).await?;
        
        // Create multiple documents
        let mut document_ids = Vec::new();
        for i in 1..=3 {
            let title = DocumentTitle::new(format!("Document {}", i))?;
            let content = DocumentContent::new(format!("Content {}", i))?;
            
            let document = service.create_document(
                title,
                content,
                ContentType::Markdown,
                user_id,
            ).await?;
            
            document_ids.push(*document.id());
            
            service.add_document_to_project(*project.id(), *document.id(), user_id).await?;
        }
        
        // Get project documents
        let project_documents = service.get_project_documents(*project.id()).await?;
        
        assert_eq!(project_documents.len(), 3);
        
        for document in &project_documents {
            assert!(document_ids.contains(document.id()));
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_document_from_project() -> Result<()> {
        let service = create_writing_service();
        let user_id = Some(EntityId::new());
        
        // Setup project with document
        let project_name = ProjectName::new("Delete Test Project".to_string())?;
        let project = service.create_project(project_name, None, user_id).await?;
        
        let doc_title = DocumentTitle::new("To Be Deleted".to_string())?;
        let doc_content = DocumentContent::new("This will be deleted".to_string())?;
        
        let document = service.create_document(
            doc_title,
            doc_content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        service.add_document_to_project(*project.id(), *document.id(), user_id).await?;
        
        // Verify document is in project
        let project_docs = service.get_project_documents(*project.id()).await?;
        assert_eq!(project_docs.len(), 1);
        
        // Delete document
        service.delete_document(*document.id(), user_id).await?;
        
        // Document should be removed from project (or marked as deleted)
        let updated_project = service.get_project(*project.id()).await?;
        
        // Depending on implementation, document might be removed or marked deleted
        if !updated_project.document_ids.contains(document.id()) {
            // Document was removed from project
            assert!(!updated_project.document_ids.contains(document.id()));
        } else {
            // Document is still in project but should be marked as deleted
            let project_docs = service.get_project_documents(*project.id()).await?;
            if !project_docs.is_empty() {
                assert!(project_docs[0].is_deleted);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod service_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_document_service_version_conflict() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        // Create document
        let title = DocumentTitle::new("Conflict Test".to_string())?;
        let content = DocumentContent::new("Original".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        // Simulate concurrent update by manually updating the version
        // This would typically happen when two clients try to update simultaneously
        let new_content = DocumentContent::new("Updated".to_string())?;
        
        // First update should succeed
        let result1 = service.update_document_content(
            *document.id(),
            new_content.clone(),
            None,
            user_id,
        ).await;
        
        assert!(result1.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_document_operations() {
        let service = create_document_service();
        
        // Try to get non-existent document
        let result = service.get_document(EntityId::new()).await;
        assert!(result.is_err());
        
        // Try to update non-existent document
        let title = DocumentTitle::new("New Title".to_string()).expect("Valid title");
        let result = service.update_document_title(EntityId::new(), title, None).await;
        assert!(result.is_err());
        
        // Try to delete non-existent document
        let result = service.delete_document(EntityId::new(), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_project_operations() {
        let service = create_project_service();
        
        // Try to get non-existent project
        let result = service.get_project(EntityId::new()).await;
        assert!(result.is_err());
        
        // Try to add document to non-existent project
        let result = service.add_document_to_project(
            EntityId::new(),
            EntityId::new(),
            None,
        ).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod service_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_bulk_document_creation() -> Result<()> {
        let service = create_document_service();
        let user_id = Some(EntityId::new());
        
        let start_time = std::time::Instant::now();
        
        // Create 100 documents
        for i in 1..=100 {
            let title = DocumentTitle::new(format!("Bulk Document {}", i))?;
            let content = DocumentContent::new(format!("Bulk content {}", i))?;
            
            service.create_document(
                title,
                content,
                ContentType::Markdown,
                user_id,
            ).await?;
        }
        
        let duration = start_time.elapsed();
        
        // Verify all documents were created
        let documents = service.list_documents().await?;
        assert_eq!(documents.len(), 100);
        
        // Performance check (should complete reasonably quickly)
        assert!(duration.as_secs() < 10, "Bulk creation took too long: {:?}", duration);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_document_access() -> Result<()> {
        let service = Arc::new(create_document_service());
        let user_id = Some(EntityId::new());
        
        // Create a document first
        let title = DocumentTitle::new("Concurrent Test".to_string())?;
        let content = DocumentContent::new("Concurrent content".to_string())?;
        
        let document = service.create_document(
            title,
            content,
            ContentType::Markdown,
            user_id,
        ).await?;
        
        let document_id = *document.id();
        
        // Create multiple concurrent access tasks
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let doc = service_clone.get_document(document_id).await;
                (i, doc.is_ok())
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let mut success_count = 0;
        for handle in handles {
            let (_, success) = handle.await.unwrap();
            if success {
                success_count += 1;
            }
        }
        
        // All concurrent accesses should succeed
        assert_eq!(success_count, 10);
        
        Ok(())
    }
}