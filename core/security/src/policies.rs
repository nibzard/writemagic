use std::collections::HashMap;

/// Content Security Policy configuration
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    /// default-src directive
    pub default_src: Vec<String>,
    /// script-src directive
    pub script_src: Vec<String>,
    /// style-src directive
    pub style_src: Vec<String>,
    /// img-src directive
    pub img_src: Vec<String>,
    /// font-src directive
    pub font_src: Vec<String>,
    /// connect-src directive
    pub connect_src: Vec<String>,
    /// media-src directive
    pub media_src: Vec<String>,
    /// object-src directive
    pub object_src: Vec<String>,
    /// frame-src directive
    pub frame_src: Vec<String>,
    /// worker-src directive
    pub worker_src: Vec<String>,
    /// manifest-src directive
    pub manifest_src: Vec<String>,
    /// base-uri directive
    pub base_uri: Vec<String>,
    /// form-action directive
    pub form_action: Vec<String>,
    /// upgrade-insecure-requests
    pub upgrade_insecure_requests: bool,
    /// block-all-mixed-content
    pub block_all_mixed_content: bool,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
            font_src: vec!["'self'".to_string(), "https:".to_string()],
            connect_src: vec!["'self'".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'none'".to_string()],
            worker_src: vec!["'self'".to_string()],
            manifest_src: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
        }
    }
}

impl ContentSecurityPolicy {
    /// Create a strict CSP for production
    pub fn strict() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()], // No unsafe-inline
            style_src: vec!["'self'".to_string()], // No unsafe-inline
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string(), "https://api.anthropic.com".to_string(), "https://api.openai.com".to_string()],
            media_src: vec!["'none'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'none'".to_string()],
            worker_src: vec!["'self'".to_string()],
            manifest_src: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
        }
    }

    /// Create a development CSP with relaxed rules
    pub fn development() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string(), "'unsafe-eval'".to_string(), "http://localhost:*".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string(), "http://localhost:*".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "http:".to_string(), "https:".to_string()],
            font_src: vec!["'self'".to_string(), "data:".to_string(), "http:".to_string(), "https:".to_string()],
            connect_src: vec!["'self'".to_string(), "http://localhost:*".to_string(), "ws://localhost:*".to_string(), "https:".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'self'".to_string()],
            worker_src: vec!["'self'".to_string(), "blob:".to_string()],
            manifest_src: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: false,
            block_all_mixed_content: false,
        }
    }

    /// Convert to CSP header value
    pub fn to_header_value(&self) -> String {
        let mut directives = Vec::new();

        directives.push(format!("default-src {}", self.default_src.join(" ")));
        directives.push(format!("script-src {}", self.script_src.join(" ")));
        directives.push(format!("style-src {}", self.style_src.join(" ")));
        directives.push(format!("img-src {}", self.img_src.join(" ")));
        directives.push(format!("font-src {}", self.font_src.join(" ")));
        directives.push(format!("connect-src {}", self.connect_src.join(" ")));
        directives.push(format!("media-src {}", self.media_src.join(" ")));
        directives.push(format!("object-src {}", self.object_src.join(" ")));
        directives.push(format!("frame-src {}", self.frame_src.join(" ")));
        directives.push(format!("worker-src {}", self.worker_src.join(" ")));
        directives.push(format!("manifest-src {}", self.manifest_src.join(" ")));
        directives.push(format!("base-uri {}", self.base_uri.join(" ")));
        directives.push(format!("form-action {}", self.form_action.join(" ")));

        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }

        if self.block_all_mixed_content {
            directives.push("block-all-mixed-content".to_string());
        }

        directives.join("; ")
    }
}

/// CORS policy configuration
#[derive(Debug, Clone)]
pub struct CorsPolicy {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    /// Max age for preflight requests
    pub max_age: u32,
    /// Allow credentials
    pub allow_credentials: bool,
}

impl Default for CorsPolicy {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string(), "X-Requested-With".to_string()],
            exposed_headers: vec![],
            max_age: 86400,
            allow_credentials: false,
        }
    }
}

impl CorsPolicy {
    /// Create a restrictive CORS policy
    pub fn restrictive() -> Self {
        Self {
            allowed_origins: vec!["https://writemagic.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            exposed_headers: vec![],
            max_age: 3600,
            allow_credentials: true,
        }
    }

    /// Create a production CORS policy
    pub fn production() -> Self {
        Self {
            allowed_origins: vec![
                "https://writemagic.com".to_string(),
                "https://www.writemagic.com".to_string(),
                "https://app.writemagic.com".to_string()
            ],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string(), "X-Requested-With".to_string()],
            exposed_headers: vec!["X-Total-Count".to_string()],
            max_age: 86400,
            allow_credentials: true,
        }
    }

    /// Create a development CORS policy
    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:8080".to_string()
            ],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec!["*".to_string()],
            max_age: 300,
            allow_credentials: true,
        }
    }

    /// Convert to CORS headers
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if !self.allowed_origins.is_empty() {
            headers.insert("Access-Control-Allow-Origin".to_string(), self.allowed_origins.join(","));
        }

        headers.insert("Access-Control-Allow-Methods".to_string(), self.allowed_methods.join(","));
        headers.insert("Access-Control-Allow-Headers".to_string(), self.allowed_headers.join(","));
        
        if !self.exposed_headers.is_empty() {
            headers.insert("Access-Control-Expose-Headers".to_string(), self.exposed_headers.join(","));
        }

        headers.insert("Access-Control-Max-Age".to_string(), self.max_age.to_string());

        if self.allow_credentials {
            headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_strict() {
        let csp = ContentSecurityPolicy::strict();
        let header_value = csp.to_header_value();
        
        assert!(header_value.contains("default-src 'self'"));
        assert!(!header_value.contains("unsafe-inline"));
        assert!(header_value.contains("object-src 'none'"));
    }

    #[test]
    fn test_csp_development() {
        let csp = ContentSecurityPolicy::development();
        let header_value = csp.to_header_value();
        
        assert!(header_value.contains("unsafe-inline"));
        assert!(header_value.contains("unsafe-eval"));
        assert!(header_value.contains("localhost"));
    }

    #[test]
    fn test_cors_production() {
        let cors = CorsPolicy::production();
        let headers = cors.to_headers();
        
        assert!(headers.get("Access-Control-Allow-Origin").unwrap().contains("writemagic.com"));
        assert_eq!(headers.get("Access-Control-Allow-Credentials").unwrap(), "true");
    }
}