//! AI Writing Service - bridges AI providers with writing domain functionality

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use writemagic_shared::{EntityId, Result, WritemagicError};

use crate::providers::{AIProvider, CompletionRequest, CompletionResponse, Message, MessageRole};
use crate::services::{AIOrchestrationService, ContextManagementService, ContentFilteringService};
use crate::value_objects::{Prompt, ModelConfiguration, TokenCount};

// Forward declarations to avoid circular dependencies
// These will be imported where the service is used

/// Writing assistance type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum WritingAssistanceType {
    ContentGeneration,
    ContentCompletion,
    Summarization,
    Improvement,
    GrammarCheck,
    StyleSuggestions,
    Brainstorming,
    Outline,
    Rewrite,
    Expand,
    Condense,
    Tone(ToneAdjustment),
}

/// Tone adjustment options
#[derive(Debug, Clone, PartialEq)]
pub enum ToneAdjustment {
    Professional,
    Casual,
    Formal,
    Creative,
    Technical,
    Persuasive,
    Friendly,
    Academic,
}

/// Text selection for content editing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextSelection {
    pub start: usize,
    pub end: usize,
}

impl TextSelection {
    pub fn new(start: usize, end: usize) -> Result<Self> {
        if start > end {
            return Err(WritemagicError::validation("Selection start cannot be greater than end"));
        }
        Ok(Self { start, end })
    }

    pub fn cursor(position: usize) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    pub fn is_cursor(&self) -> bool {
        self.start == self.end
    }

    pub fn extract_from(&self, content: &str) -> Option<String> {
        if self.end <= content.len() {
            Some(content[self.start..self.end].to_string())
        } else {
            None
        }
    }
}

/// Writing context for AI assistance
#[derive(Debug, Clone)]
pub struct WritingContext {
    pub document_id: EntityId,
    pub document_title: String,
    pub document_content: String,
    pub content_type: writemagic_shared::ContentType,
    pub selection: Option<TextSelection>,
    pub project_context: Option<ProjectContext>,
    pub conversation_history: Vec<ConversationEntry>,
    pub user_preferences: WritingPreferences,
}

/// Project context for AI assistance
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub project_id: EntityId,
    pub project_name: String,
    pub project_description: Option<String>,
    pub related_documents: Vec<RelatedDocument>,
}

/// Related document information
#[derive(Debug, Clone)]
pub struct RelatedDocument {
    pub id: EntityId,
    pub title: String,
    pub content_excerpt: String,
}

/// Conversation entry for context management
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_type: WritingAssistanceType,
    pub user_input: String,
    pub ai_response: String,
    pub applied: bool,
}

/// User writing preferences
#[derive(Debug, Clone)]
pub struct WritingPreferences {
    pub preferred_tone: Option<ToneAdjustment>,
    pub target_audience: Option<String>,
    pub writing_style: Option<String>,
    pub language: String,
    pub formality_level: FormalityLevel,
    pub vocabulary_level: VocabularyLevel,
}

/// Formality level enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FormalityLevel {
    VeryFormal,
    Formal,
    Neutral,
    Informal,
    VeryInformal,
}

/// Vocabulary level enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum VocabularyLevel {
    Simple,
    Intermediate,
    Advanced,
    Expert,
}

/// Writing assistance request
#[derive(Debug, Clone)]
pub struct WritingAssistanceRequest {
    pub context: WritingContext,
    pub assistance_type: WritingAssistanceType,
    pub user_input: Option<String>,
    pub model_config: Option<ModelConfiguration>,
    pub stream_response: bool,
}

/// Writing assistance response
#[derive(Debug, Clone)]
pub struct WritingAssistanceResponse {
    pub id: String,
    pub content: String,
    pub suggestions: Vec<WritingSuggestion>,
    pub analysis: Option<ContentAnalysis>,
    pub usage: TokenUsage,
    pub confidence_score: f32,
    pub applied_to_document: bool,
}

/// Individual writing suggestion
#[derive(Debug, Clone)]
pub struct WritingSuggestion {
    pub suggestion_type: SuggestionType,
    pub original_text: Option<String>,
    pub suggested_text: String,
    pub explanation: String,
    pub confidence: f32,
    pub position: Option<TextSelection>,
}

/// Suggestion type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    Grammar,
    Style,
    Clarity,
    Conciseness,
    Tone,
    Vocabulary,
    Structure,
    Punctuation,
    Spelling,
}

/// Readability analysis result
#[derive(Debug, Clone)]
pub struct ReadabilityAnalysis {
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade_level: f64,
    pub sentences: u32,
    pub words: u32,
    pub syllables: u32,
    pub average_words_per_sentence: f64,
    pub average_syllables_per_word: f64,
}

impl ReadabilityAnalysis {
    pub fn reading_level_description(&self) -> &'static str {
        match self.flesch_reading_ease {
            score if score >= 90.0 => "Very Easy",
            score if score >= 80.0 => "Easy",
            score if score >= 70.0 => "Fairly Easy",
            score if score >= 60.0 => "Standard",
            score if score >= 50.0 => "Fairly Difficult",
            score if score >= 30.0 => "Difficult",
            _ => "Very Difficult",
        }
    }

    pub fn grade_level_description(&self) -> String {
        format!("Grade {:.1}", self.flesch_kincaid_grade_level)
    }
}

/// Content analysis result
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub readability: ReadabilityAnalysis,
    pub tone_analysis: ToneAnalysis,
    pub key_points: Vec<String>,
    pub word_frequency: HashMap<String, u32>,
    pub sentiment: SentimentScore,
}

/// Tone analysis result
#[derive(Debug, Clone)]
pub struct ToneAnalysis {
    pub primary_tone: ToneAdjustment,
    pub tone_confidence: f32,
    pub tone_consistency: f32,
    pub detected_tones: Vec<(ToneAdjustment, f32)>,
}

/// Sentiment analysis score
#[derive(Debug, Clone)]
pub struct SentimentScore {
    pub polarity: f32,  // -1.0 to 1.0
    pub subjectivity: f32, // 0.0 to 1.0
    pub confidence: f32,
}

/// Token usage tracking
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub input_tokens: TokenCount,
    pub output_tokens: TokenCount,
    pub total_tokens: TokenCount,
    pub estimated_cost: f64,
}

/// Conversation session for managing multi-turn interactions
#[derive(Debug, Clone)]
pub struct ConversationSession {
    pub session_id: EntityId,
    pub document_id: EntityId,
    pub entries: Vec<ConversationEntry>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ConversationSession {
    pub fn new(document_id: EntityId) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id: EntityId::new(),
            document_id,
            entries: Vec::new(),
            created_at: now,
            last_activity: now,
        }
    }

    pub fn add_entry(&mut self, entry: ConversationEntry) {
        self.entries.push(entry);
        self.last_activity = chrono::Utc::now();
    }

    pub fn get_recent_context(&self, max_entries: usize) -> Vec<&ConversationEntry> {
        self.entries.iter().rev().take(max_entries).collect()
    }
}

/// AI Writing Service that provides writing-specific AI assistance
pub struct AIWritingService {
    orchestration_service: Arc<AIOrchestrationService>,
    context_service: Arc<ContextManagementService>,
    content_filter: Arc<ContentFilteringService>,
    conversation_sessions: Arc<RwLock<HashMap<EntityId, ConversationSession>>>,
    default_preferences: WritingPreferences,
}

impl AIWritingService {
    pub fn new(
        orchestration_service: Arc<AIOrchestrationService>,
        context_service: Arc<ContextManagementService>,
        content_filter: Arc<ContentFilteringService>,
    ) -> Self {
        Self {
            orchestration_service,
            context_service,
            content_filter,
            conversation_sessions: Arc::new(RwLock::new(HashMap::new())),
            default_preferences: WritingPreferences::default(),
        }
    }

    /// Get or create a conversation session for a document
    pub async fn get_conversation_session(&self, document_id: EntityId) -> ConversationSession {
        let mut sessions = self.conversation_sessions.write().await;
        sessions.entry(document_id)
            .or_insert_with(|| ConversationSession::new(document_id))
            .clone()
    }

    /// Provide writing assistance based on request
    pub async fn provide_assistance(
        &self,
        request: WritingAssistanceRequest,
    ) -> Result<WritingAssistanceResponse> {
        // Validate and filter content
        self.content_filter.filter_content(&request.context.document_content)?;

        // Get conversation session
        let mut session = self.get_conversation_session(request.context.document_id).await;

        // Build AI prompt based on assistance type
        let messages = self.build_messages(&request, &session).await?;

        // Configure model based on assistance type
        let model_config = request.model_config.unwrap_or_else(|| {
            self.get_default_model_config(&request.assistance_type)
        });

        // Create completion request
        let completion_request = self.build_completion_request(messages, model_config)?;

        // Get AI response
        let completion_response = self.orchestration_service
            .complete_with_fallback(completion_request)
            .await?;

        // Process and structure the response
        let assistance_response = self.process_response(
            &request,
            &completion_response,
        ).await?;

        // Update conversation session
        let conversation_entry = ConversationEntry {
            timestamp: chrono::Utc::now(),
            request_type: request.assistance_type.clone(),
            user_input: request.user_input.unwrap_or_default(),
            ai_response: assistance_response.content.clone(),
            applied: false,
        };

        session.add_entry(conversation_entry);
        self.conversation_sessions.write().await
            .insert(request.context.document_id, session);

        Ok(assistance_response)
    }

    /// Generate content based on prompt and context
    pub async fn generate_content(
        &self,
        context: WritingContext,
        prompt: String,
        target_length: Option<u32>,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::ContentGeneration,
            user_input: Some(prompt),
            model_config: target_length.map(|length| {
                ModelConfiguration::new("claude-3-5-sonnet-20241022")
                    .unwrap()
                    .with_max_tokens(length)
            }),
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Complete existing content
    pub async fn complete_content(
        &self,
        context: WritingContext,
        continuation_hint: Option<String>,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::ContentCompletion,
            user_input: continuation_hint,
            model_config: None,
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Summarize document content
    pub async fn summarize_content(
        &self,
        context: WritingContext,
        summary_length: Option<u32>,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::Summarization,
            user_input: None,
            model_config: summary_length.map(|length| {
                ModelConfiguration::new("claude-3-haiku-20240307")
                    .unwrap()
                    .with_max_tokens(length)
            }),
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Improve writing quality
    pub async fn improve_writing(
        &self,
        context: WritingContext,
        improvement_focus: Option<String>,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::Improvement,
            user_input: improvement_focus,
            model_config: None,
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Check grammar and provide corrections
    pub async fn check_grammar(
        &self,
        context: WritingContext,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::GrammarCheck,
            user_input: None,
            model_config: Some(
                ModelConfiguration::new("gpt-4")
                    .unwrap()
                    .with_temperature(0.1) // Lower temperature for grammar checking
            ),
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Adjust tone of writing
    pub async fn adjust_tone(
        &self,
        context: WritingContext,
        target_tone: ToneAdjustment,
    ) -> Result<WritingAssistanceResponse> {
        let request = WritingAssistanceRequest {
            context,
            assistance_type: WritingAssistanceType::Tone(target_tone),
            user_input: None,
            model_config: None,
            stream_response: false,
        };

        self.provide_assistance(request).await
    }

    /// Analyze content for insights
    pub async fn analyze_content(
        &self,
        context: WritingContext,
    ) -> Result<ContentAnalysis> {
        // Use a combination of local analysis and AI analysis
        let content = &context.document_content;
        
        // Local readability analysis
        let readability = self.calculate_readability(content);

        // AI-powered analysis
        let request = WritingAssistanceRequest {
            context: context.clone(),
            assistance_type: WritingAssistanceType::Improvement,
            user_input: Some("Provide detailed analysis of tone, key points, and sentiment".to_string()),
            model_config: Some(
                ModelConfiguration::new("claude-3-5-sonnet-20241022")
                    .unwrap()
                    .with_temperature(0.3)
            ),
            stream_response: false,
        };

        let response = self.provide_assistance(request).await?;
        
        // Parse AI response and combine with local analysis
        let tone_analysis = self.parse_tone_analysis(&response.content)?;
        let key_points = self.extract_key_points(&response.content)?;
        let sentiment = self.parse_sentiment(&response.content)?;
        let word_frequency = self.calculate_word_frequency(content);

        Ok(ContentAnalysis {
            readability,
            tone_analysis,
            key_points,
            word_frequency,
            sentiment,
        })
    }

    /// Build messages for AI completion based on request type
    async fn build_messages(
        &self,
        request: &WritingAssistanceRequest,
        session: &ConversationSession,
    ) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // System message with writing context and instructions
        let system_prompt = self.build_system_prompt(request)?;
        messages.push(Message::system(system_prompt));

        // Add conversation history for context
        let recent_context = session.get_recent_context(5); // Last 5 interactions
        for entry in recent_context.iter().rev() {
            if !entry.user_input.is_empty() {
                messages.push(Message::user(&entry.user_input));
            }
            messages.push(Message::assistant(&entry.ai_response));
        }

        // Add current request
        let user_message = self.build_user_message(request)?;
        messages.push(Message::user(user_message));

        // Apply context management to fit within token limits
        let managed_messages = self.context_service.manage_context(messages);

        Ok(managed_messages)
    }

    /// Build system prompt based on assistance type and context
    fn build_system_prompt(&self, request: &WritingAssistanceRequest) -> Result<String> {
        let base_prompt = format!(
            "You are an expert writing assistant helping with a {} document titled '{}'. ",
            self.format_content_type(&request.context.content_type),
            request.context.document_title
        );

        let context_prompt = if let Some(project) = &request.context.project_context {
            format!(
                "This document is part of the project '{}'. {}",
                project.project_name,
                project.project_description.as_deref().unwrap_or("")
            )
        } else {
            String::new()
        };

        let preferences_prompt = self.build_preferences_prompt(&request.context.user_preferences);
        
        let task_specific_prompt = match &request.assistance_type {
            WritingAssistanceType::ContentGeneration => {
                "Generate high-quality, contextually appropriate content. Be creative while maintaining consistency with the existing document style and tone."
            }
            WritingAssistanceType::ContentCompletion => {
                "Continue the existing content naturally and coherently. Maintain the established tone, style, and narrative flow."
            }
            WritingAssistanceType::Summarization => {
                "Create a concise, accurate summary that captures the main points and key insights of the content."
            }
            WritingAssistanceType::Improvement => {
                "Improve the writing quality by enhancing clarity, flow, structure, and impact while preserving the author's voice and intent."
            }
            WritingAssistanceType::GrammarCheck => {
                "Identify and correct grammatical errors, punctuation issues, and spelling mistakes. Provide clear explanations for corrections."
            }
            WritingAssistanceType::StyleSuggestions => {
                "Provide specific suggestions to improve writing style, word choice, sentence structure, and overall readability."
            }
            WritingAssistanceType::Brainstorming => {
                "Help generate creative ideas, explore different approaches, and expand on concepts related to the content."
            }
            WritingAssistanceType::Outline => {
                "Create a well-structured outline that organizes the content logically and effectively."
            }
            WritingAssistanceType::Rewrite => {
                "Rewrite the content to improve clarity, impact, and readability while maintaining the core message and meaning."
            }
            WritingAssistanceType::Expand => {
                "Expand the content with additional details, examples, explanations, or supporting information."
            }
            WritingAssistanceType::Condense => {
                "Condense the content while preserving all essential information and maintaining clarity."
            }
            WritingAssistanceType::Tone(tone) => {
                &format!("Adjust the tone of the writing to be more {}. Maintain the core message while adapting the style and language appropriately.", self.format_tone(tone))
            }
        };

        Ok(format!(
            "{}\n\n{}\n\n{}\n\nTask: {}",
            base_prompt,
            context_prompt,
            preferences_prompt,
            task_specific_prompt
        ))
    }

    /// Build user message with content and specific request
    fn build_user_message(&self, request: &WritingAssistanceRequest) -> Result<String> {
        let mut message = String::new();

        // Add document content or selection
        if let Some(selection) = &request.context.selection {
            if let Some(selected_text) = selection.extract_from(&request.context.document_content) {
                message.push_str(&format!("Selected text:\n\"\"\"\n{}\n\"\"\"\n\n", selected_text));
            }
        } else {
            message.push_str(&format!("Document content:\n\"\"\"\n{}\n\"\"\"\n\n", request.context.document_content));
        }

        // Add user input if provided
        if let Some(user_input) = &request.user_input {
            message.push_str(&format!("Additional instructions: {}\n\n", user_input));
        }

        // Add assistance type specific instructions
        match &request.assistance_type {
            WritingAssistanceType::ContentGeneration => {
                message.push_str("Please generate new content based on the context and any additional instructions provided.");
            }
            WritingAssistanceType::ContentCompletion => {
                message.push_str("Please continue this content naturally, maintaining the established style and tone.");
            }
            WritingAssistanceType::Summarization => {
                message.push_str("Please provide a comprehensive summary of this content.");
            }
            WritingAssistanceType::GrammarCheck => {
                message.push_str("Please identify and correct any grammatical errors, providing explanations for each correction.");
            }
            _ => {}
        }

        Ok(message)
    }

    /// Build completion request from messages and config
    fn build_completion_request(
        &self,
        messages: Vec<Message>,
        model_config: ModelConfiguration,
    ) -> Result<CompletionRequest> {
        Ok(CompletionRequest::new(messages, model_config.model_name)
            .with_max_tokens(model_config.max_tokens)
            .with_temperature(model_config.temperature)
            .with_metadata("service".to_string(), "ai_writing".to_string())
            .with_metadata("version".to_string(), "1.0".to_string()))
    }

    /// Process AI response into structured writing assistance response
    async fn process_response(
        &self,
        request: &WritingAssistanceRequest,
        completion_response: &CompletionResponse,
    ) -> Result<WritingAssistanceResponse> {
        let content = if let Some(choice) = completion_response.choices.first() {
            choice.message.content.clone()
        } else {
            return Err(WritemagicError::ai_provider("No response choices available"));
        };

        // Parse suggestions based on assistance type
        let suggestions = self.parse_suggestions(&content, &request.assistance_type)?;

        // Calculate usage and cost
        let usage = TokenUsage {
            input_tokens: TokenCount::new(completion_response.usage.prompt_tokens),
            output_tokens: TokenCount::new(completion_response.usage.completion_tokens),
            total_tokens: TokenCount::new(completion_response.usage.total_tokens),
            estimated_cost: 0.0, // Will be calculated by provider
        };

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&content, &request.assistance_type);

        Ok(WritingAssistanceResponse {
            id: completion_response.id.clone(),
            content,
            suggestions,
            analysis: None, // Filled separately by analyze_content
            usage,
            confidence_score,
            applied_to_document: false,
        })
    }

    /// Get default model configuration for assistance type
    fn get_default_model_config(&self, assistance_type: &WritingAssistanceType) -> ModelConfiguration {
        match assistance_type {
            WritingAssistanceType::ContentGeneration | WritingAssistanceType::Brainstorming => {
                ModelConfiguration::new("claude-3-5-sonnet-20241022")
                    .unwrap()
                    .with_temperature(0.8)
                    .with_max_tokens(4000)
            }
            WritingAssistanceType::GrammarCheck | WritingAssistanceType::StyleSuggestions => {
                ModelConfiguration::new("gpt-4")
                    .unwrap()
                    .with_temperature(0.1)
                    .with_max_tokens(2000)
            }
            WritingAssistanceType::Summarization => {
                ModelConfiguration::new("claude-3-haiku-20240307")
                    .unwrap()
                    .with_temperature(0.3)
                    .with_max_tokens(1000)
            }
            _ => {
                ModelConfiguration::new("claude-3-5-sonnet-20241022")
                    .unwrap()
                    .with_temperature(0.5)
                    .with_max_tokens(3000)
            }
        }
    }

    /// Helper methods for formatting and parsing
    fn format_content_type(&self, content_type: &writemagic_shared::ContentType) -> &str {
        match content_type {
            writemagic_shared::ContentType::PlainText => "plain text",
            writemagic_shared::ContentType::Markdown => "Markdown",
            writemagic_shared::ContentType::Html => "HTML",
            writemagic_shared::ContentType::RichText => "rich text",
        }
    }

    fn format_tone(&self, tone: &ToneAdjustment) -> &str {
        match tone {
            ToneAdjustment::Professional => "professional",
            ToneAdjustment::Casual => "casual",
            ToneAdjustment::Formal => "formal",
            ToneAdjustment::Creative => "creative",
            ToneAdjustment::Technical => "technical",
            ToneAdjustment::Persuasive => "persuasive",
            ToneAdjustment::Friendly => "friendly",
            ToneAdjustment::Academic => "academic",
        }
    }

    fn build_preferences_prompt(&self, preferences: &WritingPreferences) -> String {
        let mut prompt = String::new();
        
        if let Some(tone) = &preferences.preferred_tone {
            prompt.push_str(&format!("Preferred tone: {}. ", self.format_tone(tone)));
        }
        
        if let Some(audience) = &preferences.target_audience {
            prompt.push_str(&format!("Target audience: {}. ", audience));
        }
        
        prompt.push_str(&format!("Language: {}. ", preferences.language));
        prompt.push_str(&format!("Formality: {:?}. ", preferences.formality_level));
        prompt.push_str(&format!("Vocabulary level: {:?}.", preferences.vocabulary_level));
        
        prompt
    }

    fn parse_suggestions(
        &self,
        content: &str,
        assistance_type: &WritingAssistanceType,
    ) -> Result<Vec<WritingSuggestion>> {
        // Simple parsing logic - in production, this would be more sophisticated
        let mut suggestions = Vec::new();
        
        match assistance_type {
            WritingAssistanceType::GrammarCheck => {
                // Parse grammar corrections from AI response
                // This is a simplified implementation
                if content.contains("correction") || content.contains("error") {
                    suggestions.push(WritingSuggestion {
                        suggestion_type: SuggestionType::Grammar,
                        original_text: None,
                        suggested_text: content.clone(),
                        explanation: "Grammar improvements suggested".to_string(),
                        confidence: 0.8,
                        position: None,
                    });
                }
            }
            WritingAssistanceType::StyleSuggestions => {
                suggestions.push(WritingSuggestion {
                    suggestion_type: SuggestionType::Style,
                    original_text: None,
                    suggested_text: content.clone(),
                    explanation: "Style improvements suggested".to_string(),
                    confidence: 0.7,
                    position: None,
                });
            }
            _ => {}
        }
        
        Ok(suggestions)
    }

    fn calculate_confidence_score(
        &self,
        _content: &str,
        assistance_type: &WritingAssistanceType,
    ) -> f32 {
        // Simple confidence scoring - in production, this would analyze response quality
        match assistance_type {
            WritingAssistanceType::GrammarCheck => 0.9,
            WritingAssistanceType::Summarization => 0.8,
            WritingAssistanceType::ContentGeneration => 0.7,
            _ => 0.75,
        }
    }

    fn parse_tone_analysis(&self, _content: &str) -> Result<ToneAnalysis> {
        // Simplified tone analysis parsing
        Ok(ToneAnalysis {
            primary_tone: ToneAdjustment::Professional,
            tone_confidence: 0.75,
            tone_consistency: 0.8,
            detected_tones: vec![
                (ToneAdjustment::Professional, 0.75),
                (ToneAdjustment::Formal, 0.6),
            ],
        })
    }

    fn extract_key_points(&self, _content: &str) -> Result<Vec<String>> {
        // Simplified key point extraction
        Ok(vec![
            "Main concept discussed".to_string(),
            "Supporting evidence provided".to_string(),
            "Conclusion drawn".to_string(),
        ])
    }

    fn parse_sentiment(&self, _content: &str) -> Result<SentimentScore> {
        // Simplified sentiment analysis
        Ok(SentimentScore {
            polarity: 0.1,
            subjectivity: 0.5,
            confidence: 0.7,
        })
    }

    fn calculate_word_frequency(&self, content: &str) -> HashMap<String, u32> {
        let mut frequency = HashMap::new();
        
        for word in content.split_whitespace() {
            let word = word.to_lowercase();
            *frequency.entry(word).or_insert(0) += 1;
        }
        
        frequency
    }

    fn calculate_readability(&self, content: &str) -> ReadabilityAnalysis {
        let sentences = self.count_sentences(content);
        let words = self.count_words(content);
        let syllables = self.count_syllables(content);

        // Flesch Reading Ease Score
        let flesch_score = if sentences > 0 && words > 0 {
            206.835 - (1.015 * (words as f64 / sentences as f64)) - (84.6 * (syllables as f64 / words as f64))
        } else {
            0.0
        };

        // Flesch-Kincaid Grade Level
        let grade_level = if sentences > 0 && words > 0 {
            (0.39 * (words as f64 / sentences as f64)) + (11.8 * (syllables as f64 / words as f64)) - 15.59
        } else {
            0.0
        };

        ReadabilityAnalysis {
            flesch_reading_ease: flesch_score,
            flesch_kincaid_grade_level: grade_level,
            sentences,
            words,
            syllables,
            average_words_per_sentence: if sentences > 0 { words as f64 / sentences as f64 } else { 0.0 },
            average_syllables_per_word: if words > 0 { syllables as f64 / words as f64 } else { 0.0 },
        }
    }

    fn count_sentences(&self, text: &str) -> u32 {
        text.chars()
            .filter(|&c| c == '.' || c == '!' || c == '?')
            .count() as u32
    }

    fn count_words(&self, text: &str) -> u32 {
        text.split_whitespace()
            .filter(|word| !word.is_empty())
            .count() as u32
    }

    fn count_syllables(&self, text: &str) -> u32 {
        text.split_whitespace()
            .map(|word| self.count_syllables_in_word(word))
            .sum()
    }

    fn count_syllables_in_word(&self, word: &str) -> u32 {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u'];
        let mut syllable_count = 0;
        let mut prev_was_vowel = false;

        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_was_vowel {
                syllable_count += 1;
            }
            prev_was_vowel = is_vowel;
        }

        // Adjust for silent 'e'
        if word.ends_with('e') && syllable_count > 1 {
            syllable_count -= 1;
        }

        // Every word has at least one syllable
        std::cmp::max(1, syllable_count)
    }
}

/// Default implementations
impl Default for WritingPreferences {
    fn default() -> Self {
        Self {
            preferred_tone: None,
            target_audience: None,
            writing_style: None,
            language: "en".to_string(),
            formality_level: FormalityLevel::Neutral,
            vocabulary_level: VocabularyLevel::Intermediate,
        }
    }
}

impl ModelConfiguration {
    fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_mock_writing_context() -> WritingContext {
        WritingContext {
            document_id: EntityId::new(),
            document_title: "Test Document".to_string(),
            document_content: "This is a test document content.".to_string(),
            content_type: writemagic_shared::ContentType::Markdown,
            selection: None,
            project_context: None,
            conversation_history: Vec::new(),
            user_preferences: WritingPreferences::default(),
        }
    }

    #[tokio::test]
    async fn test_conversation_session_creation() {
        let document_id = EntityId::new();
        let session = ConversationSession::new(document_id);
        
        assert_eq!(session.document_id, document_id);
        assert!(session.entries.is_empty());
    }

    #[test]
    fn test_writing_preferences_default() {
        let preferences = WritingPreferences::default();
        assert_eq!(preferences.language, "en");
        assert_eq!(preferences.formality_level, FormalityLevel::Neutral);
        assert_eq!(preferences.vocabulary_level, VocabularyLevel::Intermediate);
    }

    #[test]
    fn test_text_selection_in_context() {
        let selection = TextSelection::new(0, 10).unwrap();
        let content = "This is a test document content.";
        let extracted = selection.extract_from(content);
        
        assert_eq!(extracted, Some("This is a ".to_string()));
    }
}