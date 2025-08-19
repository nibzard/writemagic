// Test suite for AI client service
const AIClient = require('../services/ai-client');
const config = require('../config');

// Mock axios
jest.mock('axios');
const axios = require('axios');

// Mock config
jest.mock('../config', () => ({
  cache: {
    ttlSeconds: 300,
    checkPeriodSeconds: 120,
    maxItems: 1000,
  },
  ai: {
    providers: {
      claude: {
        apiKey: 'test-claude-key',
        baseUrl: 'https://api.anthropic.com',
        defaultModel: 'claude-3-sonnet-20240229',
        maxTokens: 100000,
        temperature: 0.7,
        rateLimitConcurrency: 5,
        rateLimitIntervalMs: 200,
      },
      openai: {
        apiKey: 'test-openai-key',
        baseUrl: 'https://api.openai.com',
        defaultModel: 'gpt-4-turbo-preview',
        maxTokens: 4096,
        temperature: 0.7,
        rateLimitConcurrency: 10,
        rateLimitIntervalMs: 100,
      },
    },
    fallbackOrder: ['claude', 'openai'],
  },
}));

describe('AIClient', () => {
  let aiClient;
  
  beforeEach(() => {
    aiClient = new AIClient();
    jest.clearAllMocks();
  });

  describe('Initialization', () => {
    test('should initialize with providers', () => {
      expect(aiClient.providers).toBeDefined();
      expect(aiClient.providers.claude).toBeDefined();
      expect(aiClient.providers.openai).toBeDefined();
    });

    test('should initialize rate limiters', () => {
      expect(aiClient.rateLimiters).toBeDefined();
      expect(aiClient.rateLimiters.has('claude')).toBe(true);
      expect(aiClient.rateLimiters.has('openai')).toBe(true);
    });
  });

  describe('Request Conversion', () => {
    test('should convert to Claude format correctly', () => {
      const request = {
        messages: [
          { role: 'system', content: 'You are a helpful assistant' },
          { role: 'user', content: 'Hello' },
        ],
        model: 'claude-3-sonnet-20240229',
        max_tokens: 1000,
        temperature: 0.8,
      };

      const claudeRequest = aiClient.convertToClaudeFormat(
        request,
        config.ai.providers.claude
      );

      expect(claudeRequest).toEqual({
        model: 'claude-3-sonnet-20240229',
        messages: [{ role: 'user', content: 'Hello' }],
        max_tokens: 1000,
        temperature: 0.8,
        system: 'You are a helpful assistant',
      });
    });

    test('should convert to OpenAI format correctly', () => {
      const request = {
        messages: [
          { role: 'user', content: 'Hello' },
          { role: 'assistant', content: 'Hi there!' },
          { role: 'user', content: 'How are you?' },
        ],
        model: 'gpt-4-turbo-preview',
        max_tokens: 500,
        temperature: 0.7,
        top_p: 0.9,
      };

      const openaiRequest = aiClient.convertToOpenAIFormat(
        request,
        config.ai.providers.openai
      );

      expect(openaiRequest).toEqual({
        model: 'gpt-4-turbo-preview',
        messages: request.messages,
        max_tokens: 500,
        temperature: 0.7,
        top_p: 0.9,
        frequency_penalty: 0,
        presence_penalty: 0,
        stop: undefined,
        stream: false,
      });
    });
  });

  describe('Response Conversion', () => {
    test('should convert Claude response correctly', () => {
      const claudeResponse = {
        id: 'msg_123',
        model: 'claude-3-sonnet-20240229',
        content: [{ text: 'Hello! How can I help you?' }],
        usage: {
          input_tokens: 10,
          output_tokens: 8,
        },
      };

      const standardResponse = aiClient.convertFromClaudeFormat(claudeResponse);

      expect(standardResponse).toMatchObject({
        id: 'msg_123',
        model: 'claude-3-sonnet-20240229',
        choices: [{
          index: 0,
          message: {
            role: 'assistant',
            content: 'Hello! How can I help you?',
          },
          finish_reason: 'stop',
        }],
        usage: {
          prompt_tokens: 10,
          completion_tokens: 8,
          total_tokens: 18,
        },
      });
    });

    test('should handle OpenAI response correctly', () => {
      const openaiResponse = {
        id: 'chatcmpl-123',
        model: 'gpt-4-turbo-preview',
        choices: [{
          index: 0,
          message: {
            role: 'assistant',
            content: 'Hello! How can I help you?',
          },
          finish_reason: 'stop',
        }],
        usage: {
          prompt_tokens: 10,
          completion_tokens: 8,
          total_tokens: 18,
        },
      };

      const standardResponse = aiClient.convertFromOpenAIFormat(openaiResponse);

      expect(standardResponse).toEqual(openaiResponse);
    });
  });

  describe('Caching', () => {
    test('should generate consistent cache keys', () => {
      const request1 = {
        messages: [{ role: 'user', content: 'Hello' }],
        model: 'claude-3-sonnet-20240229',
        temperature: 0.7,
      };

      const request2 = {
        messages: [{ role: 'user', content: 'Hello' }],
        model: 'claude-3-sonnet-20240229',
        temperature: 0.7,
      };

      const key1 = aiClient.generateCacheKey(request1);
      const key2 = aiClient.generateCacheKey(request2);

      expect(key1).toBe(key2);
      expect(typeof key1).toBe('string');
      expect(key1.length).toBe(32); // SHA256 hash truncated to 32 chars
    });

    test('should generate different cache keys for different requests', () => {
      const request1 = {
        messages: [{ role: 'user', content: 'Hello' }],
        model: 'claude-3-sonnet-20240229',
      };

      const request2 = {
        messages: [{ role: 'user', content: 'Goodbye' }],
        model: 'claude-3-sonnet-20240229',
      };

      const key1 = aiClient.generateCacheKey(request1);
      const key2 = aiClient.generateCacheKey(request2);

      expect(key1).not.toBe(key2);
    });
  });

  describe('Health Check', () => {
    test('should return health status for all providers', async () => {
      // Mock successful responses
      axios.post
        .mockResolvedValueOnce({ status: 200, data: { id: 'test' } })
        .mockResolvedValueOnce({ status: 200, data: { id: 'test' } });

      const health = await aiClient.healthCheck();

      expect(health).toHaveProperty('claude');
      expect(health).toHaveProperty('openai');
      expect(health.claude.status).toBe('healthy');
      expect(health.openai.status).toBe('healthy');
    });

    test('should handle provider failures in health check', async () => {
      // Mock failure for Claude, success for OpenAI
      axios.post
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ status: 200, data: { id: 'test' } });

      const health = await aiClient.healthCheck();

      expect(health.claude.status).toBe('unhealthy');
      expect(health.claude.error).toBe('Network error');
      expect(health.openai.status).toBe('healthy');
    });
  });

  describe('Cache Management', () => {
    test('should get cache statistics', () => {
      const stats = aiClient.getCacheStats();

      expect(stats).toHaveProperty('keys');
      expect(stats).toHaveProperty('hits');
      expect(stats).toHaveProperty('misses');
      expect(stats).toHaveProperty('hitRate');
      expect(stats).toHaveProperty('memoryUsage');
    });

    test('should clear cache', () => {
      const result = aiClient.clearCache();

      expect(result.cleared).toBe(true);
      expect(result.timestamp).toBeDefined();
    });
  });

  describe('Provider Information', () => {
    test('should return available providers', () => {
      const providers = aiClient.getAvailableProviders();

      expect(Array.isArray(providers)).toBe(true);
      expect(providers.length).toBe(2);
      
      const claudeProvider = providers.find(p => p.name === 'claude');
      expect(claudeProvider).toBeDefined();
      expect(claudeProvider.config.defaultModel).toBe('claude-3-sonnet-20240229');
      
      const openaiProvider = providers.find(p => p.name === 'openai');
      expect(openaiProvider).toBeDefined();
      expect(openaiProvider.config.defaultModel).toBe('gpt-4-turbo-preview');
    });
  });

  describe('Request Sanitization', () => {
    test('should sanitize request for logging', () => {
      const request = {
        messages: [
          { role: 'user', content: 'This is a long message with sensitive content' },
          { role: 'assistant', content: 'This is another message' },
        ],
        model: 'claude-3-sonnet-20240229',
        max_tokens: 1000,
        temperature: 0.7,
        apiKey: 'secret-key', // This should not be logged
      };

      const sanitized = aiClient.sanitizeRequest(request);

      expect(sanitized).toEqual({
        model: 'claude-3-sonnet-20240229',
        messageCount: 2,
        maxTokens: 1000,
        temperature: 0.7,
      });
    });
  });
});

describe('Integration Tests', () => {
  let aiClient;
  
  beforeEach(() => {
    aiClient = new AIClient();
  });

  test('should handle complete request with fallback', async () => {
    // Mock Claude failure, OpenAI success
    const claudeError = new Error('Claude API error');
    const openaiSuccess = {
      status: 200,
      data: {
        id: 'chatcmpl-123',
        model: 'gpt-4-turbo-preview',
        choices: [{
          index: 0,
          message: {
            role: 'assistant',
            content: 'Hello from OpenAI!',
          },
          finish_reason: 'stop',
        }],
        usage: {
          prompt_tokens: 5,
          completion_tokens: 4,
          total_tokens: 9,
        },
      },
    };

    axios.post
      .mockRejectedValueOnce(claudeError)
      .mockResolvedValueOnce(openaiSuccess);

    const request = {
      messages: [{ role: 'user', content: 'Hello' }],
      model: 'claude-3-sonnet-20240229',
    };

    const response = await aiClient.complete(request);

    expect(response.provider).toBe('openai');
    expect(response.choices[0].message.content).toBe('Hello from OpenAI!');
    expect(response.cached).toBe(false);
  });

  test('should return cached response on second identical request', async () => {
    // Mock successful response
    const successResponse = {
      status: 200,
      data: {
        id: 'msg_123',
        model: 'claude-3-sonnet-20240229',
        content: [{ text: 'Cached response!' }],
        usage: {
          input_tokens: 5,
          output_tokens: 3,
        },
      },
    };

    axios.post.mockResolvedValue(successResponse);

    const request = {
      messages: [{ role: 'user', content: 'Hello' }],
      model: 'claude-3-sonnet-20240229',
    };

    // First request - should call API
    const response1 = await aiClient.complete(request);
    expect(response1.cached).toBe(false);

    // Second request - should return cached result
    const response2 = await aiClient.complete(request);
    expect(response2.cached).toBe(true);
    expect(response2.choices[0].message.content).toBe('Cached response!');

    // API should only be called once
    expect(axios.post).toHaveBeenCalledTimes(1);
  });
});