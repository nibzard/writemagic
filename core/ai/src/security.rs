//! AI security hardening with PII detection, content sanitization, and secure key management

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use regex::Regex;
use writemagic_shared::{Result, WritemagicError};
use crate::providers::{CompletionRequest, CompletionResponse};

/// Secure API key storage with automatic rotation support
#[derive(Debug, Clone)]
pub struct SecureApiKey {
    id: String,
    key: String,
    created_at: std::time::SystemTime,
    rotation_required: bool,
    usage_count: u64,
    max_usage: Option<u64>,
}

impl SecureApiKey {
    /// Create new secure API key
    pub fn new(id: String, key: String) -> Self {
        Self {
            id,
            key,
            created_at: std::time::SystemTime::now(),
            rotation_required: false,
            usage_count: 0,
            max_usage: None,
        }
    }

    /// Create key with usage limit
    pub fn with_usage_limit(id: String, key: String, max_usage: u64) -> Self {
        Self {
            id,
            key,
            created_at: std::time::SystemTime::now(),
            rotation_required: false,
            usage_count: 0,
            max_usage: Some(max_usage),
        }
    }

    /// Get the API key value
    pub fn value(&self) -> &str {
        &self.key
    }

    /// Get key ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if key needs rotation
    pub fn needs_rotation(&self) -> bool {
        if self.rotation_required {
            return true;
        }
        
        // Check usage limit
        if let Some(max_usage) = self.max_usage {
            if self.usage_count >= max_usage {
                return true;
            }
        }
        
        // Check age (rotate after 30 days)
        let age = self.created_at.elapsed().unwrap_or(std::time::Duration::ZERO);
        age > std::time::Duration::from_secs(30 * 24 * 60 * 60)
    }

    /// Mark key for rotation
    pub fn mark_for_rotation(&mut self) {
        self.rotation_required = true;
    }

    /// Record usage
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Validate key format (basic validation)
    pub fn validate(&self) -> bool {
        !self.key.is_empty() && self.key.len() >= 20
    }
}

/// Type alias for rotation callback to reduce complexity
type RotationCallback = Box<dyn Fn(&str) -> Result<SecureApiKey> + Send + Sync>;

/// Secure API key manager with rotation capabilities
pub struct SecureKeyManager {
    keys: Arc<RwLock<HashMap<String, SecureApiKey>>>,
    #[allow(dead_code)] // TODO: Implement key rotation callbacks in Phase 2
    rotation_callbacks: Arc<RwLock<Vec<RotationCallback>>>,
}

impl std::fmt::Debug for SecureKeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureKeyManager")
            .field("keys", &"[REDACTED]")
            .field("rotation_callbacks", &"[REDACTED]")
            .finish()
    }
}

impl SecureKeyManager {
    /// Create new key manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add or update API key
    pub fn add_key(&self, provider: String, key: SecureApiKey) -> Result<()> {
        if !key.validate() {
            return Err(WritemagicError::security("Invalid API key format"));
        }
        
        self.keys.write().insert(provider, key);
        Ok(())
    }

    /// Get API key for provider
    pub fn get_key(&self, provider: &str) -> Result<SecureApiKey> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(provider)
            .ok_or_else(|| WritemagicError::authentication("API key not found"))?;
        
        if key.needs_rotation() {
            return Err(WritemagicError::authentication("API key requires rotation"));
        }
        
        key.record_usage();
        Ok(key.clone())
    }

    /// Check if any keys need rotation
    pub fn check_rotation_needed(&self) -> Vec<String> {
        let keys = self.keys.read();
        keys.iter()
            .filter_map(|(provider, key)| {
                if key.needs_rotation() {
                    Some(provider.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Force rotation check for all keys
    pub fn force_rotation_check(&self) -> HashMap<String, bool> {
        let keys = self.keys.read();
        keys.iter()
            .map(|(provider, key)| (provider.clone(), key.needs_rotation()))
            .collect()
    }
}

impl Default for SecureKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// PII detection patterns with confidence scoring
#[derive(Debug, Clone)]
pub struct PIIPattern {
    pub name: String,
    pub regex: Regex,
    pub confidence: f32,
    pub severity: PIISeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PIISeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl PIIPattern {
    /// Create new PII pattern
    pub fn new(name: String, pattern: &str, confidence: f32, severity: PIISeverity) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| WritemagicError::internal(format!("Invalid PII regex: {}", e)))?;
        
        Ok(Self {
            name,
            regex,
            confidence,
            severity,
        })
    }

    /// Check if text matches this pattern
    pub fn matches(&self, text: &str) -> Vec<PIIMatch> {
        self.regex
            .find_iter(text)
            .map(|m| PIIMatch {
                pattern_name: self.name.clone(),
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                confidence: self.confidence,
                severity: self.severity.clone(),
            })
            .collect()
    }
}

/// PII match result
#[derive(Debug, Clone)]
pub struct PIIMatch {
    pub pattern_name: String,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
    pub severity: PIISeverity,
}

/// Advanced PII detection service
#[derive(Debug)]
pub struct PIIDetectionService {
    patterns: Vec<PIIPattern>,
    custom_patterns: Arc<RwLock<Vec<PIIPattern>>>,
}

impl PIIDetectionService {
    /// Create new PII detection service with default patterns
    pub fn new() -> Result<Self> {
        let patterns = Self::create_default_patterns()?;
        
        Ok(Self {
            patterns,
            custom_patterns: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create comprehensive default PII patterns
    #[allow(clippy::vec_init_then_push)] // Complex initialization with error handling
    fn create_default_patterns() -> Result<Vec<PIIPattern>> {
        let mut patterns = Vec::new();
        
        // API Keys and Secrets
        patterns.push(PIIPattern::new(
            "api_key".to_string(),
            r#"(?i)(api[_-]?key|secret|token)\s*[:=]\s*['"]?([a-zA-Z0-9_-]{20,})['"]?"#,
            0.95,
            PIISeverity::Critical,
        )?);
        
        // Authentication tokens
        patterns.push(PIIPattern::new(
            "bearer_token".to_string(),
            r"(?i)bearer\s+([a-zA-Z0-9_-]{20,})",
            0.95,
            PIISeverity::Critical,
        )?);
        
        // Database URLs
        patterns.push(PIIPattern::new(
            "database_url".to_string(),
            r"(?i)(postgres|mysql|mongodb)://[^\s\)]+",
            0.90,
            PIISeverity::High,
        )?);
        
        // AWS Access Keys
        patterns.push(PIIPattern::new(
            "aws_access_key".to_string(),
            r"(?i)(AKIA[0-9A-Z]{16})",
            0.98,
            PIISeverity::Critical,
        )?);
        
        // SSH Private Keys
        patterns.push(PIIPattern::new(
            "ssh_private_key".to_string(),
            r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
            0.99,
            PIISeverity::Critical,
        )?);
        
        // Email addresses
        patterns.push(PIIPattern::new(
            "email".to_string(),
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
            0.85,
            PIISeverity::Medium,
        )?);
        
        // Phone numbers (US format)
        patterns.push(PIIPattern::new(
            "us_phone".to_string(),
            r"(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}",
            0.80,
            PIISeverity::Medium,
        )?);
        
        // SSN (Social Security Numbers)
        patterns.push(PIIPattern::new(
            "ssn".to_string(),
            r"\b\d{3}-?\d{2}-?\d{4}\b",
            0.85,
            PIISeverity::High,
        )?);
        
        // Credit Card Numbers (basic pattern)
        patterns.push(PIIPattern::new(
            "credit_card".to_string(),
            r"\b(?:\d{4}[-\s]?){3}\d{4}\b",
            0.75,
            PIISeverity::High,
        )?);
        
        // IP Addresses
        patterns.push(PIIPattern::new(
            "ip_address".to_string(),
            r"\b(?:\d{1,3}\.){3}\d{1,3}\b",
            0.70,
            PIISeverity::Low,
        )?);
        
        // URLs with potentially sensitive paths
        patterns.push(PIIPattern::new(
            "sensitive_url".to_string(),
            r"https?://[^\s]*(?:admin|login|auth|token|key|secret)[^\s]*",
            0.80,
            PIISeverity::Medium,
        )?);
        
        Ok(patterns)
    }

    /// Add custom PII pattern
    pub fn add_custom_pattern(&self, pattern: PIIPattern) {
        self.custom_patterns.write().push(pattern);
    }

    /// Scan text for PII
    pub fn scan_text(&self, text: &str) -> Vec<PIIMatch> {
        let mut matches = Vec::new();
        
        // Check default patterns
        for pattern in &self.patterns {
            matches.extend(pattern.matches(text));
        }
        
        // Check custom patterns
        let custom_patterns = self.custom_patterns.read();
        for pattern in custom_patterns.iter() {
            matches.extend(pattern.matches(text));
        }
        
        // Sort by position
        matches.sort_by_key(|m| m.start);
        matches
    }

    /// Check if text contains high-severity PII
    pub fn contains_critical_pii(&self, text: &str) -> bool {
        self.scan_text(text)
            .iter()
            .any(|m| matches!(m.severity, PIISeverity::Critical))
    }

    /// Sanitize text by redacting PII
    pub fn sanitize_text(&self, text: &str) -> String {
        let matches = self.scan_text(text);
        if matches.is_empty() {
            return text.to_string();
        }
        
        let mut result = text.to_string();
        
        // Process matches in reverse order to maintain indices
        for m in matches.iter().rev() {
            let replacement = match m.severity {
                PIISeverity::Critical => "[REDACTED_CRITICAL]",
                PIISeverity::High => "[REDACTED_HIGH]",
                PIISeverity::Medium => "[REDACTED_MEDIUM]",
                PIISeverity::Low => "[REDACTED_LOW]",
            };
            
            result.replace_range(m.start..m.end, replacement);
        }
        
        result
    }
}

impl Default for PIIDetectionService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            log::error!("Failed to create PII detection service, using minimal implementation");
            Self {
                patterns: Vec::new(),
                custom_patterns: Arc::new(RwLock::new(Vec::new())),
            }
        })
    }
}

/// Content sanitization service
#[derive(Debug)]
pub struct ContentSanitizationService {
    pii_detector: PIIDetectionService,
    #[allow(dead_code)] // TODO: Implement key-based encryption/redaction in Phase 2
    key_manager: Arc<SecureKeyManager>,
}

impl ContentSanitizationService {
    /// Create new content sanitization service
    pub fn new(key_manager: Arc<SecureKeyManager>) -> Result<Self> {
        Ok(Self {
            pii_detector: PIIDetectionService::new()?,
            key_manager,
        })
    }

    /// Sanitize request before sending to AI provider
    pub fn sanitize_request(&self, request: &CompletionRequest) -> Result<CompletionRequest> {
        let mut sanitized = request.clone();
        
        // Sanitize messages
        for message in &mut sanitized.messages {
            let pii_matches = self.pii_detector.scan_text(&message.content);
            
            if !pii_matches.is_empty() {
                // Log security event (without PII)
                tracing::warn!(
                    "PII detected in AI request: {} matches of severity {:?}",
                    pii_matches.len(),
                    pii_matches.iter().map(|m| &m.severity).collect::<Vec<_>>()
                );
                
                // Check for critical PII
                let has_critical_pii = pii_matches.iter().any(|m| matches!(m.severity, PIISeverity::Critical));
                if has_critical_pii {
                    return Err(WritemagicError::security("Request contains critical PII and cannot be processed"));
                }
                
                // Sanitize content
                message.content = self.pii_detector.sanitize_text(&message.content);
            }
        }
        
        // Remove metadata that might contain sensitive info
        sanitized.metadata.retain(|k, _| {
            !k.to_lowercase().contains("key") && 
            !k.to_lowercase().contains("secret") &&
            !k.to_lowercase().contains("token")
        });
        
        Ok(sanitized)
    }

    /// Sanitize response from AI provider
    pub fn sanitize_response(&self, response: &CompletionResponse) -> Result<CompletionResponse> {
        let mut sanitized = response.clone();
        
        // Sanitize response choices
        for choice in &mut sanitized.choices {
            let pii_matches = self.pii_detector.scan_text(&choice.message.content);
            
            if !pii_matches.is_empty() {
                tracing::warn!(
                    "PII detected in AI response: {} matches",
                    pii_matches.len()
                );
                
                choice.message.content = self.pii_detector.sanitize_text(&choice.message.content);
            }
        }
        
        // Clean metadata
        sanitized.metadata.clear(); // Remove all response metadata to be safe
        
        Ok(sanitized)
    }

    /// Sanitize text for logging
    pub fn sanitize_for_logging(&self, text: &str) -> String {
        // More aggressive sanitization for logs
        let sanitized = self.pii_detector.sanitize_text(text);
        
        // Additionally remove any remaining API key-like patterns
        match Regex::new(r"[a-zA-Z0-9_-]{20,}") {
            Ok(api_key_pattern) => api_key_pattern.replace_all(&sanitized, "[REDACTED]").to_string(),
            Err(_) => {
                log::warn!("Failed to compile API key pattern for logging sanitization");
                sanitized // Return partially sanitized text
            }
        }
    }

    /// Check if content contains sensitive information
    pub fn contains_sensitive_content(&self, content: &str) -> bool {
        self.pii_detector
            .scan_text(content)
            .iter()
            .any(|m| matches!(m.severity, PIISeverity::Medium | PIISeverity::High))
    }
}

/// Security audit logger
#[derive(Debug)]
pub struct SecurityAuditLogger {
    events: Arc<RwLock<Vec<SecurityEvent>>>,
    max_events: usize,
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: SecurityEventType,
    pub details: String,
    pub severity: PIISeverity,
}

#[derive(Debug, Clone)]
pub enum SecurityEventType {
    PIIDetected,
    KeyRotationNeeded,
    KeyRotated,
    SecurityViolation,
    SuspiciousActivity,
}

impl SecurityAuditLogger {
    /// Create new security audit logger
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    /// Log security event
    pub fn log_event(&self, event_type: SecurityEventType, details: String, severity: PIISeverity) {
        // Log to tracing first (before moving values)
        match severity {
            PIISeverity::Critical => tracing::error!("Security event: {}", details),
            PIISeverity::High => tracing::warn!("Security event: {}", details),
            PIISeverity::Medium => tracing::info!("Security event: {}", details),
            PIISeverity::Low => tracing::debug!("Security event: {}", details),
        }
        
        let event = SecurityEvent {
            timestamp: std::time::SystemTime::now(),
            event_type,
            details,
            severity,
        };
        
        let mut events = self.events.write();
        events.push(event);
        
        // Trim to max events
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }
    }

    /// Get recent security events
    pub fn get_recent_events(&self, limit: usize) -> Vec<SecurityEvent> {
        let events = self.events.read();
        let start = if events.len() > limit { events.len() - limit } else { 0 };
        events[start..].to_vec()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: PIISeverity) -> Vec<SecurityEvent> {
        let events = self.events.read();
        events.iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }
}

impl Default for SecurityAuditLogger {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_api_key() {
        let mut key = SecureApiKey::new(
            "test".to_string(),
            "sk-1234567890abcdefghijklmnop".to_string()
        );
        
        assert!(key.validate());
        assert!(!key.needs_rotation());
        
        key.record_usage();
        assert_eq!(key.usage_count, 1);
        
        key.mark_for_rotation();
        assert!(key.needs_rotation());
    }

    #[test]
    fn test_pii_detection() -> Result<()> {
        let detector = PIIDetectionService::new()?;
        
        let text = "My API key is sk-1234567890abcdefghij and my email is test@example.com";
        let matches = detector.scan_text(text);
        
        assert!(matches.len() >= 2); // Should find API key and email
        
        let has_critical = matches.iter().any(|m| matches!(m.severity, PIISeverity::Critical));
        assert!(has_critical);
        
        Ok(())
    }

    #[test]
    fn test_content_sanitization() -> Result<()> {
        let key_manager = Arc::new(SecureKeyManager::new());
        let sanitizer = ContentSanitizationService::new(key_manager)?;
        
        let text = "Contact me at test@example.com with API key sk-1234567890abcdef";
        let sanitized = sanitizer.pii_detector.sanitize_text(text);
        
        assert!(!sanitized.contains("test@example.com"));
        assert!(!sanitized.contains("sk-1234567890abcdef"));
        assert!(sanitized.contains("[REDACTED"));
        
        Ok(())
    }

    #[test]
    fn test_security_audit_logger() {
        let logger = SecurityAuditLogger::new(100);
        
        logger.log_event(
            SecurityEventType::PIIDetected,
            "API key detected in request".to_string(),
            PIISeverity::Critical,
        );
        
        let events = logger.get_recent_events(10);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, SecurityEventType::PIIDetected));
    }
}