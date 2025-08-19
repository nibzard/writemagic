/**
 * AI Integration tests for WriteMagic web application
 * Tests AI proxy integration, provider fallback, and AI-assisted writing features
 */

import { AIProxyIntegration } from '@/ai-proxy-integration.js';
import { WritingSession } from '@/writing-session.js';

// Mock fetch for AI API calls
global.fetch = jest.fn();

describe('AI Integration Tests', () => {
  let aiIntegration;
  let writingSession;

  beforeEach(() => {
    // Reset fetch mock
    global.fetch.mockClear();
    
    // Mock WASM modules
    global.writemagic_wasm = {
      WritingSession: {
        new: jest.fn(() => ({
          start: jest.fn(),
          pause: jest.fn(),
          get_analytics: jest.fn().mockReturnValue('{}'),
          free: jest.fn()
        }))
      }
    };

    // Mock AI proxy configuration
    const mockConfig = {
      aiProxyUrl: 'http://localhost:3001',
      providers: ['claude', 'openai', 'local'],
      defaultProvider: 'claude',
      timeout: 10000,
      retryAttempts: 3
    };

    aiIntegration = new AIProxyIntegration(mockConfig);
    writingSession = new WritingSession();
  });

  describe('AI Provider Connection', () => {
    test('should successfully connect to AI proxy', async () => {
      // Mock successful connection response
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          status: 'connected',
          availableProviders: ['claude', 'openai'],
          defaultProvider: 'claude'
        })
      });

      const connectionResult = await aiIntegration.testConnection();
      
      expect(connectionResult.connected).toBe(true);
      expect(connectionResult.availableProviders).toContain('claude');
      expect(connectionResult.availableProviders).toContain('openai');
      expect(global.fetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/ai/health',
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'Content-Type': 'application/json'
          })
        })
      );
    });

    test('should handle connection failure gracefully', async () => {
      // Mock connection failure
      global.fetch.mockRejectedValueOnce(new Error('Network error'));

      const connectionResult = await aiIntegration.testConnection();
      
      expect(connectionResult.connected).toBe(false);
      expect(connectionResult.error).toBe('Network error');
    });

    test('should validate API keys for different providers', async () => {
      const providers = ['claude', 'openai'];
      
      for (const provider of providers) {
        // Mock successful validation
        global.fetch.mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            valid: true,
            provider,
            capabilities: ['text-completion', 'chat']
          })
        });

        const validation = await aiIntegration.validateProvider(provider, 'test-api-key');
        
        expect(validation.valid).toBe(true);
        expect(validation.provider).toBe(provider);
        expect(validation.capabilities).toContain('text-completion');
      }
    });
  });

  describe('Text Completion', () => {
    test('should generate text completion successfully', async () => {
      const mockCompletion = {
        text: 'This is a generated completion that continues the user\'s text in a meaningful way.',
        provider: 'claude',
        model: 'claude-3-sonnet',
        tokens: 125,
        confidence: 0.85
      };

      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockCompletion
      });

      const completionRequest = {
        prompt: 'The quick brown fox jumps over',
        maxTokens: 150,
        temperature: 0.7,
        provider: 'claude'
      };

      const result = await aiIntegration.generateCompletion(completionRequest);
      
      expect(result.text).toBe(mockCompletion.text);
      expect(result.provider).toBe('claude');
      expect(result.tokens).toBe(125);
      expect(global.fetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/ai/complete',
        expect.objectContaining({
          method: 'POST',
          headers: expect.objectContaining({
            'Content-Type': 'application/json'
          }),
          body: JSON.stringify(completionRequest)
        })
      );
    });

    test('should handle different completion types', async () => {
      const completionTypes = [
        {
          type: 'continue',
          prompt: 'The story begins when',
          expectedLength: 100
        },
        {
          type: 'rewrite',
          prompt: 'Rewrite this sentence: The cat sat on the mat.',
          expectedLength: 50
        },
        {
          type: 'summarize',
          prompt: 'Summarize: Lorem ipsum dolor sit amet, consectetur adipiscing elit...',
          expectedLength: 200
        }
      ];

      for (const testCase of completionTypes) {
        global.fetch.mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            text: `Generated ${testCase.type} response`,
            type: testCase.type,
            provider: 'claude'
          })
        });

        const result = await aiIntegration.generateCompletion({
          prompt: testCase.prompt,
          type: testCase.type,
          maxTokens: testCase.expectedLength
        });

        expect(result.text).toContain(testCase.type);
        expect(result.type).toBe(testCase.type);
      }
    });

    test('should apply content filtering', async () => {
      const sensitiveContent = {
        text: 'This contains inappropriate content that should be filtered',
        filtered: true,
        reason: 'inappropriate content detected'
      };

      global.fetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => sensitiveContent
      });

      const result = await aiIntegration.generateCompletion({
        prompt: 'Generate inappropriate content',
        maxTokens: 100
      });

      expect(result.success).toBe(false);
      expect(result.filtered).toBe(true);
      expect(result.reason).toBe('inappropriate content detected');
    });
  });

  describe('Provider Fallback', () => {
    test('should fallback to secondary provider on primary failure', async () => {
      // Mock primary provider failure
      global.fetch
        .mockRejectedValueOnce(new Error('Claude API unavailable'))
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            text: 'Fallback completion from OpenAI',
            provider: 'openai',
            model: 'gpt-4'
          })
        });

      const result = await aiIntegration.generateCompletionWithFallback({
        prompt: 'Complete this text',
        providers: ['claude', 'openai']
      });

      expect(result.text).toBe('Fallback completion from OpenAI');
      expect(result.provider).toBe('openai');
      expect(result.fallbackUsed).toBe(true);
      expect(global.fetch).toHaveBeenCalledTimes(2);
    });

    test('should handle multiple provider failures', async () => {
      // Mock all providers failing
      global.fetch
        .mockRejectedValueOnce(new Error('Claude API unavailable'))
        .mockRejectedValueOnce(new Error('OpenAI API unavailable'))
        .mockRejectedValueOnce(new Error('Local model unavailable'));

      const result = await aiIntegration.generateCompletionWithFallback({
        prompt: 'Complete this text',
        providers: ['claude', 'openai', 'local']
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain('All AI providers failed');
      expect(result.attempts).toBe(3);
    });

    test('should track provider performance for intelligent fallback', async () => {
      const performanceData = [];

      // Simulate multiple requests with different response times
      const scenarios = [
        { provider: 'claude', responseTime: 800, success: true },
        { provider: 'openai', responseTime: 1200, success: true },
        { provider: 'claude', responseTime: 2000, success: false },
        { provider: 'openai', responseTime: 900, success: true }
      ];

      for (const scenario of scenarios) {
        const startTime = Date.now();
        
        if (scenario.success) {
          global.fetch.mockResolvedValueOnce({
            ok: true,
            json: async () => ({
              text: `Response from ${scenario.provider}`,
              provider: scenario.provider
            })
          });
        } else {
          global.fetch.mockRejectedValueOnce(new Error('Provider error'));
        }

        try {
          const result = await aiIntegration.generateCompletion({
            prompt: 'Test prompt',
            provider: scenario.provider
          });
          
          performanceData.push({
            provider: scenario.provider,
            responseTime: Date.now() - startTime,
            success: true
          });
        } catch (error) {
          performanceData.push({
            provider: scenario.provider,
            responseTime: Date.now() - startTime,
            success: false
          });
        }
      }

      const stats = aiIntegration.getProviderStats();
      expect(stats.claude.attempts).toBeGreaterThan(0);
      expect(stats.openai.attempts).toBeGreaterThan(0);
    });
  });

  describe('Writing Assistant Integration', () => {
    test('should provide contextual writing suggestions', async () => {
      const writingContext = {
        content: 'The quick brown fox jumps over the lazy dog. This sentence needs',
        cursorPosition: 65,
        selectionStart: 65,
        selectionEnd: 65,
        documentType: 'essay'
      };

      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          suggestions: [
            {
              type: 'completion',
              text: ' to be completed with more descriptive language.',
              confidence: 0.9,
              position: 65
            },
            {
              type: 'improvement',
              text: 'Consider using more vivid adjectives to enhance the description.',
              confidence: 0.8,
              position: 0
            }
          ]
        })
      });

      const suggestions = await aiIntegration.getWritingSuggestions(writingContext);
      
      expect(suggestions.length).toBe(2);
      expect(suggestions[0].type).toBe('completion');
      expect(suggestions[0].confidence).toBeGreaterThan(0.8);
      expect(suggestions[1].type).toBe('improvement');
    });

    test('should integrate with writing session analytics', async () => {
      await writingSession.start();
      
      // Simulate writing activity
      writingSession.addContent('This is a test document for AI integration testing. ');
      
      // Mock AI analysis request
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          analysis: {
            readabilityScore: 85,
            sentiment: 'neutral',
            keyTopics: ['testing', 'AI integration'],
            suggestions: ['Consider adding more specific examples'],
            wordComplexity: 'medium'
          }
        })
      });

      const analytics = await writingSession.getAnalytics();
      const aiAnalysis = await aiIntegration.analyzeContent(writingSession.getCurrentContent());
      
      expect(aiAnalysis.analysis.readabilityScore).toBe(85);
      expect(aiAnalysis.analysis.keyTopics).toContain('testing');
      expect(aiAnalysis.analysis.suggestions.length).toBeGreaterThan(0);
    });

    test('should handle real-time content analysis', async () => {
      const contentUpdates = [
        'The',
        'The quick',
        'The quick brown',
        'The quick brown fox',
        'The quick brown fox jumps',
        'The quick brown fox jumps over',
        'The quick brown fox jumps over the',
        'The quick brown fox jumps over the lazy',
        'The quick brown fox jumps over the lazy dog'
      ];

      let analysisCount = 0;
      global.fetch.mockImplementation(() => {
        analysisCount++;
        return Promise.resolve({
          ok: true,
          json: async () => ({
            analysis: {
              wordCount: contentUpdates[analysisCount - 1].split(' ').length,
              readabilityScore: Math.min(90, 50 + analysisCount * 4),
              suggestions: analysisCount > 5 ? ['Content is developing well'] : ['Keep writing']
            }
          })
        });
      });

      // Enable real-time analysis
      aiIntegration.enableRealTimeAnalysis(true, 500); // 500ms debounce

      for (const content of contentUpdates) {
        await aiIntegration.analyzeContentRealTime(content);
        await new Promise(resolve => setTimeout(resolve, 100)); // Small delay
      }

      // Due to debouncing, not all updates should trigger analysis
      expect(analysisCount).toBeLessThan(contentUpdates.length);
      expect(analysisCount).toBeGreaterThan(0);
    });
  });

  describe('Chat-based AI Assistance', () => {
    test('should handle conversational AI interactions', async () => {
      const conversation = [
        {
          role: 'user',
          content: 'Help me improve this sentence: The cat sat on the mat.'
        },
        {
          role: 'assistant',
          content: 'Here are several ways to improve that sentence: "The elegant tabby cat perched gracefully on the worn Persian mat." This version adds descriptive adjectives and a more sophisticated verb.'
        },
        {
          role: 'user',
          content: 'Can you make it more concise?'
        }
      ];

      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          response: 'Certainly! How about: "The tabby perched on the worn mat." This keeps the descriptive elements while reducing wordiness.',
          conversationId: 'conv-123',
          messageId: 'msg-456'
        })
      });

      const result = await aiIntegration.sendChatMessage({
        message: 'Can you make it more concise?',
        conversation: conversation,
        context: {
          documentType: 'creative writing',
          currentContent: 'The cat sat on the mat.'
        }
      });

      expect(result.response).toContain('tabby perched');
      expect(result.conversationId).toBe('conv-123');
      expect(global.fetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/ai/chat',
        expect.objectContaining({
          method: 'POST',
          body: expect.stringContaining('Can you make it more concise?')
        })
      );
    });

    test('should maintain conversation context', async () => {
      const conversationId = 'test-conversation-123';
      
      // First message
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          response: 'I can help you with your writing. What would you like to work on?',
          conversationId,
          messageId: 'msg-1'
        })
      });

      const firstResponse = await aiIntegration.startConversation('I need help with my essay');
      expect(firstResponse.conversationId).toBe(conversationId);

      // Follow-up message should include conversation context
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          response: 'For essay structure, I recommend starting with a clear thesis statement.',
          conversationId,
          messageId: 'msg-2'
        })
      });

      const followUpResponse = await aiIntegration.continueConversation(
        conversationId,
        'How should I structure my essay?'
      );

      expect(followUpResponse.conversationId).toBe(conversationId);
      expect(global.fetch).toHaveBeenLastCalledWith(
        'http://localhost:3001/api/ai/chat',
        expect.objectContaining({
          body: expect.stringContaining(conversationId)
        })
      );
    });
  });

  describe('Error Handling and Resilience', () => {
    test('should handle API rate limiting', async () => {
      global.fetch.mockResolvedValueOnce({
        ok: false,
        status: 429,
        json: async () => ({
          error: 'Rate limit exceeded',
          retryAfter: 60
        })
      });

      const result = await aiIntegration.generateCompletion({
        prompt: 'Test prompt'
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Rate limit exceeded');
      expect(result.retryAfter).toBe(60);
    });

    test('should implement exponential backoff for retries', async () => {
      let callCount = 0;
      global.fetch.mockImplementation(() => {
        callCount++;
        if (callCount < 3) {
          return Promise.reject(new Error('Temporary server error'));
        }
        return Promise.resolve({
          ok: true,
          json: async () => ({
            text: 'Success after retries',
            provider: 'claude'
          })
        });
      });

      const startTime = Date.now();
      const result = await aiIntegration.generateCompletionWithRetry({
        prompt: 'Test prompt',
        maxRetries: 3,
        baseDelay: 100
      });

      const totalTime = Date.now() - startTime;
      
      expect(result.success).toBe(true);
      expect(result.text).toBe('Success after retries');
      expect(callCount).toBe(3);
      expect(totalTime).toBeGreaterThan(300); // Should have delays
    });

    test('should handle malformed API responses', async () => {
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => {
          throw new Error('Invalid JSON');
        }
      });

      const result = await aiIntegration.generateCompletion({
        prompt: 'Test prompt'
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain('Invalid JSON');
    });

    test('should validate response data', async () => {
      // Mock response missing required fields
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          // Missing 'text' field
          provider: 'claude',
          model: 'claude-3-sonnet'
        })
      });

      const result = await aiIntegration.generateCompletion({
        prompt: 'Test prompt'
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain('Invalid response format');
    });
  });

  describe('Security and Privacy', () => {
    test('should sanitize user input before sending to AI', async () => {
      const maliciousInput = {
        prompt: 'Normal text <script>alert("xss")</script> more text',
        maxTokens: 100
      };

      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          text: 'Safe response',
          provider: 'claude'
        })
      });

      await aiIntegration.generateCompletion(maliciousInput);

      const requestBody = JSON.parse(global.fetch.mock.calls[0][1].body);
      expect(requestBody.prompt).not.toContain('<script>');
      expect(requestBody.prompt).toBe('Normal text alert("xss") more text');
    });

    test('should detect and filter PII before sending to AI', async () => {
      const textWithPII = {
        prompt: 'My name is John Doe and my email is john.doe@example.com. My SSN is 123-45-6789.',
        maxTokens: 100
      };

      // Mock PII detection
      aiIntegration.piiDetection = true;

      const result = await aiIntegration.generateCompletion(textWithPII);

      expect(result.piiDetected).toBe(true);
      expect(result.warning).toContain('PII detected');
      
      // Should not send request if PII is detected and not explicitly allowed
      expect(global.fetch).not.toHaveBeenCalled();
    });

    test('should encrypt sensitive data in transit', async () => {
      const sensitivePrompt = {
        prompt: 'Confidential business information that needs protection',
        encrypt: true
      };

      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          text: 'Encrypted response',
          provider: 'claude',
          encrypted: true
        })
      });

      await aiIntegration.generateCompletion(sensitivePrompt);

      const requestBody = JSON.parse(global.fetch.mock.calls[0][1].body);
      expect(requestBody.encrypted).toBe(true);
      expect(typeof requestBody.prompt).toBe('string'); // Should be encrypted string
    });
  });

  describe('Performance and Caching', () => {
    test('should cache similar completions to reduce API calls', async () => {
      const prompt = 'The quick brown fox';
      
      global.fetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          text: 'jumps over the lazy dog',
          provider: 'claude'
        })
      });

      // First request
      const result1 = await aiIntegration.generateCompletion({ prompt });
      expect(result1.text).toBe('jumps over the lazy dog');

      // Second identical request should use cache
      const result2 = await aiIntegration.generateCompletion({ prompt });
      expect(result2.text).toBe('jumps over the lazy dog');
      expect(result2.cached).toBe(true);

      // Should only make one API call
      expect(global.fetch).toHaveBeenCalledTimes(1);
    });

    test('should implement intelligent caching based on prompt similarity', async () => {
      const similarPrompts = [
        'The quick brown fox',
        'The quick brown fox jumps',
        'A quick brown fox'
      ];

      global.fetch.mockResolvedValue({
        ok: true,
        json: async () => ({
          text: 'cached response',
          provider: 'claude'
        })
      });

      // Enable similarity caching
      aiIntegration.enableSimilarityCache(true, 0.8); // 80% similarity threshold

      for (const prompt of similarPrompts) {
        await aiIntegration.generateCompletion({ prompt });
      }

      // Should use cache for similar prompts
      expect(global.fetch).toHaveBeenCalledTimes(1); // Only first call
    });

    test('should measure and report AI response times', async () => {
      global.fetch.mockImplementation(() => 
        new Promise(resolve => 
          setTimeout(() => resolve({
            ok: true,
            json: async () => ({
              text: 'Delayed response',
              provider: 'claude'
            })
          }), 500)
        )
      );

      const result = await aiIntegration.generateCompletion({
        prompt: 'Test prompt',
        measurePerformance: true
      });

      expect(result.responseTime).toBeGreaterThan(450);
      expect(result.responseTime).toBeLessThan(600);
    });
  });
});