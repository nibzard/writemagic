use serde_json::json;

use crate::common::{assertions::*, TestApp};

#[tokio::test]
async fn test_create_document_success() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let create_request = json!({
        "title": "Test Document",
        "description": "A test document",
        "content": "This is the content of the test document.",
        "tags": ["test", "example"]
    });

    let response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    assert_status(&response, 201);
    assert_json_field(&response, "id");
    assert_json_field(&response, "title");
    assert_json_field(&response, "content");
    assert_json_field(&response, "tags");
    assert_json_field(&response, "created_at");
    assert_json_field(&response, "updated_at");
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["title"], "Test Document");
    assert_eq!(json_response["description"], "A test document");
    assert_eq!(json_response["content"], "This is the content of the test document.");
    assert_eq!(json_response["tags"], json!(["test", "example"]));
}

#[tokio::test]
async fn test_create_document_minimal() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let create_request = json!({
        "title": "Minimal Document"
    });

    let response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    assert_status(&response, 201);
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["title"], "Minimal Document");
    assert!(json_response["description"].is_null());
    assert!(json_response["content"].is_null());
    assert_eq!(json_response["tags"], json!([]));
}

#[tokio::test]
async fn test_create_document_unauthorized() {
    let app = TestApp::new().await;

    let create_request = json!({
        "title": "Unauthorized Document"
    });

    let response = app.server
        .post("/api/v1/documents")
        .json(&create_request)
        .await;

    assert_auth_error(&response);
}

#[tokio::test]
async fn test_create_document_validation_errors() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Test empty title
    let create_request = json!({
        "title": ""
    });

    let response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    assert_validation_error(&response);

    // Test too many tags
    let create_request = json!({
        "title": "Test Document",
        "tags": (0..15).map(|i| format!("tag{}", i)).collect::<Vec<_>>()
    });

    let response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    assert_validation_error(&response);
}

#[tokio::test]
async fn test_get_document_success() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create document
    let create_request = json!({
        "title": "Test Document",
        "content": "Document content"
    });

    let create_response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    let created_doc: serde_json::Value = create_response.json();
    let document_id = created_doc["id"].as_str().unwrap();

    // Get document
    let response = app.server
        .get(&format!("/api/v1/documents/{}", document_id))
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&response);
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["id"], document_id);
    assert_eq!(json_response["title"], "Test Document");
    assert_eq!(json_response["content"], "Document content");
}

#[tokio::test]
async fn test_get_document_not_found() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let response = app.server
        .get("/api/v1/documents/nonexistent-id")
        .add_header("Authorization", &auth_header)
        .await;

    assert_status(&response, 404);
}

#[tokio::test]
async fn test_update_document_success() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create document
    let create_request = json!({
        "title": "Original Title",
        "content": "Original content"
    });

    let create_response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    let created_doc: serde_json::Value = create_response.json();
    let document_id = created_doc["id"].as_str().unwrap();

    // Update document
    let update_request = json!({
        "title": "Updated Title",
        "content": "Updated content",
        "tags": ["updated", "test"]
    });

    let response = app.server
        .put(&format!("/api/v1/documents/{}", document_id))
        .add_header("Authorization", &auth_header)
        .json(&update_request)
        .await;

    assert_success(&response);
    
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["id"], document_id);
    assert_eq!(json_response["title"], "Updated Title");
    assert_eq!(json_response["content"], "Updated content");
    assert_eq!(json_response["tags"], json!(["updated", "test"]));
    
    // Updated timestamp should be different from created timestamp
    assert_ne!(json_response["created_at"], json_response["updated_at"]);
}

#[tokio::test]
async fn test_delete_document_success() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create document
    let create_request = json!({
        "title": "Document to Delete"
    });

    let create_response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    let created_doc: serde_json::Value = create_response.json();
    let document_id = created_doc["id"].as_str().unwrap();

    // Delete document
    let response = app.server
        .delete(&format!("/api/v1/documents/{}", document_id))
        .add_header("Authorization", &auth_header)
        .await;

    assert_status(&response, 204);

    // Verify document is deleted
    let get_response = app.server
        .get(&format!("/api/v1/documents/{}", document_id))
        .add_header("Authorization", &auth_header)
        .await;

    assert_status(&get_response, 404);
}

#[tokio::test]
async fn test_list_documents_success() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create multiple documents
    for i in 1..=5 {
        let create_request = json!({
            "title": format!("Document {}", i),
            "content": format!("Content for document {}", i),
            "tags": [format!("tag{}", i)]
        });

        app.server
            .post("/api/v1/documents")
            .add_header("Authorization", &auth_header)
            .json(&create_request)
            .await;
    }

    // List documents
    let response = app.server
        .get("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&response);
    assert_json_field(&response, "documents");
    assert_json_field(&response, "total");
    assert_json_field(&response, "page");
    assert_json_field(&response, "per_page");
    
    let json_response: serde_json::Value = response.json();
    let documents = json_response["documents"].as_array().unwrap();
    assert!(documents.len() >= 5);
    assert_eq!(json_response["page"], 1);
    assert_eq!(json_response["per_page"], 20);
}

#[tokio::test]
async fn test_list_documents_pagination() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create documents
    for i in 1..=25 {
        let create_request = json!({
            "title": format!("Document {}", i)
        });

        app.server
            .post("/api/v1/documents")
            .add_header("Authorization", &auth_header)
            .json(&create_request)
            .await;
    }

    // Test pagination - page 1 with 10 per page
    let response = app.server
        .get("/api/v1/documents?page=1&per_page=10")
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&response);
    
    let json_response: serde_json::Value = response.json();
    let documents = json_response["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 10);
    assert_eq!(json_response["page"], 1);
    assert_eq!(json_response["per_page"], 10);

    // Test page 2
    let response = app.server
        .get("/api/v1/documents?page=2&per_page=10")
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&response);
    
    let json_response: serde_json::Value = response.json();
    let documents = json_response["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 10);
    assert_eq!(json_response["page"], 2);
    assert_eq!(json_response["per_page"], 10);
}

#[tokio::test]
async fn test_concurrent_document_operations() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    // Create multiple documents concurrently
    let create_tasks = (0..10).map(|i| {
        let app = app.server.clone();
        let auth_header = auth_header.clone();
        tokio::spawn(async move {
            let create_request = json!({
                "title": format!("Concurrent Document {}", i),
                "content": format!("Content {}", i)
            });

            app.post("/api/v1/documents")
                .add_header("Authorization", &auth_header)
                .json(&create_request)
                .await
        })
    });

    let results = futures::future::join_all(create_tasks).await;

    // All creations should succeed
    for result in results {
        let response = result.expect("Task should complete");
        assert_status(&response, 201);
    }

    // List all documents to verify they were created
    let list_response = app.server
        .get("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .await;

    assert_success(&list_response);
    
    let json_response: serde_json::Value = list_response.json();
    let documents = json_response["documents"].as_array().unwrap();
    assert!(documents.len() >= 10);
}

#[tokio::test]
async fn test_document_word_count_calculation() {
    let app = TestApp::new().await;

    // Create and authenticate user
    app.create_test_user("testuser", "test@example.com", "password123").await;
    let auth_header = app.auth_header_for_user("testuser", "password123").await;

    let content = "This is a test document with exactly ten words.";
    let create_request = json!({
        "title": "Word Count Test",
        "content": content
    });

    let response = app.server
        .post("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .json(&create_request)
        .await;

    assert_status(&response, 201);
    
    // Get the created document to check word count in summary
    let list_response = app.server
        .get("/api/v1/documents")
        .add_header("Authorization", &auth_header)
        .await;

    let json_response: serde_json::Value = list_response.json();
    let documents = json_response["documents"].as_array().unwrap();
    let document = &documents[0];
    
    assert_eq!(document["word_count"], 10);
}