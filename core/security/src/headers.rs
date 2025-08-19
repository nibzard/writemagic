use std::collections::HashMap;

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    /// X-Content-Type-Options
    pub content_type_options: String,
    /// X-Frame-Options
    pub frame_options: String,
    /// X-XSS-Protection
    pub xss_protection: String,
    /// Referrer-Policy
    pub referrer_policy: String,
    /// Strict-Transport-Security
    pub hsts: Option<String>,
    /// Permissions-Policy
    pub permissions_policy: String,
    /// X-Permitted-Cross-Domain-Policies
    pub cross_domain_policies: String,
    /// Custom security headers
    pub custom_headers: HashMap<String, String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            content_type_options: "nosniff".to_string(),
            frame_options: "DENY".to_string(),
            xss_protection: "1; mode=block".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            hsts: Some("max-age=31536000; includeSubDomains; preload".to_string()),
            permissions_policy: "geolocation=(), microphone=(), camera=(), payment=(), usb=()".to_string(),
            cross_domain_policies: "none".to_string(),
            custom_headers: HashMap::new(),
        }
    }
}

impl SecurityHeaders {
    /// Create strict security headers for production
    pub fn strict() -> Self {
        let mut headers = Self::default();
        headers.hsts = Some("max-age=63072000; includeSubDomains; preload".to_string());
        headers.frame_options = "DENY".to_string();
        headers.permissions_policy = "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=()".to_string();
        headers
    }

    /// Create relaxed security headers for development
    pub fn development() -> Self {
        let mut headers = Self::default();
        headers.hsts = None; // No HSTS for development
        headers.frame_options = "SAMEORIGIN".to_string();
        headers
    }

    /// Convert to HTTP headers map
    pub fn to_http_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        headers.insert("X-Content-Type-Options".to_string(), self.content_type_options.clone());
        headers.insert("X-Frame-Options".to_string(), self.frame_options.clone());
        headers.insert("X-XSS-Protection".to_string(), self.xss_protection.clone());
        headers.insert("Referrer-Policy".to_string(), self.referrer_policy.clone());
        headers.insert("Permissions-Policy".to_string(), self.permissions_policy.clone());
        headers.insert("X-Permitted-Cross-Domain-Policies".to_string(), self.cross_domain_policies.clone());
        
        if let Some(hsts) = &self.hsts {
            headers.insert("Strict-Transport-Security".to_string(), hsts.clone());
        }
        
        // Add custom headers
        for (key, value) in &self.custom_headers {
            headers.insert(key.clone(), value.clone());
        }
        
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_security_headers() {
        let headers = SecurityHeaders::default();
        assert_eq!(headers.content_type_options, "nosniff");
        assert_eq!(headers.frame_options, "DENY");
        assert!(headers.hsts.is_some());
    }

    #[test]
    fn test_strict_security_headers() {
        let headers = SecurityHeaders::strict();
        assert!(headers.hsts.as_ref().unwrap().contains("max-age=63072000"));
        assert_eq!(headers.frame_options, "DENY");
    }

    #[test]
    fn test_development_security_headers() {
        let headers = SecurityHeaders::development();
        assert!(headers.hsts.is_none());
        assert_eq!(headers.frame_options, "SAMEORIGIN");
    }

    #[test]
    fn test_to_http_headers() {
        let security_headers = SecurityHeaders::default();
        let http_headers = security_headers.to_http_headers();
        
        assert!(http_headers.contains_key("X-Content-Type-Options"));
        assert!(http_headers.contains_key("X-Frame-Options"));
        assert!(http_headers.contains_key("Strict-Transport-Security"));
    }
}