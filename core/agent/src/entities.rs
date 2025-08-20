//! Agent domain entities

use writemagic_shared::{EntityId, WritemagicError, Result};
use chrono::{DateTime, Utc};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use garde::Validate;

/// An agent that executes YAML-based workflows and automation tasks
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Agent {
    #[garde(skip)]
    pub id: EntityId,
    #[garde(length(min = 1, max = 100))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(skip)]
    pub workflow: AgentWorkflow,
    #[garde(skip)]
    pub config: AgentConfig,
    #[garde(skip)]
    pub state: AgentState,
    #[garde(skip)]
    pub metadata: AgentMetadata,
    #[garde(skip)]
    pub created_at: DateTime<Utc>,
    #[garde(skip)]
    pub updated_at: DateTime<Utc>,
    #[garde(skip)]
    pub created_by: EntityId,
    #[garde(skip)]
    pub is_active: bool,
}

/// YAML-based workflow definition for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWorkflow {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub triggers: Vec<WorkflowTrigger>,
    pub variables: BTreeMap<String, WorkflowVariable>,
    pub jobs: BTreeMap<String, WorkflowJob>,
    pub on_success: Option<Vec<WorkflowAction>>,
    pub on_failure: Option<Vec<WorkflowAction>>,
}

/// Triggers that can initiate workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    pub trigger_type: TriggerType,
    pub conditions: Vec<TriggerCondition>,
    pub schedule: Option<CronSchedule>,
}

/// Types of triggers available
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    DocumentSaved,
    DocumentCreated,
    ProjectCreated,
    CommitMade,
    TimeSchedule,
    FileChanged,
    AIResponseReceived,
    GoalAchieved,
    Manual,
    WebhookReceived,
}

/// Conditions for trigger activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

/// Operators for condition evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    MatchesRegex,
    In,
    NotIn,
}

/// Cron-style scheduling for time-based triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    pub expression: String,
    pub timezone: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Workflow variables that can be used in jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariable {
    pub name: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub is_secret: bool,
}

/// A job within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowJob {
    pub name: String,
    pub description: Option<String>,
    pub depends_on: Vec<String>,
    pub if_condition: Option<String>,
    pub timeout: Option<Duration>,
    pub retry: Option<RetryConfig>,
    pub steps: Vec<WorkflowStep>,
}

/// Individual steps within a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: WorkflowAction,
    pub if_condition: Option<String>,
    pub with: Option<BTreeMap<String, serde_json::Value>>,
    pub env: Option<BTreeMap<String, String>>,
}

/// Actions that can be performed by workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    /// Document operations
    CreateDocument {
        title: String,
        content: Option<String>,
        project_id: Option<EntityId>,
    },
    UpdateDocument {
        document_id: EntityId,
        content: Option<String>,
        title: Option<String>,
    },
    
    /// Project operations
    CreateProject {
        name: String,
        description: Option<String>,
        template: Option<String>,
    },
    
    /// AI operations
    AIGenerate {
        prompt: String,
        provider: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    },
    
    /// File operations
    WriteFile {
        path: String,
        content: String,
        append: Option<bool>,
    },
    
    /// Version control operations
    CreateCommit {
        document_id: EntityId,
        message: String,
        branch: Option<String>,
    },
    
    /// Notification operations
    SendNotification {
        title: String,
        message: String,
        recipients: Vec<String>,
    },
    
    /// Wait/delay operations
    Sleep {
        duration: Duration,
    },
}

/// Agent configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_concurrent_executions: u32,
    pub execution_timeout: Duration,
    pub retry_failed_executions: bool,
    pub log_level: LogLevel,
    pub enable_monitoring: bool,
    pub resource_limits: ResourceLimits,
    pub security_settings: SecuritySettings,
}

/// Logging levels for agent execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Resource limits for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub max_disk_space_mb: u64,
    pub max_network_requests_per_minute: u32,
}

/// Security settings for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub allow_file_access: bool,
    pub allow_network_access: bool,
    pub allow_system_commands: bool,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub sandbox_mode: bool,
}

/// Current state of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub current_execution_id: Option<EntityId>,
    pub last_execution_at: Option<DateTime<Utc>>,
    pub last_execution_result: Option<ExecutionResult>,
    pub next_scheduled_run: Option<DateTime<Utc>>,
    pub execution_count: u64,
    pub failure_count: u64,
    pub average_execution_time: Option<Duration>,
}

/// Status of an agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Paused,
    Disabled,
    Running,
    Error,
}

/// Result of an execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success {
        duration: Duration,
        outputs: BTreeMap<String, serde_json::Value>,
    },
    Failure {
        error: String,
        step_id: Option<String>,
        duration: Duration,
    },
    Cancelled {
        reason: String,
        duration: Duration,
    },
}

/// Agent metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub tags: Vec<String>,
    pub category: AgentCategory,
    pub version: String,
    pub author: Option<String>,
    pub documentation_url: Option<String>,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub last_modified_by: Option<EntityId>,
}

/// Categories for organizing agents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentCategory {
    Automation,
    ContentGeneration,
    DocumentProcessing,
    ProjectManagement,
    Integration,
    Monitoring,
    Backup,
    Custom,
}

/// Execution context for running agent workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: EntityId,
    pub agent_id: EntityId,
    pub trigger: WorkflowTrigger,
    pub variables: BTreeMap<String, serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub user_id: Option<EntityId>,
    pub project_id: Option<EntityId>,
    pub document_id: Option<EntityId>,
    pub environment: ExecutionEnvironment,
}

/// Environment for agent execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    Development,
    Staging,
    Production,
}

/// Retry configuration for jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: Duration,
    pub backoff_strategy: BackoffStrategy,
}

/// Backoff strategies for retries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
}

impl Agent {
    /// Create a new agent from a YAML workflow definition
    pub fn new(
        name: String,
        workflow: AgentWorkflow,
        created_by: EntityId,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: EntityId::new(),
            name,
            description: workflow.description.clone(),
            workflow,
            config: AgentConfig::default(),
            state: AgentState::default(),
            metadata: AgentMetadata::default(),
            created_at: now,
            updated_at: now,
            created_by,
            is_active: true,
        }
    }
    
    /// Validate the workflow configuration
    pub fn validate_workflow(&self, workflow: &AgentWorkflow) -> Result<()> {
        // Basic validation
        if workflow.name.trim().is_empty() {
            return Err(WritemagicError::validation("Workflow name cannot be empty"));
        }
        
        if workflow.version.trim().is_empty() {
            return Err(WritemagicError::validation("Workflow version cannot be empty"));
        }
        
        if workflow.triggers.is_empty() {
            return Err(WritemagicError::validation("Workflow must have at least one trigger"));
        }
        
        // Validate jobs
        if workflow.jobs.len() > 50 {
            return Err(WritemagicError::validation("Too many jobs in workflow"));
        }
        
        // Validate variables
        if workflow.variables.len() > 100 {
            return Err(WritemagicError::validation("Too many variables in workflow"));
        }
        
        // Validate each job
        for (job_name, job) in &workflow.jobs {
            if job_name.trim().is_empty() {
                return Err(WritemagicError::validation("Job name cannot be empty"));
            }
            
            if job.steps.len() > 20 {
                return Err(WritemagicError::validation(format!(
                    "Too many steps in job '{}': {} (max: 20)", 
                    job_name, 
                    job.steps.len()
                )));
            }
            
            // Check for circular dependencies
            if job.depends_on.contains(job_name) {
                return Err(WritemagicError::validation(format!(
                    "Job '{}' cannot depend on itself",
                    job_name
                )));
            }
        }
        
        Ok(())
    }
    
    /// Check if the agent can be triggered by the given conditions
    pub fn can_trigger(&self, trigger_type: &TriggerType, context: &BTreeMap<String, serde_json::Value>) -> bool {
        if !self.is_active || self.state.status == AgentStatus::Disabled {
            return false;
        }
        
        self.workflow.triggers.iter().any(|trigger| {
            trigger.trigger_type == *trigger_type && 
            self.evaluate_conditions(&trigger.conditions, context)
        })
    }
    
    /// Evaluate trigger conditions against context
    fn evaluate_conditions(&self, conditions: &[TriggerCondition], context: &BTreeMap<String, serde_json::Value>) -> bool {
        if conditions.is_empty() {
            return true;
        }
        
        conditions.iter().all(|condition| {
            let context_value = context.get(&condition.field);
            self.evaluate_condition(condition, context_value)
        })
    }
    
    /// Evaluate a single condition
    fn evaluate_condition(&self, condition: &TriggerCondition, context_value: Option<&serde_json::Value>) -> bool {
        match context_value {
            Some(value) => {
                match condition.operator {
                    ConditionOperator::Equals => value == &condition.value,
                    ConditionOperator::NotEquals => value != &condition.value,
                    ConditionOperator::Contains => {
                        if let (Some(text), Some(search)) = (value.as_str(), condition.value.as_str()) {
                            text.contains(search)
                        } else {
                            false
                        }
                    },
                    ConditionOperator::GreaterThan => {
                        if let (Some(num1), Some(num2)) = (value.as_f64(), condition.value.as_f64()) {
                            num1 > num2
                        } else {
                            false
                        }
                    },
                    ConditionOperator::LessThan => {
                        if let (Some(num1), Some(num2)) = (value.as_f64(), condition.value.as_f64()) {
                            num1 < num2
                        } else {
                            false
                        }
                    },
                    _ => false, // Other operators simplified for this implementation
                }
            },
            None => false,
        }
    }
    
    /// Record execution result
    pub fn record_execution(&mut self, result: ExecutionResult) {
        self.state.last_execution_at = Some(Utc::now());
        self.state.execution_count += 1;
        
        match &result {
            ExecutionResult::Success { .. } => {
                self.metadata.successful_executions += 1;
            },
            ExecutionResult::Failure { .. } => {
                self.state.failure_count += 1;
                self.metadata.failed_executions += 1;
            },
            ExecutionResult::Cancelled { .. } => {
                // Cancelled executions don't count as failures
            },
        }
        
        self.state.last_execution_result = Some(result);
        self.metadata.total_executions += 1;
        self.updated_at = Utc::now();
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 1,
            execution_timeout: Duration::from_secs(300), // 5 minutes
            retry_failed_executions: true,
            log_level: LogLevel::Info,
            enable_monitoring: true,
            resource_limits: ResourceLimits::default(),
            security_settings: SecuritySettings::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            max_disk_space_mb: 1024,
            max_network_requests_per_minute: 60,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            allow_file_access: false,
            allow_network_access: true,
            allow_system_commands: false,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            sandbox_mode: true,
        }
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Active,
            current_execution_id: None,
            last_execution_at: None,
            last_execution_result: None,
            next_scheduled_run: None,
            execution_count: 0,
            failure_count: 0,
            average_execution_time: None,
        }
    }
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            category: AgentCategory::Custom,
            version: "1.0.0".to_string(),
            author: None,
            documentation_url: None,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time: Duration::from_secs(0),
            last_modified_by: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_agent() {
        let workflow = AgentWorkflow {
            version: "1.0".to_string(),
            name: "Test Workflow".to_string(),
            description: Some("A test workflow".to_string()),
            triggers: vec![
                WorkflowTrigger {
                    trigger_type: TriggerType::DocumentSaved,
                    conditions: vec![],
                    schedule: None,
                }
            ],
            variables: BTreeMap::new(),
            jobs: BTreeMap::new(),
            on_success: None,
            on_failure: None,
        };
        
        let agent = Agent::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        );
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.state.status, AgentStatus::Active);
        assert!(agent.is_active);
    }
    
    #[test]
    fn test_trigger_evaluation() {
        let mut context = BTreeMap::new();
        context.insert("document_type".to_string(), json!("text"));
        context.insert("word_count".to_string(), json!(1000));
        
        let conditions = vec![
            TriggerCondition {
                field: "document_type".to_string(),
                operator: ConditionOperator::Equals,
                value: json!("text"),
            },
            TriggerCondition {
                field: "word_count".to_string(),
                operator: ConditionOperator::GreaterThan,
                value: json!(500),
            }
        ];
        
        let workflow = AgentWorkflow {
            version: "1.0".to_string(),
            name: "Test Workflow".to_string(),
            description: None,
            triggers: vec![
                WorkflowTrigger {
                    trigger_type: TriggerType::DocumentSaved,
                    conditions,
                    schedule: None,
                }
            ],
            variables: BTreeMap::new(),
            jobs: BTreeMap::new(),
            on_success: None,
            on_failure: None,
        };
        
        let agent = Agent::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        );
        
        assert!(agent.can_trigger(&TriggerType::DocumentSaved, &context));
        assert!(!agent.can_trigger(&TriggerType::DocumentCreated, &context));
    }
}
