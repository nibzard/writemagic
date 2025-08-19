//! Agent domain - File-based YAML agents for background processing
//!
//! This domain provides a comprehensive system for managing YAML-based automation agents
//! that can execute workflows based on triggers and perform various actions within the
//! WriteMagic ecosystem.

pub mod entities;
pub mod value_objects;
pub mod aggregates;
pub mod services;
pub mod repositories;

// Re-export main types for convenience
pub use entities::{Agent, AgentWorkflow, ExecutionContext, ExecutionResult, TriggerType, WorkflowAction};
pub use value_objects::{ExecutionPriority, ExecutionStrategy, ResourceQuota, AgentVersion};
pub use aggregates::{AgentAggregate, QueuedExecution, ExecutionRecord};
pub use services::{AgentManagementService, AgentExecutionService, AgentOrchestrationService};
pub use repositories::{AgentRepository, AgentWorkflowRepository, ExecutionRepository};