//! Writing domain services

use async_trait::async_trait;
use writemagic_shared::{EntityId, DomainService, Result, WritemagicError};
use crate::aggregates::{DocumentAggregate, ProjectAggregate};
use crate::entities::{Document, Project};
use crate::value_objects::{DocumentTitle, DocumentContent, ProjectName, TextSelection};
use crate::repositories::{DocumentRepository, ProjectRepository};
use std::sync::Arc;

/// Document management service
pub struct DocumentManagementService {
    document_repository: Arc<dyn DocumentRepository>,
}

impl DocumentManagementService {
    pub fn new(document_repository: Arc<dyn DocumentRepository>) -> Self {
        Self {
            document_repository,
        }
    }

    pub async fn create_document(
        &self,
        title: DocumentTitle,
        content: DocumentContent,
        content_type: writemagic_shared::ContentType,
        created_by: Option<EntityId>,
    ) -> Result<DocumentAggregate> {
        // Create new document aggregate
        let mut aggregate = DocumentAggregate::new(title, content, content_type, created_by);

        // Save to repository
        let document = self.document_repository.save(aggregate.document()).await?;
        
        // Update aggregate with saved document
        *aggregate = DocumentAggregate::load_from_document(document);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }

    pub async fn update_document_content(
        &self,
        document_id: EntityId,
        content: DocumentContent,
        selection: Option<TextSelection>,
        updated_by: Option<EntityId>,
    ) -> Result<DocumentAggregate> {
        // Load existing document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Create aggregate and update content
        let mut aggregate = DocumentAggregate::load_from_document(document);
        aggregate.update_content(content, selection, updated_by)?;

        // Save changes
        let updated_document = self.document_repository.save(aggregate.document()).await?;
        
        // Update aggregate with saved document
        *aggregate = DocumentAggregate::load_from_document(updated_document);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }

    pub async fn delete_document(
        &self,
        document_id: EntityId,
        deleted_by: Option<EntityId>,
    ) -> Result<()> {
        // Load existing document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Create aggregate and delete
        let mut aggregate = DocumentAggregate::load_from_document(document);
        aggregate.delete(deleted_by)?;

        // Save changes
        self.document_repository.save(aggregate.document()).await?;

        Ok(())
    }

    pub async fn restore_document(
        &self,
        document_id: EntityId,
        restored_by: Option<EntityId>,
    ) -> Result<DocumentAggregate> {
        // Load existing document
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Create aggregate and restore
        let mut aggregate = DocumentAggregate::load_from_document(document);
        aggregate.restore(restored_by)?;

        // Save changes
        let updated_document = self.document_repository.save(aggregate.document()).await?;
        
        // Update aggregate with saved document
        *aggregate = DocumentAggregate::load_from_document(updated_document);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }
}

/// Project management service
pub struct ProjectManagementService {
    project_repository: Arc<dyn ProjectRepository>,
    document_repository: Arc<dyn DocumentRepository>,
}

impl ProjectManagementService {
    pub fn new(
        project_repository: Arc<dyn ProjectRepository>,
        document_repository: Arc<dyn DocumentRepository>,
    ) -> Self {
        Self {
            project_repository,
            document_repository,
        }
    }

    pub async fn create_project(
        &self,
        name: ProjectName,
        description: Option<String>,
        created_by: Option<EntityId>,
    ) -> Result<ProjectAggregate> {
        // Create new project aggregate
        let mut aggregate = ProjectAggregate::new(name, description, created_by);

        // Save to repository
        let project = self.project_repository.save(aggregate.project()).await?;
        
        // Update aggregate with saved project
        *aggregate = ProjectAggregate::load_from_project(project);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }

    pub async fn add_document_to_project(
        &self,
        project_id: EntityId,
        document_id: EntityId,
        updated_by: Option<EntityId>,
    ) -> Result<ProjectAggregate> {
        // Load existing project
        let project = self.project_repository
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Project not found"))?;

        // Verify document exists
        let document = self.document_repository
            .find_by_id(&document_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Document not found"))?;

        // Create aggregate and add document
        let mut aggregate = ProjectAggregate::load_from_project(project);
        aggregate.add_document(document_id, document.title, updated_by)?;

        // Save changes
        let updated_project = self.project_repository.save(aggregate.project()).await?;
        
        // Update aggregate with saved project
        *aggregate = ProjectAggregate::load_from_project(updated_project);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }

    pub async fn remove_document_from_project(
        &self,
        project_id: EntityId,
        document_id: EntityId,
        updated_by: Option<EntityId>,
    ) -> Result<ProjectAggregate> {
        // Load existing project
        let project = self.project_repository
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Project not found"))?;

        // Create aggregate and remove document
        let mut aggregate = ProjectAggregate::load_from_project(project);
        aggregate.remove_document(&document_id, updated_by)?;

        // Save changes
        let updated_project = self.project_repository.save(aggregate.project()).await?;
        
        // Update aggregate with saved project
        *aggregate = ProjectAggregate::load_from_project(updated_project);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }

    pub async fn update_project_name(
        &self,
        project_id: EntityId,
        name: ProjectName,
        updated_by: Option<EntityId>,
    ) -> Result<ProjectAggregate> {
        // Load existing project
        let project = self.project_repository
            .find_by_id(&project_id)
            .await?
            .ok_or_else(|| WritemagicError::repository("Project not found"))?;

        // Create aggregate and update name
        let mut aggregate = ProjectAggregate::load_from_project(project);
        aggregate.update_name(name, updated_by)?;

        // Save changes
        let updated_project = self.project_repository.save(aggregate.project()).await?;
        
        // Update aggregate with saved project
        *aggregate = ProjectAggregate::load_from_project(updated_project);
        aggregate.mark_events_as_committed();

        Ok(aggregate)
    }
}

/// Content analysis service
pub struct ContentAnalysisService;

impl ContentAnalysisService {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_readability(&self, content: &DocumentContent) -> ReadabilityAnalysis {
        let text = content.as_str();
        let sentences = self.count_sentences(text);
        let words = content.word_count().value();
        let syllables = self.count_syllables(text);

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

impl Default for ContentAnalysisService {
    fn default() -> Self {
        Self::new()
    }
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