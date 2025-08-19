/**
 * WriteMagic AI Proxy Integration Examples
 * 
 * This file demonstrates how to integrate the AI proxy service
 * with the WriteMagic web application.
 */

class WriteMagicAI {
  constructor(options = {}) {
    this.baseUrl = options.proxyUrl || 'http://localhost:3001';
    this.timeout = options.timeout || 30000;
    this.retryAttempts = options.retryAttempts || 3;
    this.retryDelay = options.retryDelay || 1000;
    
    // Request ID for tracking
    this.generateRequestId = () => 
      `client_${Date.now()}_${Math.random().toString(36).substring(2)}`;
  }

  /**
   * Make a completion request with automatic retry and error handling
   */
  async complete(messages, options = {}) {
    const request = {
      messages,
      model: options.model,
      max_tokens: options.maxTokens || 1000,
      temperature: options.temperature || 0.7,
      top_p: options.topP,
      stop: options.stop,
      stream: options.stream || false,
    };

    return this.makeRequest('/api/ai/complete', request, options);
  }

  /**
   * Chat interface with conversation management
   */
  async chat(message, options = {}) {
    const request = {
      message,
      conversation_id: options.conversationId,
      context: options.context || [],
      model: options.model,
      max_tokens: options.maxTokens || 500,
      temperature: options.temperature || 0.7,
    };

    return this.makeRequest('/api/ai/chat', request, options);
  }

  /**
   * Get AI provider health status
   */
  async getProviderHealth() {
    try {
      const response = await fetch(`${this.baseUrl}/api/ai/providers/health`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Failed to get provider health:', error);
      throw error;
    }
  }

  /**
   * Get available providers
   */
  async getProviders() {
    try {
      const response = await fetch(`${this.baseUrl}/api/ai/providers`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Failed to get providers:', error);
      throw error;
    }
  }

  /**
   * Internal method to make requests with retry logic
   */
  async makeRequest(endpoint, data, options = {}) {
    let lastError;

    for (let attempt = 1; attempt <= this.retryAttempts; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);

        const response = await fetch(`${this.baseUrl}${endpoint}`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Request-ID': this.generateRequestId(),
            ...options.headers,
          },
          credentials: 'include',
          signal: controller.signal,
          body: JSON.stringify(data),
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}));
          throw new AIError(response.status, errorData.message || response.statusText, errorData);
        }

        const result = await response.json();
        return result;

      } catch (error) {
        lastError = error;
        
        // Don't retry on certain errors
        if (error instanceof AIError) {
          if (error.status === 400 || error.status === 401 || error.status === 403) {
            throw error;
          }
        }

        // Don't retry on the last attempt
        if (attempt === this.retryAttempts) {
          break;
        }

        // Wait before retrying
        await new Promise(resolve => 
          setTimeout(resolve, this.retryDelay * attempt)
        );
      }
    }

    throw lastError;
  }
}

/**
 * Custom error class for AI-related errors
 */
class AIError extends Error {
  constructor(status, message, details = {}) {
    super(message);
    this.name = 'AIError';
    this.status = status;
    this.details = details;
  }

  isRateLimit() {
    return this.status === 429;
  }

  isServerError() {
    return this.status >= 500;
  }

  isClientError() {
    return this.status >= 400 && this.status < 500;
  }
}

/**
 * Conversation manager for maintaining chat context
 */
class ConversationManager {
  constructor(aiClient) {
    this.aiClient = aiClient;
    this.conversations = new Map();
    this.maxContextLength = 10; // Maximum messages to keep in context
  }

  async startConversation(systemPrompt = null) {
    const conversationId = `conv_${Date.now()}_${Math.random().toString(36).substring(2)}`;
    
    const conversation = {
      id: conversationId,
      messages: [],
      created: new Date(),
    };

    if (systemPrompt) {
      conversation.messages.push({
        role: 'system',
        content: systemPrompt,
      });
    }

    this.conversations.set(conversationId, conversation);
    return conversationId;
  }

  async sendMessage(conversationId, message, options = {}) {
    const conversation = this.conversations.get(conversationId);
    if (!conversation) {
      throw new Error(`Conversation ${conversationId} not found`);
    }

    // Add user message
    conversation.messages.push({
      role: 'user',
      content: message,
    });

    // Keep context within limits
    const context = this.trimContext(conversation.messages);

    // Make AI request
    const response = await this.aiClient.chat(message, {
      conversationId,
      context: context.slice(0, -1), // Exclude current message
      ...options,
    });

    // Add assistant response
    conversation.messages.push({
      role: 'assistant',
      content: response.message,
    });

    return response;
  }

  trimContext(messages) {
    if (messages.length <= this.maxContextLength) {
      return messages;
    }

    // Keep system message if present
    const hasSystem = messages[0]?.role === 'system';
    const systemMessage = hasSystem ? [messages[0]] : [];
    const otherMessages = hasSystem ? messages.slice(1) : messages;

    // Keep most recent messages
    const recentMessages = otherMessages.slice(-this.maxContextLength + systemMessage.length);
    
    return [...systemMessage, ...recentMessages];
  }

  getConversation(conversationId) {
    return this.conversations.get(conversationId);
  }

  deleteConversation(conversationId) {
    return this.conversations.delete(conversationId);
  }

  listConversations() {
    return Array.from(this.conversations.entries()).map(([id, conv]) => ({
      id,
      messageCount: conv.messages.length,
      created: conv.created,
    }));
  }
}

/**
 * Writing assistant with specialized prompts
 */
class WritingAssistant {
  constructor(aiClient) {
    this.aiClient = aiClient;
    this.conversationManager = new ConversationManager(aiClient);
  }

  async improveText(text, instructions = 'Improve this text') {
    const messages = [
      {
        role: 'system',
        content: 'You are a professional writing assistant. Help improve text while maintaining the author\'s voice and intent.',
      },
      {
        role: 'user',
        content: `${instructions}:\n\n${text}`,
      },
    ];

    const response = await this.aiClient.complete(messages, {
      temperature: 0.3, // Lower temperature for consistency
      maxTokens: text.length * 2, // Roughly twice the input length
    });

    return response.choices[0].message.content;
  }

  async generateIdeas(topic, count = 5) {
    const messages = [
      {
        role: 'system',
        content: 'You are a creative writing assistant. Generate creative and diverse ideas.',
      },
      {
        role: 'user',
        content: `Generate ${count} creative writing ideas about: ${topic}`,
      },
    ];

    const response = await this.aiClient.complete(messages, {
      temperature: 0.8, // Higher temperature for creativity
      maxTokens: 500,
    });

    return response.choices[0].message.content;
  }

  async checkGrammar(text) {
    const messages = [
      {
        role: 'system',
        content: 'You are a grammar and style checker. Identify and suggest corrections for grammatical errors, style issues, and clarity improvements.',
      },
      {
        role: 'user',
        content: `Please check this text for grammar, style, and clarity issues:\n\n${text}`,
      },
    ];

    const response = await this.aiClient.complete(messages, {
      temperature: 0.1, // Very low temperature for accuracy
      maxTokens: 1000,
    });

    return response.choices[0].message.content;
  }

  async startWritingSession(topic, genre = 'general') {
    const systemPrompt = `You are a writing coach helping with ${genre} writing. 
    The topic is: ${topic}. 
    Provide helpful suggestions, ask clarifying questions, and offer encouragement.
    Keep responses concise and actionable.`;

    return await this.conversationManager.startConversation(systemPrompt);
  }

  async continueWritingSession(conversationId, message) {
    return await this.conversationManager.sendMessage(conversationId, message, {
      temperature: 0.6,
      maxTokens: 300,
    });
  }
}

/**
 * Usage examples
 */
async function examples() {
  // Initialize AI client
  const ai = new WriteMagicAI({
    proxyUrl: 'http://localhost:3001',
    timeout: 30000,
    retryAttempts: 3,
  });

  try {
    // Example 1: Simple completion
    console.log('Example 1: Simple completion');
    const completion = await ai.complete([
      { role: 'user', content: 'Write a short poem about coding' }
    ], {
      maxTokens: 200,
      temperature: 0.8,
    });
    console.log('Poem:', completion.choices[0].message.content);
    console.log('Used provider:', completion.metadata.provider);

    // Example 2: Chat conversation
    console.log('\nExample 2: Chat conversation');
    const chatResponse = await ai.chat('What are the best practices for writing clean code?', {
      maxTokens: 300,
    });
    console.log('Response:', chatResponse.message);

    // Example 3: Writing assistant
    console.log('\nExample 3: Writing assistant');
    const assistant = new WritingAssistant(ai);
    
    const improvedText = await assistant.improveText(
      'This is a text that could be better written and more clear.',
      'Make this text more professional and clear'
    );
    console.log('Improved text:', improvedText);

    // Example 4: Provider health check
    console.log('\nExample 4: Provider health');
    const health = await ai.getProviderHealth();
    console.log('Provider status:', health.status);
    console.log('Available providers:', Object.keys(health.providers));

    // Example 5: Writing session
    console.log('\nExample 5: Writing session');
    const sessionId = await assistant.startWritingSession('science fiction', 'creative');
    const sessionResponse = await assistant.continueWritingSession(
      sessionId, 
      'I want to write about time travel but avoid clichés'
    );
    console.log('Writing advice:', sessionResponse.message);

  } catch (error) {
    if (error instanceof AIError) {
      console.error(`AI Error (${error.status}):`, error.message);
      if (error.isRateLimit()) {
        console.log('Rate limit exceeded, please wait before retrying');
      }
    } else {
      console.error('Unexpected error:', error.message);
    }
  }
}

// Export for use in web applications
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    WriteMagicAI,
    AIError,
    ConversationManager,
    WritingAssistant,
  };
}

// Run examples if called directly
if (typeof require !== 'undefined' && require.main === module) {
  examples().catch(console.error);
}