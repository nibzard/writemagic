//! Agent domain aggregates

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::entities::{Agent, AgentWorkflow, AgentStatus, ExecutionContext, ExecutionResult, TriggerType, WorkflowTrigger};
use crate::value_objects::ExecutionPriority;
use chrono::{DateTime, Utc};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Agent aggregate root that manages agent lifecycle and execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAggregate {
    agent: Agent,
    execution_queue: VecDeque<QueuedExecution>,
    execution_history: Vec<ExecutionRecord>,
    resource_usage: ResourceUsage,
    version: u64,
    events: Vec<AgentEvent>,
}

/// Queued execution waiting to be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedExecution {
    pub id: EntityId,
    pub agent_id: EntityId,
    pub trigger_type: TriggerType,
    pub context: BTreeMap<String, serde_json::Value>,
    pub priority: ExecutionPriority,
    pub queued_at: DateTime<Utc>,
    pub execute_after: Option<DateTime<Utc>>,
    pub max_retries: u32,
    pub current_retry: u32,
}

/// Historical record of agent executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub execution_id: EntityId,
    pub agent_id: EntityId,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<ExecutionResult>,
    pub trigger_type: TriggerType,
    pub context: BTreeMap<String, serde_json::Value>,
    pub resource_usage: ExecutionResourceUsage,
}

/// Resource usage for a single execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResourceUsage {
    pub cpu_time_ms: u64,
    pub memory_peak_mb: u64,
    pub disk_io_bytes: u64,
    pub network_io_bytes: u64,
    pub duration: Duration,
}

/// Current resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub current_cpu_percent: f32,
    pub current_memory_mb: u64,
    pub total_executions: u64,
    pub total_cpu_time: Duration,
    pub total_memory_used: u64,
    pub average_execution_time: Duration,
    pub last_updated: DateTime<Utc>,
}

/// Agent domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    AgentCreated {
        agent_id: EntityId,
        name: String,
        version: String,
        created_by: EntityId,
        timestamp: DateTime<Utc>,
    },
    AgentUpdated {
        agent_id: EntityId,
        old_version: String,
        new_version: String,
        updated_by: EntityId,
        timestamp: DateTime<Utc>,
    },
    AgentActivated {
        agent_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    AgentDeactivated {
        agent_id: EntityId,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    ExecutionQueued {
        agent_id: EntityId,
        execution_id: EntityId,
        trigger_type: TriggerType,
        priority: ExecutionPriority,
        timestamp: DateTime<Utc>,
    },
    ExecutionStarted {
        agent_id: EntityId,
        execution_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    ExecutionCompleted {
        agent_id: EntityId,
        execution_id: EntityId,
        result: ExecutionResult,
        duration: Duration,
        timestamp: DateTime<Utc>,
    },
    ExecutionFailed {
        agent_id: EntityId,
        execution_id: EntityId,
        error: String,
        retry_count: u32,
        timestamp: DateTime<Utc>,
    },
    WorkflowUpdated {
        agent_id: EntityId,
        old_workflow_version: String,
        new_workflow_version: String,
        updated_by: EntityId,
        timestamp: DateTime<Utc>,
    },
    ResourceLimitExceeded {
        agent_id: EntityId,
        resource_type: String,
        limit: String,
        actual: String,
        timestamp: DateTime<Utc>,
    },
}

impl AgentAggregate {
    /// Create a new agent aggregate
    pub fn new(
        name: String,
        workflow: AgentWorkflow,
        created_by: EntityId,
    ) -> Result<Self> {
        let agent = Agent::new(name.clone(), workflow.clone(), created_by);
        
        let mut aggregate = Self {
            agent,
            execution_queue: VecDeque::new(),
            execution_history: Vec::new(),
            resource_usage: ResourceUsage::default(),
            version: 1,
            events: Vec::new(),
        };
        
        aggregate.add_event(AgentEvent::AgentCreated {
            agent_id: aggregate.agent.id,
            name,
            version: workflow.version,
            created_by,
            timestamp: Utc::now(),
        });
        
        Ok(aggregate)
    }
    
    /// Get the agent entity
    pub fn agent(&self) -> &Agent {
        &self.agent
    }
    
    /// Get agent ID
    pub fn id(&self) -> EntityId {
        self.agent.id
    }
    
    /// Get aggregate version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get pending events
    pub fn events(&self) -> &[AgentEvent] {
        &self.events
    }
    
    /// Update agent workflow
    pub fn update_workflow(&mut self, workflow: AgentWorkflow, updated_by: EntityId) -> Result<()> {
        let _old_version = self.agent.workflow.version.clone();
        
        // Validate new workflow
        self.agent.validate_workflow(&workflow)?;
        
        let old_workflow_version = self.agent.workflow.version.clone();
        self.agent.workflow = workflow.clone();
        self.agent.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(AgentEvent::WorkflowUpdated {
            agent_id: self.agent.id,
            old_workflow_version,
            new_workflow_version: workflow.version,
            updated_by,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Activate the agent
    pub fn activate(&mut self) -> Result<()> {
        if self.agent.is_active {
            return Err(WritemagicError::validation("Agent is already active"));
        }
        
        self.agent.is_active = true;
        self.agent.state.status = AgentStatus::Active;
        self.agent.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(AgentEvent::AgentActivated {
            agent_id: self.agent.id,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Deactivate the agent
    pub fn deactivate(&mut self, reason: String) -> Result<()> {
        if !self.agent.is_active {
            return Err(WritemagicError::validation("Agent is already inactive"));
        }
        
        // Clear execution queue when deactivating
        self.execution_queue.clear();
        
        self.agent.is_active = false;
        self.agent.state.status = AgentStatus::Disabled;
        self.agent.updated_at = Utc::now();
        self.version += 1;
        
        self.add_event(AgentEvent::AgentDeactivated {
            agent_id: self.agent.id,
            reason,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Queue an execution
    pub fn queue_execution(
        &mut self,
        trigger_type: TriggerType,
        context: BTreeMap<String, serde_json::Value>,
        priority: ExecutionPriority,
        execute_after: Option<DateTime<Utc>>,
    ) -> Result<EntityId> {
        if !self.agent.is_active {
            return Err(WritemagicError::validation("Cannot queue execution for inactive agent"));
        }
        
        // Check if agent can be triggered
        if !self.agent.can_trigger(&trigger_type, &context) {
            return Err(WritemagicError::validation("Agent trigger conditions not met"));
        }
        
        // Check queue size limits
        if self.execution_queue.len() >= 100 { // Configurable limit
            return Err(WritemagicError::validation("Execution queue is full"));
        }
        
        let priority_for_comparison = priority.clone();
        let execution = QueuedExecution {
            id: EntityId::new(),
            agent_id: self.agent.id,
            trigger_type: trigger_type.clone(),
            context,
            priority,
            queued_at: Utc::now(),
            execute_after,
            max_retries: 3, // Configurable
            current_retry: 0,
        };
        
        let execution_id = execution.id;
        
        // Insert in priority order
        let insert_pos = self.execution_queue
            .iter()
            .position(|e| e.priority < priority_for_comparison)
            .unwrap_or(self.execution_queue.len());
        
        self.execution_queue.insert(insert_pos, execution);
        self.version += 1;
        
        self.add_event(AgentEvent::ExecutionQueued {
            agent_id: self.agent.id,
            execution_id,
            trigger_type,
            priority: priority_for_comparison,
            timestamp: Utc::now(),
        });
        
        Ok(execution_id)
    }
    
    /// Get next execution to process
    pub fn get_next_execution(&mut self) -> Option<QueuedExecution> {
        let now = Utc::now();
        
        // Find first execution that is ready to run
        if let Some(pos) = self.execution_queue.iter().position(|e| {
            e.execute_after.map_or(true, |time| now >= time)
        }) {
            let execution = self.execution_queue.remove(pos)?;
            self.version += 1;
            Some(execution)
        } else {
            None
        }
    }
    
    /// Start execution
    pub fn start_execution(&mut self, execution: &QueuedExecution) -> Result<ExecutionContext> {
        if self.agent.state.status == AgentStatus::Running && 
           self.agent.config.max_concurrent_executions <= 1 {
            return Err(WritemagicError::validation("Agent is already running an execution"));
        }
        
        self.agent.state.status = AgentStatus::Running;
        self.agent.state.current_execution_id = Some(execution.id);
        self.agent.updated_at = Utc::now();
        self.version += 1;
        
        let trigger = WorkflowTrigger {
            trigger_type: execution.trigger_type.clone(),
            conditions: vec![],
            schedule: None,
        };
        
        let context = ExecutionContext {
            execution_id: execution.id,
            agent_id: self.agent.id,
            trigger,
            variables: execution.context.clone(),
            started_at: Utc::now(),
            user_id: Some(self.agent.created_by),
            project_id: execution.context.get("project_id")
                .and_then(|v| v.as_str())
                .and_then(|s| EntityId::from_string(s).ok()),
            document_id: execution.context.get("document_id")
                .and_then(|v| v.as_str())
                .and_then(|s| EntityId::from_string(s).ok()),
            environment: crate::entities::ExecutionEnvironment::Production,
        };
        
        self.add_event(AgentEvent::ExecutionStarted {
            agent_id: self.agent.id,
            execution_id: execution.id,
            timestamp: Utc::now(),
        });
        
        Ok(context)
    }
    
    /// Complete execution
    pub fn complete_execution(
        &mut self,
        execution_id: EntityId,
        result: ExecutionResult,
        resource_usage: ExecutionResourceUsage,
    ) -> Result<()> {
        // Update agent state
        self.agent.record_execution(result.clone());
        self.agent.state.status = AgentStatus::Active;
        self.agent.state.current_execution_id = None;
        
        // Update resource usage
        self.update_resource_usage(&resource_usage);
        
        // Record execution history
        let record = ExecutionRecord {
            execution_id,
            agent_id: self.agent.id,
            started_at: Utc::now() - resource_usage.duration, // Approximate start time
            completed_at: Some(Utc::now()),
            result: Some(result.clone()),
            trigger_type: TriggerType::Manual, // Would need to store this from context
            context: BTreeMap::new(), // Would need to store this from context
            resource_usage,
        };
        
        self.execution_history.push(record);
        
        // Keep only last 1000 executions
        if self.execution_history.len() > 1000 {
            self.execution_history.remove(0);
        }
        
        self.version += 1;
        
        let duration = match &result {
            ExecutionResult::Success { duration, .. } => *duration,
            ExecutionResult::Failure { duration, .. } => *duration,
            ExecutionResult::Cancelled { duration, .. } => *duration,
        };
        
        self.add_event(AgentEvent::ExecutionCompleted {
            agent_id: self.agent.id,
            execution_id,
            result,
            duration,
            timestamp: Utc::now(),
        });
        
        Ok(())
    }
    
    /// Handle execution failure
    pub fn handle_execution_failure(
        &mut self,
        execution_id: EntityId,
        error: String,
        retry_count: u32,
    ) -> Result<bool> {
        self.agent.state.status = AgentStatus::Active;
        self.agent.state.current_execution_id = None;
        self.version += 1;
        
        self.add_event(AgentEvent::ExecutionFailed {
            agent_id: self.agent.id,
            execution_id,
            error,
            retry_count,
            timestamp: Utc::now(),
        });
        
        // Return whether to retry
        Ok(retry_count < 3 && self.agent.config.retry_failed_executions)
    }
    
    /// Get execution statistics
    pub fn get_execution_statistics(&self) -> ExecutionStatistics {
        let total_executions = self.execution_history.len() as u64;
        let successful_executions = self.execution_history.iter()
            .filter(|r| matches!(r.result, Some(ExecutionResult::Success { .. })))
            .count() as u64;
        
        let failed_executions = self.execution_history.iter()
            .filter(|r| matches!(r.result, Some(ExecutionResult::Failure { .. })))
            .count() as u64;
        
        let average_duration = if total_executions > 0 {
            let total_duration: Duration = self.execution_history.iter()
                .filter_map(|r| match &r.result {
                    Some(ExecutionResult::Success { duration, .. }) => Some(*duration),
                    Some(ExecutionResult::Failure { duration, .. }) => Some(*duration),
                    Some(ExecutionResult::Cancelled { duration, .. }) => Some(*duration),
                    None => None,
                })
                .sum();
            
            total_duration / total_executions as u32
        } else {
            Duration::from_secs(0)
        };
        
        ExecutionStatistics {
            total_executions,
            successful_executions,
            failed_executions,
            success_rate: if total_executions > 0 {
                successful_executions as f32 / total_executions as f32 * 100.0
            } else {
                0.0
            },
            average_duration,
            queue_size: self.execution_queue.len(),
            current_status: self.agent.state.status.clone(),
            last_execution: self.agent.state.last_execution_at,
            resource_usage: self.resource_usage.clone(),
        }
    }
    
    /// Update resource usage tracking
    fn update_resource_usage(&mut self, execution_usage: &ExecutionResourceUsage) {
        self.resource_usage.total_executions += 1;
        self.resource_usage.total_cpu_time += execution_usage.duration;
        self.resource_usage.total_memory_used += execution_usage.memory_peak_mb;
        
        // Update average execution time
        if self.resource_usage.total_executions > 0 {
            self.resource_usage.average_execution_time = 
                self.resource_usage.total_cpu_time / self.resource_usage.total_executions as u32;
        }
        
        self.resource_usage.last_updated = Utc::now();
    }
    
    /// Clear events after persistence
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
    
    /// Add an event to the aggregate
    fn add_event(&mut self, event: AgentEvent) {
        self.events.push(event);
    }
    
    /// Get queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        let now = Utc::now();
        
        let ready_count = self.execution_queue.iter()
            .filter(|e| e.execute_after.map_or(true, |time| now >= time))
            .count();
        
        let scheduled_count = self.execution_queue.len() - ready_count;
        
        let priority_breakdown = {
            let mut breakdown = BTreeMap::new();
            for execution in &self.execution_queue {
                *breakdown.entry(execution.priority.clone()).or_insert(0) += 1;
            }
            breakdown
        };
        
        QueueStatus {
            total_queued: self.execution_queue.len(),
            ready_to_execute: ready_count,
            scheduled_for_later: scheduled_count,
            priority_breakdown,
            oldest_queued: self.execution_queue.front().map(|e| e.queued_at),
        }
    }
}

/// Execution statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f32,
    pub average_duration: Duration,
    pub queue_size: usize,
    pub current_status: AgentStatus,
    pub last_execution: Option<DateTime<Utc>>,
    pub resource_usage: ResourceUsage,
}

/// Queue status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub total_queued: usize,
    pub ready_to_execute: usize,
    pub scheduled_for_later: usize,
    pub priority_breakdown: BTreeMap<ExecutionPriority, usize>,
    pub oldest_queued: Option<DateTime<Utc>>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            current_cpu_percent: 0.0,
            current_memory_mb: 0,
            total_executions: 0,
            total_cpu_time: Duration::from_secs(0),
            total_memory_used: 0,
            average_execution_time: Duration::from_secs(0),
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{WorkflowTrigger, AgentWorkflow};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn test_create_agent_aggregate() {
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
        
        let aggregate = AgentAggregate::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        ).unwrap();
        
        assert_eq!(aggregate.agent().name, "Test Agent");
        assert!(aggregate.agent().is_active);
        assert_eq!(aggregate.version(), 1);
        assert_eq!(aggregate.events().len(), 1); // AgentCreated event
    }
    
    #[test]
    fn test_queue_execution() {
        let workflow = AgentWorkflow {
            version: "1.0".to_string(),
            name: "Test Workflow".to_string(),
            description: None,
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
        
        let mut aggregate = AgentAggregate::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        ).unwrap();
        
        let mut context = BTreeMap::new();
        context.insert("document_id".to_string(), json!("doc123"));
        
        let execution_id = aggregate.queue_execution(
            TriggerType::DocumentSaved,
            context,
            ExecutionPriority::Normal,
            None,
        ).unwrap();
        
        assert_eq!(aggregate.execution_queue.len(), 1);
        assert_eq!(aggregate.execution_queue[0].id, execution_id);
    }
    
    #[test]
    fn test_execution_priority_ordering() {
        let workflow = AgentWorkflow {
            version: "1.0".to_string(),
            name: "Test Workflow".to_string(),
            description: None,
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
        
        let mut aggregate = AgentAggregate::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        ).unwrap();
        
        let context = BTreeMap::new();
        
        // Queue executions with different priorities
        aggregate.queue_execution(
            TriggerType::DocumentSaved,
            context.clone(),
            ExecutionPriority::Low,
            None,
        ).unwrap();
        
        aggregate.queue_execution(
            TriggerType::DocumentSaved,
            context.clone(),
            ExecutionPriority::High,
            None,
        ).unwrap();
        
        aggregate.queue_execution(
            TriggerType::DocumentSaved,
            context.clone(),
            ExecutionPriority::Normal,
            None,
        ).unwrap();
        
        // Check that high priority is first
        assert_eq!(aggregate.execution_queue[0].priority, ExecutionPriority::High);
        assert_eq!(aggregate.execution_queue[1].priority, ExecutionPriority::Normal);
        assert_eq!(aggregate.execution_queue[2].priority, ExecutionPriority::Low);
    }
    
    #[test]
    fn test_activate_deactivate_agent() {
        let workflow = AgentWorkflow {
            version: "1.0".to_string(),
            name: "Test Workflow".to_string(),
            description: None,
            triggers: vec![],
            variables: BTreeMap::new(),
            jobs: BTreeMap::new(),
            on_success: None,
            on_failure: None,
        };
        
        let mut aggregate = AgentAggregate::new(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        ).unwrap();
        
        // Agent starts active
        assert!(aggregate.agent().is_active);
        
        // Deactivate
        aggregate.deactivate("Testing".to_string()).unwrap();
        assert!(!aggregate.agent().is_active);
        assert_eq!(aggregate.agent().state.status, AgentStatus::Disabled);
        
        // Reactivate
        aggregate.activate().unwrap();
        assert!(aggregate.agent().is_active);
        assert_eq!(aggregate.agent().state.status, AgentStatus::Active);
        
        // Test double activation fails
        assert!(aggregate.activate().is_err());
    }
}
