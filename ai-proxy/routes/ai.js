// AI proxy routes
const express = require('express');
const AIClient = require('../services/ai-client');
const { 
  completionValidation, 
  chatValidation, 
  handleValidationErrors,
  contentFilter,
  sanitizeRequest,
} = require('../middleware/security');
const { logError, logSecurityEvent } = require('../utils/logger');
const config = require('../config');

const router = express.Router();
const aiClient = new AIClient();

// Apply middleware to all AI routes
router.use(sanitizeRequest);
router.use(contentFilter);

/**
 * POST /api/ai/complete
 * Text completion with provider fallback
 */
router.post('/complete', completionValidation, handleValidationErrors, async (req, res) => {
  try {
    const startTime = Date.now();
    
    // Extract and validate request
    const completionRequest = {
      messages: req.body.messages,
      model: req.body.model,
      max_tokens: req.body.max_tokens,
      temperature: req.body.temperature,
      top_p: req.body.top_p,
      frequency_penalty: req.body.frequency_penalty,
      presence_penalty: req.body.presence_penalty,
      stop: req.body.stop,
      stream: req.body.stream || false,
    };

    // Log PII warning if detected
    if (req.containsPII) {
      logSecurityEvent('pii_in_request', {
        ip: req.ip,
        endpoint: '/complete',
        requestId: req.requestId,
      });
    }

    // Make AI request
    const response = await aiClient.complete(completionRequest);
    const duration = Date.now() - startTime;

    // Add metadata to response
    const fullResponse = {
      ...response,
      metadata: {
        requestId: req.requestId,
        processingTime: duration,
        cached: response.cached,
        provider: response.provider,
        timestamp: new Date().toISOString(),
      },
    };

    res.json(fullResponse);
  } catch (error) {
    logError(error, { 
      endpoint: '/complete',
      ip: req.ip,
      requestId: req.requestId,
    });

    // Return appropriate error response
    const errorResponse = {
      error: 'AI request failed',
      message: getPublicErrorMessage(error),
      requestId: req.requestId,
      timestamp: new Date().toISOString(),
    };

    const statusCode = getErrorStatusCode(error);
    res.status(statusCode).json(errorResponse);
  }
});

/**
 * POST /api/ai/chat
 * Conversational AI interactions
 */
router.post('/chat', chatValidation, handleValidationErrors, async (req, res) => {
  try {
    const startTime = Date.now();
    
    // Build conversation context
    const messages = [];
    
    // Add context if provided
    if (req.body.context && Array.isArray(req.body.context)) {
      messages.push(...req.body.context);
    }
    
    // Add current message
    messages.push({
      role: 'user',
      content: req.body.message,
    });

    const completionRequest = {
      messages,
      model: req.body.model,
      max_tokens: req.body.max_tokens,
      temperature: req.body.temperature,
    };

    // Log PII warning if detected
    if (req.containsPII) {
      logSecurityEvent('pii_in_request', {
        ip: req.ip,
        endpoint: '/chat',
        requestId: req.requestId,
      });
    }

    const response = await aiClient.complete(completionRequest);
    const duration = Date.now() - startTime;

    // Format chat response
    const chatResponse = {
      message: response.choices?.[0]?.message?.content || '',
      conversationId: req.body.conversation_id || generateConversationId(),
      messageId: generateMessageId(),
      provider: response.provider,
      usage: response.usage,
      metadata: {
        requestId: req.requestId,
        processingTime: duration,
        cached: response.cached,
        timestamp: new Date().toISOString(),
      },
    };

    res.json(chatResponse);
  } catch (error) {
    logError(error, { 
      endpoint: '/chat',
      ip: req.ip,
      requestId: req.requestId,
    });

    const errorResponse = {
      error: 'Chat request failed',
      message: getPublicErrorMessage(error),
      requestId: req.requestId,
      timestamp: new Date().toISOString(),
    };

    const statusCode = getErrorStatusCode(error);
    res.status(statusCode).json(errorResponse);
  }
});

/**
 * GET /api/ai/providers/health
 * Provider status check
 */
router.get('/providers/health', async (req, res) => {
  try {
    const healthResults = await aiClient.healthCheck();
    
    const overallStatus = Object.values(healthResults).every(provider => 
      provider.status === 'healthy'
    ) ? 'healthy' : 'degraded';

    res.json({
      status: overallStatus,
      providers: healthResults,
      timestamp: new Date().toISOString(),
      fallbackOrder: config.ai.fallbackOrder,
    });
  } catch (error) {
    logError(error, { endpoint: '/providers/health' });
    
    res.status(500).json({
      status: 'unhealthy',
      error: 'Health check failed',
      message: 'Unable to check provider status',
      timestamp: new Date().toISOString(),
    });
  }
});

/**
 * GET /api/ai/providers
 * List available providers
 */
router.get('/providers', (req, res) => {
  try {
    const providers = aiClient.getAvailableProviders();
    
    res.json({
      providers,
      defaultProvider: config.ai.defaultProvider,
      fallbackOrder: config.ai.fallbackOrder,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logError(error, { endpoint: '/providers' });
    
    res.status(500).json({
      error: 'Failed to get providers',
      message: 'Unable to retrieve provider information',
      timestamp: new Date().toISOString(),
    });
  }
});

/**
 * GET /api/ai/cache/stats
 * Cache statistics
 */
router.get('/cache/stats', (req, res) => {
  try {
    const stats = aiClient.getCacheStats();
    
    res.json({
      cache: stats,
      config: {
        ttlSeconds: config.cache.ttlSeconds,
        maxItems: config.cache.maxItems,
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logError(error, { endpoint: '/cache/stats' });
    
    res.status(500).json({
      error: 'Failed to get cache stats',
      message: 'Unable to retrieve cache statistics',
      timestamp: new Date().toISOString(),
    });
  }
});

/**
 * POST /api/ai/cache/clear
 * Clear response cache
 */
router.post('/cache/clear', (req, res) => {
  try {
    const result = aiClient.clearCache();
    
    logSecurityEvent('cache_cleared', {
      ip: req.ip,
      requestId: req.requestId,
    });
    
    res.json({
      message: 'Cache cleared successfully',
      result,
    });
  } catch (error) {
    logError(error, { endpoint: '/cache/clear' });
    
    res.status(500).json({
      error: 'Failed to clear cache',
      message: 'Unable to clear response cache',
      timestamp: new Date().toISOString(),
    });
  }
});

// Utility functions

function getPublicErrorMessage(error) {
  // Don't expose internal error details in production
  if (config.server.nodeEnv === 'production') {
    if (error.message.includes('API key') || error.message.includes('authentication')) {
      return 'Authentication error with AI provider';
    }
    if (error.message.includes('rate limit')) {
      return 'AI provider rate limit exceeded. Please try again later.';
    }
    if (error.message.includes('timeout')) {
      return 'AI request timed out. Please try again.';
    }
    return 'An error occurred while processing your request';
  }
  
  return error.message;
}

function getErrorStatusCode(error) {
  if (error.message.includes('authentication') || error.message.includes('API key')) {
    return 502; // Bad Gateway (upstream auth error)
  }
  if (error.message.includes('rate limit')) {
    return 429; // Too Many Requests
  }
  if (error.message.includes('timeout')) {
    return 504; // Gateway Timeout
  }
  if (error.message.includes('validation')) {
    return 400; // Bad Request
  }
  
  return 500; // Internal Server Error
}

function generateConversationId() {
  return `conv_${Date.now()}_${Math.random().toString(36).substring(2, 15)}`;
}

function generateMessageId() {
  return `msg_${Date.now()}_${Math.random().toString(36).substring(2, 15)}`;
}

module.exports = router;