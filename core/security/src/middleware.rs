use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{SecurityConfig, SecurityHeaders, ContentSecurityPolicy, CorsPolicy};

/// Security middleware for HTTP requests
pub struct SecurityMiddleware {
    config: Arc<SecurityConfig>,
    rate_limiter: Option<RateLimiter>,
}

impl SecurityMiddleware {
    pub fn new(config: SecurityConfig) -> Self {
        let rate_limiter = if config.enable_rate_limiting {
            Some(RateLimiter::new(config.rate_limit_per_minute))
        } else {
            None
        };

        Self {
            config: Arc::new(config),
            rate_limiter,
        }
    }

    /// Apply security headers to response
    pub fn apply_security_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if self.config.enable_security_headers {
            // Apply standard security headers
            let security_headers = self.config.security_headers.to_http_headers();
            headers.extend(security_headers);

            // Apply Content Security Policy
            let csp_header = self.config.csp_policy.to_header_value();
            headers.insert("Content-Security-Policy".to_string(), csp_header);

            // Apply CORS headers
            let cors_headers = self.config.cors_policy.to_headers();
            headers.extend(cors_headers);
        }

        // Add server information hiding
        headers.insert("Server".to_string(), "WriteMagic".to_string());
        
        // Add security-focused headers
        headers.insert("X-Robots-Tag".to_string(), "noindex, nofollow".to_string());
        
        headers
    }

    /// Check if request should be rate limited
    pub fn check_rate_limit(&mut self, client_ip: &str) -> Result<(), RateLimitError> {
        if let Some(ref mut limiter) = self.rate_limiter {
            limiter.check_rate_limit(client_ip)
        } else {
            Ok(())
        }
    }

    /// Validate request for security issues
    pub fn validate_request(&self, request: &HttpRequest) -> Result<(), SecurityError> {
        // Check for suspicious headers
        if let Some(user_agent) = request.headers.get("user-agent") {
            if self.is_suspicious_user_agent(user_agent) {
                return Err(SecurityError::SuspiciousRequest("Suspicious User-Agent".to_string()));
            }
        }

        // Check for path traversal attempts
        if request.path.contains("..") || request.path.contains("~") {
            return Err(SecurityError::PathTraversal);
        }

        // Check for SQL injection patterns (basic)
        if self.contains_sql_injection_pattern(&request.path) || 
           self.contains_sql_injection_pattern(&request.query_string.unwrap_or_default()) {
            return Err(SecurityError::SqlInjectionAttempt);
        }

        // Check request size
        if request.content_length.unwrap_or(0) > 10_000_000 { // 10MB limit
            return Err(SecurityError::RequestTooLarge);
        }

        Ok(())
    }

    fn is_suspicious_user_agent(&self, user_agent: &str) -> bool {
        let suspicious_patterns = [
            "sqlmap", "nmap", "nikto", "dirb", "gobuster", "wfuzz", "burp",
            "havij", "pangolin", "jsql", "NoSQLMap", "bsqlbf"
        ];

        let user_agent_lower = user_agent.to_lowercase();
        suspicious_patterns.iter().any(|pattern| user_agent_lower.contains(pattern))
    }

    fn contains_sql_injection_pattern(&self, input: &str) -> bool {
        let sql_patterns = [
            "union select", "drop table", "insert into", "delete from",
            "update set", "create table", "alter table", "truncate",
            "'or'1'='1", "1=1", "admin'--", "' or 1=1--"
        ];

        let input_lower = input.to_lowercase();
        sql_patterns.iter().any(|pattern| input_lower.contains(pattern))
    }
}

/// Simple rate limiter implementation
pub struct RateLimiter {
    requests_per_minute: u32,
    client_requests: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            client_requests: HashMap::new(),
        }
    }

    pub fn check_rate_limit(&mut self, client_ip: &str) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);

        // Get or create client request history
        let requests = self.client_requests.entry(client_ip.to_string()).or_insert_with(Vec::new);

        // Remove requests older than 1 minute
        requests.retain(|&request_time| request_time > minute_ago);

        // Check if rate limit exceeded
        if requests.len() >= self.requests_per_minute as usize {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Add current request
        requests.push(now);

        Ok(())
    }
}

/// HTTP request representation for security validation
pub struct HttpRequest {
    pub path: String,
    pub query_string: Option<String>,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub content_length: Option<usize>,
}

/// Security errors
#[derive(Debug)]
pub enum SecurityError {
    SuspiciousRequest(String),
    PathTraversal,
    SqlInjectionAttempt,
    RequestTooLarge,
}

/// Rate limiting errors
#[derive(Debug)]
pub enum RateLimitError {
    RateLimitExceeded,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::SuspiciousRequest(msg) => write!(f, "Suspicious request: {}", msg),
            SecurityError::PathTraversal => write!(f, "Path traversal attempt detected"),
            SecurityError::SqlInjectionAttempt => write!(f, "SQL injection attempt detected"),
            SecurityError::RequestTooLarge => write!(f, "Request too large"),
        }
    }
}

impl std::error::Error for SecurityError {}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware_creation() {
        let config = SecurityConfig::default();
        let middleware = SecurityMiddleware::new(config);
        
        let headers = middleware.apply_security_headers();
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("Content-Security-Policy"));
    }

    #[test]
    fn test_suspicious_user_agent_detection() {
        let config = SecurityConfig::default();
        let middleware = SecurityMiddleware::new(config);
        
        assert!(middleware.is_suspicious_user_agent("sqlmap/1.4.7"));
        assert!(middleware.is_suspicious_user_agent("Mozilla/5.0 nikto/2.1.6"));
        assert!(!middleware.is_suspicious_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));
    }

    #[test]
    fn test_sql_injection_detection() {
        let config = SecurityConfig::default();
        let middleware = SecurityMiddleware::new(config);
        
        assert!(middleware.contains_sql_injection_pattern("' UNION SELECT * FROM users--"));
        assert!(middleware.contains_sql_injection_pattern("admin' OR '1'='1"));
        assert!(!middleware.contains_sql_injection_pattern("/api/users/123"));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5);
        
        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("127.0.0.1").is_ok());
        }
        
        // Should block 6th request
        assert!(limiter.check_rate_limit("127.0.0.1").is_err());
    }

    #[test]
    fn test_request_validation() {
        let config = SecurityConfig::default();
        let middleware = SecurityMiddleware::new(config);
        
        let mut headers = HashMap::new();
        headers.insert("user-agent".to_string(), "Mozilla/5.0".to_string());
        
        let valid_request = HttpRequest {
            path: "/api/documents".to_string(),
            query_string: Some("limit=10".to_string()),
            headers,
            method: "GET".to_string(),
            content_length: Some(1024),
        };
        
        assert!(middleware.validate_request(&valid_request).is_ok());
        
        let malicious_request = HttpRequest {
            path: "/api/users/../../../etc/passwd".to_string(),
            query_string: None,
            headers: HashMap::new(),
            method: "GET".to_string(),
            content_length: Some(1024),
        };
        
        assert!(middleware.validate_request(&malicious_request).is_err());
    }
}