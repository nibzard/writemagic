//! AI domain services

use async_trait::async_trait;
use writemagic_shared::{EntityId, DomainService, Result, WritemagicError};
use crate::providers::{AIProvider, CompletionRequest, CompletionResponse, Message};
use crate::value_objects::{Prompt, ModelConfiguration};
use std::sync::Arc;
use std::collections::HashMap;

/// AI orchestration service with provider fallback
pub struct AIOrchestrationService {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    fallback_order: Vec<String>,
}

impl AIOrchestrationService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            fallback_order: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Arc<dyn AIProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name.clone(), provider);
        self.fallback_order.push(name);
    }

    pub fn set_fallback_order(&mut self, order: Vec<String>) {
        self.fallback_order = order;
    }

    pub async fn complete_with_fallback(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut last_error = None;

        for provider_name in &self.fallback_order {
            if let Some(provider) = self.providers.get(provider_name) {
                match provider.complete(&request).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        log::warn!("Provider {} failed: {}", provider_name, e);
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| WritemagicError::ai_provider("No providers available")))
    }
}

/// Context management service
pub struct ContextManagementService {
    max_context_length: usize,
}

impl ContextManagementService {
    pub fn new(max_context_length: usize) -> Self {
        Self { max_context_length }
    }

    pub fn manage_context(&self, messages: Vec<Message>) -> Vec<Message> {
        let total_length: usize = messages.iter().map(|m| m.content.len()).sum();
        
        if total_length <= self.max_context_length {
            return messages;
        }

        // Keep system message and recent messages
        let mut result = Vec::new();
        let mut current_length = 0;

        // Add system messages first
        for msg in &messages {
            if matches!(msg.role, crate::providers::MessageRole::System) {
                result.push(msg.clone());
                current_length += msg.content.len();
            }
        }

        // Add recent messages in reverse order
        for msg in messages.iter().rev() {
            if !matches!(msg.role, crate::providers::MessageRole::System) {
                if current_length + msg.content.len() <= self.max_context_length {
                    result.insert(result.len(), msg.clone());
                    current_length += msg.content.len();
                } else {
                    break;
                }
            }
        }

        result
    }
}

/// Content filtering service
pub struct ContentFilteringService {
    prohibited_patterns: Vec<regex::Regex>,
}

impl ContentFilteringService {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            r"(?i)(password|api[_-]?key|secret|token)\s*[:=]\s*[^\s]+",
            r"(?i)(credit[_-]?card|ssn|social[_-]?security)",
        ];

        let mut prohibited_patterns = Vec::new();
        for pattern in patterns {
            prohibited_patterns.push(regex::Regex::new(pattern)
                .map_err(|e| WritemagicError::internal(format!("Invalid regex: {}", e)))?);
        }

        Ok(Self { prohibited_patterns })
    }

    pub fn filter_content(&self, content: &str) -> Result<String> {
        for pattern in &self.prohibited_patterns {
            if pattern.is_match(content) {
                return Err(WritemagicError::validation("Content contains sensitive information"));
            }
        }
        Ok(content.to_string())
    }

    pub fn detect_sensitive_info(&self, content: &str) -> Vec<String> {
        let mut findings = Vec::new();
        for (i, pattern) in self.prohibited_patterns.iter().enumerate() {
            if pattern.is_match(content) {
                findings.push(format!("Pattern {} matched", i));
            }
        }
        findings
    }
}

impl Default for ContentFilteringService {
    fn default() -> Self {
        Self::new().expect("Failed to create content filtering service")
    }
}