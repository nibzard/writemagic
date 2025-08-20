//! Agent domain services

use writemagic_shared::{EntityId, WritemagicError, Result};
use crate::aggregates::{AgentAggregate, QueuedExecution, ExecutionStatistics, ResourceUsage};
use crate::entities::{Agent, AgentWorkflow, ExecutionContext, ExecutionResult, TriggerType, AgentStatus};
use crate::repositories::{AgentRepository, AgentWorkflowRepository, ExecutionRepository, AgentSearchCriteria, WorkflowSearchCriteria};
use crate::value_objects::{ExecutionPriority, ExecutionStrategy, WorkflowValidation};
use chrono::{DateTime, Utc};
use std::time::Duration;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};

/// Service for managing agent lifecycle and operations
pub struct AgentManagementService {
    agent_repository: Arc<dyn AgentRepository>,
    workflow_repository: Arc<dyn AgentWorkflowRepository>,
    execution_repository: Arc<dyn ExecutionRepository>,
    running_agents: Arc<RwLock<HashMap<EntityId, Arc<Mutex<AgentAggregate>>>>>,
}

impl AgentManagementService {
    /// Create a new agent management service
    pub fn new(
        agent_repository: Arc<dyn AgentRepository>,
        workflow_repository: Arc<dyn AgentWorkflowRepository>,
        execution_repository: Arc<dyn ExecutionRepository>,
    ) -> Self {
        Self {
            agent_repository,
            workflow_repository,
            execution_repository,
            running_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new agent from a workflow
    pub async fn create_agent(
        &self,
        name: String,
        workflow: AgentWorkflow,
        created_by: EntityId,
    ) -> Result<EntityId> {
        // Validate workflow before creating agent
        self.validate_workflow(&workflow)?;
        
        // Create agent aggregate
        let mut aggregate = AgentAggregate::new(name, workflow.clone(), created_by)?;
        let agent_id = aggregate.id();
        
        // Save to repository
        self.agent_repository.save(&mut aggregate).await?;
        
        // Store workflow template
        self.workflow_repository.save_workflow(&workflow).await?;
        
        Ok(agent_id)
    }
    
    /// Load and start an agent
    pub async fn start_agent(&self, agent_id: &EntityId) -> Result<()> {
        let mut aggregate = self.load_agent(agent_id).await?;
        
        // Activate the agent
        aggregate.activate()?;
        
        // Save updated state
        self.agent_repository.save(&mut aggregate).await?;
        
        // Add to running agents
        let mut running = self.running_agents.write().await;
        running.insert(*agent_id, Arc::new(Mutex::new(aggregate)));
        
        Ok(())
    }
    
    /// Stop an agent
    pub async fn stop_agent(&self, agent_id: &EntityId, reason: String) -> Result<()> {
        // Remove from running agents first
        let aggregate_mutex = {
            let mut running = self.running_agents.write().await;
            running.remove(agent_id)
        };
        
        if let Some(aggregate_mutex) = aggregate_mutex {
            let mut aggregate = aggregate_mutex.lock().await;
            
            // Deactivate the agent
            aggregate.deactivate(reason)?;
            
            // Save final state
            self.agent_repository.save(&mut aggregate).await?;
        } else {
            // Agent not running, just update in repository
            let mut aggregate = self.load_agent(agent_id).await?;
            aggregate.deactivate(reason)?;
            self.agent_repository.save(&mut aggregate).await?;
        }
        
        Ok(())
    }
    
    /// Update an agent's workflow
    pub async fn update_agent_workflow(
        &self,
        agent_id: &EntityId,
        workflow: AgentWorkflow,
        updated_by: EntityId,
    ) -> Result<()> {
        // Validate new workflow
        self.validate_workflow(&workflow)?;
        
        let mut aggregate = self.load_agent(agent_id).await?;
        
        // Update workflow
        aggregate.update_workflow(workflow.clone(), updated_by)?;
        
        // Save updated agent
        self.agent_repository.save(&mut aggregate).await?;
        
        // Save new workflow version
        self.workflow_repository.save_workflow(&workflow).await?;
        
        // Update running agent if it exists
        if let Some(running_aggregate) = self.get_running_agent(agent_id).await {
            let mut running = running_aggregate.lock().await;
            *running = aggregate;
        }
        
        Ok(())
    }
    
    /// Find agents by search criteria
    pub async fn find_agents(&self, criteria: AgentSearchCriteria) -> Result<Vec<Agent>> {
        self.agent_repository.find_by_criteria(criteria).await
    }
    
    /// Get agent statistics
    pub async fn get_agent_statistics(&self, agent_id: &EntityId) -> Result<ExecutionStatistics> {
        let repo_stats = self.execution_repository.get_execution_stats(agent_id).await?;
        
        // Convert from repository stats to aggregates stats
        Ok(ExecutionStatistics {
            total_executions: repo_stats.total_executions,
            successful_executions: repo_stats.successful_executions,
            failed_executions: repo_stats.failed_executions,
            success_rate: repo_stats.success_rate as f32,
            average_duration: std::time::Duration::from_millis(repo_stats.average_duration_ms),
            queue_size: 0, // Would need to get this from somewhere else
            current_status: AgentStatus::Active, // Would need to determine this
            last_execution: repo_stats.last_execution_at,
            resource_usage: ResourceUsage::default(),
        })
    }
    
    /// List all active agents
    pub async fn list_active_agents(&self) -> Result<Vec<Agent>> {
        self.agent_repository.list_active().await
    }
    
    /// Get system-wide agent status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let status_counts = self.agent_repository.count_by_status().await?;
        let running_count = self.running_agents.read().await.len() as u64;
        
        Ok(SystemStatus {
            total_agents: status_counts.values().sum(),
            active_agents: *status_counts.get("active").unwrap_or(&0),
            running_agents: running_count,
            disabled_agents: *status_counts.get("disabled").unwrap_or(&0),
            error_agents: *status_counts.get("error").unwrap_or(&0),
        })
    }
    
    /// Load agent from repository
    async fn load_agent(&self, agent_id: &EntityId) -> Result<AgentAggregate> {
        self.agent_repository
            .load(agent_id)
            .await?
            .ok_or_else(|| WritemagicError::not_found(format!("Agent not found: {}", agent_id)))
    }
    
    /// Get running agent reference
    async fn get_running_agent(&self, agent_id: &EntityId) -> Option<Arc<Mutex<AgentAggregate>>> {
        let running = self.running_agents.read().await;
        running.get(agent_id).cloned()
    }
    
    /// Validate workflow before saving
    fn validate_workflow(&self, workflow: &AgentWorkflow) -> Result<()> {
        let validation = WorkflowValidation::default_rules();
        
        // Validate job count
        validation.validate_job_count(workflow.jobs.len() as u32)?;
        
        // Validate variable count
        validation.validate_variable_count(workflow.variables.len() as u32)?;
        
        // Validate required fields
        if workflow.name.trim().is_empty() {
            return Err(WritemagicError::validation("Workflow name cannot be empty"));
        }
        
        if workflow.version.trim().is_empty() {
            return Err(WritemagicError::validation("Workflow version cannot be empty"));
        }
        
        // Validate triggers
        if workflow.triggers.is_empty() {
            return Err(WritemagicError::validation("Workflow must have at least one trigger"));
        }
        
        Ok(())
    }
}

/// Type alias for running agents map to reduce complexity
type RunningAgents = Arc<RwLock<HashMap<EntityId, Arc<Mutex<AgentAggregate>>>>>;

/// Service for executing agent workflows
pub struct AgentExecutionService {
    agent_repository: Arc<dyn AgentRepository>,
    #[allow(dead_code)] // TODO: Implement execution tracking in Phase 2
    execution_repository: Arc<dyn ExecutionRepository>,
    running_agents: RunningAgents,
    #[allow(dead_code)] // TODO: Implement execution queue processing in Phase 2
    execution_queue: Arc<Mutex<VecDeque<QueuedExecution>>>,
}

impl AgentExecutionService {
    /// Create a new execution service
    pub fn new(
        agent_repository: Arc<dyn AgentRepository>,
        execution_repository: Arc<dyn ExecutionRepository>,
        running_agents: RunningAgents,
    ) -> Self {
        Self {
            agent_repository,
            execution_repository,
            running_agents,
            execution_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    /// Trigger agent execution
    pub async fn trigger_execution(
        &self,
        agent_id: &EntityId,
        trigger_type: TriggerType,
        context: BTreeMap<String, Value>,
        priority: ExecutionPriority,
        execute_after: Option<DateTime<Utc>>,
    ) -> Result<EntityId> {
        // Get running agent
        let agent_mutex = self.get_running_agent(agent_id).await
            .ok_or_else(|| WritemagicError::not_found(format!("Running agent not found: {}", agent_id)))?;
        
        let mut agent = agent_mutex.lock().await;
        
        // Queue execution on agent
        let execution_id = agent.queue_execution(trigger_type, context, priority, execute_after)?;
        
        // Save updated agent state
        self.agent_repository.save(&mut agent).await?;
        
        Ok(execution_id)
    }
    
    /// Execute next queued execution
    pub async fn execute_next(&self) -> Result<Option<ExecutionResult>> {
        // Find agent with highest priority queued execution
        let (agent_id, execution) = self.find_next_execution().await?;
        
        if let Some((agent_id, execution)) = agent_id.zip(execution) {
            self.execute_workflow(agent_id, execution).await.map(Some)
        } else {
            Ok(None)
        }
    }
    
    /// Execute workflow for a specific execution
    async fn execute_workflow(
        &self,
        agent_id: EntityId,
        execution: QueuedExecution,
    ) -> Result<ExecutionResult> {
        let agent_mutex = self.get_running_agent(&agent_id).await
            .ok_or_else(|| WritemagicError::not_found(format!("Running agent not found: {}", agent_id)))?;
        
        let mut agent = agent_mutex.lock().await;
        
        // Start execution
        let context = agent.start_execution(&execution)?;
        let start_time = Utc::now();
        
        // Execute workflow steps
        let result = self.execute_workflow_steps(&context).await;
        let end_time = Utc::now();
        let duration = (end_time - start_time).to_std().unwrap_or(std::time::Duration::from_secs(0));
        
        // Create execution result
        let execution_result = match result {
            Ok(outputs) => ExecutionResult::Success { duration, outputs },
            Err(error) => ExecutionResult::Failure {
                error: error.to_string(),
                step_id: None, // Would be set during step execution
                duration,
            },
        };
        
        // Record execution completion
        let resource_usage = crate::aggregates::ExecutionResourceUsage {
            cpu_time_ms: duration.as_millis() as u64,
            memory_peak_mb: 0, // Would be measured during execution
            disk_io_bytes: 0,  // Would be measured during execution
            network_io_bytes: 0, // Would be measured during execution
            duration,
        };
        
        agent.complete_execution(execution.id, execution_result.clone(), resource_usage)?;
        
        // Save updated agent state
        self.agent_repository.save(&mut agent).await?;
        
        Ok(execution_result)
    }
    
    /// Execute workflow steps
    async fn execute_workflow_steps(
        &self,
        _context: &ExecutionContext,
    ) -> Result<BTreeMap<String, Value>> {
        let outputs = BTreeMap::new();
        
        // For now, just return success with empty outputs
        // In a real implementation, this would:
        // 1. Get the workflow from the agent
        // 2. Execute each job in the workflow
        // 3. Handle dependencies between jobs
        // 4. Execute steps within each job
        // 5. Handle workflow variables and outputs
        // 6. Implement retry logic and error handling
        
        Ok(outputs)
    }
    
    /// Find the next execution to process
    async fn find_next_execution(&self) -> Result<(Option<EntityId>, Option<QueuedExecution>)> {
        let running = self.running_agents.read().await;
        
        let mut best_execution: Option<(EntityId, QueuedExecution)> = None;
        let mut best_priority = ExecutionPriority::Low;
        let mut best_time = Utc::now();
        
        for (agent_id, agent_mutex) in running.iter() {
            let mut agent = agent_mutex.lock().await;
            
            if let Some(execution) = agent.get_next_execution() {
                let should_select = match &best_execution {
                    None => true,
                    Some(_) => {
                        execution.priority > best_priority ||
                        (execution.priority == best_priority && execution.queued_at < best_time)
                    }
                };
                
                if should_select {
                    best_priority = execution.priority.clone();
                    best_time = execution.queued_at;
                    best_execution = Some((*agent_id, execution));
                }
            }
        }
        
        if let Some((agent_id, execution)) = best_execution {
            Ok((Some(agent_id), Some(execution)))
        } else {
            Ok((None, None))
        }
    }
    
    /// Get running agent reference
    async fn get_running_agent(&self, agent_id: &EntityId) -> Option<Arc<Mutex<AgentAggregate>>> {
        let running = self.running_agents.read().await;
        running.get(agent_id).cloned()
    }
}

/// Service for managing agent workflows and templates
pub struct AgentWorkflowService {
    workflow_repository: Arc<dyn AgentWorkflowRepository>,
}

impl AgentWorkflowService {
    /// Create a new workflow service
    pub fn new(workflow_repository: Arc<dyn AgentWorkflowRepository>) -> Self {
        Self {
            workflow_repository,
        }
    }
    
    /// Create a workflow template
    pub async fn create_workflow_template(&self, workflow: AgentWorkflow) -> Result<()> {
        // Validate workflow
        let validation = WorkflowValidation::default_rules();
        validation.validate_job_count(workflow.jobs.len() as u32)?;
        validation.validate_variable_count(workflow.variables.len() as u32)?;
        
        self.workflow_repository.save_workflow(&workflow).await
    }
    
    /// Get workflow by name and version
    pub async fn get_workflow(&self, name: &str, version: &str) -> Result<Option<AgentWorkflow>> {
        self.workflow_repository.load_workflow(name, version).await
    }
    
    /// List all versions of a workflow
    pub async fn list_workflow_versions(&self, name: &str) -> Result<Vec<AgentWorkflow>> {
        self.workflow_repository.list_workflow_versions(name).await
    }
    
    /// Get latest version of a workflow
    pub async fn get_latest_workflow(&self, name: &str) -> Result<Option<AgentWorkflow>> {
        self.workflow_repository.get_latest_version(name).await
    }
    
    /// Find workflows by criteria
    pub async fn find_workflows(&self, criteria: WorkflowSearchCriteria) -> Result<Vec<AgentWorkflow>> {
        self.workflow_repository.find_workflows(criteria).await
    }
    
    /// Clone workflow with new version
    pub async fn clone_workflow(
        &self,
        source_name: &str,
        source_version: &str,
        new_version: &str,
    ) -> Result<AgentWorkflow> {
        let mut workflow = self.workflow_repository
            .load_workflow(source_name, source_version)
            .await?
            .ok_or_else(|| WritemagicError::not_found("Source workflow not found"))?;
        
        // Update version
        workflow.version = new_version.to_string();
        
        // Save new version
        self.workflow_repository.save_workflow(&workflow).await?;
        
        Ok(workflow)
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub total_agents: u64,
    pub active_agents: u64,
    pub running_agents: u64,
    pub disabled_agents: u64,
    pub error_agents: u64,
}

/// Agent orchestration service that coordinates all agent operations
pub struct AgentOrchestrationService {
    management_service: Arc<AgentManagementService>,
    execution_service: Arc<AgentExecutionService>,
    #[allow(dead_code)] // TODO: Implement workflow orchestration in Phase 2
    workflow_service: Arc<AgentWorkflowService>,
}

impl AgentOrchestrationService {
    /// Create a new orchestration service
    pub fn new(
        management_service: Arc<AgentManagementService>,
        execution_service: Arc<AgentExecutionService>,
        workflow_service: Arc<AgentWorkflowService>,
    ) -> Self {
        Self {
            management_service,
            execution_service,
            workflow_service,
        }
    }
    
    /// Create and start an agent in one operation
    pub async fn create_and_start_agent(
        &self,
        name: String,
        workflow: AgentWorkflow,
        created_by: EntityId,
    ) -> Result<EntityId> {
        // Create agent
        let agent_id = self.management_service.create_agent(name, workflow, created_by).await?;
        
        // Start agent
        self.management_service.start_agent(&agent_id).await?;
        
        Ok(agent_id)
    }
    
    /// Trigger execution with automatic priority handling
    pub async fn smart_trigger(
        &self,
        agent_id: &EntityId,
        trigger_type: TriggerType,
        context: BTreeMap<String, Value>,
        strategy: ExecutionStrategy,
    ) -> Result<ExecutionResult> {
        let priority = match strategy {
            ExecutionStrategy::Immediate => ExecutionPriority::Critical,
            ExecutionStrategy::Queued => ExecutionPriority::Normal,
            ExecutionStrategy::Scheduled => ExecutionPriority::Low,
            ExecutionStrategy::Conditional => ExecutionPriority::Normal,
            ExecutionStrategy::Manual => ExecutionPriority::High,
        };
        
        let execute_after = match strategy {
            ExecutionStrategy::Immediate => None,
            ExecutionStrategy::Scheduled => Some(Utc::now() + chrono::Duration::minutes(1)),
            _ => None,
        };
        
        // Trigger execution
        let _execution_id = self.execution_service
            .trigger_execution(agent_id, trigger_type, context, priority, execute_after)
            .await?;
        
        // For immediate execution, process right away
        if matches!(strategy, ExecutionStrategy::Immediate) {
            if let Some(result) = self.execution_service.execute_next().await? {
                Ok(result)
            } else {
                // If no execution was processed, return a queued result
                Ok(ExecutionResult::Success {
                    duration: Duration::from_secs(0),
                    outputs: BTreeMap::new(),
                })
            }
        } else {
            // Return a queued result
            Ok(ExecutionResult::Success {
                duration: Duration::from_secs(0),
                outputs: BTreeMap::new(),
            })
        }
    }
    
    /// Get comprehensive system status
    pub async fn get_comprehensive_status(&self) -> Result<ComprehensiveSystemStatus> {
        let system_status = self.management_service.get_system_status().await?;
        
        Ok(ComprehensiveSystemStatus {
            system: system_status,
            workflow_templates: 0, // Would query workflow repository
            execution_queue_size: 0, // Would query execution service
            average_execution_time_ms: 0, // Would calculate from statistics
        })
    }
}

/// Comprehensive system status
#[derive(Debug, Clone)]
pub struct ComprehensiveSystemStatus {
    pub system: SystemStatus,
    pub workflow_templates: u64,
    pub execution_queue_size: u64,
    pub average_execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{WorkflowTrigger, TriggerType};
    use crate::repositories::SqliteAgentRepository;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_agent_management_service() {
        let agent_repo = Arc::new(SqliteAgentRepository::new());
        let workflow_repo = Arc::new(crate::repositories::SqliteAgentWorkflowRepository::new());
        let execution_repo = Arc::new(crate::repositories::SqliteExecutionRepository::new());
        
        let service = AgentManagementService::new(agent_repo, workflow_repo, execution_repo);
        
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
        
        let agent_id = service.create_agent(
            "Test Agent".to_string(),
            workflow,
            EntityId::new(),
        ).await;
        
        assert!(agent_id.is_ok());
    }
    
    #[test]
    fn test_system_status() {
        let status = SystemStatus {
            total_agents: 10,
            active_agents: 8,
            running_agents: 3,
            disabled_agents: 2,
            error_agents: 0,
        };
        
        assert_eq!(status.active_agents + status.disabled_agents, status.total_agents);
    }
    
    #[test]
    fn test_comprehensive_system_status() {
        let system_status = SystemStatus {
            total_agents: 5,
            active_agents: 4,
            running_agents: 2,
            disabled_agents: 1,
            error_agents: 0,
        };
        
        let comprehensive = ComprehensiveSystemStatus {
            system: system_status,
            workflow_templates: 3,
            execution_queue_size: 5,
            average_execution_time_ms: 1500,
        };
        
        assert_eq!(comprehensive.system.total_agents, 5);
        assert_eq!(comprehensive.workflow_templates, 3);
    }
}
