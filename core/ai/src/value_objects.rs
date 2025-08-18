//! AI domain value objects

use serde::{Deserialize, Serialize};
use writemagic_shared::{ValueObject, Result, WritemagicError};
use validator::Validate;

/// AI prompt value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct Prompt {
    #[validate(length(min = 1, max = 100000))]
    pub content: String,
    pub template_variables: std::collections::HashMap<String, String>,
}

impl Prompt {
    pub fn new(content: impl Into<String>) -> Result<Self> {
        let content = content.into();
        let prompt = Self {
            content,
            template_variables: std::collections::HashMap::new(),
        };
        prompt.validate().map_err(|e| {
            WritemagicError::validation(format!("Invalid prompt: {}", e))
        })?;
        Ok(prompt)
    }

    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.template_variables.insert(key.into(), value.into());
        self
    }

    pub fn render(&self) -> String {
        let mut content = self.content.clone();
        for (key, value) in &self.template_variables {
            content = content.replace(&format!("{{{}}}", key), value);
        }
        content
    }
}

impl ValueObject for Prompt {}

/// Model configuration value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct ModelConfiguration {
    pub model_name: String,
    #[validate(range(min = 1, max = 100000))]
    pub max_tokens: u32,
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: f32,
    #[validate(range(min = 0.0, max = 1.0))]
    pub top_p: f32,
    #[validate(range(min = -2.0, max = 2.0))]
    pub frequency_penalty: f32,
    #[validate(range(min = -2.0, max = 2.0))]
    pub presence_penalty: f32,
}

impl ModelConfiguration {
    pub fn new(model_name: impl Into<String>) -> Result<Self> {
        let config = Self {
            model_name: model_name.into(),
            max_tokens: 4096,
            temperature: 0.7,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        };
        config.validate().map_err(|e| {
            WritemagicError::validation(format!("Invalid model configuration: {}", e))
        })?;
        Ok(config)
    }
}

impl ValueObject for ModelConfiguration {}

/// Token count value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenCount(pub u32);

impl TokenCount {
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn add(&self, other: TokenCount) -> Self {
        Self(self.0 + other.0)
    }
}

impl ValueObject for TokenCount {}

impl std::fmt::Display for TokenCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} tokens", self.0)
    }
}