// AI provider client service with fallback strategy
const axios = require('axios');
const NodeCache = require('node-cache');
const crypto = require('crypto');
const config = require('../config');
const { logAIRequest, logError, logPerformance } = require('../utils/logger');

class AIClient {
  constructor() {
    this.cache = new NodeCache({
      stdTTL: config.cache.ttlSeconds,
      checkperiod: config.cache.checkPeriodSeconds,
      maxKeys: config.cache.maxItems,
    });
    
    this.providers = this.initializeProviders();
    this.fallbackOrder = config.ai.fallbackOrder;
    this.rateLimiters = new Map();
    
    // Initialize rate limiters for each provider
    this.initializeRateLimiters();
  }

  initializeProviders() {
    const providers = {};
    
    // Claude provider
    if (config.ai.providers.claude.apiKey) {
      providers.claude = {
        name: 'claude',
        config: config.ai.providers.claude,
        client: axios.create({
          baseURL: config.ai.providers.claude.baseUrl,
          timeout: 120000,
          headers: {
            'Content-Type': 'application/json',
            'anthropic-version': '2023-06-01',
            'x-api-key': config.ai.providers.claude.apiKey,
          },
        }),
      };
    }
    
    // OpenAI provider
    if (config.ai.providers.openai.apiKey) {
      providers.openai = {
        name: 'openai',
        config: config.ai.providers.openai,
        client: axios.create({
          baseURL: config.ai.providers.openai.baseUrl,
          timeout: 120000,
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${config.ai.providers.openai.apiKey}`,
          },
        }),
      };
    }
    
    return providers;
  }

  initializeRateLimiters() {
    for (const [providerName, provider] of Object.entries(this.providers)) {
      this.rateLimiters.set(providerName, {
        requests: new Map(),
        maxConcurrent: provider.config.rateLimitConcurrency,
        minInterval: provider.config.rateLimitIntervalMs,
        lastRequest: Date.now() - provider.config.rateLimitIntervalMs,
      });
    }
  }

  // Main completion method with provider fallback
  async complete(request) {
    const cacheKey = this.generateCacheKey(request);
    
    // Check cache first
    const cachedResponse = this.cache.get(cacheKey);
    if (cachedResponse) {
      return {
        ...cachedResponse,
        cached: true,
        provider: cachedResponse.provider,
      };
    }

    // Try providers in fallback order
    let lastError = null;
    
    for (const providerName of this.fallbackOrder) {
      if (!this.providers[providerName]) {
        continue;
      }
      
      try {
        const startTime = Date.now();
        const response = await this.makeProviderRequest(providerName, request);
        const duration = Date.now() - startTime;
        
        logAIRequest(providerName, request.model || 'default', request, response, null, duration);
        logPerformance('ai_completion', duration, { provider: providerName });
        
        // Cache the response
        this.cache.set(cacheKey, { ...response, provider: providerName });
        
        return {
          ...response,
          cached: false,
          provider: providerName,
        };
      } catch (error) {
        lastError = error;
        logAIRequest(providerName, request.model || 'default', request, null, error);
        logError(error, { provider: providerName, request: this.sanitizeRequest(request) });
        
        // Continue to next provider if this one fails
        continue;
      }
    }
    
    // All providers failed
    throw new Error(`All AI providers failed. Last error: ${lastError?.message || 'Unknown error'}`);
  }

  // Make request to specific provider
  async makeProviderRequest(providerName, request) {
    const provider = this.providers[providerName];
    if (!provider) {
      throw new Error(`Provider ${providerName} not available`);
    }

    // Apply rate limiting
    await this.applyRateLimit(providerName);

    switch (providerName) {
      case 'claude':
        return this.makeClaudeRequest(provider, request);
      case 'openai':
        return this.makeOpenAIRequest(provider, request);
      default:
        throw new Error(`Unsupported provider: ${providerName}`);
    }
  }

  // Claude API request
  async makeClaudeRequest(provider, request) {
    const claudeRequest = this.convertToClaudeFormat(request, provider.config);
    
    const response = await provider.client.post('/v1/messages', claudeRequest);
    
    if (response.status !== 200) {
      throw new Error(`Claude API error: ${response.status} ${response.statusText}`);
    }
    
    return this.convertFromClaudeFormat(response.data);
  }

  // OpenAI API request
  async makeOpenAIRequest(provider, request) {
    const openaiRequest = this.convertToOpenAIFormat(request, provider.config);
    
    const response = await provider.client.post('/v1/chat/completions', openaiRequest);
    
    if (response.status !== 200) {
      throw new Error(`OpenAI API error: ${response.status} ${response.statusText}`);
    }
    
    return this.convertFromOpenAIFormat(response.data);
  }

  // Convert request to Claude format
  convertToClaudeFormat(request, providerConfig) {
    const messages = [];
    let systemMessage = null;
    
    for (const message of request.messages || []) {
      if (message.role === 'system') {
        systemMessage = message.content;
      } else if (message.role === 'user' || message.role === 'assistant') {
        messages.push({
          role: message.role,
          content: message.content,
        });
      }
    }
    
    const claudeRequest = {
      model: request.model || providerConfig.defaultModel,
      messages,
      max_tokens: Math.min(
        request.max_tokens || providerConfig.maxTokens,
        providerConfig.maxTokens
      ),
      temperature: request.temperature ?? providerConfig.temperature,
    };
    
    if (systemMessage) {
      claudeRequest.system = systemMessage;
    }
    
    if (request.top_p !== undefined) {
      claudeRequest.top_p = request.top_p;
    }
    
    if (request.stop) {
      claudeRequest.stop_sequences = request.stop;
    }
    
    return claudeRequest;
  }

  // Convert request to OpenAI format
  convertToOpenAIFormat(request, providerConfig) {
    return {
      model: request.model || providerConfig.defaultModel,
      messages: request.messages || [],
      max_tokens: Math.min(
        request.max_tokens || providerConfig.maxTokens,
        providerConfig.maxTokens
      ),
      temperature: request.temperature ?? providerConfig.temperature,
      top_p: request.top_p ?? 1.0,
      frequency_penalty: request.frequency_penalty ?? 0,
      presence_penalty: request.presence_penalty ?? 0,
      stop: request.stop,
      stream: request.stream || false,
    };
  }

  // Convert Claude response to standard format
  convertFromClaudeFormat(response) {
    const message = {
      role: 'assistant',
      content: response.content?.[0]?.text || '',
    };
    
    return {
      id: response.id,
      choices: [{
        index: 0,
        message,
        finish_reason: 'stop',
      }],
      usage: {
        prompt_tokens: response.usage?.input_tokens || 0,
        completion_tokens: response.usage?.output_tokens || 0,
        total_tokens: (response.usage?.input_tokens || 0) + (response.usage?.output_tokens || 0),
      },
      model: response.model,
      created: Math.floor(Date.now() / 1000),
    };
  }

  // Convert OpenAI response to standard format (already in correct format)
  convertFromOpenAIFormat(response) {
    return response;
  }

  // Apply rate limiting for provider
  async applyRateLimit(providerName) {
    const limiter = this.rateLimiters.get(providerName);
    if (!limiter) return;
    
    // Check if we need to wait based on minimum interval
    const now = Date.now();
    const timeSinceLastRequest = now - limiter.lastRequest;
    
    if (timeSinceLastRequest < limiter.minInterval) {
      const waitTime = limiter.minInterval - timeSinceLastRequest;
      await new Promise(resolve => setTimeout(resolve, waitTime));
    }
    
    limiter.lastRequest = Date.now();
  }

  // Generate cache key for request
  generateCacheKey(request) {
    const keyData = {
      model: request.model,
      messages: request.messages,
      max_tokens: request.max_tokens,
      temperature: request.temperature,
      top_p: request.top_p,
    };
    
    return crypto
      .createHash('sha256')
      .update(JSON.stringify(keyData))
      .digest('hex')
      .substring(0, 32);
  }

  // Sanitize request for logging (remove sensitive data)
  sanitizeRequest(request) {
    return {
      model: request.model,
      messageCount: request.messages?.length || 0,
      maxTokens: request.max_tokens,
      temperature: request.temperature,
    };
  }

  // Health check for all providers
  async healthCheck() {
    const results = {};
    
    for (const [providerName, provider] of Object.entries(this.providers)) {
      try {
        const testRequest = {
          messages: [{ role: 'user', content: 'Hello' }],
          max_tokens: 1,
        };
        
        const startTime = Date.now();
        await this.makeProviderRequest(providerName, testRequest);
        const responseTime = Date.now() - startTime;
        
        results[providerName] = {
          status: 'healthy',
          responseTime,
          lastChecked: new Date().toISOString(),
        };
      } catch (error) {
        results[providerName] = {
          status: 'unhealthy',
          error: error.message,
          lastChecked: new Date().toISOString(),
        };
      }
    }
    
    return results;
  }

  // Get cache statistics
  getCacheStats() {
    return {
      keys: this.cache.keys().length,
      hits: this.cache.getStats().hits,
      misses: this.cache.getStats().misses,
      hitRate: this.cache.getStats().hits / (this.cache.getStats().hits + this.cache.getStats().misses) || 0,
      memoryUsage: process.memoryUsage(),
    };
  }

  // Clear cache
  clearCache() {
    this.cache.flushAll();
    return { cleared: true, timestamp: new Date().toISOString() };
  }

  // Get available providers
  getAvailableProviders() {
    return Object.keys(this.providers).map(name => ({
      name,
      config: {
        defaultModel: this.providers[name].config.defaultModel,
        maxTokens: this.providers[name].config.maxTokens,
        temperature: this.providers[name].config.temperature,
      },
    }));
  }
}

module.exports = AIClient;