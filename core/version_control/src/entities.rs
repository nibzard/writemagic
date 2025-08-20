//! Version control domain entities

use writemagic_shared::EntityId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use garde::Validate;

/// A commit in the version control system
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Commit {
    #[garde(skip)]
    pub id: EntityId,
    #[garde(skip)]
    pub document_id: EntityId,
    #[garde(skip)]
    pub parent_id: Option<EntityId>,
    #[garde(length(min = 1, max = 500))]
    pub message: String,
    #[garde(skip)]
    pub author: EntityId,
    #[garde(skip)]
    pub timestamp: DateTime<Utc>,
    #[garde(skip)]
    pub content_hash: String,
    #[garde(skip)]
    pub changes: Vec<Change>,
    #[garde(skip)]
    pub metadata: CommitMetadata,
}

/// Metadata associated with a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub word_count: usize,
    pub character_count: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_modified: usize,
    pub tags: Vec<String>,
    pub branch: Option<String>,
}

/// A change within a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_type: ChangeType,
    pub line_number: usize,
    pub content_before: Option<String>,
    pub content_after: Option<String>,
    pub position: ChangePosition,
}

/// Position information for a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePosition {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

/// Types of changes that can occur
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Addition,
    Deletion,
    Modification,
    Move,
}

/// A branch in the version control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: EntityId,
    pub document_id: EntityId,
    pub name: String,
    pub head_commit_id: EntityId,
    pub created_by: EntityId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_default: bool,
    pub is_protected: bool,
    pub description: Option<String>,
}

/// A tag pointing to a specific commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: EntityId,
    pub name: String,
    pub commit_id: EntityId,
    pub document_id: EntityId,
    pub created_by: EntityId,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub tag_type: TagType,
}

/// Types of tags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagType {
    Release,
    Milestone,
    Draft,
    Review,
    Archive,
}

/// A diff between two versions of content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    pub id: EntityId,
    pub source_commit_id: EntityId,
    pub target_commit_id: EntityId,
    pub document_id: EntityId,
    pub created_at: DateTime<Utc>,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

/// Statistics about a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_modified: usize,
    pub words_added: usize,
    pub words_removed: usize,
    pub characters_added: usize,
    pub characters_removed: usize,
}

/// A hunk is a contiguous block of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub start_line_old: usize,
    pub start_line_new: usize,
    pub lines_old: usize,
    pub lines_new: usize,
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub line_number_old: Option<usize>,
    pub line_number_new: Option<usize>,
}

/// Types of lines in a diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,  // Unchanged line
    Addition, // Added line
    Deletion, // Removed line
}

/// Timeline entry for version control history visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id: EntityId,
    pub document_id: EntityId,
    pub entry_type: TimelineEntryType,
    pub timestamp: DateTime<Utc>,
    pub author: EntityId,
    pub title: String,
    pub description: Option<String>,
    pub related_commit_id: Option<EntityId>,
    pub related_branch_id: Option<EntityId>,
    pub related_tag_id: Option<EntityId>,
    pub metadata: HashMap<String, String>,
}

/// Types of timeline entries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineEntryType {
    Commit,
    BranchCreated,
    BranchMerged,
    TagCreated,
    MergeConflict,
    DocumentCreated,
    MajorEdit,
    Review,
    Milestone,
}

impl Commit {
    /// Create a new commit
    pub fn new(
        document_id: EntityId,
        parent_id: Option<EntityId>,
        message: String,
        author: EntityId,
        content_hash: String,
        changes: Vec<Change>,
    ) -> Self {
        let now = Utc::now();
        
        // Calculate metadata from changes
        let mut word_count = 0;
        let mut character_count = 0;
        let mut lines_added = 0;
        let mut lines_removed = 0;
        let mut lines_modified = 0;
        
        for change in &changes {
            match change.change_type {
                ChangeType::Addition => {
                    lines_added += 1;
                    if let Some(content) = &change.content_after {
                        word_count += content.split_whitespace().count();
                        character_count += content.len();
                    }
                },
                ChangeType::Deletion => {
                    lines_removed += 1;
                },
                ChangeType::Modification => {
                    lines_modified += 1;
                    if let Some(content) = &change.content_after {
                        word_count += content.split_whitespace().count();
                        character_count += content.len();
                    }
                },
                ChangeType::Move => {
                    lines_modified += 1;
                },
            }
        }
        
        Self {
            id: EntityId::new(),
            document_id,
            parent_id,
            message,
            author,
            timestamp: now,
            content_hash,
            changes,
            metadata: CommitMetadata {
                word_count,
                character_count,
                lines_added,
                lines_removed,
                lines_modified,
                tags: Vec::new(),
                branch: None,
            },
        }
    }
    
    /// Get the commit's ancestry depth
    pub fn get_ancestry_depth(&self, all_commits: &HashMap<EntityId, Commit>) -> usize {
        let mut depth = 0;
        let mut current_id = self.parent_id;
        
        while let Some(parent_id) = current_id {
            if let Some(parent_commit) = all_commits.get(&parent_id) {
                depth += 1;
                current_id = parent_commit.parent_id;
            } else {
                break;
            }
        }
        
        depth
    }
    
    /// Check if this commit is an ancestor of another commit
    pub fn is_ancestor_of(&self, other: &Commit, all_commits: &HashMap<EntityId, Commit>) -> bool {
        let mut current_id = other.parent_id;
        
        while let Some(parent_id) = current_id {
            if parent_id == self.id {
                return true;
            }
            
            if let Some(parent_commit) = all_commits.get(&parent_id) {
                current_id = parent_commit.parent_id;
            } else {
                break;
            }
        }
        
        false
    }
}

impl Branch {
    /// Create a new branch
    pub fn new(
        document_id: EntityId,
        name: String,
        head_commit_id: EntityId,
        created_by: EntityId,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: EntityId::new(),
            document_id,
            name,
            head_commit_id,
            created_by,
            created_at: now,
            updated_at: now,
            is_default: false,
            is_protected: false,
            description: None,
        }
    }
    
    /// Update the branch head to a new commit
    pub fn update_head(&mut self, commit_id: EntityId) {
        self.head_commit_id = commit_id;
        self.updated_at = Utc::now();
    }
}

impl Tag {
    /// Create a new tag
    pub fn new(
        name: String,
        commit_id: EntityId,
        document_id: EntityId,
        created_by: EntityId,
        tag_type: TagType,
    ) -> Self {
        Self {
            id: EntityId::new(),
            name,
            commit_id,
            document_id,
            created_by,
            created_at: Utc::now(),
            description: None,
            tag_type,
        }
    }
}

impl Diff {
    /// Create a new diff between two commits
    pub fn new(
        source_commit_id: EntityId,
        target_commit_id: EntityId,
        document_id: EntityId,
        hunks: Vec<DiffHunk>,
    ) -> Self {
        // Calculate diff statistics
        let mut stats = DiffStats {
            files_changed: 1, // For now, assuming single document
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
            words_added: 0,
            words_removed: 0,
            characters_added: 0,
            characters_removed: 0,
        };
        
        for hunk in &hunks {
            for line in &hunk.lines {
                match line.line_type {
                    DiffLineType::Addition => {
                        stats.lines_added += 1;
                        stats.words_added += line.content.split_whitespace().count();
                        stats.characters_added += line.content.len();
                    },
                    DiffLineType::Deletion => {
                        stats.lines_removed += 1;
                        stats.words_removed += line.content.split_whitespace().count();
                        stats.characters_removed += line.content.len();
                    },
                    DiffLineType::Context => {
                        // Context lines don't affect stats
                    },
                }
            }
        }
        
        Self {
            id: EntityId::new(),
            source_commit_id,
            target_commit_id,
            document_id,
            created_at: Utc::now(),
            hunks,
            stats,
        }
    }
    
    /// Check if this is a clean diff (no conflicts)
    pub fn is_clean(&self) -> bool {
        !self.hunks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_commit() {
        let changes = vec![
            Change {
                change_type: ChangeType::Addition,
                line_number: 1,
                content_before: None,
                content_after: Some("Hello world".to_string()),
                position: ChangePosition {
                    start_line: 1,
                    end_line: 1,
                    start_column: 0,
                    end_column: 11,
                },
            }
        ];
        
        let commit = Commit::new(
            EntityId::new(),
            None,
            "Initial commit".to_string(),
            EntityId::new(),
            "hash123".to_string(),
            changes,
        );
        
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.metadata.lines_added, 1);
        assert_eq!(commit.metadata.word_count, 2);
    }
}
