//! Unit tests for shared services and service container

use crate::{
    CrossDomainServiceRegistry, CrossDomainCoordinator, ServiceContainer, 
    ServiceRef, ProviderRegistry, WritingDomainService, AIDomainService,
    ProjectDomainService, EntityId, Result, WritemagicError
};
use std::sync::{Arc, Mutex};
use std::any::Any;

// Mock service implementations for testing
struct MockWritingService {
    pub operations: Arc<Mutex<Vec<String>>>,
}

impl MockWritingService {
    fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn record_operation(&self, operation: &str) {
        self.operations.lock().unwrap().push(operation.to_string());
    }
    
    fn get_operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }
}

impl WritingDomainService for MockWritingService {
    fn create_document(&self, title: String, content: String) -> Result<EntityId> {
        self.record_operation(&format!("create_document: {} - {}", title, content));
        Ok(EntityId::new())
    }
    
    fn update_document(&self, id: EntityId, content: String) -> Result<()> {
        self.record_operation(&format!("update_document: {} - {}", id, content));
        Ok(())
    }
    
    fn delete_document(&self, id: EntityId) -> Result<()> {
        self.record_operation(&format!("delete_document: {}", id));
        Ok(())
    }
}

struct MockAIService {
    pub operations: Arc<Mutex<Vec<String>>>,
}

impl MockAIService {
    fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn record_operation(&self, operation: &str) {
        self.operations.lock().unwrap().push(operation.to_string());
    }
    
    fn get_operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }
}

// Note: AIDomainService implementation temporarily removed due to missing type definitions
// This will be re-added when the service types are properly defined

struct MockProjectService {
    pub operations: Arc<Mutex<Vec<String>>>,
}

impl MockProjectService {
    fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn record_operation(&self, operation: &str) {
        self.operations.lock().unwrap().push(operation.to_string());
    }
    
    fn get_operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }
}

impl ProjectDomainService for MockProjectService {
    fn create_project(&self, name: String, description: Option<String>) -> Result<EntityId> {
        self.record_operation(&format!("create_project: {} - {:?}", name, description));
        Ok(EntityId::new())
    }
    
    fn add_document_to_project(&self, project_id: EntityId, document_id: EntityId) -> Result<()> {
        self.record_operation(&format!("add_document_to_project: {} -> {}", project_id, document_id));
        Ok(())
    }
    
    fn remove_document_from_project(&self, project_id: EntityId, document_id: EntityId) -> Result<()> {
        self.record_operation(&format!("remove_document_from_project: {} -> {}", project_id, document_id));
        Ok(())
    }
}

#[cfg(test)]
mod cross_domain_service_registry_tests {
    use super::*;

    #[test]
    fn test_service_registry_creation() {
        let registry = CrossDomainServiceRegistry::new();
        
        // Registry should start empty
        assert!(registry.get_writing_service().is_none());
        assert!(registry.get_ai_service().is_none());
        assert!(registry.get_project_service().is_none());
    }

    #[test]
    fn test_service_registration() {
        let mut registry = CrossDomainServiceRegistry::new();
        let writing_service = Arc::new(MockWritingService::new());
        let ai_service = Arc::new(MockAIService::new());
        let project_service = Arc::new(MockProjectService::new());
        
        registry.register_writing_service(Arc::clone(&writing_service));
        registry.register_ai_service(Arc::clone(&ai_service));
        registry.register_project_service(Arc::clone(&project_service));
        
        // Services should now be available
        assert!(registry.get_writing_service().is_some());
        assert!(registry.get_ai_service().is_some());
        assert!(registry.get_project_service().is_some());
    }

    #[test]
    fn test_service_usage() {
        let mut registry = CrossDomainServiceRegistry::new();
        let writing_service = Arc::new(MockWritingService::new());
        
        registry.register_writing_service(Arc::clone(&writing_service));
        
        let retrieved_service = registry.get_writing_service().expect("Writing service should be available");
        let document_id = retrieved_service.create_document("Test Title".to_string(), "Test Content".to_string())
            .expect("Create document should succeed");
        
        let operations = writing_service.get_operations();
        assert_eq!(operations.len(), 1);
        assert!(operations[0].contains("create_document"));
        assert!(operations[0].contains("Test Title"));
    }
}

#[cfg(test)]
mod cross_domain_coordinator_tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let registry = Arc::new(CrossDomainServiceRegistry::new());
        let coordinator = CrossDomainCoordinator::new(registry);
        
        // Coordinator should be created successfully
        assert!(coordinator.is_ok());
    }

    #[tokio::test]
    async fn test_document_creation_workflow() {
        let mut registry = CrossDomainServiceRegistry::new();
        let writing_service = Arc::new(MockWritingService::new());
        let ai_service = Arc::new(MockAIService::new());
        
        registry.register_writing_service(Arc::clone(&writing_service));
        registry.register_ai_service(Arc::clone(&ai_service));
        
        let coordinator = CrossDomainCoordinator::new(Arc::new(registry))
            .expect("Coordinator creation should succeed");
        
        // Simulate a cross-domain workflow
        let result = coordinator.create_document_with_ai_assistance(
            "Document Title".to_string(),
            "Initial content".to_string(),
        ).await;
        
        assert!(result.is_ok());
        
        // Check that both services were called
        let writing_ops = writing_service.get_operations();
        let ai_ops = ai_service.get_operations();
        
        assert!(!writing_ops.is_empty());
        assert!(!ai_ops.is_empty());
    }
}

// Simple test service trait and implementation
trait TestService: Send + Sync {
    fn get_name(&self) -> &str;
    fn perform_operation(&self, input: &str) -> String;
}

struct ConcreteTestService {
    name: String,
}

impl ConcreteTestService {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl TestService for ConcreteTestService {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn perform_operation(&self, input: &str) -> String {
        format!("{}: {}", self.name, input)
    }
}

#[cfg(test)]
mod service_container_tests {
    use super::*;

    #[test]
    fn test_service_container_registration() {
        let mut container = ServiceContainer::new();
        let service = Arc::new(ConcreteTestService::new("TestService".to_string()));
        
        container.register("test_service", Arc::clone(&service) as Arc<dyn Any + Send + Sync>);
        
        // Service should be retrievable
        let retrieved = container.resolve::<Arc<ConcreteTestService>>("test_service");
        assert!(retrieved.is_some());
        
        let retrieved_service = retrieved.unwrap();
        assert_eq!(retrieved_service.get_name(), "TestService");
    }

    #[test]
    fn test_service_container_missing_service() {
        let container = ServiceContainer::new();
        
        let result = container.resolve::<Arc<ConcreteTestService>>("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_service_container_type_mismatch() {
        let mut container = ServiceContainer::new();
        let service = Arc::new(ConcreteTestService::new("TestService".to_string()));
        
        container.register("test_service", service as Arc<dyn Any + Send + Sync>);
        
        // Try to resolve as wrong type
        let result = container.resolve::<Arc<MockWritingService>>("test_service");
        assert!(result.is_none());
    }
}

#[cfg(test)]
mod provider_registry_tests {
    use super::*;

    #[test]
    fn test_provider_registry_creation() {
        let registry = ProviderRegistry::new();
        
        // Registry should start empty
        assert!(registry.get_provider("test_provider").is_none());
    }

    #[test]
    fn test_provider_registration() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(ConcreteTestService::new("TestProvider".to_string()));
        
        registry.register("test_provider", Arc::clone(&provider) as Arc<dyn Any + Send + Sync>);
        
        // Provider should be retrievable
        let retrieved = registry.get_provider("test_provider");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_provider_factory() {
        let mut registry = ProviderRegistry::new();
        
        registry.register_factory("dynamic_provider", Box::new(|| {
            Arc::new(ConcreteTestService::new("DynamicProvider".to_string())) as Arc<dyn Any + Send + Sync>
        }));
        
        let provider1 = registry.get_provider("dynamic_provider");
        let provider2 = registry.get_provider("dynamic_provider");
        
        assert!(provider1.is_some());
        assert!(provider2.is_some());
        
        // Should create different instances each time
        let p1_ptr = Arc::as_ptr(&provider1.unwrap());
        let p2_ptr = Arc::as_ptr(&provider2.unwrap());
        assert_ne!(p1_ptr, p2_ptr);
    }
}

#[cfg(test)]
mod service_ref_tests {
    use super::*;

    #[test]
    fn test_service_ref_creation() {
        let service = Arc::new(ConcreteTestService::new("RefTest".to_string()));
        let service_ref = ServiceRef::new(Arc::clone(&service) as Arc<dyn Any + Send + Sync>);
        
        // Should be able to downcast back to original type
        let downcast_result = service_ref.downcast::<ConcreteTestService>();
        assert!(downcast_result.is_some());
        
        let downcast_service = downcast_result.unwrap();
        assert_eq!(downcast_service.get_name(), "RefTest");
    }

    #[test]
    fn test_service_ref_wrong_type() {
        let service = Arc::new(ConcreteTestService::new("RefTest".to_string()));
        let service_ref = ServiceRef::new(service as Arc<dyn Any + Send + Sync>);
        
        // Should fail to downcast to wrong type
        let downcast_result = service_ref.downcast::<MockWritingService>();
        assert!(downcast_result.is_none());
    }
}

// Additional integration test for service interactions
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_service_interaction() {
        // Set up all services
        let mut registry = CrossDomainServiceRegistry::new();
        let writing_service = Arc::new(MockWritingService::new());
        let ai_service = Arc::new(MockAIService::new());
        let project_service = Arc::new(MockProjectService::new());
        
        registry.register_writing_service(Arc::clone(&writing_service));
        registry.register_ai_service(Arc::clone(&ai_service));
        registry.register_project_service(Arc::clone(&project_service));
        
        let coordinator = CrossDomainCoordinator::new(Arc::new(registry))
            .expect("Coordinator creation should succeed");
        
        // Execute a complex workflow
        let project_id = coordinator.create_project("Test Project".to_string(), Some("A test project".to_string()))
            .expect("Create project should succeed");
        
        let document_id = coordinator.create_document_with_ai_assistance(
            "AI Generated Document".to_string(),
            "Please write about testing".to_string(),
        ).await.expect("Create document with AI should succeed");
        
        coordinator.add_document_to_project(project_id, document_id)
            .expect("Add document to project should succeed");
        
        // Verify all services were called appropriately
        let writing_ops = writing_service.get_operations();
        let ai_ops = ai_service.get_operations();
        let project_ops = project_service.get_operations();
        
        assert!(!writing_ops.is_empty());
        assert!(!ai_ops.is_empty());
        assert!(!project_ops.is_empty());
        
        // Verify specific operations
        assert!(project_ops.iter().any(|op| op.contains("create_project")));
        assert!(writing_ops.iter().any(|op| op.contains("create_document")));
        assert!(ai_ops.iter().any(|op| op.contains("complete_text") || op.contains("analyze_content")));
        assert!(project_ops.iter().any(|op| op.contains("add_document_to_project")));
    }
}