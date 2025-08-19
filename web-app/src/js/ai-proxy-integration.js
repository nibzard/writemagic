/**
 * AI Proxy Integration for WriteMagic Web App
 * 
 * This module integrates the secure AI proxy service with the WriteMagic web application,
 * providing a seamless bridge between the client-side WASM engine and the server-side AI proxy.
 */

/**
 * AI Proxy Client for WriteMagic
 */
class WriteMagicAIProxy {
    constructor(options = {}) {
        this.baseUrl = options.proxyUrl || 'http://localhost:3001';
        this.timeout = options.timeout || 30000;
        this.retryAttempts = options.retryAttempts || 3;
        this.retryDelay = options.retryDelay || 1000;
        
        // Event callbacks for integration with WriteMagic
        this.eventCallbacks = new Map();
        
        // Provider health tracking
        this.providerHealth = null;
        this.lastHealthCheck = null;
        
        // Request tracking for analytics
        this.requestCount = 0;
        this.totalTokensUsed = 0;
        this.totalCost = 0;
        
        // Initialize health check
        this.checkProviderHealth();
    }

    /**
     * Initialize AI proxy integration with WriteMagic instance
     */
    static async integrate(writeMagicInstance, proxyOptions = {}) {
        const proxy = new WriteMagicAIProxy(proxyOptions);
        
        // Wait for WriteMagic to be initialized
        if (!writeMagicInstance.isInitialized) {
            await new Promise((resolve) => {
                writeMagicInstance.on('writemagic:initialized', resolve);
            });
        }
        
        // Override AI methods in WriteMagic to use proxy
        proxy.overrideAIMethods(writeMagicInstance);
        
        // Set up event forwarding
        proxy.setupEventForwarding(writeMagicInstance);
        
        return proxy;
    }

    /**
     * Override AI methods in WriteMagic instance to use proxy instead of direct WASM calls
     */
    overrideAIMethods(writeMagic) {
        const originalCompleteText = writeMagic.completeText.bind(writeMagic);
        const originalGetWritingSuggestions = writeMagic.getWritingSuggestions.bind(writeMagic);
        const originalCheckAIHealth = writeMagic.checkAIHealth.bind(writeMagic);
        
        // Override completeText method
        writeMagic.completeText = async (prompt, options = {}) => {
            try {
                const response = await this.complete([
                    { role: 'user', content: prompt }
                ], {
                    model: options.model,
                    max_tokens: options.maxTokens,
                    temperature: options.temperature,
                });

                // Track usage
                this.trackUsage(response);

                // Emit WriteMagic events
                writeMagic.emit('writemagic:ai_completion', {
                    prompt,
                    response: response.choices[0].message.content,
                    model: response.model,
                    tokensUsed: response.usage.total_tokens,
                    provider: response.metadata.provider,
                });

                return {
                    content: response.choices[0].message.content,
                    model: response.model,
                    tokensUsed: response.usage.total_tokens,
                    finishReason: response.choices[0].finish_reason,
                    provider: response.metadata.provider,
                    cached: response.metadata.cached,
                };
            } catch (error) {
                writeMagic.emit('writemagic:ai_error', { error, prompt });
                
                // Fallback to WASM engine if proxy fails
                console.warn('AI proxy failed, falling back to WASM engine:', error.message);
                return originalCompleteText(prompt, options);
            }
        };

        // Override getWritingSuggestions method
        writeMagic.getWritingSuggestions = async (content, suggestionType = 'improve') => {
            try {
                const prompts = {
                    improve: `Please suggest improvements for this text while maintaining the author's voice:\n\n${content}`,
                    expand: `Please expand on this text with additional details and examples:\n\n${content}`,
                    summarize: `Please provide a concise summary of this text:\n\n${content}`,
                    rewrite: `Please rewrite this text for better clarity and flow:\n\n${content}`,
                    grammar: `Please fix any grammar and spelling errors in this text:\n\n${content}`,
                };

                const prompt = prompts[suggestionType] || prompts.improve;

                const response = await this.complete([
                    { role: 'system', content: 'You are a professional writing assistant. Provide helpful, specific suggestions while maintaining the author\'s voice.' },
                    { role: 'user', content: prompt }
                ], {
                    max_tokens: 1000,
                    temperature: 0.3,
                });

                this.trackUsage(response);

                return {
                    content: response.choices[0].message.content,
                    model: response.model,
                    tokensUsed: response.usage.total_tokens,
                    suggestionType,
                    provider: response.metadata.provider,
                };
            } catch (error) {
                console.warn('AI proxy suggestion failed, falling back to WASM engine:', error.message);
                return originalGetWritingSuggestions(content, suggestionType);
            }
        };

        // Override checkAIHealth method
        writeMagic.checkAIHealth = async () => {
            try {
                return await this.getProviderHealth();
            } catch (error) {
                console.warn('AI proxy health check failed, falling back to WASM engine:', error.message);
                return originalCheckAIHealth();
            }
        };

        // Add new methods for proxy-specific functionality
        writeMagic.getAIUsageStats = () => this.getUsageStats();
        writeMagic.clearAICache = () => this.clearCache();
        writeMagic.getAvailableAIProviders = () => this.getProviders();
    }

    /**
     * Set up event forwarding between proxy and WriteMagic
     */
    setupEventForwarding(writeMagic) {
        // Forward proxy events to WriteMagic events
        this.on('provider_health_changed', (data) => {
            writeMagic.emit('writemagic:ai_provider_health_changed', data);
        });

        this.on('usage_stats_updated', (data) => {
            writeMagic.emit('writemagic:ai_usage_updated', data);
        });
    }

    /**
     * Make a completion request through the proxy
     */
    async complete(messages, options = {}) {
        const request = {
            messages,
            model: options.model,
            max_tokens: options.max_tokens || options.maxTokens,
            temperature: options.temperature,
            top_p: options.top_p || options.topP,
            stop: options.stop,
            stream: options.stream || false,
        };

        return this.makeRequest('/api/ai/complete', request, options);
    }

    /**
     * Chat interface through the proxy
     */
    async chat(message, options = {}) {
        const request = {
            message,
            conversation_id: options.conversationId,
            context: options.context || [],
            model: options.model,
            max_tokens: options.maxTokens,
            temperature: options.temperature,
        };

        return this.makeRequest('/api/ai/chat', request, options);
    }

    /**
     * Get provider health status
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

            const health = await response.json();
            
            // Update cached health status
            const previousHealth = this.providerHealth;
            this.providerHealth = health;
            this.lastHealthCheck = Date.now();

            // Emit health change event if status changed
            if (previousHealth && previousHealth.status !== health.status) {
                this.emit('provider_health_changed', {
                    previous: previousHealth,
                    current: health,
                });
            }

            return health;
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
     * Get cache statistics
     */
    async getCacheStats() {
        try {
            const response = await fetch(`${this.baseUrl}/api/ai/cache/stats`, {
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
            console.error('Failed to get cache stats:', error);
            throw error;
        }
    }

    /**
     * Clear response cache
     */
    async clearCache() {
        try {
            const response = await fetch(`${this.baseUrl}/api/ai/cache/clear`, {
                method: 'POST',
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
            console.error('Failed to clear cache:', error);
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
                    throw new AIProxyError(response.status, errorData.message || response.statusText, errorData);
                }

                const result = await response.json();
                this.requestCount++;
                
                return result;

            } catch (error) {
                lastError = error;
                
                // Don't retry on certain errors
                if (error instanceof AIProxyError) {
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

    /**
     * Track usage statistics
     */
    trackUsage(response) {
        if (response.usage) {
            this.totalTokensUsed += response.usage.total_tokens;
            
            // Estimate cost based on provider
            const provider = response.metadata?.provider;
            if (provider === 'claude') {
                // Claude pricing (approximate)
                this.totalCost += (response.usage.prompt_tokens * 0.00001) + 
                                 (response.usage.completion_tokens * 0.00003);
            } else if (provider === 'openai') {
                // OpenAI pricing (approximate)
                this.totalCost += (response.usage.prompt_tokens * 0.00001) + 
                                 (response.usage.completion_tokens * 0.00003);
            }

            this.emit('usage_stats_updated', {
                requestCount: this.requestCount,
                totalTokens: this.totalTokensUsed,
                estimatedCost: this.totalCost,
            });
        }
    }

    /**
     * Get usage statistics
     */
    getUsageStats() {
        return {
            requestCount: this.requestCount,
            totalTokensUsed: this.totalTokensUsed,
            estimatedCost: this.totalCost,
            lastHealthCheck: this.lastHealthCheck,
            providerHealth: this.providerHealth,
        };
    }

    /**
     * Periodic health check
     */
    async checkProviderHealth() {
        try {
            await this.getProviderHealth();
        } catch (error) {
            console.warn('Provider health check failed:', error.message);
        }

        // Schedule next health check
        setTimeout(() => this.checkProviderHealth(), 60000); // Every minute
    }

    /**
     * Generate unique request ID
     */
    generateRequestId() {
        return `web_${Date.now()}_${Math.random().toString(36).substring(2)}`;
    }

    /**
     * Event management
     */
    on(event, callback) {
        if (!this.eventCallbacks.has(event)) {
            this.eventCallbacks.set(event, new Set());
        }
        this.eventCallbacks.get(event).add(callback);
        return this;
    }

    off(event, callback) {
        if (this.eventCallbacks.has(event)) {
            this.eventCallbacks.get(event).delete(callback);
        }
        return this;
    }

    emit(event, data) {
        if (this.eventCallbacks.has(event)) {
            for (const callback of this.eventCallbacks.get(event)) {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`Error in event callback for '${event}':`, error);
                }
            }
        }
    }
}

/**
 * Custom error class for AI proxy errors
 */
class AIProxyError extends Error {
    constructor(status, message, details = {}) {
        super(message);
        this.name = 'AIProxyError';
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
 * Enhanced Writing Assistant with proxy integration
 */
class ProxyWritingAssistant {
    constructor(aiProxy) {
        this.aiProxy = aiProxy;
        this.conversations = new Map();
        this.maxContextLength = 10;
    }

    /**
     * Improve text with specific instructions
     */
    async improveText(text, instructions = 'Improve this text for clarity and flow') {
        try {
            const response = await this.aiProxy.complete([
                {
                    role: 'system',
                    content: 'You are a professional writing assistant. Help improve text while maintaining the author\'s voice and intent. Provide specific, actionable improvements.',
                },
                {
                    role: 'user',
                    content: `${instructions}:\n\n${text}`,
                },
            ], {
                temperature: 0.3,
                max_tokens: Math.max(500, text.length),
            });

            return {
                improvedText: response.choices[0].message.content,
                originalLength: text.length,
                improvedLength: response.choices[0].message.content.length,
                tokensUsed: response.usage.total_tokens,
                provider: response.metadata.provider,
            };
        } catch (error) {
            console.error('Text improvement failed:', error);
            throw error;
        }
    }

    /**
     * Generate writing ideas
     */
    async generateIdeas(topic, style = 'general', count = 5) {
        try {
            const response = await this.aiProxy.complete([
                {
                    role: 'system',
                    content: `You are a creative writing assistant. Generate diverse, original ideas for ${style} writing that inspire and engage writers.`,
                },
                {
                    role: 'user',
                    content: `Generate ${count} creative writing ideas about: ${topic}`,
                },
            ], {
                temperature: 0.8,
                max_tokens: 600,
            });

            return {
                ideas: response.choices[0].message.content,
                topic,
                style,
                count,
                provider: response.metadata.provider,
            };
        } catch (error) {
            console.error('Idea generation failed:', error);
            throw error;
        }
    }

    /**
     * Check grammar and style
     */
    async checkGrammar(text) {
        try {
            const response = await this.aiProxy.complete([
                {
                    role: 'system',
                    content: 'You are an expert grammar and style checker. Identify errors and provide clear explanations for improvements. Be specific and helpful.',
                },
                {
                    role: 'user',
                    content: `Please check this text for grammar, style, and clarity issues:\n\n${text}`,
                },
            ], {
                temperature: 0.1,
                max_tokens: 1000,
            });

            return {
                feedback: response.choices[0].message.content,
                originalText: text,
                tokensUsed: response.usage.total_tokens,
                provider: response.metadata.provider,
            };
        } catch (error) {
            console.error('Grammar check failed:', error);
            throw error;
        }
    }

    /**
     * Start a writing conversation
     */
    async startWritingConversation(topic, genre = 'general') {
        const conversationId = `conv_${Date.now()}_${Math.random().toString(36).substring(2)}`;
        
        const systemPrompt = `You are an encouraging writing coach specializing in ${genre} writing. 
        Help the writer explore ideas about: ${topic}
        Ask thoughtful questions, provide constructive feedback, and offer specific suggestions.
        Keep responses concise but insightful.`;

        const conversation = {
            id: conversationId,
            topic,
            genre,
            messages: [{ role: 'system', content: systemPrompt }],
            created: new Date(),
        };

        this.conversations.set(conversationId, conversation);
        return conversationId;
    }

    /**
     * Continue writing conversation
     */
    async continueConversation(conversationId, userMessage) {
        const conversation = this.conversations.get(conversationId);
        if (!conversation) {
            throw new Error(`Conversation ${conversationId} not found`);
        }

        // Add user message
        conversation.messages.push({
            role: 'user',
            content: userMessage,
        });

        // Trim context if needed
        const context = this.trimContext(conversation.messages);

        try {
            const response = await this.aiProxy.complete(context, {
                temperature: 0.6,
                max_tokens: 400,
            });

            // Add assistant response
            conversation.messages.push({
                role: 'assistant',
                content: response.choices[0].message.content,
            });

            return {
                response: response.choices[0].message.content,
                conversationId,
                messageCount: conversation.messages.length,
                provider: response.metadata.provider,
            };
        } catch (error) {
            console.error('Conversation continuation failed:', error);
            throw error;
        }
    }

    /**
     * Trim conversation context
     */
    trimContext(messages) {
        if (messages.length <= this.maxContextLength) {
            return messages;
        }

        // Keep system message and most recent messages
        const systemMessage = messages[0].role === 'system' ? [messages[0]] : [];
        const recentMessages = messages.slice(-this.maxContextLength + systemMessage.length);
        
        return [...systemMessage, ...recentMessages];
    }
}

/**
 * Usage example
 */
async function integrateAIProxy() {
    // Assume WriteMagic instance exists
    if (typeof WriteMagic === 'undefined') {
        console.error('WriteMagic not loaded. Make sure to load the main WriteMagic library first.');
        return;
    }

    try {
        // Create WriteMagic instance
        const writeMagic = new WriteMagic({
            claude_api_key: null, // Will use proxy instead
            openai_api_key: null, // Will use proxy instead
            default_model: 'claude-3-sonnet-20240229',
            enable_analytics: true,
        });

        // Integrate AI proxy
        const aiProxy = await WriteMagicAIProxy.integrate(writeMagic, {
            proxyUrl: 'http://localhost:3001',
            timeout: 30000,
            retryAttempts: 3,
        });

        // Create enhanced writing assistant
        const assistant = new ProxyWritingAssistant(aiProxy);

        // Set up event listeners
        writeMagic.on('writemagic:ai_completion', (data) => {
            console.log('AI completion:', {
                provider: data.provider,
                tokensUsed: data.tokensUsed,
                cached: data.cached,
            });
        });

        writeMagic.on('writemagic:ai_provider_health_changed', (data) => {
            console.log('AI provider health changed:', data.current.status);
        });

        // Example usage
        console.log('WriteMagic AI Proxy integration complete!');
        
        // Test text completion
        const completion = await writeMagic.completeText('Write a short paragraph about the importance of clear writing');
        console.log('Completion result:', completion);

        // Test writing assistance
        const improved = await assistant.improveText('This text needs improvement and better clarity.');
        console.log('Improved text:', improved.improvedText);

        // Check provider health
        const health = await writeMagic.checkAIHealth();
        console.log('Provider health:', health.status);

        return { writeMagic, aiProxy, assistant };

    } catch (error) {
        console.error('AI proxy integration failed:', error);
        throw error;
    }
}

// Export for use in web applications
export {
    WriteMagicAIProxy,
    AIProxyError,
    ProxyWritingAssistant,
    integrateAIProxy,
};

// Auto-integrate if WriteMagic is available globally
if (typeof window !== 'undefined' && window.WriteMagic) {
    window.integrateAIProxy = integrateAIProxy;
    window.WriteMagicAIProxy = WriteMagicAIProxy;
    window.ProxyWritingAssistant = ProxyWritingAssistant;
}