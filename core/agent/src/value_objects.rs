//! Agent domain value objects

use writemagic_shared::{WritemagicError, Result};
// Remove unused chrono imports
use std::time::Duration;
use serde::{Deserialize, Serialize};
// Remove unused BTreeMap import
use std::fmt;

/// Agent execution priority level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExecutionPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl fmt::Display for ExecutionPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionPriority::Low => write!(f, "Low"),
            ExecutionPriority::Normal => write!(f, "Normal"),
            ExecutionPriority::High => write!(f, "High"),
            ExecutionPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Agent workflow execution mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Sequential,  // Jobs run one after another
    Parallel,    // Jobs run simultaneously where possible
    Pipeline,    // Jobs pass outputs to next job
    Matrix,      // Jobs run with different variable combinations
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Sequential => write!(f, "Sequential"),
            ExecutionMode::Parallel => write!(f, "Parallel"),
            ExecutionMode::Pipeline => write!(f, "Pipeline"),
            ExecutionMode::Matrix => write!(f, "Matrix"),
        }
    }
}

/// Agent version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentVersion {
    major: u32,
    minor: u32,
    patch: u32,
    pre_release: Option<String>,
}

impl AgentVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }
    
    /// Create a pre-release version
    pub fn new_pre_release(major: u32, minor: u32, patch: u32, pre_release: String) -> Result<Self> {
        if pre_release.trim().is_empty() {
            return Err(WritemagicError::validation("Pre-release tag cannot be empty"));
        }
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        })
    }
    
    /// Parse version from string (e.g., "1.2.3" or "1.2.3-beta.1")
    pub fn from_string(version: &str) -> Result<Self> {
        let parts: Vec<&str> = version.split('-').collect();
        let version_part = parts[0];
        let pre_release = if parts.len() > 1 {
            Some(parts[1..].join("-"))
        } else {
            None
        };
        
        let version_numbers: Vec<&str> = version_part.split('.').collect();
        if version_numbers.len() != 3 {
            return Err(WritemagicError::validation("Version must be in format major.minor.patch"));
        }
        
        let major = version_numbers[0].parse()
            .map_err(|_| WritemagicError::validation("Invalid major version number"))?;
        let minor = version_numbers[1].parse()
            .map_err(|_| WritemagicError::validation("Invalid minor version number"))?;
        let patch = version_numbers[2].parse()
            .map_err(|_| WritemagicError::validation("Invalid patch version number"))?;
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
    
    /// Get version components
    pub fn components(&self) -> (u32, u32, u32) {
        (self.major, self.minor, self.patch)
    }
    
    /// Check if this is a pre-release version
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
    
    /// Compare versions (semantic versioning)
    pub fn is_compatible_with(&self, other: &AgentVersion) -> bool {
        // Major versions must match for compatibility
        self.major == other.major
    }
    
    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &AgentVersion) -> bool {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                match self.minor.cmp(&other.minor) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => {
                        match self.patch.cmp(&other.patch) {
                            std::cmp::Ordering::Greater => true,
                            std::cmp::Ordering::Less => false,
                            std::cmp::Ordering::Equal => {
                                // If both have no pre-release, they're equal
                                // If one has pre-release and other doesn't, the one without is newer
                                // If both have pre-release, compare lexicographically
                                match (&self.pre_release, &other.pre_release) {
                                    (None, Some(_)) => true,
                                    (Some(_), None) => false,
                                    (None, None) => false,
                                    (Some(a), Some(b)) => a > b,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl fmt::Display for AgentVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pre) = &self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Agent execution timeout configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionTimeout {
    duration: Duration,
    action: TimeoutAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutAction {
    Cancel,
    Kill,
    Retry,
    Continue,
}

impl ExecutionTimeout {
    /// Create a new timeout configuration
    pub fn new(duration: Duration, action: TimeoutAction) -> Result<Self> {
        if duration.is_zero() {
            return Err(WritemagicError::validation("Timeout duration cannot be zero"));
        }
        
        Ok(Self { duration, action })
    }
    
    /// Get timeout duration
    pub fn duration(&self) -> Duration {
        self.duration
    }
    
    /// Get timeout action
    pub fn action(&self) -> &TimeoutAction {
        &self.action
    }
    
    /// Create a cancel timeout
    pub fn cancel_after(duration: Duration) -> Result<Self> {
        Self::new(duration, TimeoutAction::Cancel)
    }
    
    /// Create a kill timeout
    pub fn kill_after(duration: Duration) -> Result<Self> {
        Self::new(duration, TimeoutAction::Kill)
    }
}

/// Agent execution schedule using cron-like syntax
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionSchedule {
    expression: String,
    timezone: String,
    enabled: bool,
}

impl ExecutionSchedule {
    /// Create a new execution schedule
    pub fn new(expression: String, timezone: Option<String>) -> Result<Self> {
        if expression.trim().is_empty() {
            return Err(WritemagicError::validation("Schedule expression cannot be empty"));
        }
        
        // Basic validation of cron expression format
        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 5 && parts.len() != 6 {
            return Err(WritemagicError::validation("Invalid cron expression format"));
        }
        
        Ok(Self {
            expression,
            timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
            enabled: true,
        })
    }
    
    /// Get cron expression
    pub fn expression(&self) -> &str {
        &self.expression
    }
    
    /// Get timezone
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
    
    /// Check if schedule is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable the schedule
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Create common schedule presets
    pub fn daily_at(hour: u8, minute: u8) -> Result<Self> {
        if hour > 23 || minute > 59 {
            return Err(WritemagicError::validation("Invalid time format"));
        }
        
        let expression = format!("{} {} * * *", minute, hour);
        Self::new(expression, None)
    }
    
    pub fn weekly_on(day: u8, hour: u8, minute: u8) -> Result<Self> {
        if day > 7 || hour > 23 || minute > 59 || day == 0 {
            return Err(WritemagicError::validation("Invalid schedule parameters"));
        }
        
        let expression = format!("{} {} * * {}", minute, hour, day - 1); // Cron uses 0-6 for days
        Self::new(expression, None)
    }
    
    pub fn every_n_minutes(minutes: u8) -> Result<Self> {
        if minutes == 0 || minutes > 59 {
            return Err(WritemagicError::validation("Minutes must be between 1 and 59"));
        }
        
        let expression = format!("*/{} * * * *", minutes);
        Self::new(expression, None)
    }
}

/// Agent resource quota for execution limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceQuota {
    max_cpu_cores: Option<f32>,
    max_memory_mb: Option<u64>,
    max_disk_io_mbps: Option<u64>,
    max_network_io_mbps: Option<u64>,
    max_execution_time: Option<Duration>,
}

impl ResourceQuota {
    /// Create unlimited quota
    pub fn unlimited() -> Self {
        Self {
            max_cpu_cores: None,
            max_memory_mb: None,
            max_disk_io_mbps: None,
            max_network_io_mbps: None,
            max_execution_time: None,
        }
    }
    
    /// Create a basic quota
    pub fn basic() -> Self {
        Self {
            max_cpu_cores: Some(0.5),
            max_memory_mb: Some(256),
            max_disk_io_mbps: Some(10),
            max_network_io_mbps: Some(5),
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
    
    /// Create a premium quota
    pub fn premium() -> Self {
        Self {
            max_cpu_cores: Some(2.0),
            max_memory_mb: Some(1024),
            max_disk_io_mbps: Some(50),
            max_network_io_mbps: Some(25),
            max_execution_time: Some(Duration::from_secs(1800)), // 30 minutes
        }
    }
    
    /// Set CPU limit
    pub fn with_cpu_limit(mut self, cores: f32) -> Result<Self> {
        if cores <= 0.0 {
            return Err(WritemagicError::validation("CPU cores must be greater than 0"));
        }
        self.max_cpu_cores = Some(cores);
        Ok(self)
    }
    
    /// Set memory limit
    pub fn with_memory_limit(mut self, memory_mb: u64) -> Result<Self> {
        if memory_mb == 0 {
            return Err(WritemagicError::validation("Memory limit must be greater than 0"));
        }
        self.max_memory_mb = Some(memory_mb);
        Ok(self)
    }
    
    /// Set execution time limit
    pub fn with_time_limit(mut self, duration: Duration) -> Result<Self> {
        if duration.is_zero() {
            return Err(WritemagicError::validation("Execution time limit must be greater than 0"));
        }
        self.max_execution_time = Some(duration);
        Ok(self)
    }
    
    /// Check if quota allows the given resource usage
    pub fn allows_usage(&self, cpu: f32, memory_mb: u64, duration: Duration) -> bool {
        if let Some(max_cpu) = self.max_cpu_cores {
            if cpu > max_cpu {
                return false;
            }
        }
        
        if let Some(max_memory) = self.max_memory_mb {
            if memory_mb > max_memory {
                return false;
            }
        }
        
        if let Some(max_time) = self.max_execution_time {
            if duration > max_time {
                return false;
            }
        }
        
        true
    }
}

/// Agent permission level for security
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PermissionLevel {
    Restricted,  // Can only perform basic document operations
    Standard,    // Can perform most operations within sandbox
    Elevated,    // Can perform advanced operations with some restrictions
    Full,        // Can perform all operations (dangerous)
}

impl fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionLevel::Restricted => write!(f, "Restricted"),
            PermissionLevel::Standard => write!(f, "Standard"),
            PermissionLevel::Elevated => write!(f, "Elevated"),
            PermissionLevel::Full => write!(f, "Full"),
        }
    }
}

/// Agent execution strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Immediate,      // Execute as soon as triggered
    Queued,        // Add to execution queue
    Scheduled,     // Execute at scheduled time
    Conditional,   // Execute only if conditions are met
    Manual,        // Require manual approval
}

impl fmt::Display for ExecutionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionStrategy::Immediate => write!(f, "Immediate"),
            ExecutionStrategy::Queued => write!(f, "Queued"),
            ExecutionStrategy::Scheduled => write!(f, "Scheduled"),
            ExecutionStrategy::Conditional => write!(f, "Conditional"),
            ExecutionStrategy::Manual => write!(f, "Manual"),
        }
    }
}

/// Agent workflow validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowValidation {
    allow_loops: bool,
    max_job_depth: u32,
    max_variables: u32,
    max_steps_per_job: u32,
    required_fields: Vec<String>,
}

impl WorkflowValidation {
    /// Create default validation rules
    pub fn default_rules() -> Self {
        Self {
            allow_loops: false,
            max_job_depth: 10,
            max_variables: 50,
            max_steps_per_job: 20,
            required_fields: vec!["version".to_string(), "name".to_string()],
        }
    }
    
    /// Create strict validation rules
    pub fn strict_rules() -> Self {
        Self {
            allow_loops: false,
            max_job_depth: 5,
            max_variables: 20,
            max_steps_per_job: 10,
            required_fields: vec![
                "version".to_string(), 
                "name".to_string(), 
                "description".to_string()
            ],
        }
    }
    
    /// Create lenient validation rules
    pub fn lenient_rules() -> Self {
        Self {
            allow_loops: true,
            max_job_depth: 50,
            max_variables: 200,
            max_steps_per_job: 100,
            required_fields: vec!["version".to_string()],
        }
    }
    
    /// Validate job count
    pub fn validate_job_count(&self, count: u32) -> Result<()> {
        if count > self.max_job_depth {
            return Err(WritemagicError::validation(format!(
                "Too many jobs: {} (max: {})", count, self.max_job_depth
            )));
        }
        Ok(())
    }
    
    /// Validate variable count
    pub fn validate_variable_count(&self, count: u32) -> Result<()> {
        if count > self.max_variables {
            return Err(WritemagicError::validation(format!(
                "Too many variables: {} (max: {})", count, self.max_variables
            )));
        }
        Ok(())
    }
    
    /// Validate steps per job
    pub fn validate_steps_count(&self, count: u32) -> Result<()> {
        if count > self.max_steps_per_job {
            return Err(WritemagicError::validation(format!(
                "Too many steps in job: {} (max: {})", count, self.max_steps_per_job
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_version() {
        let v1 = AgentVersion::new(1, 2, 3);
        let v2 = AgentVersion::new(1, 2, 4);
        let v3 = AgentVersion::new(2, 0, 0);
        
        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
        
        assert_eq!(v1.to_string(), "1.2.3");
        
        let pre_release = AgentVersion::new_pre_release(1, 0, 0, "beta.1".to_string()).unwrap();
        assert!(pre_release.is_pre_release());
        assert_eq!(pre_release.to_string(), "1.0.0-beta.1");
    }
    
    #[test]
    fn test_version_parsing() {
        let v1 = AgentVersion::from_string("1.2.3").unwrap();
        assert_eq!(v1.components(), (1, 2, 3));
        assert!(!v1.is_pre_release());
        
        let v2 = AgentVersion::from_string("2.0.0-alpha.1").unwrap();
        assert_eq!(v2.components(), (2, 0, 0));
        assert!(v2.is_pre_release());
        
        assert!(AgentVersion::from_string("invalid").is_err());
        assert!(AgentVersion::from_string("1.2").is_err());
    }
    
    #[test]
    fn test_execution_timeout() {
        let timeout = ExecutionTimeout::cancel_after(Duration::from_secs(60)).unwrap();
        assert_eq!(timeout.duration(), Duration::from_secs(60));
        assert_eq!(timeout.action(), &TimeoutAction::Cancel);
        
        assert!(ExecutionTimeout::new(Duration::from_secs(0), TimeoutAction::Cancel).is_err());
    }
    
    #[test]
    fn test_execution_schedule() {
        let schedule = ExecutionSchedule::daily_at(14, 30).unwrap();
        assert_eq!(schedule.expression(), "30 14 * * *");
        assert!(schedule.is_enabled());
        
        let weekly = ExecutionSchedule::weekly_on(1, 9, 0).unwrap(); // Monday at 9:00
        assert_eq!(weekly.expression(), "0 9 * * 0");
        
        let every_5_min = ExecutionSchedule::every_n_minutes(5).unwrap();
        assert_eq!(every_5_min.expression(), "*/5 * * * *");
        
        assert!(ExecutionSchedule::daily_at(25, 30).is_err()); // Invalid hour
        assert!(ExecutionSchedule::every_n_minutes(0).is_err()); // Invalid minutes
    }
    
    #[test]
    fn test_resource_quota() {
        let basic = ResourceQuota::basic();
        assert!(basic.allows_usage(0.3, 200, Duration::from_secs(100)));
        assert!(!basic.allows_usage(1.0, 200, Duration::from_secs(100))); // CPU too high
        assert!(!basic.allows_usage(0.3, 500, Duration::from_secs(100))); // Memory too high
        assert!(!basic.allows_usage(0.3, 200, Duration::from_secs(400))); // Time too long
        
        let unlimited = ResourceQuota::unlimited();
        assert!(unlimited.allows_usage(10.0, 10000, Duration::from_secs(3600))); // All allowed
        
        let custom = ResourceQuota::unlimited()
            .with_cpu_limit(1.0).unwrap()
            .with_memory_limit(512).unwrap();
        assert!(custom.allows_usage(0.8, 400, Duration::from_secs(3600)));
        assert!(!custom.allows_usage(1.2, 400, Duration::from_secs(3600)));
    }
    
    #[test]
    fn test_workflow_validation() {
        let strict = WorkflowValidation::strict_rules();
        assert!(strict.validate_job_count(3).is_ok());
        assert!(strict.validate_job_count(10).is_err());
        
        let lenient = WorkflowValidation::lenient_rules();
        assert!(lenient.validate_job_count(30).is_ok());
        assert!(lenient.validate_variable_count(150).is_ok());
    }
}
