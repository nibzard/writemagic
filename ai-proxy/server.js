#!/usr/bin/env node

/**
 * WriteMagic AI Proxy Server
 * Secure API proxy service for client-side AI integration
 */

const express = require('express');
const path = require('path');
const config = require('./config');
const { logger, requestLogger } = require('./utils/logger');

// Import middleware
const { securityHeaders, rateLimiter } = require('./middleware/security');
const { 
  corsMiddleware, 
  handlePreFlight, 
  trackOrigins, 
  developmentCORS, 
  corsSecurityHeaders 
} = require('./middleware/cors');

// Import routes
const aiRoutes = require('./routes/ai');

// Create Express app
const app = express();

// Trust proxy for correct IP addresses
app.set('trust proxy', true);

// Apply security headers first
app.use(securityHeaders);

// Handle pre-flight requests
app.use(handlePreFlight);

// Development CORS (must be before main CORS)
app.use(developmentCORS);

// Main CORS configuration
app.use(corsMiddleware);

// CORS security headers
app.use(corsSecurityHeaders);

// Origin tracking
app.use(trackOrigins);

// Body parsing middleware
app.use(express.json({ 
  limit: config.security.requestSizeLimit,
  strict: true,
}));
app.use(express.urlencoded({ 
  extended: false,
  limit: config.security.requestSizeLimit,
}));

// Request logging
app.use(requestLogger);

// Apply rate limiting
app.use(rateLimiter);

// Health check endpoint (before auth)
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env.npm_package_version || '1.0.0',
    uptime: process.uptime(),
    nodeVersion: process.version,
    environment: config.server.nodeEnv,
  });
});

// API routes
app.use('/api/ai', aiRoutes);

// Root endpoint
app.get('/', (req, res) => {
  res.json({
    service: 'WriteMagic AI Proxy',
    version: process.env.npm_package_version || '1.0.0',
    status: 'running',
    timestamp: new Date().toISOString(),
    endpoints: [
      'GET /health - Health check',
      'GET /api/ai/providers - List available AI providers',
      'GET /api/ai/providers/health - Check provider health',
      'POST /api/ai/complete - Text completion with fallback',
      'POST /api/ai/chat - Conversational AI',
      'GET /api/ai/cache/stats - Cache statistics',
      'POST /api/ai/cache/clear - Clear response cache',
    ],
  });
});

// 404 handler
app.use('*', (req, res) => {
  logger.warn('Not Found', {
    path: req.originalUrl,
    method: req.method,
    ip: req.ip,
    userAgent: req.headers['user-agent'],
  });
  
  res.status(404).json({
    error: 'Not Found',
    message: 'The requested resource was not found',
    path: req.originalUrl,
    timestamp: new Date().toISOString(),
  });
});

// Global error handler
app.use((error, req, res, next) => {
  logger.error('Unhandled Error', {
    error: error.message,
    stack: error.stack,
    path: req.originalUrl,
    method: req.method,
    ip: req.ip,
    requestId: req.requestId,
  });

  // Don't expose internal errors in production
  const isDevelopment = config.server.nodeEnv === 'development';
  
  res.status(500).json({
    error: 'Internal Server Error',
    message: isDevelopment ? error.message : 'An unexpected error occurred',
    requestId: req.requestId,
    timestamp: new Date().toISOString(),
    ...(isDevelopment && { stack: error.stack }),
  });
});

// Graceful shutdown handling
process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

function gracefulShutdown(signal) {
  logger.info(`Received ${signal}, starting graceful shutdown`);
  
  server.close((err) => {
    if (err) {
      logger.error('Error during server shutdown', { error: err.message });
      process.exit(1);
    }
    
    logger.info('Server closed successfully');
    process.exit(0);
  });
  
  // Force exit after 30 seconds
  setTimeout(() => {
    logger.error('Forced shutdown after timeout');
    process.exit(1);
  }, 30000);
}

// Unhandled promise rejection handler
process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled Promise Rejection', {
    reason: reason?.message || reason,
    stack: reason?.stack,
  });
});

// Uncaught exception handler
process.on('uncaughtException', (error) => {
  logger.error('Uncaught Exception', {
    error: error.message,
    stack: error.stack,
  });
  
  // Exit on uncaught exceptions
  process.exit(1);
});

// Start server
const server = app.listen(config.server.port, config.server.host, () => {
  logger.info('AI Proxy Server Started', {
    host: config.server.host,
    port: config.server.port,
    environment: config.server.nodeEnv,
    nodeVersion: process.version,
    pid: process.pid,
    allowedOrigins: config.cors.allowedOrigins,
    availableProviders: Object.keys(config.ai.providers).filter(
      provider => config.ai.providers[provider].apiKey
    ),
  });
});

// Configure server timeouts
server.keepAliveTimeout = 65000; // 65 seconds
server.headersTimeout = 66000; // 66 seconds

module.exports = app;