// Security middleware for AI proxy service
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const { body, validationResult } = require('express-validator');
const config = require('../config');
const { logSecurityEvent } = require('../utils/logger');

// Helmet security headers
const securityHeaders = helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      fontSrc: ["'self'"],
      connectSrc: ["'self'"],
      frameSrc: ["'none'"],
      objectSrc: ["'none'"],
      baseUri: ["'self'"],
      formAction: ["'self'"],
    },
  },
  crossOriginEmbedderPolicy: false, // Allow CORS
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
});

// Rate limiting
const rateLimiter = rateLimit({
  windowMs: config.rateLimit.windowMs,
  max: config.rateLimit.maxRequests,
  skipSuccessfulRequests: config.rateLimit.skipSuccessfulRequests,
  message: {
    error: 'Too many requests',
    message: config.rateLimit.message,
    retryAfter: Math.ceil(config.rateLimit.windowMs / 1000),
  },
  standardHeaders: true,
  legacyHeaders: false,
  handler: (req, res) => {
    logSecurityEvent('rate_limit_exceeded', {
      ip: req.ip,
      userAgent: req.headers['user-agent'],
      path: req.path,
    });
    
    res.status(429).json({
      error: 'Rate limit exceeded',
      message: config.rateLimit.message,
      retryAfter: Math.ceil(config.rateLimit.windowMs / 1000),
    });
  },
});

// Content filtering middleware
function contentFilter(req, res, next) {
  if (!config.contentFilter.enabled) {
    return next();
  }

  const content = extractContentFromRequest(req);
  
  if (!content) {
    return next();
  }

  // Check content length
  if (content.length > config.contentFilter.maxInputLength) {
    logSecurityEvent('content_length_exceeded', {
      ip: req.ip,
      contentLength: content.length,
      maxLength: config.contentFilter.maxInputLength,
    });
    
    return res.status(413).json({
      error: 'Content too long',
      message: `Content exceeds maximum length of ${config.contentFilter.maxInputLength} characters`,
      contentLength: content.length,
      maxLength: config.contentFilter.maxInputLength,
    });
  }

  // Check for blocked patterns
  for (const pattern of config.contentFilter.blockedPatterns) {
    if (pattern && content.toLowerCase().includes(pattern.toLowerCase())) {
      logSecurityEvent('blocked_content_detected', {
        ip: req.ip,
        pattern: pattern,
      });
      
      return res.status(400).json({
        error: 'Content not allowed',
        message: 'Content contains blocked patterns',
      });
    }
  }

  // Check for PII patterns
  const piiFound = config.contentFilter.piiPatterns.some(pattern => {
    return pattern.test(content);
  });

  if (piiFound) {
    logSecurityEvent('pii_detected', {
      ip: req.ip,
      contentLength: content.length,
    });
    
    // Allow request but log the event
    req.containsPII = true;
  }

  next();
}

// Extract content from request for filtering
function extractContentFromRequest(req) {
  let content = '';
  
  if (req.body) {
    // Extract from messages array
    if (req.body.messages && Array.isArray(req.body.messages)) {
      content = req.body.messages
        .map(msg => msg.content || '')
        .join(' ');
    }
    
    // Extract from direct content field
    if (req.body.content) {
      content += ' ' + req.body.content;
    }
    
    // Extract from prompt field
    if (req.body.prompt) {
      content += ' ' + req.body.prompt;
    }
  }
  
  return content.trim();
}

// Input validation schemas
const completionValidation = [
  body('messages')
    .isArray({ min: 1 })
    .withMessage('Messages must be a non-empty array'),
  body('messages.*.role')
    .isIn(['system', 'user', 'assistant', 'function'])
    .withMessage('Invalid message role'),
  body('messages.*.content')
    .isString()
    .isLength({ min: 1, max: config.contentFilter.maxInputLength })
    .withMessage('Message content must be a non-empty string'),
  body('model')
    .optional()
    .isString()
    .isLength({ min: 1, max: 100 })
    .withMessage('Model must be a valid string'),
  body('max_tokens')
    .optional()
    .isInt({ min: 1, max: 100000 })
    .withMessage('Max tokens must be between 1 and 100000'),
  body('temperature')
    .optional()
    .isFloat({ min: 0, max: 2 })
    .withMessage('Temperature must be between 0 and 2'),
  body('top_p')
    .optional()
    .isFloat({ min: 0, max: 1 })
    .withMessage('Top_p must be between 0 and 1'),
  body('stream')
    .optional()
    .isBoolean()
    .withMessage('Stream must be a boolean'),
];

const chatValidation = [
  body('message')
    .isString()
    .isLength({ min: 1, max: config.contentFilter.maxInputLength })
    .withMessage('Message must be a non-empty string'),
  body('conversation_id')
    .optional()
    .isString()
    .isLength({ min: 1, max: 100 })
    .withMessage('Conversation ID must be a valid string'),
  body('context')
    .optional()
    .isArray()
    .withMessage('Context must be an array'),
];

// Validation error handler
function handleValidationErrors(req, res, next) {
  const errors = validationResult(req);
  
  if (!errors.isEmpty()) {
    logSecurityEvent('validation_failed', {
      ip: req.ip,
      path: req.path,
      errors: errors.array(),
    });
    
    return res.status(400).json({
      error: 'Validation failed',
      message: 'Invalid input data',
      details: errors.array(),
    });
  }
  
  next();
}

// CORS origin validation
function validateOrigin(origin, callback) {
  if (!origin) {
    // Allow requests with no origin (e.g., mobile apps, Postman)
    return callback(null, true);
  }
  
  if (config.cors.allowedOrigins.includes(origin)) {
    return callback(null, true);
  }
  
  logSecurityEvent('cors_violation', {
    origin,
    allowedOrigins: config.cors.allowedOrigins,
  });
  
  return callback(new Error('Not allowed by CORS'));
}

// Request sanitization
function sanitizeRequest(req, res, next) {
  // Remove potential XSS vectors from string fields
  function sanitizeValue(value) {
    if (typeof value === 'string') {
      return value
        .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
        .replace(/javascript:/gi, '')
        .replace(/on\w+\s*=/gi, '');
    }
    return value;
  }
  
  function sanitizeObject(obj) {
    if (obj && typeof obj === 'object') {
      for (const key in obj) {
        if (Array.isArray(obj[key])) {
          obj[key] = obj[key].map(item => 
            typeof item === 'object' ? sanitizeObject(item) : sanitizeValue(item)
          );
        } else if (typeof obj[key] === 'object') {
          sanitizeObject(obj[key]);
        } else {
          obj[key] = sanitizeValue(obj[key]);
        }
      }
    }
    return obj;
  }
  
  if (req.body) {
    req.body = sanitizeObject(req.body);
  }
  
  next();
}

// IP whitelist middleware (optional)
function ipWhitelist(allowedIPs = []) {
  return (req, res, next) => {
    if (allowedIPs.length === 0) {
      return next();
    }
    
    const clientIP = req.ip || req.connection.remoteAddress;
    
    if (!allowedIPs.includes(clientIP)) {
      logSecurityEvent('ip_blocked', {
        ip: clientIP,
        allowedIPs,
      });
      
      return res.status(403).json({
        error: 'Access denied',
        message: 'IP address not allowed',
      });
    }
    
    next();
  };
}

module.exports = {
  securityHeaders,
  rateLimiter,
  contentFilter,
  completionValidation,
  chatValidation,
  handleValidationErrors,
  validateOrigin,
  sanitizeRequest,
  ipWhitelist,
};