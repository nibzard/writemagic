// CORS middleware configuration
const cors = require('cors');
const config = require('../config');
const { validateOrigin } = require('./security');
const { logSecurityEvent } = require('../utils/logger');

// CORS configuration
const corsOptions = {
  origin: (origin, callback) => {
    validateOrigin(origin, callback);
  },
  credentials: config.cors.credentials,
  methods: config.cors.methods,
  allowedHeaders: config.cors.allowedHeaders,
  optionsSuccessStatus: 200, // For legacy browser support
  maxAge: 86400, // 24 hours
  
  // Custom error handler
  onError: (error, req, res, next) => {
    logSecurityEvent('cors_error', {
      origin: req.headers.origin,
      method: req.method,
      path: req.path,
      ip: req.ip,
      error: error.message,
    });
    
    res.status(403).json({
      error: 'CORS policy violation',
      message: 'Origin not allowed',
      origin: req.headers.origin,
    });
  },
};

// CORS middleware
const corsMiddleware = cors(corsOptions);

// Pre-flight handler with additional validation
function handlePreFlight(req, res, next) {
  if (req.method === 'OPTIONS') {
    // Additional pre-flight checks
    const origin = req.headers.origin;
    const requestedMethod = req.headers['access-control-request-method'];
    const requestedHeaders = req.headers['access-control-request-headers'];
    
    // Log pre-flight requests for monitoring
    logSecurityEvent('preflight_request', {
      origin,
      requestedMethod,
      requestedHeaders,
      ip: req.ip,
    });
    
    // Validate requested method
    if (requestedMethod && !config.cors.methods.includes(requestedMethod)) {
      logSecurityEvent('invalid_preflight_method', {
        origin,
        requestedMethod,
        allowedMethods: config.cors.methods,
        ip: req.ip,
      });
      
      return res.status(405).json({
        error: 'Method not allowed',
        method: requestedMethod,
        allowedMethods: config.cors.methods,
      });
    }
    
    // Validate requested headers
    if (requestedHeaders) {
      const headers = requestedHeaders.split(',').map(h => h.trim().toLowerCase());
      const allowedHeaders = config.cors.allowedHeaders.map(h => h.toLowerCase());
      
      const invalidHeaders = headers.filter(h => !allowedHeaders.includes(h));
      if (invalidHeaders.length > 0) {
        logSecurityEvent('invalid_preflight_headers', {
          origin,
          invalidHeaders,
          allowedHeaders: config.cors.allowedHeaders,
          ip: req.ip,
        });
        
        return res.status(400).json({
          error: 'Headers not allowed',
          invalidHeaders,
          allowedHeaders: config.cors.allowedHeaders,
        });
      }
    }
  }
  
  next();
}

// Origin tracking middleware
function trackOrigins(req, res, next) {
  const origin = req.headers.origin;
  
  if (origin) {
    // Track origin usage for analytics
    req.clientOrigin = origin;
    
    // Log new origins (not in allowed list) for security monitoring
    if (!config.cors.allowedOrigins.includes(origin)) {
      logSecurityEvent('unknown_origin_access', {
        origin,
        path: req.path,
        method: req.method,
        ip: req.ip,
        userAgent: req.headers['user-agent'],
      });
    }
  }
  
  next();
}

// Dynamic CORS for development
function developmentCORS(req, res, next) {
  if (config.server.nodeEnv === 'development') {
    // Allow localhost origins in development
    const origin = req.headers.origin;
    if (origin && (origin.includes('localhost') || origin.includes('127.0.0.1'))) {
      res.header('Access-Control-Allow-Origin', origin);
      res.header('Access-Control-Allow-Credentials', 'true');
      res.header('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE,OPTIONS');
      res.header('Access-Control-Allow-Headers', 
        'Origin,X-Requested-With,Content-Type,Accept,Authorization,X-Session-ID,X-Request-ID'
      );
      
      if (req.method === 'OPTIONS') {
        return res.sendStatus(200);
      }
    }
  }
  
  next();
}

// Security headers for CORS
function corsSecurityHeaders(req, res, next) {
  const origin = req.headers.origin;
  
  // Add security headers based on origin
  if (origin) {
    // Strict transport security for HTTPS origins
    if (origin.startsWith('https://')) {
      res.header('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
    }
    
    // Content security policy
    res.header('Content-Security-Policy', 
      `default-src 'self'; connect-src 'self' ${origin}; frame-ancestors 'none';`
    );
    
    // Referrer policy
    res.header('Referrer-Policy', 'strict-origin-when-cross-origin');
    
    // Feature policy
    res.header('Permissions-Policy', 
      'geolocation=(), microphone=(), camera=(), payment=(), usb=()'
    );
  }
  
  next();
}

module.exports = {
  corsMiddleware,
  handlePreFlight,
  trackOrigins,
  developmentCORS,
  corsSecurityHeaders,
};