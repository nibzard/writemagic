//! Accurate tokenization system with model-specific support

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tiktoken_rs::{CoreBPE, get_bpe_from_model};
use writemagic_shared::{Result, WritemagicError};
use crate::providers::CompletionRequest;

/// Model-specific tokenizer configuration
#[derive(Debug, Clone)]
pub struct ModelTokenizerConfig {
    pub name: String,
    pub encoding_name: String,
    pub max_tokens: u32,
    pub context_window: u32,
    pub special_tokens: HashMap<String, u32>,
}

impl ModelTokenizerConfig {
    /// Create config for Claude models
    pub fn claude_3() -> Self {
        Self {
            name: "claude-3".to_string(),
            encoding_name: "cl100k_base".to_string(),
            max_tokens: 100000,
            context_window: 200000,
            special_tokens: HashMap::new(),
        }
    }

    /// Create config for GPT-4 models  
    pub fn gpt_4() -> Self {
        Self {
            name: "gpt-4".to_string(),
            encoding_name: "cl100k_base".to_string(),
            max_tokens: 4096,
            context_window: 128000,
            special_tokens: HashMap::new(),
        }
    }

    /// Create config for GPT-3.5-turbo
    pub fn gpt_3_5_turbo() -> Self {
        Self {
            name: "gpt-3.5-turbo".to_string(),
            encoding_name: "cl100k_base".to_string(),
            max_tokens: 4096,
            context_window: 16385,
            special_tokens: HashMap::new(),
        }
    }
}

/// Token usage statistics with accurate counting
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost: f64,
}

impl TokenUsage {
    pub fn new(input_tokens: u32, output_tokens: u32, input_cost_per_token: f64, output_cost_per_token: f64) -> Self {
        let total_tokens = input_tokens + output_tokens;
        let estimated_cost = (input_tokens as f64 * input_cost_per_token) + (output_tokens as f64 * output_cost_per_token);
        
        Self {
            input_tokens,
            output_tokens,
            total_tokens,
            estimated_cost,
        }
    }
}

/// Model-specific tokenizer with caching
pub struct ModelTokenizer {
    config: ModelTokenizerConfig,
    encoder: CoreBPE,
    cache: Arc<RwLock<HashMap<String, (u32, std::time::Instant)>>>,
    cache_ttl: std::time::Duration,
}

impl ModelTokenizer {
    /// Create a new tokenizer for the specified model
    pub fn new(config: ModelTokenizerConfig) -> Result<Self> {
        let encoder = match get_bpe_from_model(&config.name) {
            Ok(encoder) => encoder,
            Err(_) => {
                // Fallback to cl100k_base encoding if model-specific fails
                tiktoken_rs::get_bpe_from_model("cl100k_base")
                    .map_err(|e| WritemagicError::internal(format!("Failed to load tokenizer: {}", e)))?
            }
        };

        Ok(Self {
            config,
            encoder,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300), // 5-minute cache
        })
    }

    /// Count tokens in text with caching
    pub fn count_tokens(&self, text: &str) -> Result<u32> {
        // Check cache first
        let cache_key = blake3::hash(text.as_bytes()).to_string();
        
        {
            let cache = self.cache.read();
            if let Some((count, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(*count);
                }
            }
        }

        // Tokenize text
        let tokens = self.encoder.encode_with_special_tokens(text);
        let count = tokens.len() as u32;

        // Cache result
        {
            let mut cache = self.cache.write();
            cache.insert(cache_key, (count, std::time::Instant::now()));
            
            // Clean expired entries periodically
            if cache.len() > 1000 {
                let now = std::time::Instant::now();
                cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.cache_ttl);
            }
        }

        Ok(count)
    }

    /// Count tokens in a completion request
    pub fn count_request_tokens(&self, request: &CompletionRequest) -> Result<u32> {
        let mut total = 0;
        
        // Count message tokens
        for message in &request.messages {
            total += self.count_tokens(&message.content)?;
            
            // Add overhead tokens for message formatting
            total += match message.role {
                crate::providers::MessageRole::System => 4, // <|start|>system<|message|>{content}<|end|>
                crate::providers::MessageRole::User => 4,   // <|start|>user<|message|>{content}<|end|>
                crate::providers::MessageRole::Assistant => 4, // <|start|>assistant<|message|>{content}<|end|>
                crate::providers::MessageRole::Function => 6, // Additional overhead for function calls
            };
        }
        
        // Add base conversation overhead
        total += 3; // Base conversation tokens
        
        Ok(total)
    }

    /// Check if request fits within context window
    pub fn validate_context_window(&self, request: &CompletionRequest) -> Result<()> {
        let input_tokens = self.count_request_tokens(request)?;
        let max_output_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);
        let total_tokens = input_tokens + max_output_tokens;
        
        if total_tokens > self.config.context_window {
            return Err(WritemagicError::validation(format!(
                "Request exceeds context window: {} tokens (max: {})",
                total_tokens, self.config.context_window
            )));
        }
        
        Ok(())
    }

    /// Get optimal max_tokens for request within budget
    pub fn optimize_max_tokens(&self, request: &CompletionRequest, token_budget: u32) -> Result<u32> {
        let input_tokens = self.count_request_tokens(request)?;
        
        if input_tokens >= token_budget {
            return Err(WritemagicError::validation(
                "Input tokens exceed available budget"
            ));
        }
        
        let available_output_tokens = token_budget - input_tokens;
        let requested_output_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);
        
        Ok(available_output_tokens.min(requested_output_tokens).min(self.config.max_tokens))
    }

    /// Get model configuration
    pub fn config(&self) -> &ModelTokenizerConfig {
        &self.config
    }

    /// Clear token cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

/// Multi-model tokenization service
pub struct TokenizationService {
    tokenizers: HashMap<String, Arc<ModelTokenizer>>,
    default_tokenizer: Arc<ModelTokenizer>,
}

impl TokenizationService {
    /// Create new tokenization service with common models
    pub fn new() -> Result<Self> {
        let mut tokenizers = HashMap::new();
        
        // Initialize tokenizers for common models
        let claude_3 = Arc::new(ModelTokenizer::new(ModelTokenizerConfig::claude_3())?);
        let gpt_4 = Arc::new(ModelTokenizer::new(ModelTokenizerConfig::gpt_4())?);
        let gpt_3_5 = Arc::new(ModelTokenizer::new(ModelTokenizerConfig::gpt_3_5_turbo())?);
        
        // Map model names to tokenizers
        tokenizers.insert("claude-3-sonnet".to_string(), claude_3.clone());
        tokenizers.insert("claude-3-opus".to_string(), claude_3.clone());
        tokenizers.insert("claude-3-haiku".to_string(), claude_3.clone());
        
        tokenizers.insert("gpt-4".to_string(), gpt_4.clone());
        tokenizers.insert("gpt-4-turbo".to_string(), gpt_4.clone());
        tokenizers.insert("gpt-4o".to_string(), gpt_4.clone());
        
        tokenizers.insert("gpt-3.5-turbo".to_string(), gpt_3_5.clone());
        
        Ok(Self {
            tokenizers,
            default_tokenizer: gpt_4, // Use GPT-4 as default
        })
    }

    /// Get tokenizer for specific model
    pub fn get_tokenizer(&self, model_name: &str) -> Arc<ModelTokenizer> {
        // Try exact match first
        if let Some(tokenizer) = self.tokenizers.get(model_name) {
            return tokenizer.clone();
        }
        
        // Try prefix matching for model families
        for (registered_model, tokenizer) in &self.tokenizers {
            if model_name.starts_with(registered_model.split('-').next().unwrap_or("")) {
                return tokenizer.clone();
            }
        }
        
        // Return default tokenizer
        tracing::warn!("No specific tokenizer found for model '{}', using default", model_name);
        self.default_tokenizer.clone()
    }

    /// Count tokens for any model
    pub fn count_tokens(&self, text: &str, model_name: &str) -> Result<u32> {
        let tokenizer = self.get_tokenizer(model_name);
        tokenizer.count_tokens(text)
    }

    /// Count request tokens for any model
    pub fn count_request_tokens(&self, request: &CompletionRequest) -> Result<u32> {
        let tokenizer = self.get_tokenizer(&request.model);
        tokenizer.count_request_tokens(request)
    }

    /// Validate request against context window
    pub fn validate_request(&self, request: &CompletionRequest) -> Result<()> {
        let tokenizer = self.get_tokenizer(&request.model);
        tokenizer.validate_context_window(request)
    }

    /// Calculate accurate token usage from response
    pub fn calculate_usage(&self, request: &CompletionRequest, response_content: &str, cost_per_input_token: f64, cost_per_output_token: f64) -> Result<TokenUsage> {
        let tokenizer = self.get_tokenizer(&request.model);
        let input_tokens = tokenizer.count_request_tokens(request)?;
        let output_tokens = tokenizer.count_tokens(response_content)?;
        
        Ok(TokenUsage::new(
            input_tokens,
            output_tokens,
            cost_per_input_token,
            cost_per_output_token,
        ))
    }

    /// Get all available models
    pub fn available_models(&self) -> Vec<String> {
        self.tokenizers.keys().cloned().collect()
    }
}

impl Default for TokenizationService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            log::error!("Failed to create tokenization service, using minimal implementation");
            // Create a minimal default tokenizer
            let default_config = ModelTokenizerConfig::gpt_4();
            let default_tokenizer = ModelTokenizer::new(default_config)
                .unwrap_or_else(|_| panic!("Failed to create default tokenizer"));
            
            Self {
                tokenizers: std::collections::HashMap::new(),
                default_tokenizer: Arc::new(default_tokenizer),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Message;

    #[test]
    fn test_model_configs() {
        let claude = ModelTokenizerConfig::claude_3();
        assert_eq!(claude.name, "claude-3");
        assert_eq!(claude.context_window, 200000);
        
        let gpt4 = ModelTokenizerConfig::gpt_4();
        assert_eq!(gpt4.name, "gpt-4");
        assert_eq!(gpt4.context_window, 128000);
    }

    #[tokio::test]
    async fn test_tokenization_service() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let test_text = "Hello, how are you doing today?";
        let count = service.count_tokens(test_text, "gpt-4")?;
        
        assert!(count > 0);
        assert!(count < 20); // Should be reasonable for this short text
        
        Ok(())
    }

    #[tokio::test] 
    async fn test_request_token_counting() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let request = CompletionRequest::new(
            vec![
                Message::system("You are a helpful assistant."),
                Message::user("Hello, how are you?"),
            ],
            "gpt-4".to_string(),
        );
        
        let count = service.count_request_tokens(&request)?;
        assert!(count > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_context_window_validation() -> Result<()> {
        let service = TokenizationService::new()?;
        
        let normal_request = CompletionRequest::new(
            vec![Message::user("Short message")],
            "gpt-4".to_string(),
        );
        
        // Should not error for normal request
        service.validate_request(&normal_request)?;
        
        Ok(())
    }
}