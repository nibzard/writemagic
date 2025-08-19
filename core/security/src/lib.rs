use std::collections::HashMap;

pub mod headers;
pub mod middleware;
pub mod policies;
pub mod scanner;

pub use headers::SecurityHeaders;
pub use middleware::SecurityMiddleware;
pub use policies::{ContentSecurityPolicy, CorsPolicy};
pub use scanner::SecurityScanner;

/// Security configuration for the application
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable security headers
    pub enable_security_headers: bool,
    /// Content Security Policy configuration
    pub csp_policy: ContentSecurityPolicy,
    /// CORS policy configuration
    pub cors_policy: CorsPolicy,
    /// Enable HTTPS redirection
    pub force_https: bool,
    /// Enable request rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit requests per minute
    pub rate_limit_per_minute: u32,
    /// Enable vulnerability scanning
    pub enable_vulnerability_scanning: bool,
    /// Security headers configuration
    pub security_headers: SecurityHeaders,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_headers: true,
            csp_policy: ContentSecurityPolicy::strict(),
            cors_policy: CorsPolicy::restrictive(),
            force_https: true,
            enable_rate_limiting: true,
            rate_limit_per_minute: 100,
            enable_vulnerability_scanning: true,
            security_headers: SecurityHeaders::default(),
        }
    }
}

impl SecurityConfig {
    /// Create a production-ready security configuration
    pub fn production() -> Self {
        Self {
            enable_security_headers: true,
            csp_policy: ContentSecurityPolicy::strict(),
            cors_policy: CorsPolicy::production(),
            force_https: true,
            enable_rate_limiting: true,
            rate_limit_per_minute: 60, // More restrictive for production
            enable_vulnerability_scanning: true,
            security_headers: SecurityHeaders::strict(),
        }
    }

    /// Create a development configuration with relaxed policies
    pub fn development() -> Self {
        Self {
            enable_security_headers: true,
            csp_policy: ContentSecurityPolicy::development(),
            cors_policy: CorsPolicy::development(),
            force_https: false,
            enable_rate_limiting: false,
            rate_limit_per_minute: 1000,
            enable_vulnerability_scanning: false,
            security_headers: SecurityHeaders::development(),
        }
    }
}