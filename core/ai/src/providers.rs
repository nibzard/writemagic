//! AI provider abstractions and implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use writemagic_shared::{Result, WritemagicError};
use std::collections::HashMap;

/// AI provider trait following the pattern from CLAUDE.md
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Complete a request with the AI provider
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;

    /// Get provider capabilities
    fn capabilities(&self) -> ModelCapabilities;

    /// Validate API key or credentials
    async fn validate_credentials(&self) -> Result<bool>;

    /// Get usage statistics
    async fn get_usage_stats(&self) -> Result<UsageStats>;
}

/// Completion request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
    pub metadata: HashMap<String, String>,
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>, model: String) -> Self {
        Self {
            messages,
            model,
            max_tokens: None,
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Completion response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub model: String,
    pub created: i64,
    pub metadata: HashMap<String, String>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Message role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Choice in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<FinishReason>,
}

/// Finish reason enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    FunctionCall,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub context_window: u32,
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub requests_today: u64,
    pub tokens_today: u64,
    pub cost_today: f64,
}

/// Claude AI provider implementation
pub struct ClaudeProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/v1/messages", self.base_url);
        
        // Convert to Claude API format
        let claude_request = self.convert_to_claude_format(request)?;
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| WritemagicError::network(format!("Claude API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(WritemagicError::ai_provider(format!("Claude API error: {}", error_text)));
        }

        let claude_response: serde_json::Value = response.json().await
            .map_err(|e| WritemagicError::ai_provider(format!("Failed to parse Claude response: {}", e)))?;

        self.convert_from_claude_format(&claude_response)
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            max_tokens: 100000,
            supports_streaming: true,
            supports_functions: false,
            supports_vision: true,
            context_window: 200000,
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00003,
        }
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // Simple validation by making a minimal request
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "claude-3-haiku-20240307".to_string(),
        ).with_max_tokens(1);

        match self.complete(&test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_usage_stats(&self) -> Result<UsageStats> {
        // This would typically query the provider's usage API
        // For now, return default stats
        Ok(UsageStats {
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            requests_today: 0,
            tokens_today: 0,
            cost_today: 0.0,
        })
    }
}

impl ClaudeProvider {
    fn convert_to_claude_format(&self, request: &CompletionRequest) -> Result<serde_json::Value> {
        let mut claude_messages = Vec::new();
        let mut system_message = None;

        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    system_message = Some(msg.content.clone());
                }
                MessageRole::User | MessageRole::Assistant => {
                    claude_messages.push(serde_json::json!({
                        "role": msg.role,
                        "content": msg.content
                    }));
                }
                MessageRole::Function => {
                    // Skip function messages for Claude
                    continue;
                }
            }
        }

        let mut claude_request = serde_json::json!({
            "model": request.model,
            "messages": claude_messages,
            "max_tokens": request.max_tokens.unwrap_or(4096)
        });

        if let Some(system) = system_message {
            claude_request["system"] = serde_json::Value::String(system);
        }

        if let Some(temperature) = request.temperature {
            claude_request["temperature"] = serde_json::Value::from(temperature);
        }

        if let Some(top_p) = request.top_p {
            claude_request["top_p"] = serde_json::Value::from(top_p);
        }

        if let Some(stop) = &request.stop {
            claude_request["stop_sequences"] = serde_json::Value::Array(
                stop.iter().map(|s| serde_json::Value::String(s.clone())).collect()
            );
        }

        Ok(claude_request)
    }

    fn convert_from_claude_format(&self, response: &serde_json::Value) -> Result<CompletionResponse> {
        let id = response["id"].as_str().unwrap_or_default().to_string();
        let model = response["model"].as_str().unwrap_or_default().to_string();
        
        let content = response["content"][0]["text"].as_str().unwrap_or_default();
        let message = Message::assistant(content);

        let choice = Choice {
            index: 0,
            message,
            finish_reason: Some(FinishReason::Stop),
        };

        let usage = Usage {
            prompt_tokens: response["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: response["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (response["usage"]["input_tokens"].as_u64().unwrap_or(0) + 
                          response["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
        };

        Ok(CompletionResponse {
            id,
            choices: vec![choice],
            usage,
            model,
            created: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        })
    }
}

/// OpenAI GPT provider implementation
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| WritemagicError::network(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(WritemagicError::ai_provider(format!("OpenAI API error: {}", error_text)));
        }

        response.json().await
            .map_err(|e| WritemagicError::ai_provider(format!("Failed to parse OpenAI response: {}", e)))
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            max_tokens: 4096,
            supports_streaming: true,
            supports_functions: true,
            supports_vision: true,
            context_window: 128000,
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00003,
        }
    }

    async fn validate_credentials(&self) -> Result<bool> {
        let test_request = CompletionRequest::new(
            vec![Message::user("Test")],
            "gpt-3.5-turbo".to_string(),
        ).with_max_tokens(1);

        match self.complete(&test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_usage_stats(&self) -> Result<UsageStats> {
        // This would typically query OpenAI's usage API
        Ok(UsageStats {
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            requests_today: 0,
            tokens_today: 0,
            cost_today: 0.0,
        })
    }
}