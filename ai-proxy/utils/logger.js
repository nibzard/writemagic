// Winston logger configuration for AI proxy service
const winston = require('winston');
const config = require('../config');

// Create logger with appropriate transports
const logger = winston.createLogger({
  level: config.security.logLevel,
  format: winston.format.combine(
    winston.format.timestamp({
      format: 'YYYY-MM-DD HH:mm:ss.SSS'
    }),
    winston.format.errors({ stack: true }),
    winston.format.json(),
    winston.format.printf(({ timestamp, level, message, ...meta }) => {
      let logMessage = `${timestamp} [${level.toUpperCase()}] ${message}`;
      
      // Add metadata if present
      if (Object.keys(meta).length > 0) {
        // Filter out sensitive information
        const filteredMeta = filterSensitiveData(meta);
        if (Object.keys(filteredMeta).length > 0) {
          logMessage += ` ${JSON.stringify(filteredMeta)}`;
        }
      }
      
      return logMessage;
    })
  ),
  transports: [
    new winston.transports.Console({
      handleExceptions: true,
      handleRejections: true,
    }),
  ],
});

// Add file logging in production
if (config.server.nodeEnv === 'production') {
  logger.add(new winston.transports.File({
    filename: 'logs/error.log',
    level: 'error',
    maxsize: 5242880, // 5MB
    maxFiles: 5,
  }));
  
  logger.add(new winston.transports.File({
    filename: 'logs/combined.log',
    maxsize: 5242880, // 5MB
    maxFiles: 5,
  }));
}

// Filter sensitive data from logs
function filterSensitiveData(obj) {
  const sensitiveKeys = [
    'password',
    'apiKey',
    'api_key',
    'authorization',
    'auth',
    'token',
    'secret',
    'key',
    'credential',
  ];
  
  const filtered = {};
  
  for (const [key, value] of Object.entries(obj)) {
    const lowerKey = key.toLowerCase();
    
    if (sensitiveKeys.some(sensitive => lowerKey.includes(sensitive))) {
      filtered[key] = '[REDACTED]';
    } else if (typeof value === 'object' && value !== null) {
      filtered[key] = filterSensitiveData(value);
    } else {
      filtered[key] = value;
    }
  }
  
  return filtered;
}

// Request logging middleware
function requestLogger(req, res, next) {
  if (!config.security.enableRequestLogging) {
    return next();
  }

  const startTime = Date.now();
  const requestId = req.headers['x-request-id'] || generateRequestId();
  req.requestId = requestId;

  // Log request
  logger.info('HTTP Request', {
    requestId,
    method: req.method,
    url: req.originalUrl,
    userAgent: req.headers['user-agent'],
    ip: req.ip,
    referer: req.headers.referer,
  });

  // Log response
  const originalSend = res.send;
  res.send = function(body) {
    const duration = Date.now() - startTime;
    
    logger.info('HTTP Response', {
      requestId,
      statusCode: res.statusCode,
      duration,
      contentLength: body?.length || 0,
    });

    return originalSend.call(this, body);
  };

  next();
}

// Generate unique request ID
function generateRequestId() {
  return `req_${Date.now()}_${Math.random().toString(36).substring(2, 15)}`;
}

// AI request logging
function logAIRequest(provider, model, request, response = null, error = null, duration = null) {
  const logData = {
    provider,
    model,
    requestTokens: calculateTokens(request.messages),
    responseTokens: response?.usage?.total_tokens || 0,
    duration,
    success: !error,
  };

  if (error) {
    logger.error('AI Request Failed', { ...logData, error: error.message });
  } else {
    logger.info('AI Request Completed', logData);
  }
}

// Simple token estimation (rough approximation)
function calculateTokens(messages) {
  if (!messages || !Array.isArray(messages)) return 0;
  
  return messages.reduce((total, message) => {
    // Rough approximation: 1 token per 4 characters
    return total + Math.ceil((message.content || '').length / 4);
  }, 0);
}

// Error logging helper
function logError(error, context = {}) {
  logger.error('Application Error', {
    message: error.message,
    stack: error.stack,
    ...context,
  });
}

// Security event logging
function logSecurityEvent(event, details = {}) {
  logger.warn('Security Event', {
    event,
    timestamp: new Date().toISOString(),
    ...details,
  });
}

// Performance logging
function logPerformance(operation, duration, metadata = {}) {
  logger.info('Performance Metric', {
    operation,
    duration,
    ...metadata,
  });
}

module.exports = {
  logger,
  requestLogger,
  logAIRequest,
  logError,
  logSecurityEvent,
  logPerformance,
  generateRequestId,
};