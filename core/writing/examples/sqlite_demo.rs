//! SQLite repository demonstration

use writemagic_writing::{CoreEngine, Document, Project, entities::*};
use writemagic_shared::{ContentType, EntityId, Pagination};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 WriteMagic SQLite Repository Demo");
    
    // Initialize the core engine with SQLite in-memory database
    let engine = CoreEngine::new_sqlite_in_memory().await?;
    println!("✅ Initialized SQLite in-memory database");
    
    // Get repositories
    let doc_repo = engine.document_repository();
    let project_repo = engine.project_repository();
    
    // Create some test documents
    println!("\n📄 Creating test documents...");
    
    let doc1 = Document::new(
        "My First Document".to_string(),
        "This is a sample markdown document with some **bold** text.".to_string(),
        ContentType::Markdown,
        Some(EntityId::new()),
    );
    
    let doc2 = Document::new(
        "Meeting Notes".to_string(),
        "Notes from the team meeting:\n- Discussed project timeline\n- Assigned tasks".to_string(),
        ContentType::PlainText,
        Some(EntityId::new()),
    );
    
    let doc3 = Document::new(
        "Code Example".to_string(),
        r#"fn main() {
    println!("Hello, world!");
}"#.to_string(),
        ContentType::Code { language: "rust".to_string() },
        Some(EntityId::new()),
    );
    
    // Save documents
    let saved_doc1 = doc_repo.save(&doc1).await?;
    let saved_doc2 = doc_repo.save(&doc2).await?;
    let saved_doc3 = doc_repo.save(&doc3).await?;
    
    println!("✅ Saved {} documents", 3);
    
    // Create a project with documents
    println!("\n📁 Creating project...");
    
    let mut project = Project::new(
        "My Writing Project".to_string(),
        Some("A collection of documents for my project".to_string()),
        Some(EntityId::new()),
    );
    
    project.add_document(saved_doc1.id, None);
    project.add_document(saved_doc2.id, None);
    
    let saved_project = project_repo.save(&project).await?;
    println!("✅ Created project: {}", saved_project.name);
    
    // Demonstrate repository operations
    println!("\n🔍 Testing repository operations...");
    
    // Count documents
    let doc_count = doc_repo.count().await?;
    println!("📊 Total documents: {}", doc_count);
    
    // Find all documents
    let all_docs = doc_repo.find_all(Pagination::new(0, 10)?).await?;
    println!("📋 Found {} documents:", all_docs.len());
    for doc in &all_docs {
        println!("  - {}: {} ({:?})", doc.id, doc.title, doc.content_type);
    }
    
    // Search by title
    let search_results = doc_repo.search_by_title("Meeting", Pagination::new(0, 10)?).await?;
    println!("🔎 Search results for 'Meeting': {} documents", search_results.len());
    
    // Search by content type
    let markdown_docs = doc_repo.find_by_content_type(&ContentType::Markdown, Pagination::new(0, 10)?).await?;
    println!("📝 Markdown documents: {} found", markdown_docs.len());
    
    // Find documents by project
    let project_docs = doc_repo.find_by_project_id(&saved_project.id, Pagination::new(0, 10)?).await?;
    println!("📁 Documents in project: {} found", project_docs.len());
    
    // Get statistics
    let stats = doc_repo.get_statistics().await?;
    println!("📊 Document Statistics:");
    println!("  - Total documents: {}", stats.total_documents);
    println!("  - Total word count: {}", stats.total_word_count);
    println!("  - Average word count: {:.2}", stats.average_word_count);
    println!("  - Documents by type: {:?}", stats.documents_by_type);
    
    // Test document update
    println!("\n✏️  Testing document update...");
    let mut doc_to_update = saved_doc1;
    doc_to_update.update_content("This is updated content with more text!".to_string(), None);
    let updated_doc = doc_repo.save(&doc_to_update).await?;
    println!("✅ Updated document: {} (version {})", updated_doc.title, updated_doc.version);
    
    // Test content search
    println!("\n🔍 Testing content search...");
    let content_search = doc_repo.search_by_content("updated", Pagination::new(0, 10)?).await?;
    println!("📄 Found {} documents containing 'updated'", content_search.len());
    
    // Project statistics
    let project_stats = project_repo.get_statistics().await?;
    println!("\n📁 Project Statistics:");
    println!("  - Total projects: {}", project_stats.total_projects);
    println!("  - Total documents in projects: {}", project_stats.total_documents_in_projects);
    println!("  - Average documents per project: {:.2}", project_stats.average_documents_per_project);
    
    // Check migration status
    if let Some(migrations) = engine.get_migration_status().await? {
        println!("\n🗄️  Database Migrations:");
        for migration in migrations {
            let status = if migration.applied { "✅ Applied" } else { "❌ Pending" };
            println!("  - {}: {}", migration.name, status);
        }
    }
    
    println!("\n🎉 Demo completed successfully!");
    Ok(())
}