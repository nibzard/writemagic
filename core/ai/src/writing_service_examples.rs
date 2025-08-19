//! Examples demonstrating AI writing service usage

use writemagic_shared::{EntityId, Result, ContentType};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, 
    Document, Project,
    DocumentTitle, DocumentContent, ProjectName,
    IntegratedWritingService,
};
use crate::writing_service::{
    WritingAssistanceType, ToneAdjustment, WritingPreferences, 
    FormalityLevel, VocabularyLevel, TextSelection,
};

/// Example: Generate content for a new document
pub async fn example_generate_content(
    engine: &CoreEngine,
    claude_api_key: &str,
) -> Result<()> {
    // Get the integrated writing service
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Create a new document
    let document_service = engine.document_management_service();
    let title = DocumentTitle::new("My AI-Assisted Article")?;
    let initial_content = DocumentContent::new("Introduction: This article will explore...")?;
    
    let document_aggregate = document_service
        .create_document(title, initial_content, ContentType::Markdown, None)
        .await?;
    
    let document_id = document_aggregate.document().id;

    // Generate content for the document
    let response = writing_service.generate_content_for_document(
        document_id,
        "Write a comprehensive section about the benefits of AI in writing. Make it engaging and informative.".to_string(),
        Some(500), // Target ~500 tokens
        true,      // Apply to document
        None,      // No specific user
    ).await?;

    println!("Generated content applied to document: {}", response.applied_to_document);
    println!("Content preview: {}", &response.content[..response.content.len().min(200)]);
    println!("Confidence score: {}", response.confidence_score);
    println!("Token usage: {}", response.usage.total_tokens);

    Ok(())
}

/// Example: Improve existing document content
pub async fn example_improve_writing(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Select a portion of text to improve
    let selection = Some(TextSelection::new(0, 100)?); // First 100 characters

    // Improve the selected content
    let response = writing_service.improve_document_content(
        document_id,
        selection,
        Some("Focus on clarity and conciseness".to_string()),
        true, // Apply improvements
        None,
    ).await?;

    println!("Improved content: {}", response.content);
    println!("Suggestions provided: {}", response.suggestions.len());
    
    for suggestion in &response.suggestions {
        println!("- {}: {}", suggestion.suggestion_type, suggestion.explanation);
    }

    Ok(())
}

/// Example: Grammar checking and correction
pub async fn example_grammar_check(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Check grammar for entire document
    let response = writing_service.check_document_grammar(
        document_id,
        None, // Check entire document
        false, // Don't auto-apply corrections
        None,
    ).await?;

    println!("Grammar check completed");
    println!("Corrections found: {}", response.suggestions.len());

    for suggestion in &response.suggestions {
        if let Some(original) = &suggestion.original_text {
            println!("Original: {}", original);
            println!("Suggested: {}", suggestion.suggested_text);
            println!("Reason: {}", suggestion.explanation);
            println!("---");
        }
    }

    Ok(())
}

/// Example: Tone adjustment
pub async fn example_adjust_tone(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Adjust tone to be more professional
    let response = writing_service.adjust_document_tone(
        document_id,
        ToneAdjustment::Professional,
        None, // Adjust entire document
        true, // Apply changes
        None,
    ).await?;

    println!("Tone adjusted to professional");
    println!("New content preview: {}", &response.content[..response.content.len().min(300)]);

    Ok(())
}

/// Example: Content analysis
pub async fn example_analyze_content(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Analyze document content
    let analysis = writing_service.analyze_document_content(document_id, None).await?;

    println!("Content Analysis Results:");
    println!("========================");
    
    // Readability analysis
    println!("Readability:");
    println!("  - Flesch Reading Ease: {:.1}", analysis.readability.flesch_reading_ease);
    println!("  - Grade Level: {}", analysis.readability.grade_level_description());
    println!("  - Reading Level: {}", analysis.readability.reading_level_description());
    println!("  - Word Count: {}", analysis.readability.words);
    println!("  - Sentence Count: {}", analysis.readability.sentences);

    // Tone analysis
    println!("\nTone Analysis:");
    println!("  - Primary Tone: {:?}", analysis.tone_analysis.primary_tone);
    println!("  - Confidence: {:.1}%", analysis.tone_analysis.tone_confidence * 100.0);
    println!("  - Consistency: {:.1}%", analysis.tone_analysis.tone_consistency * 100.0);

    // Key points
    println!("\nKey Points:");
    for (i, point) in analysis.key_points.iter().enumerate() {
        println!("  {}. {}", i + 1, point);
    }

    // Sentiment
    println!("\nSentiment:");
    println!("  - Polarity: {:.2} (negative ← 0 → positive)", analysis.sentiment.polarity);
    println!("  - Subjectivity: {:.2} (objective ← 0 → subjective)", analysis.sentiment.subjectivity);

    Ok(())
}

/// Example: Content completion
pub async fn example_complete_content(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Complete content from current cursor position
    let cursor_position = 150; // Assume cursor is at character 150
    let selection = Some(TextSelection::cursor(cursor_position));

    let response = writing_service.complete_document_content(
        document_id,
        selection,
        Some("Continue with examples and practical applications".to_string()),
        true, // Apply completion
        None,
    ).await?;

    println!("Content completion applied");
    println!("Added text: {}", response.content);
    println!("Character count: {}", response.content.len());

    Ok(())
}

/// Example: Document summarization
pub async fn example_summarize_document(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Create summary as a new document
    let response = writing_service.summarize_document(
        document_id,
        Some(200), // ~200 tokens
        true,      // Create summary document
        None,
    ).await?;

    println!("Document summarized");
    println!("Summary: {}", response.content);
    
    if response.applied_to_document {
        println!("Summary document created successfully");
    }

    Ok(())
}

/// Example: Custom assistance with preferences
pub async fn example_custom_assistance(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Set custom writing preferences
    let preferences = WritingPreferences {
        preferred_tone: Some(ToneAdjustment::Academic),
        target_audience: Some("Graduate students and researchers".to_string()),
        writing_style: Some("Formal academic style with clear explanations".to_string()),
        language: "en".to_string(),
        formality_level: FormalityLevel::VeryFormal,
        vocabulary_level: VocabularyLevel::Advanced,
    };

    // Rewrite content with specific preferences
    let response = writing_service.provide_custom_assistance(
        document_id,
        WritingAssistanceType::Rewrite,
        Some("Rewrite this for an academic audience, emphasizing theoretical foundations".to_string()),
        None, // Rewrite entire document
        Some(preferences),
        true, // Apply changes
        None,
    ).await?;

    println!("Custom rewrite completed");
    println!("Academic version: {}", &response.content[..response.content.len().min(400)]);

    Ok(())
}

/// Example: Working with conversation sessions
pub async fn example_conversation_session(
    engine: &CoreEngine,
    document_id: EntityId,
) -> Result<()> {
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // Get conversation session for document
    let session = writing_service.get_document_conversation_session(document_id).await?;

    println!("Conversation session for document {}", document_id);
    println!("Session ID: {}", session.session_id);
    println!("Total interactions: {}", session.entries.len());
    println!("Last activity: {}", session.last_activity);

    // Recent context (last 3 interactions)
    let recent = session.get_recent_context(3);
    println!("\nRecent conversation context:");
    for entry in recent {
        println!("  - {}: {} -> {}", 
                 entry.timestamp.format("%H:%M"),
                 entry.request_type,
                 &entry.ai_response[..entry.ai_response.len().min(50)]);
    }

    Ok(())
}

/// Example: Complete workflow - Create document and enhance with AI
pub async fn example_complete_workflow() -> Result<()> {
    // Initialize engine with AI capabilities
    let engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key("your-claude-api-key".to_string())
        .with_default_model("claude-3-5-sonnet-20241022".to_string())
        .with_content_filtering(true)
        .build()
        .await?;

    // Initialize logging
    engine.init_logging()?;

    // Create a new document
    let doc_service = engine.document_management_service();
    let title = DocumentTitle::new("AI-Enhanced Writing Example")?;
    let initial_content = DocumentContent::new("This document will be enhanced using AI assistance.")?;
    
    let doc_aggregate = doc_service
        .create_document(title, initial_content, ContentType::Markdown, None)
        .await?;
    
    let document_id = doc_aggregate.document().id;
    println!("Created document: {}", document_id);

    // Step 1: Generate content
    println!("\n1. Generating content...");
    example_generate_content(&engine, "your-claude-api-key").await?;

    // Step 2: Analyze the content
    println!("\n2. Analyzing content...");
    example_analyze_content(&engine, document_id).await?;

    // Step 3: Check grammar
    println!("\n3. Checking grammar...");
    example_grammar_check(&engine, document_id).await?;

    // Step 4: Improve writing quality
    println!("\n4. Improving writing...");
    example_improve_writing(&engine, document_id).await?;

    // Step 5: Adjust tone
    println!("\n5. Adjusting tone...");
    example_adjust_tone(&engine, document_id).await?;

    // Step 6: Create summary
    println!("\n6. Creating summary...");
    example_summarize_document(&engine, document_id).await?;

    // Shutdown engine
    engine.shutdown().await;
    println!("\nWorkflow completed successfully!");

    Ok(())
}

/// Example: Project-based writing assistance
pub async fn example_project_writing() -> Result<()> {
    let engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key("your-claude-api-key".to_string())
        .build()
        .await?;

    // Create a project
    let project_service = engine.project_management_service();
    let project_name = ProjectName::new("AI Research Paper")?;
    let description = Some("Research paper exploring AI applications in writing".to_string());
    
    let project_aggregate = project_service
        .create_project(project_name, description, None)
        .await?;
    
    let project_id = project_aggregate.project().id;
    println!("Created project: {}", project_id);

    // Create multiple documents in the project
    let doc_service = engine.document_management_service();
    let documents = vec![
        ("Introduction", "This paper introduces..."),
        ("Literature Review", "Previous research shows..."),
        ("Methodology", "Our approach involves..."),
        ("Results", "The findings indicate..."),
        ("Conclusion", "In conclusion..."),
    ];

    let mut document_ids = Vec::new();
    for (title, content) in documents {
        let doc_title = DocumentTitle::new(title)?;
        let doc_content = DocumentContent::new(content)?;
        
        let doc_aggregate = doc_service
            .create_document(doc_title, doc_content, ContentType::Markdown, None)
            .await?;
        
        let doc_id = doc_aggregate.document().id;
        document_ids.push(doc_id);

        // Add document to project
        project_service
            .add_document_to_project(project_id, doc_id, None)
            .await?;
    }

    println!("Created {} documents in project", document_ids.len());

    // Use AI to enhance each document with project context
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    for (i, doc_id) in document_ids.iter().enumerate() {
        println!("\nEnhancing document {} with project context...", i + 1);
        
        // Generate content aware of project context
        let response = writing_service.generate_content_for_document(
            *doc_id,
            "Expand this section with detailed content, keeping in mind this is part of an AI research paper".to_string(),
            Some(300),
            true,
            None,
        ).await?;

        println!("Enhanced document with {} tokens", response.usage.total_tokens);
    }

    engine.shutdown().await;
    println!("Project-based writing assistance completed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_complete_workflow() {
        // This test requires actual API keys and should be run manually
        // example_complete_workflow().await.unwrap();
    }

    #[test]
    fn test_writing_preferences() {
        let preferences = WritingPreferences {
            preferred_tone: Some(ToneAdjustment::Academic),
            target_audience: Some("Students".to_string()),
            writing_style: Some("Clear and concise".to_string()),
            language: "en".to_string(),
            formality_level: FormalityLevel::Formal,
            vocabulary_level: VocabularyLevel::Intermediate,
        };

        assert_eq!(preferences.language, "en");
        assert_eq!(preferences.formality_level, FormalityLevel::Formal);
        assert_eq!(preferences.vocabulary_level, VocabularyLevel::Intermediate);
        assert!(preferences.target_audience.is_some());
    }
}