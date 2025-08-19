// Configuration management for AI proxy service
require('dotenv').config();

const config = {
  server: {
    port: parseInt(process.env.PORT, 10) || 3001,
    host: process.env.HOST || 'localhost',
    nodeEnv: process.env.NODE_ENV || 'development',
  },

  cors: {
    origin: process.env.CORS_ORIGIN?.split(',') || ['http://localhost:3000'],
    allowedOrigins: process.env.ALLOWED_ORIGINS?.split(',') || [
      'http://localhost:3000',
      'http://localhost:8080',
      'https://writemagic.app'
    ],
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowedHeaders: [
      'Content-Type',
      'Authorization',
      'X-Requested-With',
      'X-Session-ID',
      'X-Request-ID'
    ],
  },

  ai: {
    providers: {
      claude: {
        apiKey: process.env.CLAUDE_API_KEY,
        baseUrl: process.env.CLAUDE_BASE_URL || 'https://api.anthropic.com',
        defaultModel: 'claude-3-sonnet-20240229',
        maxTokens: parseInt(process.env.CLAUDE_MAX_TOKENS, 10) || 100000,
        temperature: parseFloat(process.env.CLAUDE_TEMPERATURE) || 0.7,
        rateLimitConcurrency: 5,
        rateLimitIntervalMs: 200,
      },
      openai: {
        apiKey: process.env.OPENAI_API_KEY,
        baseUrl: process.env.OPENAI_BASE_URL || 'https://api.openai.com',
        defaultModel: 'gpt-4-turbo-preview',
        maxTokens: parseInt(process.env.OPENAI_MAX_TOKENS, 10) || 4096,
        temperature: parseFloat(process.env.OPENAI_TEMPERATURE) || 0.7,
        rateLimitConcurrency: 10,
        rateLimitIntervalMs: 100,
      },
    },
    defaultProvider: process.env.DEFAULT_PROVIDER || 'claude',
    fallbackOrder: ['claude', 'openai'],
  },

  rateLimit: {
    windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS, 10) || 15 * 60 * 1000, // 15 minutes
    maxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS, 10) || 100,
    skipSuccessfulRequests: process.env.RATE_LIMIT_SKIP_SUCCESSFUL_REQUESTS === 'true',
    message: 'Too many requests from this IP, please try again later.',
  },

  cache: {
    ttlSeconds: parseInt(process.env.RESPONSE_CACHE_TTL_SECONDS, 10) || 300, // 5 minutes
    maxItems: parseInt(process.env.RESPONSE_CACHE_MAX_ITEMS, 10) || 1000,
    checkPeriodSeconds: 120, // 2 minutes
  },

  security: {
    requestSizeLimit: process.env.REQUEST_SIZE_LIMIT || '10mb',
    enableRequestLogging: process.env.ENABLE_REQUEST_LOGGING === 'true',
    logLevel: process.env.LOG_LEVEL || 'info',
  },

  contentFilter: {
    enabled: process.env.ENABLE_CONTENT_FILTER !== 'false',
    maxInputLength: parseInt(process.env.MAX_INPUT_LENGTH, 10) || 50000,
    blockedPatterns: process.env.BLOCKED_PATTERNS?.split(',') || [],
    piiPatterns: [
      // Email addresses
      /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g,
      // Phone numbers (US format)
      /\b\d{3}-?\d{3}-?\d{4}\b/g,
      // Social Security Numbers
      /\b\d{3}-?\d{2}-?\d{4}\b/g,
      // Credit card patterns (basic)
      /\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b/g,
    ],
  },

  monitoring: {
    enabled: process.env.ENABLE_METRICS !== 'false',
    healthCheckIntervalMs: parseInt(process.env.HEALTH_CHECK_INTERVAL_MS, 10) || 30000,
  },
};

// Validation
function validateConfig() {
  const errors = [];

  if (!config.ai.providers.claude.apiKey && !config.ai.providers.openai.apiKey) {
    errors.push('At least one AI provider API key must be configured');
  }

  if (config.server.port < 1 || config.server.port > 65535) {
    errors.push('Invalid server port number');
  }

  if (config.rateLimit.maxRequests < 1) {
    errors.push('Rate limit max requests must be greater than 0');
  }

  if (config.cache.ttlSeconds < 0) {
    errors.push('Cache TTL must be non-negative');
  }

  if (errors.length > 0) {
    throw new Error(`Configuration validation failed:\n${errors.join('\n')}`);
  }
}

// Validate configuration on startup
validateConfig();

module.exports = config;