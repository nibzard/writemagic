use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Security scanner for vulnerability detection
pub struct SecurityScanner {
    enabled: bool,
}

impl SecurityScanner {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Scan dependencies for vulnerabilities
    pub fn scan_dependencies(&self) -> Result<DependencyScanResult, ScanError> {
        if !self.enabled {
            return Ok(DependencyScanResult::disabled());
        }

        // Run cargo audit
        let cargo_audit = self.run_cargo_audit()?;
        
        // Run cargo deny (license and security checks)
        let cargo_deny = self.run_cargo_deny()?;

        Ok(DependencyScanResult {
            vulnerabilities: cargo_audit.vulnerabilities,
            license_violations: cargo_deny.license_violations,
            security_advisories: cargo_audit.security_advisories,
            total_dependencies: cargo_audit.total_dependencies,
            scan_timestamp: chrono::Utc::now(),
            status: if cargo_audit.vulnerabilities.is_empty() && cargo_deny.license_violations.is_empty() {
                ScanStatus::Clean
            } else if cargo_audit.vulnerabilities.iter().any(|v| v.severity == Severity::Critical) {
                ScanStatus::Critical
            } else {
                ScanStatus::Warning
            }
        })
    }

    /// Scan container images for vulnerabilities
    pub fn scan_container(&self, image: &str) -> Result<ContainerScanResult, ScanError> {
        if !self.enabled {
            return Ok(ContainerScanResult::disabled());
        }

        // Use Trivy for container scanning
        let output = Command::new("trivy")
            .arg("image")
            .arg("--format")
            .arg("json")
            .arg("--severity")
            .arg("HIGH,CRITICAL")
            .arg(image)
            .output()
            .map_err(|e| ScanError::ScannerNotFound(format!("Trivy not found: {}", e)))?;

        if !output.status.success() {
            return Err(ScanError::ScanFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        let trivy_result: TrivyResult = serde_json::from_slice(&output.stdout)
            .map_err(|e| ScanError::ParseError(format!("Failed to parse Trivy output: {}", e)))?;

        Ok(ContainerScanResult {
            image: image.to_string(),
            vulnerabilities: trivy_result.results.into_iter()
                .flat_map(|r| r.vulnerabilities)
                .map(|v| Vulnerability {
                    id: v.vulnerability_id,
                    title: v.title,
                    description: v.description,
                    severity: match v.severity.as_str() {
                        "CRITICAL" => Severity::Critical,
                        "HIGH" => Severity::High,
                        "MEDIUM" => Severity::Medium,
                        "LOW" => Severity::Low,
                        _ => Severity::Unknown,
                    },
                    package: v.pkg_name,
                    fixed_version: v.fixed_version,
                })
                .collect(),
            scan_timestamp: chrono::Utc::now(),
            status: ScanStatus::Clean, // Will be updated based on vulnerabilities
        })
    }

    /// Run static analysis on source code
    pub fn scan_source_code(&self) -> Result<SourceScanResult, ScanError> {
        if !self.enabled {
            return Ok(SourceScanResult::disabled());
        }

        // Run clippy for Rust code analysis
        let clippy_output = Command::new("cargo")
            .arg("clippy")
            .arg("--message-format")
            .arg("json")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .output()
            .map_err(|e| ScanError::ScannerNotFound(format!("cargo clippy failed: {}", e)))?;

        let clippy_messages = String::from_utf8_lossy(&clippy_output.stdout);
        let issues = self.parse_clippy_output(&clippy_messages)?;

        Ok(SourceScanResult {
            issues,
            scan_timestamp: chrono::Utc::now(),
            status: ScanStatus::Clean, // Will be updated based on issues
        })
    }

    fn run_cargo_audit(&self) -> Result<CargoAuditResult, ScanError> {
        let output = Command::new("cargo")
            .arg("audit")
            .arg("--format")
            .arg("json")
            .output()
            .map_err(|e| ScanError::ScannerNotFound(format!("cargo audit failed: {}", e)))?;

        if !output.status.success() {
            return Err(ScanError::ScanFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        // Parse cargo audit JSON output
        let audit_data: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| ScanError::ParseError(format!("Failed to parse cargo audit output: {}", e)))?;

        Ok(CargoAuditResult {
            vulnerabilities: Vec::new(), // TODO: Parse actual vulnerabilities
            security_advisories: Vec::new(),
            total_dependencies: 0,
        })
    }

    fn run_cargo_deny(&self) -> Result<CargoDenyResult, ScanError> {
        let output = Command::new("cargo")
            .arg("deny")
            .arg("check")
            .output()
            .map_err(|e| ScanError::ScannerNotFound(format!("cargo deny failed: {}", e)))?;

        // Parse output for license violations
        Ok(CargoDenyResult {
            license_violations: Vec::new(), // TODO: Parse actual violations
        })
    }

    fn parse_clippy_output(&self, output: &str) -> Result<Vec<CodeIssue>, ScanError> {
        let mut issues = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON
            if let Ok(message) = serde_json::from_str::<ClippyMessage>(line) {
                if message.reason == "compiler-message" && message.message.level == "error" {
                    issues.push(CodeIssue {
                        file: message.message.spans.first()
                            .map(|s| s.file_name.clone())
                            .unwrap_or_default(),
                        line: message.message.spans.first()
                            .map(|s| s.line_start)
                            .unwrap_or(0),
                        column: message.message.spans.first()
                            .map(|s| s.column_start)
                            .unwrap_or(0),
                        severity: Severity::High,
                        message: message.message.message,
                        rule: message.message.code
                            .map(|c| c.code)
                            .unwrap_or_default(),
                    });
                }
            }
        }

        Ok(issues)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub license_violations: Vec<LicenseViolation>,
    pub security_advisories: Vec<SecurityAdvisory>,
    pub total_dependencies: usize,
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
    pub status: ScanStatus,
}

impl DependencyScanResult {
    pub fn disabled() -> Self {
        Self {
            vulnerabilities: Vec::new(),
            license_violations: Vec::new(),
            security_advisories: Vec::new(),
            total_dependencies: 0,
            scan_timestamp: chrono::Utc::now(),
            status: ScanStatus::Disabled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerScanResult {
    pub image: String,
    pub vulnerabilities: Vec<Vulnerability>,
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
    pub status: ScanStatus,
}

impl ContainerScanResult {
    pub fn disabled() -> Self {
        Self {
            image: String::new(),
            vulnerabilities: Vec::new(),
            scan_timestamp: chrono::Utc::now(),
            status: ScanStatus::Disabled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceScanResult {
    pub issues: Vec<CodeIssue>,
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
    pub status: ScanStatus,
}

impl SourceScanResult {
    pub fn disabled() -> Self {
        Self {
            issues: Vec::new(),
            scan_timestamp: chrono::Utc::now(),
            status: ScanStatus::Disabled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub package: String,
    pub fixed_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub package: String,
    pub license: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub message: String,
    pub rule: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ScanStatus {
    Clean,
    Warning,
    Critical,
    Disabled,
}

// Internal structs for parsing tool outputs
#[derive(Debug, Deserialize)]
struct CargoAuditResult {
    vulnerabilities: Vec<Vulnerability>,
    security_advisories: Vec<SecurityAdvisory>,
    total_dependencies: usize,
}

#[derive(Debug, Deserialize)]
struct CargoDenyResult {
    license_violations: Vec<LicenseViolation>,
}

#[derive(Debug, Deserialize)]
struct TrivyResult {
    results: Vec<TrivyResultEntry>,
}

#[derive(Debug, Deserialize)]
struct TrivyResultEntry {
    #[serde(rename = "Vulnerabilities")]
    vulnerabilities: Vec<TrivyVulnerability>,
}

#[derive(Debug, Deserialize)]
struct TrivyVulnerability {
    #[serde(rename = "VulnerabilityID")]
    vulnerability_id: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Severity")]
    severity: String,
    #[serde(rename = "PkgName")]
    pkg_name: String,
    #[serde(rename = "FixedVersion")]
    fixed_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClippyMessage {
    reason: String,
    message: ClippyMessageContent,
}

#[derive(Debug, Deserialize)]
struct ClippyMessageContent {
    level: String,
    message: String,
    code: Option<ClippyCode>,
    spans: Vec<ClippySpan>,
}

#[derive(Debug, Deserialize)]
struct ClippyCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct ClippySpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
}

#[derive(Debug)]
pub enum ScanError {
    ScannerNotFound(String),
    ScanFailed(String),
    ParseError(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::ScannerNotFound(msg) => write!(f, "Scanner not found: {}", msg),
            ScanError::ScanFailed(msg) => write!(f, "Scan failed: {}", msg),
            ScanError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ScanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_scanner_creation() {
        let scanner = SecurityScanner::new(true);
        assert!(scanner.enabled);
    }

    #[test]
    fn test_disabled_scanner() {
        let scanner = SecurityScanner::new(false);
        let result = scanner.scan_dependencies().unwrap();
        assert!(matches!(result.status, ScanStatus::Disabled));
    }
}