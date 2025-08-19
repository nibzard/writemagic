//! Integration layer between AI writing service and writing domain

use async_trait::async_trait;
use std::sync::Arc;
use writemagic_shared::{EntityId, Result, WritemagicError};

use crate::entities::{Document, Project};
use crate::repositories::{DocumentRepository, ProjectRepository};
use crate::services::{DocumentManagementService, ProjectManagementService, ContentAnalysisService};
use crate::value_objects::{DocumentContent, DocumentTitle};

use writemagic_ai::{
    AIWritingService, 
    WritingContext, 
    ProjectContext, 
    WritingAssistanceRequest, 
    WritingAssistanceResponse,
    WritingAssistanceType,
    WritingPreferences,
    ToneAdjustment,
    ConversationSession,
    ContentAnalysis,
    RelatedDocument,
    TextSelection, // Import from AI module
};

/// Integrated writing assistance service that combines AI with document management
pub struct IntegratedWritingService {
    ai_writing_service: Arc<AIWritingService>,
    document_service: Arc<DocumentManagementService>,
    project_service: Arc<ProjectManagementService>,
    content_analysis_service: Arc<ContentAnalysisService>,
    document_repository: Arc<dyn DocumentRepository>,
    project_repository: Arc<dyn ProjectRepository>,
}

impl IntegratedWritingService {
    pub fn new(
        ai_writing_service: Arc<AIWritingService>,
        document_service: Arc<DocumentManagementService>,
        project_service: Arc<ProjectManagementService>,
        content_analysis_service: Arc<ContentAnalysisService>,
        document_repository: Arc<dyn DocumentRepository>,
        project_repository: Arc<dyn ProjectRepository>,
    ) -> Self {
        Self {
            ai_writing_service,
            document_service,
            project_service,
            content_analysis_service,
            document_repository,
            project_repository,
        }
    }

    /// Generate content and optionally apply it to a document
    pub async fn generate_content_for_document(
        &self,
        document_id: EntityId,
        prompt: String,
        target_length: Option<u32>,
        apply_to_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, None, None).await?;

        // Generate content
        let mut response = self.ai_writing_service
            .generate_content(context, prompt, target_length)
            .await?;

        // Apply to document if requested
        if apply_to_document {
            let new_content = if document.content.is_empty() {
                response.content.clone()
            } else {
                format!("{}\n\n{}", document.content, response.content)
            };

            let content = DocumentContent::new(new_content)?;
            self.document_service
                .update_document_content(document_id, content, None, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Complete existing content in a document
    pub async fn complete_document_content(
        &self,
        document_id: EntityId,
        selection: Option<TextSelection>,
        continuation_hint: Option<String>,
        apply_to_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, selection.clone(), None).await?;

        // Complete content
        let mut response = self.ai_writing_service
            .complete_content(context, continuation_hint)
            .await?;

        // Apply to document if requested
        if apply_to_document {
            let mut new_content = document.content.clone();
            
            if let Some(sel) = &selection {
                // Insert at selection position
                if sel.end <= new_content.len() {
                    new_content.insert_str(sel.end, &response.content);
                }
            } else {
                // Append to end
                new_content.push_str(&response.content);
            }

            let content = DocumentContent::new(new_content)?;
            self.document_service
                .update_document_content(document_id, content, selection, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Improve existing document content
    pub async fn improve_document_content(
        &self,
        document_id: EntityId,
        selection: Option<TextSelection>,
        improvement_focus: Option<String>,
        apply_to_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, selection.clone(), None).await?;

        // Improve content
        let mut response = self.ai_writing_service
            .improve_writing(context, improvement_focus)
            .await?;

        // Apply to document if requested
        if apply_to_document {
            let content = if let Some(sel) = &selection {
                // Replace selected content
                let mut new_content = document.content.clone();
                if sel.start <= new_content.len() && sel.end <= new_content.len() {
                    new_content.replace_range(sel.start..sel.end, &response.content);
                }
                new_content
            } else {
                // Replace entire content
                response.content.clone()
            };

            let content = DocumentContent::new(content)?;
            self.document_service
                .update_document_content(document_id, content, selection, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Summarize document content
    pub async fn summarize_document(
        &self,
        document_id: EntityId,
        summary_length: Option<u32>,
        create_summary_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, None, None).await?;

        // Generate summary
        let mut response = self.ai_writing_service
            .summarize_content(context, summary_length)
            .await?;

        // Create summary document if requested
        if create_summary_document {
            let summary_title = DocumentTitle::new(format!("Summary of {}", document.title))?;
            let summary_content = DocumentContent::new(response.content.clone())?;
            
            let summary_doc = self.document_service
                .create_document(summary_title, summary_content, document.content_type.clone(), updated_by)
                .await?;

            response.applied_to_document = true;
            // Store summary document ID in metadata
            response.content = format!("Summary created as document: {}\n\n{}", summary_doc.document().id, response.content);
        }

        Ok(response)
    }

    /// Check grammar and provide corrections
    pub async fn check_document_grammar(
        &self,
        document_id: EntityId,
        selection: Option<TextSelection>,
        apply_corrections: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, selection.clone(), None).await?;

        // Check grammar
        let mut response = self.ai_writing_service
            .check_grammar(context)
            .await?;

        // Apply corrections if requested
        if apply_corrections && !response.suggestions.is_empty() {
            // For simplicity, apply the main suggestion content
            let content = if let Some(sel) = &selection {
                // Replace selected content
                let mut new_content = document.content.clone();
                if sel.start <= new_content.len() && sel.end <= new_content.len() {
                    new_content.replace_range(sel.start..sel.end, &response.content);
                }
                new_content
            } else {
                // Use the corrected content
                response.content.clone()
            };

            let content = DocumentContent::new(content)?;
            self.document_service
                .update_document_content(document_id, content, selection, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Adjust document tone
    pub async fn adjust_document_tone(
        &self,
        document_id: EntityId,
        target_tone: ToneAdjustment,
        selection: Option<TextSelection>,
        apply_to_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, selection.clone(), None).await?;

        // Adjust tone
        let mut response = self.ai_writing_service
            .adjust_tone(context, target_tone)
            .await?;

        // Apply to document if requested
        if apply_to_document {
            let content = if let Some(sel) = &selection {
                // Replace selected content
                let mut new_content = document.content.clone();
                if sel.start <= new_content.len() && sel.end <= new_content.len() {
                    new_content.replace_range(sel.start..sel.end, &response.content);
                }
                new_content
            } else {
                // Replace entire content
                response.content.clone()
            };

            let content = DocumentContent::new(content)?;
            self.document_service
                .update_document_content(document_id, content, selection, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Analyze document content comprehensively
    pub async fn analyze_document_content(
        &self,
        document_id: EntityId,
        selection: Option<TextSelection>,
    ) -> Result<ContentAnalysis> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context
        let context = self.build_writing_context(&document, selection, None).await?;

        // Analyze content
        self.ai_writing_service
            .analyze_content(context)
            .await
    }

    /// Get conversation session for a document
    pub async fn get_document_conversation_session(
        &self,
        document_id: EntityId,
    ) -> Result<ConversationSession> {
        // Verify document exists
        let _document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        Ok(self.ai_writing_service.get_conversation_session(document_id).await)
    }

    /// Custom writing assistance with full request control
    pub async fn provide_custom_assistance(
        &self,
        document_id: EntityId,
        assistance_type: WritingAssistanceType,
        user_input: Option<String>,
        selection: Option<TextSelection>,
        preferences: Option<WritingPreferences>,
        apply_to_document: bool,
        updated_by: Option<EntityId>,
    ) -> Result<WritingAssistanceResponse> {
        // Load document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Build writing context with custom preferences
        let mut context = self.build_writing_context(&document, selection.clone(), None).await?;
        if let Some(prefs) = preferences {
            context.user_preferences = prefs;
        }

        // Create assistance request
        let request = WritingAssistanceRequest {
            context,
            assistance_type: assistance_type.clone(),
            user_input,
            model_config: None,
            stream_response: false,
        };

        // Get assistance
        let mut response = self.ai_writing_service
            .provide_assistance(request)
            .await?;

        // Apply to document if requested and content should be applied
        if apply_to_document && self.should_apply_content(&assistance_type) {
            let content = if let Some(sel) = &selection {
                // Replace selected content or insert at position
                let mut new_content = document.content.clone();
                if matches!(assistance_type, WritingAssistanceType::ContentGeneration | WritingAssistanceType::ContentCompletion) {
                    // Insert new content
                    if sel.end <= new_content.len() {
                        new_content.insert_str(sel.end, &response.content);
                    }
                } else {
                    // Replace selected content
                    if sel.start <= new_content.len() && sel.end <= new_content.len() {
                        new_content.replace_range(sel.start..sel.end, &response.content);
                    }
                }
                new_content
            } else {
                match assistance_type {
                    WritingAssistanceType::ContentGeneration | WritingAssistanceType::ContentCompletion => {
                        // Append to document
                        format!("{}\n\n{}", document.content, response.content)
                    }
                    _ => {
                        // Replace entire content
                        response.content.clone()
                    }
                }
            };

            let content = DocumentContent::new(content)?;
            self.document_service
                .update_document_content(document_id, content, selection, updated_by)
                .await?;

            response.applied_to_document = true;
        }

        Ok(response)
    }

    /// Build writing context from document and optional project
    async fn build_writing_context(
        &self,
        document: &Document,
        selection: Option<TextSelection>,
        project_id: Option<EntityId>,
    ) -> Result<WritingContext> {
        // Build project context if document is part of a project or project_id is provided
        let project_context = if let Some(pid) = project_id {
            self.build_project_context(pid).await?
        } else {
            // Try to find project containing this document
            self.find_project_for_document(document.id).await?
        };

        Ok(WritingContext {
            document_id: document.id,
            document_title: document.title.clone(),
            document_content: document.content.clone(),
            content_type: document.content_type.clone(),
            selection,
            project_context,
            conversation_history: Vec::new(), // Will be populated by AI service
            user_preferences: WritingPreferences::default(),
        })
    }

    /// Build project context from project ID
    async fn build_project_context(&self, project_id: EntityId) -> Result<Option<ProjectContext>> {
        if let Some(project) = self.project_repository.find_by_id(&project_id).await? {
            // Get related documents (limit to first 5 for context)
            let mut related_documents = Vec::new();
            for doc_id in project.document_ids.iter().take(5) {
                if let Some(doc) = self.document_repository.find_by_id(doc_id).await? {
                    let excerpt = if doc.content.len() > 200 {
                        format!("{}...", &doc.content[..200])
                    } else {
                        doc.content.clone()
                    };

                    related_documents.push(RelatedDocument {
                        id: doc.id,
                        title: doc.title,
                        content_excerpt: excerpt,
                    });
                }
            }

            Ok(Some(ProjectContext {
                project_id: project.id,
                project_name: project.name,
                project_description: project.description,
                related_documents,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find project containing a document
    async fn find_project_for_document(&self, document_id: EntityId) -> Result<Option<ProjectContext>> {
        // This would require a more sophisticated query or index
        // For now, we'll skip this optimization
        Ok(None)
    }

    /// Determine if content should be applied to document for a given assistance type
    fn should_apply_content(&self, assistance_type: &WritingAssistanceType) -> bool {
        matches!(
            assistance_type,
            WritingAssistanceType::ContentGeneration |
            WritingAssistanceType::ContentCompletion |
            WritingAssistanceType::Improvement |
            WritingAssistanceType::Rewrite |
            WritingAssistanceType::Expand |
            WritingAssistanceType::Condense |
            WritingAssistanceType::Tone(_) |
            WritingAssistanceType::GrammarCheck
        )
    }
}

/// Builder for integrated writing service
pub struct IntegratedWritingServiceBuilder {
    ai_writing_service: Option<Arc<AIWritingService>>,
    document_service: Option<Arc<DocumentManagementService>>,
    project_service: Option<Arc<ProjectManagementService>>,
    content_analysis_service: Option<Arc<ContentAnalysisService>>,
    document_repository: Option<Arc<dyn DocumentRepository>>,
    project_repository: Option<Arc<dyn ProjectRepository>>,
}

impl IntegratedWritingServiceBuilder {
    pub fn new() -> Self {
        Self {
            ai_writing_service: None,
            document_service: None,
            project_service: None,
            content_analysis_service: None,
            document_repository: None,
            project_repository: None,
        }
    }

    pub fn with_ai_writing_service(mut self, service: Arc<AIWritingService>) -> Self {
        self.ai_writing_service = Some(service);
        self
    }

    pub fn with_document_service(mut self, service: Arc<DocumentManagementService>) -> Self {
        self.document_service = Some(service);
        self
    }

    pub fn with_project_service(mut self, service: Arc<ProjectManagementService>) -> Self {
        self.project_service = Some(service);
        self
    }

    pub fn with_content_analysis_service(mut self, service: Arc<ContentAnalysisService>) -> Self {
        self.content_analysis_service = Some(service);
        self
    }

    pub fn with_document_repository(mut self, repo: Arc<dyn DocumentRepository>) -> Self {
        self.document_repository = Some(repo);
        self
    }

    pub fn with_project_repository(mut self, repo: Arc<dyn ProjectRepository>) -> Self {
        self.project_repository = Some(repo);
        self
    }

    pub fn build(self) -> Result<IntegratedWritingService> {
        Ok(IntegratedWritingService::new(
            self.ai_writing_service
                .ok_or_else(|| WritemagicError::configuration("AI writing service is required"))?,
            self.document_service
                .ok_or_else(|| WritemagicError::configuration("Document service is required"))?,
            self.project_service
                .ok_or_else(|| WritemagicError::configuration("Project service is required"))?,
            self.content_analysis_service
                .ok_or_else(|| WritemagicError::configuration("Content analysis service is required"))?,
            self.document_repository
                .ok_or_else(|| WritemagicError::configuration("Document repository is required"))?,
            self.project_repository
                .ok_or_else(|| WritemagicError::configuration("Project repository is required"))?,
        ))
    }
}

impl Default for IntegratedWritingServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writemagic_shared::{ContentType, EntityId};
    use crate::entities::Document;

    fn create_test_document() -> Document {
        Document::new(
            "Test Document".to_string(),
            "This is a test document content.".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        )
    }

    #[test]
    fn test_should_apply_content_logic() {
        // Test the logic for determining when to apply content
        // This is a simple test without needing the full service
        
        fn should_apply_content(assistance_type: &WritingAssistanceType) -> bool {
            matches!(
                assistance_type,
                WritingAssistanceType::ContentGeneration |
                WritingAssistanceType::ContentCompletion |
                WritingAssistanceType::Improvement |
                WritingAssistanceType::Rewrite |
                WritingAssistanceType::Expand |
                WritingAssistanceType::Condense |
                WritingAssistanceType::Tone(_) |
                WritingAssistanceType::GrammarCheck
            )
        }
        
        assert!(should_apply_content(&WritingAssistanceType::ContentGeneration));
        assert!(should_apply_content(&WritingAssistanceType::GrammarCheck));
        assert!(!should_apply_content(&WritingAssistanceType::Brainstorming));
    }

    #[tokio::test]
    async fn test_text_selection_extraction() {
        let content = "This is a test document content.";
        let selection = TextSelection::new(0, 10).unwrap();
        let extracted = selection.extract_from(content);
        
        assert_eq!(extracted, Some("This is a ".to_string()));
    }
}