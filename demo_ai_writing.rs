//! Demonstration of AI Writing Service Integration
//! 
//! This file shows how the AI writing service integrates with WriteMagic's
//! writing domain to provide contextual, document-aware AI assistance.

use writemagic_shared::{EntityId, Result, ContentType};
use writemagic_writing::{
    CoreEngine, ApplicationConfigBuilder, 
    DocumentTitle, DocumentContent, ProjectName,
};
use writemagic_ai::{
    WritingAssistanceType, ToneAdjustment, WritingPreferences, 
    FormalityLevel, VocabularyLevel, TextSelection,
};

/// Demo: Basic AI-Assisted Document Creation and Enhancement
pub async fn demo_basic_ai_assistance() -> Result<()> {
    println!("🚀 WriteMagic AI Writing Service Demo");
    println!("=====================================\n");

    // 1. Initialize the engine with AI capabilities
    println!("1. Initializing WriteMagic Core Engine with AI...");
    let engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key("demo-claude-key".to_string()) // Replace with real key
        .with_openai_key("demo-openai-key".to_string())  // Replace with real key
        .with_default_model("claude-3-5-sonnet-20241022".to_string())
        .with_content_filtering(true)
        .with_max_context_length(8000)
        .build()
        .await?;

    engine.init_logging()?;
    println!("✅ Engine initialized with AI capabilities\n");

    // 2. Create a new document
    println!("2. Creating a new document...");
    let doc_service = engine.document_management_service();
    let title = DocumentTitle::new("AI-Enhanced Technical Article")?;
    let initial_content = DocumentContent::new(
        "# Introduction\n\nThis article explores advanced techniques in software architecture."
    )?;
    
    let doc_aggregate = doc_service
        .create_document(title, initial_content, ContentType::Markdown, None)
        .await?;
    
    let document_id = doc_aggregate.document().id;
    println!("✅ Document created: {}\n", document_id);

    // 3. Get the integrated writing service
    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI writing service not available"))?;

    // 4. Generate content using AI
    println!("3. Generating content with AI assistance...");
    let generation_response = writing_service.generate_content_for_document(
        document_id,
        "Write a comprehensive section about microservices architecture patterns. \
         Include benefits, challenges, and best practices. Make it engaging for senior developers."
            .to_string(),
        Some(800), // Target ~800 tokens
        true,      // Apply to document
        None,
    ).await?;

    println!("✅ Generated {} characters of content", generation_response.content.len());
    println!("   Confidence: {:.1}%", generation_response.confidence_score * 100.0);
    println!("   Tokens used: {}\n", generation_response.usage.total_tokens);

    // 5. Analyze the content quality
    println!("4. Analyzing content quality...");
    let analysis = writing_service.analyze_document_content(document_id, None).await?;
    
    println!("📊 Content Analysis:");
    println!("   - Readability: {} (Grade {:.1})", 
             analysis.readability.reading_level_description(),
             analysis.readability.flesch_kincaid_grade_level);
    println!("   - Word Count: {}", analysis.readability.words);
    println!("   - Primary Tone: {:?}", analysis.tone_analysis.primary_tone);
    println!("   - Sentiment: {:.2} (polarity)", analysis.sentiment.polarity);
    println!("   - Key Points: {}", analysis.key_points.len());
    
    for (i, point) in analysis.key_points.iter().take(3).enumerate() {
        println!("     {}. {}", i + 1, point);
    }
    println!();

    // 6. Improve writing quality
    println!("5. Improving writing quality...");
    let improvement_response = writing_service.improve_document_content(
        document_id,
        None, // Improve entire document
        Some("Focus on clarity, conciseness, and technical accuracy".to_string()),
        true, // Apply improvements
        None,
    ).await?;

    println!("✅ Writing improvements applied");
    println!("   Suggestions provided: {}", improvement_response.suggestions.len());
    
    for suggestion in improvement_response.suggestions.iter().take(3) {
        println!("   - {}: {}", 
                 format!("{:?}", suggestion.suggestion_type),
                 suggestion.explanation);
    }
    println!();

    // 7. Adjust tone for different audiences
    println!("6. Adjusting tone for academic audience...");
    let tone_response = writing_service.adjust_document_tone(
        document_id,
        ToneAdjustment::Academic,
        None, // Adjust entire document
        false, // Don't apply yet, just preview
        None,
    ).await?;

    println!("✅ Tone adjustment preview generated");
    println!("   Preview: {}...", &tone_response.content[..200.min(tone_response.content.len())]);
    println!();

    // 8. Check grammar and provide corrections
    println!("7. Checking grammar...");
    let grammar_response = writing_service.check_document_grammar(
        document_id,
        None, // Check entire document
        false, // Don't auto-apply corrections
        None,
    ).await?;

    println!("✅ Grammar check completed");
    println!("   Issues found: {}", grammar_response.suggestions.len());
    
    for suggestion in grammar_response.suggestions.iter().take(2) {
        if let Some(original) = &suggestion.original_text {
            println!("   - Original: \"{}\"", original);
            println!("     Suggested: \"{}\"", suggestion.suggested_text);
            println!("     Reason: {}", suggestion.explanation);
        }
    }
    println!();

    // 9. Create a summary
    println!("8. Creating document summary...");
    let summary_response = writing_service.summarize_document(
        document_id,
        Some(200), // ~200 tokens
        true,      // Create summary document
        None,
    ).await?;

    println!("✅ Summary created");
    println!("   Summary: {}", summary_response.content);
    println!();

    // 10. Get conversation history
    println!("9. Retrieving conversation history...");
    let session = writing_service.get_document_conversation_session(document_id).await?;
    
    println!("💬 Conversation Session:");
    println!("   Session ID: {}", session.session_id);
    println!("   Total interactions: {}", session.entries.len());
    println!("   Last activity: {}", session.last_activity);
    
    let recent = session.get_recent_context(3);
    for entry in recent {
        println!("   - {}: {:?}", 
                 entry.timestamp.format("%H:%M:%S"),
                 entry.request_type);
    }
    println!();

    // Cleanup
    engine.shutdown().await;
    println!("🎉 Demo completed successfully!");
    
    Ok(())
}

/// Demo: Project-Based AI Writing Workflow
pub async fn demo_project_workflow() -> Result<()> {
    println!("📚 Project-Based AI Writing Workflow Demo");
    println!("=========================================\n");

    let engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key("demo-claude-key".to_string())
        .build()
        .await?;

    // Create a research project
    let project_service = engine.project_management_service();
    let project_name = ProjectName::new("Machine Learning Research Paper")?;
    let description = Some("Comprehensive research on deep learning applications".to_string());
    
    let project_aggregate = project_service
        .create_project(project_name, description, None)
        .await?;
    
    let project_id = project_aggregate.project().id;
    println!("📝 Created project: {}", project_id);

    // Create multiple documents for the project
    let doc_service = engine.document_management_service();
    let sections = vec![
        ("Abstract", "This paper presents..."),
        ("Introduction", "Machine learning has revolutionized..."),
        ("Literature Review", "Previous research in this area..."),
        ("Methodology", "Our approach involves..."),
        ("Results", "The experimental results show..."),
        ("Discussion", "The implications of these findings..."),
        ("Conclusion", "In conclusion, this research..."),
    ];

    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI service not available"))?;

    for (title, content) in sections {
        // Create document
        let doc_title = DocumentTitle::new(title)?;
        let doc_content = DocumentContent::new(content)?;
        
        let doc_aggregate = doc_service
            .create_document(doc_title, doc_content, ContentType::Markdown, None)
            .await?;
        
        let doc_id = doc_aggregate.document().id;

        // Add to project
        project_service
            .add_document_to_project(project_id, doc_id, None)
            .await?;

        // Use AI to enhance each section
        println!("🤖 Enhancing '{}' section...", title);
        
        let enhancement_response = writing_service.generate_content_for_document(
            doc_id,
            format!("Expand this {} section for a machine learning research paper. \
                     Make it comprehensive and academically rigorous.", title.to_lowercase()),
            Some(500),
            true,
            None,
        ).await?;

        println!("   ✅ Enhanced with {} tokens", enhancement_response.usage.total_tokens);
    }

    println!("\n🎯 Project workflow completed with AI assistance!");
    engine.shutdown().await;
    
    Ok(())
}

/// Demo: Advanced AI Features
pub async fn demo_advanced_features() -> Result<()> {
    println!("🧠 Advanced AI Writing Features Demo");
    println!("=====================================\n");

    let engine = ApplicationConfigBuilder::new()
        .with_sqlite_in_memory()
        .with_claude_key("demo-claude-key".to_string())
        .build()
        .await?;

    // Create test document
    let doc_service = engine.document_management_service();
    let title = DocumentTitle::new("Technical Documentation")?;
    let content = DocumentContent::new(
        "API Reference\n\nThis API provides endpoints for user management. \
         The authentication is handled via JWT tokens. Each endpoint requires \
         proper authorization headers."
    )?;
    
    let doc_aggregate = doc_service
        .create_document(title, content, ContentType::Markdown, None)
        .await?;
    
    let document_id = doc_aggregate.document().id;

    let writing_service = engine.integrated_writing_service()
        .ok_or_else(|| writemagic_shared::WritemagicError::configuration("AI service not available"))?;

    // 1. Custom assistance with specific preferences
    println!("1. Custom assistance with user preferences...");
    
    let preferences = WritingPreferences {
        preferred_tone: Some(ToneAdjustment::Technical),
        target_audience: Some("Software developers".to_string()),
        writing_style: Some("Clear, concise technical documentation".to_string()),
        language: "en".to_string(),
        formality_level: FormalityLevel::Formal,
        vocabulary_level: VocabularyLevel::Advanced,
    };

    let custom_response = writing_service.provide_custom_assistance(
        document_id,
        WritingAssistanceType::Rewrite,
        Some("Rewrite this for senior developers, emphasizing technical details and best practices".to_string()),
        None,
        Some(preferences),
        false, // Don't apply, just preview
        None,
    ).await?;

    println!("✅ Technical rewrite completed");
    println!("   Preview: {}...", &custom_response.content[..300.min(custom_response.content.len())]);
    println!();

    // 2. Content completion with context
    println!("2. Intelligent content completion...");
    
    let cursor_position = 150; // Simulate cursor position
    let selection = Some(TextSelection::cursor(cursor_position));
    
    let completion_response = writing_service.complete_document_content(
        document_id,
        selection,
        Some("Continue with detailed examples and code snippets".to_string()),
        false, // Don't apply, just preview
        None,
    ).await?;

    println!("✅ Content completion generated");
    println!("   Completion: {}", completion_response.content);
    println!();

    // 3. Multiple tone variations
    println!("3. Generating multiple tone variations...");
    
    let tones = vec![
        ToneAdjustment::Casual,
        ToneAdjustment::Professional,
        ToneAdjustment::Academic,
        ToneAdjustment::Friendly,
    ];

    for tone in tones {
        let tone_response = writing_service.adjust_document_tone(
            document_id,
            tone.clone(),
            None,
            false,
            None,
        ).await?;

        println!("   {:?} tone: {}...", 
                 tone, 
                 &tone_response.content[..100.min(tone_response.content.len())]);
    }
    println!();

    // 4. Advanced content analysis
    println!("4. Advanced content analysis...");
    
    let analysis = writing_service.analyze_document_content(document_id, None).await?;
    
    println!("📈 Detailed Analysis:");
    println!("   Readability Metrics:");
    println!("     - Flesch Reading Ease: {:.1}", analysis.readability.flesch_reading_ease);
    println!("     - Grade Level: {:.1}", analysis.readability.flesch_kincaid_grade_level);
    println!("     - Avg Words/Sentence: {:.1}", analysis.readability.average_words_per_sentence);
    println!("     - Avg Syllables/Word: {:.2}", analysis.readability.average_syllables_per_word);
    
    println!("   Tone Analysis:");
    println!("     - Primary: {:?} ({:.1}% confidence)", 
             analysis.tone_analysis.primary_tone,
             analysis.tone_analysis.tone_confidence * 100.0);
    println!("     - Consistency: {:.1}%", analysis.tone_analysis.tone_consistency * 100.0);
    
    println!("   Word Frequency (Top 5):");
    let mut freq_vec: Vec<_> = analysis.word_frequency.iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (word, count) in freq_vec.iter().take(5) {
        println!("     - '{}': {} times", word, count);
    }

    engine.shutdown().await;
    println!("\n🏁 Advanced features demo completed!");
    
    Ok(())
}

/// Main demo function
#[tokio::main]
async fn main() -> Result<()> {
    // Run basic demo
    if let Err(e) = demo_basic_ai_assistance().await {
        eprintln!("Basic demo error: {}", e);
    }

    println!("\n" + "=".repeat(50) + "\n");

    // Run project workflow demo
    if let Err(e) = demo_project_workflow().await {
        eprintln!("Project workflow demo error: {}", e);
    }

    println!("\n" + "=".repeat(50) + "\n");

    // Run advanced features demo
    if let Err(e) = demo_advanced_features().await {
        eprintln!("Advanced features demo error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writing_preferences() {
        let prefs = WritingPreferences {
            preferred_tone: Some(ToneAdjustment::Professional),
            target_audience: Some("Engineers".to_string()),
            writing_style: Some("Technical".to_string()),
            language: "en".to_string(),
            formality_level: FormalityLevel::Formal,
            vocabulary_level: VocabularyLevel::Advanced,
        };

        assert_eq!(prefs.language, "en");
        assert_eq!(prefs.formality_level, FormalityLevel::Formal);
        assert!(prefs.preferred_tone.is_some());
    }

    #[test]
    fn test_text_selection() {
        let selection = TextSelection::new(10, 20).unwrap();
        assert_eq!(selection.start, 10);
        assert_eq!(selection.end, 20);
        assert!(!selection.is_cursor());

        let cursor = TextSelection::cursor(15);
        assert!(cursor.is_cursor());
        assert_eq!(cursor.start, 15);
        assert_eq!(cursor.end, 15);
    }

    // Note: Integration tests would require actual API keys
    // They should be run with `cargo test --ignored` and proper credentials
    
    #[tokio::test]
    #[ignore]
    async fn integration_test_basic_workflow() {
        // This test requires real API keys
        // demo_basic_ai_assistance().await.unwrap();
    }
}