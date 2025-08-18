//! Writing domain value objects

use serde::{Deserialize, Serialize};
use validator::Validate;
use writemagic_shared::{ValueObject, Result, WritemagicError};

/// Word count value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WordCount(pub u32);

impl WordCount {
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn decrement(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }

    pub fn add(&mut self, count: u32) {
        self.0 += count;
    }

    pub fn subtract(&mut self, count: u32) {
        self.0 = self.0.saturating_sub(count);
    }
}

impl ValueObject for WordCount {}

impl Default for WordCount {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for WordCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} words", self.0)
    }
}

/// Character count value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterCount(pub u32);

impl CharacterCount {
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl ValueObject for CharacterCount {}

impl Default for CharacterCount {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for CharacterCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} characters", self.0)
    }
}

/// Document title value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct DocumentTitle {
    #[validate(length(min = 1, max = 255))]
    pub value: String,
}

impl DocumentTitle {
    pub fn new(title: impl Into<String>) -> Result<Self> {
        let title = title.into().trim().to_string();
        let document_title = Self { value: title };
        document_title.validate().map_err(|e| {
            WritemagicError::validation(format!("Invalid document title: {}", e))
        })?;
        Ok(document_title)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl ValueObject for DocumentTitle {}

impl std::fmt::Display for DocumentTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Project name value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct ProjectName {
    #[validate(length(min = 1, max = 100))]
    pub value: String,
}

impl ProjectName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into().trim().to_string();
        let project_name = Self { value: name };
        project_name.validate().map_err(|e| {
            WritemagicError::validation(format!("Invalid project name: {}", e))
        })?;
        Ok(project_name)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl ValueObject for ProjectName {}

impl std::fmt::Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Document content value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct DocumentContent {
    #[validate(length(max = 10485760))] // 10MB max
    pub value: String,
}

impl DocumentContent {
    pub fn new(content: impl Into<String>) -> Result<Self> {
        let content = content.into();
        let document_content = Self { value: content };
        document_content.validate().map_err(|e| {
            WritemagicError::validation(format!("Invalid document content: {}", e))
        })?;
        Ok(document_content)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.trim().is_empty()
    }

    pub fn word_count(&self) -> WordCount {
        let count = self.value
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .count() as u32;
        WordCount::new(count)
    }

    pub fn character_count(&self) -> CharacterCount {
        CharacterCount::new(self.value.len() as u32)
    }

    pub fn character_count_no_spaces(&self) -> CharacterCount {
        let count = self.value.chars().filter(|c| !c.is_whitespace()).count() as u32;
        CharacterCount::new(count)
    }

    pub fn append(&mut self, text: &str) {
        self.value.push_str(text);
    }

    pub fn prepend(&mut self, text: &str) {
        self.value = format!("{}{}", text, self.value);
    }

    pub fn insert_at(&mut self, position: usize, text: &str) {
        if position <= self.value.len() {
            self.value.insert_str(position, text);
        }
    }

    pub fn replace_range(&mut self, range: std::ops::Range<usize>, text: &str) {
        if range.start <= self.value.len() && range.end <= self.value.len() && range.start <= range.end {
            self.value.replace_range(range, text);
        }
    }
}

impl ValueObject for DocumentContent {}

impl std::fmt::Display for DocumentContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value.len() > 100 {
            write!(f, "{}...", &self.value[..100])
        } else {
            write!(f, "{}", self.value)
        }
    }
}

/// Text selection value object for editing operations
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

    pub fn length(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    pub fn contains(&self, position: usize) -> bool {
        position >= self.start && position <= self.end
    }

    pub fn extract_from(&self, content: &str) -> Option<String> {
        if self.end <= content.len() {
            Some(content[self.start..self.end].to_string())
        } else {
            None
        }
    }
}

impl ValueObject for TextSelection {}

impl std::fmt::Display for TextSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_cursor() {
            write!(f, "cursor at {}", self.start)
        } else {
            write!(f, "selection {}..{}", self.start, self.end)
        }
    }
}