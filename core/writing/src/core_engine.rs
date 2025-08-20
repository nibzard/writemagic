//! Core engine for WriteMagic - orchestrates all domains and repositories

use std::sync::Arc;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use writemagic_shared::{DatabaseManager, DatabaseConfig, Result, WritemagicError, EventBus, InMemoryEventBus, CrossDomainServiceRegistry, CrossDomainCoordinator, EntityId};

#[cfg(target_arch = "wasm32")]
use writemagic_shared::{Result, WritemagicError, EventBus, InMemoryEventBus, CrossDomainServiceRegistry, CrossDomainCoordinator, EntityId};
use crate::repositories::{DocumentRepository, ProjectRepository};
use crate::{InMemoryDocumentRepository, InMemoryProjectRepository};
#[cfg(feature = "database")]
use crate::{SqliteDocumentRepository, SqliteProjectRepository};
use crate::services::{DocumentManagementService, ProjectManagementService, ContentAnalysisService};
#[cfg(feature = "ai")]
use crate::ai_writing_integration::{IntegratedWritingService, IntegratedWritingServiceBuilder};

// Import IndexedDB repositories for WASM builds
#[cfg(target_arch = "wasm32")]
use crate::web_persistence::{IndexedDbManager, IndexedDbConfig, IndexedDbDocumentRepository, IndexedDbProjectRepository, check_indexeddb_support};

// Import AI components (conditional for WASM builds)
#[cfg(feature = "ai")]
use writemagic_ai::{
    AIOrchestrationService, 
    AIProviderRegistry, 
    ContextManagementService, 
    ContentFilteringService,
    AIWritingService,
};
use writemagic_agent::{TriggerType, ExecutionStrategy};

// Import domain services
// TODO: Add back when these modules are available
// use writemagic_project::services::{ProjectManagementService as ProjectDomainService, ProjectTemplateService, ProjectAnalyticsService};
// use writemagic_version_control::services::{VersionControlService, DiffService, TimelineService};
// use writemagic_agent::services::{AgentManagementService, AgentExecutionService, AgentWorkflowService, AgentOrchestrationService};

// Import cross-domain coordination
// TODO: Add back cross-domain services when available
// use writemagic_shared::{
//     CrossDomainServiceRegistry, CrossDomainCoordinator, InMemoryEventBus, EventBus, 
//     WritingDomainService, AIDomainService, ProjectDomainService as ProjectService,
//     VersionControlDomainService, AgentDomainService as AgentService
// };

// Import repositories
// TODO: Add back when these modules are available
// use writemagic_project::repositories::{ProjectRepository as ProjectDomainRepository, ProjectRepositoryFactory as ProjectFactory};
// use writemagic_version_control::repositories::{VersionControlRepository, RepositoryFactory as VcFactory};
// use writemagic_agent::repositories::{AgentRepository, AgentRepositoryFactory};

/// Application configuration for the entire WriteMagic stack
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    #[cfg(feature = "ai")]
    pub ai: AIConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
}

/// Storage configuration for different platforms
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageConfig {
    pub storage_type: StorageType,
    #[cfg(not(target_arch = "wasm32"))]
    pub database_config: Option<DatabaseConfig>,
    #[cfg(target_arch = "wasm32")]
    pub indexeddb_config: Option<IndexedDbConfig>,
}

/// Storage backend types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StorageType {
    InMemory,
    SQLite,
    #[cfg(target_arch = "wasm32")]
    IndexedDB,
}

/// AI provider configuration
#[cfg(feature = "ai")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIConfig {
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub default_model: String,
    pub max_context_length: usize,
    pub enable_content_filtering: bool,
    pub cache_ttl_seconds: u64,
}

#[cfg(feature = "ai")]
impl Default for AIConfig {
    fn default() -> Self {
        Self {
            claude_api_key: None,
            openai_api_key: None,
            default_model: "gpt-4".to_string(),
            max_context_length: 32000,
            enable_content_filtering: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_tracing: bool,
}

/// Security configuration  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityConfig {
    pub encrypt_at_rest: bool,
    pub api_rate_limit_per_hour: u32,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let storage = StorageConfig {
            storage_type: StorageType::IndexedDB,
            database_config: None,
            indexeddb_config: Some(IndexedDbConfig::default()),
        };
        
        #[cfg(not(target_arch = "wasm32"))]
        let storage = StorageConfig {
            storage_type: StorageType::SQLite,
            #[cfg(not(target_arch = "wasm32"))]
            database_config: Some(DatabaseConfig::default()),
        };
        
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            database: DatabaseConfig::default(), // For backwards compatibility
            storage,
            #[cfg(feature = "ai")]
            #[cfg(feature = "ai")]
            ai: AIConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                storage_type: StorageType::IndexedDB,
                database_config: None,
                indexeddb_config: Some(IndexedDbConfig::default()),
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                storage_type: StorageType::SQLite,
                #[cfg(not(target_arch = "wasm32"))]
            database_config: Some(DatabaseConfig::default()),
            }
        }
    }
}


impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_tracing: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encrypt_at_rest: true,
            api_rate_limit_per_hour: 1000,
        }
    }
}

/// Core engine configuration (legacy, for backwards compatibility)
#[derive(Debug, Clone)]
pub struct CoreEngineConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub database_config: Option<DatabaseConfig>,
    pub use_in_memory: bool,
}

impl Default for CoreEngineConfig {
    fn default() -> Self {
        Self {
            database_config: None,
            use_in_memory: false,
        }
    }
}

impl CoreEngineConfig {
    /// Create config for in-memory storage (for testing)
    pub fn in_memory() -> Self {
        Self {
            database_config: None,
            use_in_memory: true,
        }
    }

    /// Create config for SQLite with default settings
    pub fn sqlite() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            database_config: Some(DatabaseConfig::default()),
            use_in_memory: false,
        }
    }

    /// Create config for SQLite with custom settings
    #[cfg(not(target_arch = "wasm32"))]
    pub fn sqlite_with_config(config: DatabaseConfig) -> Self {
        Self {
            database_config: Some(config),
            use_in_memory: false,
        }
    }

    /// Create config for SQLite in-memory database (for testing)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn sqlite_in_memory() -> Self {
        Self {
            database_config: Some(DatabaseConfig {
                database_url: "sqlite::memory:".to_string(),
                max_connections: 1,
                min_connections: 1,
                enable_wal: false,
                enable_foreign_keys: true,
            }),
            use_in_memory: false,
        }
    }
}

/// Enhanced Core engine that orchestrates all domains and services
pub struct CoreEngine {
    // Configuration
    config: ApplicationConfig,
    
    // Database manager (if using SQLite)
    #[cfg(not(target_arch = "wasm32"))]
    database_manager: Option<DatabaseManager>,
    
    // IndexedDB manager (if using IndexedDB)
    #[cfg(target_arch = "wasm32")]
    indexeddb_manager: Option<Arc<tokio::sync::Mutex<IndexedDbManager>>>,
    
    // Repository implementations - Writing domain
    document_repository: Arc<dyn DocumentRepository>,
    project_repository: Arc<dyn ProjectRepository>,
    
    // TODO: Uncomment when dependencies are available
    // // Repository implementations - New domains
    // project_domain_repository: Arc<dyn ProjectDomainRepository>,
    // version_control_repository: Arc<dyn VersionControlRepository>,
    // agent_repository: Arc<dyn AgentRepository>,
    
    // AI services (conditional)
    #[cfg(feature = "ai")]
    ai_orchestration_service: Option<AIOrchestrationService>,
    #[cfg(feature = "ai")]
    context_management_service: ContextManagementService,
    #[cfg(feature = "ai")]
    content_filtering_service: Option<ContentFilteringService>,
    #[cfg(feature = "ai")]
    ai_writing_service: Option<AIWritingService>,
    
    // Writing domain services
    document_management_service: Arc<DocumentManagementService>,
    project_management_service: Arc<ProjectManagementService>,
    content_analysis_service: Arc<ContentAnalysisService>,
    #[cfg(feature = "ai")]
    integrated_writing_service: Option<Arc<IntegratedWritingService>>,
    
    // TODO: Uncomment when dependencies are available
    // // New domain services
    // project_domain_service: Arc<ProjectDomainService>,
    // project_template_service: Arc<ProjectTemplateService>,
    // project_analytics_service: Arc<ProjectAnalyticsService>,
    // version_control_service: Arc<VersionControlService>,
    // diff_service: Arc<DiffService>,
    // timeline_service: Arc<TimelineService>,
    // agent_management_service: Arc<AgentManagementService>,
    // agent_execution_service: Arc<AgentExecutionService>,
    // agent_workflow_service: Arc<AgentWorkflowService>,
    // agent_orchestration_service: Arc<AgentOrchestrationService>,
    // 
    // // Cross-domain coordination
    // event_bus: Arc<dyn EventBus>,
    // service_registry: Arc<CrossDomainServiceRegistry>,
    // cross_domain_coordinator: Arc<CrossDomainCoordinator>,
    
    // Runtime for async operations
    tokio_runtime: Arc<tokio::runtime::Runtime>,
}

impl CoreEngine {
    /// Initialize the enhanced core engine with full application configuration
    pub async fn new_with_config(config: ApplicationConfig) -> Result<Self> {
        log::info!("Initializing WriteMagic CoreEngine with full configuration");
        
        // Create tokio runtime
        let tokio_runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .map_err(|e| WritemagicError::internal(format!("Failed to create tokio runtime: {}", e)))?
        );

        // Initialize storage based on configuration
        let (database_manager, document_repository, project_repository) = match config.storage.storage_type {
            StorageType::InMemory => {
                log::info!("Using in-memory storage");
                (
                    None,
                    Arc::new(InMemoryDocumentRepository::new()) as Arc<dyn DocumentRepository>,
                    Arc::new(InMemoryProjectRepository::new()) as Arc<dyn ProjectRepository>,
                )
            },
            StorageType::SQLite => {
                let db_config = config.storage.database_config.as_ref()
                    .unwrap_or(&config.database);
                    
                if db_config.database_url == "sqlite::memory:" {
                    log::info!("Using SQLite in-memory storage");
                    (
                        None,
                        Arc::new(InMemoryDocumentRepository::new()) as Arc<dyn DocumentRepository>,
                        Arc::new(InMemoryProjectRepository::new()) as Arc<dyn ProjectRepository>,
                    )
                } else {
                    log::info!("Using SQLite storage at: {}", db_config.database_url);
                    let database_manager = DatabaseManager::new(db_config.clone()).await?;
                    let pool = database_manager.pool().clone();
                    #[cfg(feature = "database")]
                    {
                        (
                            Some(database_manager),
                            Arc::new(SqliteDocumentRepository::new(pool.clone())) as Arc<dyn DocumentRepository>,
                            Arc::new(SqliteProjectRepository::new(pool)) as Arc<dyn ProjectRepository>,
                        )
                    }
                    #[cfg(not(feature = "database"))]
                    {
                        let _ = pool; // Avoid unused variable warning
                        (
                            Some(database_manager),
                            Arc::new(InMemoryDocumentRepository::new()) as Arc<dyn DocumentRepository>,
                            Arc::new(InMemoryProjectRepository::new()) as Arc<dyn ProjectRepository>,
                        )
                    }
                }
            },
            #[cfg(target_arch = "wasm32")]
            StorageType::IndexedDB => {
                return Err(WritemagicError::configuration(
                    "IndexedDB initialization should be handled separately in WASM environment"
                ));
            },
        };

        // Initialize AI services
        #[cfg(feature = "ai")]
        let (mut ai_orchestration_service, mut content_filtering_service) = Self::initialize_ai_services(&config.ai).await?;
        
        // Initialize context management service
        #[cfg(feature = "ai")]
        let context_management_service = ContextManagementService::new(config.ai.max_context_length.try_into().unwrap())?;

        // Create Arc for context management service
        #[cfg(feature = "ai")]
        let context_arc = Arc::new(context_management_service);
        
        // Initialize AI writing service if AI orchestration is available  
        #[cfg(feature = "ai")]
        let ai_writing_service = if ai_orchestration_service.is_some() && content_filtering_service.is_some() {
            // Take ownership of the services
            let orchestration_arc = Arc::new(ai_orchestration_service.take().unwrap());
            let filter_arc = Arc::new(content_filtering_service.take().unwrap());
            
            Some(AIWritingService::new(orchestration_arc, context_arc.clone(), filter_arc))
        } else {
            None
        };

        // Initialize domain services
        let document_management_service = Arc::new(DocumentManagementService::new(
            document_repository.clone()
        ));
        let project_management_service = Arc::new(ProjectManagementService::new(
            project_repository.clone(),
            document_repository.clone(),
        ));
        let content_analysis_service = Arc::new(ContentAnalysisService::new());
        
        // TODO: Initialize additional domain services when implemented
        // These services will be added in future phases when their dependencies are available

        // Initialize integrated writing service if AI is available
        #[cfg(feature = "ai")]
        let integrated_writing_service = if let Some(ai_writing) = &ai_writing_service {
            let integrated = IntegratedWritingServiceBuilder::new()
                .with_ai_writing_service(Arc::new(ai_writing.clone()))
                .with_document_service(document_management_service.clone())
                .with_project_service(project_management_service.clone())
                .with_content_analysis_service(content_analysis_service.clone())
                .with_document_repository(document_repository.clone())
                .with_project_repository(project_repository.clone())
                .build()?;
            Some(Arc::new(integrated))
        } else {
            None
        };
        
        // TODO: Initialize cross-domain coordination when dependencies are available
        // let event_bus = Arc::new(InMemoryEventBus::new()) as Arc<dyn EventBus>;
        // let mut service_registry = CrossDomainServiceRegistry::new(event_bus.clone());
        // let service_registry = Arc::new(service_registry);
        // let cross_domain_coordinator = Arc::new(CrossDomainCoordinator::new(service_registry.clone()));

        Ok(Self {
            config,
            database_manager,
            #[cfg(target_arch = "wasm32")]
            indexeddb_manager: None,
            document_repository,
            project_repository,
            #[cfg(feature = "ai")]
            ai_orchestration_service,
            #[cfg(feature = "ai")]
            context_management_service: (*context_arc).clone(),
            #[cfg(feature = "ai")]
            content_filtering_service,
            #[cfg(feature = "ai")]
            ai_writing_service,
            document_management_service,
            project_management_service,
            content_analysis_service,
            #[cfg(feature = "ai")]
            integrated_writing_service,
            tokio_runtime,
        })
    }

    /// Initialize AI services based on configuration
    #[cfg(feature = "ai")]
    async fn initialize_ai_services(ai_config: &AIConfig) -> Result<(Option<AIOrchestrationService>, Option<ContentFilteringService>)> {
        let mut ai_service = None;
        let mut content_filter = None;

        // Initialize AI orchestration if any API keys are provided
        if ai_config.claude_api_key.is_some() || ai_config.openai_api_key.is_some() {
            log::info!("Initializing AI orchestration service");
            
            let registry = AIProviderRegistry::new();
            
            if let Some(claude_key) = &ai_config.claude_api_key {
                registry.add_claude_key(claude_key.clone())?;
                log::info!("Claude provider configured");
            }
            
            if let Some(openai_key) = &ai_config.openai_api_key {
                registry.add_openai_key(openai_key.clone())?;
                log::info!("OpenAI provider configured");
            }
            
            ai_service = Some(registry.create_orchestration_service().await?);
        } else {
            log::warn!("No AI API keys configured - AI features will be disabled");
        }

        // Initialize content filtering if enabled
        if ai_config.enable_content_filtering {
            content_filter = Some(ContentFilteringService::new()?);
            log::info!("Content filtering service enabled");
        }

        Ok((ai_service, content_filter))
    }

    /// Initialize the core engine with legacy configuration (backwards compatibility)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(config: CoreEngineConfig) -> Result<Self> {
        let app_config = ApplicationConfig {
            database: config.database_config.unwrap_or_else(|| {
                if config.use_in_memory {
                    DatabaseConfig {
                        database_url: "sqlite::memory:".to_string(),
                        max_connections: 1,
                        min_connections: 1,
                        enable_wal: false,
                        enable_foreign_keys: true,
                    }
                } else {
                    DatabaseConfig::default()
                }
            }),
            storage: if config.use_in_memory {
                StorageConfig {
                    storage_type: StorageType::InMemory,
                    database_config: None,
                    #[cfg(target_arch = "wasm32")]
                    indexeddb_config: None,
                }
            } else {
                StorageConfig::default()
            },
            #[cfg(feature = "ai")]
            ai: AIConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
        };
        
        Self::new_with_config(app_config).await
    }

    /// Create engine with default SQLite configuration
    pub async fn new_default() -> Result<Self> {
        Self::new(CoreEngineConfig::sqlite()).await
    }

    /// Initialize the core engine with default configuration (web handler compatibility)
    pub async fn initialize() -> Result<Self> {
        Self::new_default().await
    }

    /// Create engine with in-memory storage for testing
    pub async fn new_in_memory() -> Result<Self> {
        Self::new(CoreEngineConfig::in_memory()).await
    }

    /// Create engine with SQLite in-memory database for testing
    pub async fn new_sqlite_in_memory() -> Result<Self> {
        Self::new(CoreEngineConfig::sqlite_in_memory()).await
    }

    /// Create engine with full configuration including AI providers
    #[cfg(feature = "ai")]
    pub async fn new_with_ai(claude_key: Option<String>, openai_key: Option<String>) -> Result<Self> {
        let mut ai_config = AIConfig::default();
        ai_config.claude_api_key = claude_key;
        ai_config.openai_api_key = openai_key;
        
        let app_config = ApplicationConfig {
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
            ai: ai_config,
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
        };
        
        Self::new_with_config(app_config).await
    }

    /// Create engine with in-memory storage and AI providers  
    #[cfg(feature = "ai")]
    pub async fn new_in_memory_with_ai(claude_key: Option<String>, openai_key: Option<String>) -> Result<Self> {
        let mut ai_config = AIConfig::default();
        ai_config.claude_api_key = claude_key;
        ai_config.openai_api_key = openai_key;
        
        let app_config = ApplicationConfig {
            database: DatabaseConfig {
                database_url: "sqlite::memory:".to_string(),
                max_connections: 1,
                min_connections: 1,
                enable_wal: false,
                enable_foreign_keys: true,
            },
            storage: StorageConfig {
                storage_type: StorageType::InMemory,
                database_config: None,
                #[cfg(target_arch = "wasm32")]
                indexeddb_config: None,
            },
            ai: ai_config,
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
        };
        
        Self::new_with_config(app_config).await
    }

    /// Create engine with IndexedDB storage for WASM environment
    #[cfg(target_arch = "wasm32")]
    pub async fn new_with_indexeddb(config: ApplicationConfig) -> Result<Self> {
        log::info!("Initializing WriteMagic CoreEngine with IndexedDB");
        
        // Check IndexedDB support
        check_indexeddb_support()
            .map_err(|e| WritemagicError::configuration(&format!("IndexedDB not supported: {:?}", e)))?;
        
        // Create tokio runtime
        let tokio_runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .map_err(|e| WritemagicError::internal(format!("Failed to create tokio runtime: {}", e)))?
        );
        
        // Initialize IndexedDB
        let indexeddb_config = config.storage.indexeddb_config.as_ref()
            .cloned()
            .unwrap_or_default();
        
        let mut indexeddb_manager = IndexedDbManager::new(indexeddb_config);
        indexeddb_manager.initialize().await
            .map_err(|e| WritemagicError::database(&format!("IndexedDB initialization failed: {:?}", e)))?;
        
        let indexeddb_manager = Arc::new(tokio::sync::Mutex::new(indexeddb_manager));
        
        // Create IndexedDB repositories
        let document_repository = Arc::new(IndexedDbDocumentRepository::new(indexeddb_manager.clone())) as Arc<dyn DocumentRepository>;
        let project_repository = Arc::new(IndexedDbProjectRepository::new(indexeddb_manager.clone())) as Arc<dyn ProjectRepository>;
        
        // Create new domain repositories (using factory pattern for cross-platform compatibility)
        let project_domain_repository = ProjectFactory::create_repository();
        let version_control_repository = VcFactory::create_repository();
        let agent_repository = AgentRepositoryFactory::create_agent_repository();
        
        log::info!("IndexedDB repositories initialized");
        
        // Initialize AI services
        #[cfg(feature = "ai")]
        let (mut ai_orchestration_service, mut content_filtering_service) = Self::initialize_ai_services(&config.ai).await?;
        
        // Initialize context management service
        #[cfg(feature = "ai")]
        let context_management_service = ContextManagementService::new(config.ai.max_context_length.try_into().unwrap())?;
        
        // Initialize AI writing service if AI orchestration is available
        #[cfg(feature = "ai")]
        let ai_writing_service = if ai_orchestration_service.is_some() && content_filtering_service.is_some() {
            // Take ownership of the services
            let orchestration_arc = Arc::new(ai_orchestration_service.take().unwrap());
            let context_arc = Arc::new(context_management_service);
            let filter_arc = Arc::new(content_filtering_service.take().unwrap());
            
            Some(AIWritingService::new(orchestration_arc, context_arc, filter_arc))
        } else {
            None
        };
        
        #[cfg(not(feature = "ai"))]
        let ai_writing_service = None;
        
        // Initialize domain services
        let document_management_service = Arc::new(DocumentManagementService::new(
            document_repository.clone()
        ));
        let project_management_service = Arc::new(ProjectManagementService::new(
            project_repository.clone(),
            document_repository.clone(),
        ));
        let content_analysis_service = Arc::new(ContentAnalysisService::new());
        
        // TODO: Initialize additional domain services when implemented
        // These services will be added in future phases when their dependencies are available
        
        // Initialize integrated writing service if AI is available
        #[cfg(feature = "ai")]
        let integrated_writing_service = if let Some(ai_writing) = &ai_writing_service {
            let integrated = IntegratedWritingServiceBuilder::new()
                .with_ai_writing_service(Arc::new(ai_writing.clone()))
                .with_document_service(document_management_service.clone())
                .with_project_service(project_management_service.clone())
                .with_content_analysis_service(content_analysis_service.clone())
                .with_document_repository(document_repository.clone())
                .with_project_repository(project_repository.clone())
                .build()?;
            Some(Arc::new(integrated))
        } else {
            None
        };
        
        // Initialize cross-domain coordination for IndexedDB constructor
        let event_bus = Arc::new(InMemoryEventBus::new()) as Arc<dyn EventBus>;
        let mut service_registry = CrossDomainServiceRegistry::new(event_bus.clone());
        
        // Register domain service adapters - these would need to be implemented
        // For now, we'll create the structure without the actual adapters
        let service_registry = Arc::new(service_registry);
        let cross_domain_coordinator = Arc::new(CrossDomainCoordinator::new(service_registry.clone()));
        
        Ok(Self {
            config,
            database_manager: None,
            indexeddb_manager: Some(indexeddb_manager),
            document_repository,
            project_repository,
            #[cfg(feature = "ai")]
            ai_orchestration_service,
            #[cfg(feature = "ai")]
            context_management_service: (*context_arc).clone(),
            #[cfg(feature = "ai")]
            content_filtering_service,
            #[cfg(feature = "ai")]
            ai_writing_service,
            document_management_service,
            project_management_service,
            content_analysis_service,
            #[cfg(feature = "ai")]
            integrated_writing_service,
            tokio_runtime,
        })
    }
    
    /// Create engine with default IndexedDB configuration for WASM
    #[cfg(target_arch = "wasm32")]
    pub async fn new_indexeddb_default() -> Result<Self> {
        let config = ApplicationConfig::default(); // Uses IndexedDB by default in WASM
        Self::new_with_indexeddb(config).await
    }
    
    /// Create engine with IndexedDB and AI providers for WASM
    #[cfg(all(target_arch = "wasm32", feature = "ai"))]
    pub async fn new_indexeddb_with_ai(claude_key: Option<String>, openai_key: Option<String>) -> Result<Self> {
        let mut ai_config = AIConfig::default();
        ai_config.claude_api_key = claude_key;
        ai_config.openai_api_key = openai_key;
        
        let mut app_config = ApplicationConfig::default(); // Uses IndexedDB by default in WASM
        app_config.ai = ai_config;
        
        Self::new_with_indexeddb(app_config).await
    }

    // Repository access methods
    /// Get document repository
    pub fn document_repository(&self) -> Arc<dyn DocumentRepository> {
        Arc::clone(&self.document_repository)
    }

    /// Get project repository
    pub fn project_repository(&self) -> Arc<dyn ProjectRepository> {
        Arc::clone(&self.project_repository)
    }

    // Database access methods
    /// Get database manager (if using SQLite)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn database_manager(&self) -> Option<&DatabaseManager> {
        self.database_manager.as_ref()
    }
    
    /// Get IndexedDB manager (if using IndexedDB)
    #[cfg(target_arch = "wasm32")]
    pub fn indexeddb_manager(&self) -> Option<Arc<tokio::sync::Mutex<IndexedDbManager>>> {
        self.indexeddb_manager.as_ref().cloned()
    }

    /// Check if the engine is using in-memory storage
    pub fn is_in_memory(&self) -> bool {
        matches!(self.config.storage.storage_type, StorageType::InMemory)
    }
    
    /// Check if the engine is using IndexedDB storage
    #[cfg(target_arch = "wasm32")]
    pub fn is_indexeddb(&self) -> bool {
        matches!(self.config.storage.storage_type, StorageType::IndexedDB)
    }
    
    /// Check if the engine is using SQLite storage
    pub fn is_sqlite(&self) -> bool {
        matches!(self.config.storage.storage_type, StorageType::SQLite)
    }

    // AI service access methods
    #[cfg(feature = "ai")]
    /// Get AI orchestration service
    pub fn ai_orchestration_service(&self) -> Option<&AIOrchestrationService> {
        self.ai_orchestration_service.as_ref()
    }

    #[cfg(feature = "ai")]
    /// Get context management service
    pub fn context_management_service(&self) -> &ContextManagementService {
        &self.context_management_service
    }

    #[cfg(feature = "ai")]
    /// Get content filtering service
    pub fn content_filtering_service(&self) -> Option<&ContentFilteringService> {
        self.content_filtering_service.as_ref()
    }

    #[cfg(feature = "ai")]
    /// Get AI writing service
    pub fn ai_writing_service(&self) -> Option<&AIWritingService> {
        self.ai_writing_service.as_ref()
    }

    // Domain service access methods
    /// Get document management service
    pub fn document_management_service(&self) -> Arc<DocumentManagementService> {
        self.document_management_service.clone()
    }

    /// Get project management service
    pub fn project_management_service(&self) -> Arc<ProjectManagementService> {
        self.project_management_service.clone()
    }

    /// Get content analysis service
    pub fn content_analysis_service(&self) -> Arc<ContentAnalysisService> {
        self.content_analysis_service.clone()
    }


    /// Get integrated writing service
    #[cfg(feature = "ai")]
    pub fn integrated_writing_service(&self) -> Option<Arc<IntegratedWritingService>> {
        self.integrated_writing_service.clone()
    }

    // New domain service access methods
    /// Get project domain service (advanced project management)
    // TODO: Uncomment when dependencies are available
    // pub fn project_domain_service(&self) -> Arc<ProjectDomainService> {
    //     self.project_domain_service.clone()
    // }

    // TODO: Uncomment when dependencies are available
    // /// Get project template service
    // pub fn project_template_service(&self) -> Arc<ProjectTemplateService> {
    //     self.project_template_service.clone()
    // }

    // /// Get project analytics service
    // pub fn project_analytics_service(&self) -> Arc<ProjectAnalyticsService> {
    //     self.project_analytics_service.clone()
    // }

    // TODO: Uncomment when dependencies are available
    // /// Get version control service
    // pub fn version_control_service(&self) -> Arc<VersionControlService> {
    //     self.version_control_service.clone()
    // }

    // TODO: Uncomment when dependencies are available
    // /// Get diff service
    // pub fn diff_service(&self) -> Arc<DiffService> {
    //     self.diff_service.clone()
    // }

    // /// Get timeline service
    // pub fn timeline_service(&self) -> Arc<TimelineService> {
    //     self.timeline_service.clone()
    // }

    // /// Get agent management service
    // pub fn agent_management_service(&self) -> Arc<AgentManagementService> {
    //     self.agent_management_service.clone()
    // }

    // /// Get agent execution service
    // pub fn agent_execution_service(&self) -> Arc<AgentExecutionService> {
    //     self.agent_execution_service.clone()
    // }

    // /// Get agent workflow service
    // pub fn agent_workflow_service(&self) -> Arc<AgentWorkflowService> {
    //     self.agent_workflow_service.clone()
    // }

    // /// Get agent orchestration service
    // pub fn agent_orchestration_service(&self) -> Arc<AgentOrchestrationService> {
    //     self.agent_orchestration_service.clone()
    // }

    // // New domain repository access methods
    // /// Get project domain repository
    // pub fn project_domain_repository(&self) -> Arc<dyn ProjectDomainRepository> {
    //     self.project_domain_repository.clone()
    // }

    // /// Get version control repository
    // pub fn version_control_repository(&self) -> Arc<dyn VersionControlRepository> {
    //     self.version_control_repository.clone()
    // }

    // /// Get agent repository
    // pub fn agent_repository(&self) -> Arc<dyn AgentRepository> {
    //     self.agent_repository.clone()
    // }

    // // Configuration access methods
    // /// Get event bus
    // pub fn event_bus(&self) -> Arc<dyn EventBus> {
    //     self.event_bus.clone()
    // }

    // /// Get cross-domain service registry
    // pub fn service_registry(&self) -> Arc<CrossDomainServiceRegistry> {
    //     self.service_registry.clone()
    // }

    // /// Get cross-domain coordinator
    // pub fn cross_domain_coordinator(&self) -> Arc<CrossDomainCoordinator> {
    //     self.cross_domain_coordinator.clone()
    // }

    /// Get application configuration
    pub fn config(&self) -> &ApplicationConfig {
        &self.config
    }

    /// Get tokio runtime
    pub fn runtime(&self) -> &Arc<tokio::runtime::Runtime> {
        &self.tokio_runtime
    }

    // AI integration methods
    /// Complete text using AI with automatic provider fallback
    #[cfg(feature = "ai")]
    pub async fn complete_text(&self, prompt: String, model: Option<String>) -> Result<String> {
        match &self.ai_orchestration_service {
            Some(ai_service) => {
                // Apply content filtering if enabled
                let filtered_prompt = if let Some(filter) = &self.content_filtering_service {
                    filter.filter_content(&prompt)?
                } else {
                    prompt
                };

                // Create completion request
                let model = model.unwrap_or_else(|| self.config.ai.default_model.clone());
                let messages = vec![
                    writemagic_ai::Message::user(filtered_prompt)
                ];

                let request = writemagic_ai::CompletionRequest::new(messages, model)
                    .with_max_tokens(1000)
                    .with_temperature(0.7);

                // Get completion with fallback
                let response = ai_service.complete_with_fallback(request).await?;
                
                if let Some(choice) = response.choices.first() {
                    Ok(choice.message.content.clone())
                } else {
                    Err(WritemagicError::ai_provider("No completion choices returned"))
                }
            }
            None => Err(WritemagicError::configuration("AI services not configured"))
        }
    }

    /// Check AI provider health status
    #[cfg(feature = "ai")]
    pub async fn check_ai_provider_health(&self) -> Result<HashMap<String, bool>> {
        match &self.ai_orchestration_service {
            Some(ai_service) => {
                ai_service.health_check_all_providers().await
            }
            None => Ok(HashMap::new())
        }
    }

    /// Get AI provider statistics
    #[cfg(feature = "ai")]
    pub async fn get_ai_provider_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        match &self.ai_orchestration_service {
            Some(ai_service) => {
                let health = ai_service.get_provider_health().await;
                let stats = health.into_iter().map(|(name, health)| {
                    let stat_value = serde_json::json!({
                        "isHealthy": health.is_healthy,
                        "consecutiveFailures": health.consecutive_failures,
                        "avgResponseTimeMs": health.avg_response_time.as_millis(),
                        "lastSuccess": health.last_success.map(|t| t.elapsed().as_secs()),
                        "lastFailure": health.last_failure.map(|t| t.elapsed().as_secs())
                    });
                    (name, stat_value)
                }).collect();
                Ok(stats)
            }
            None => Ok(HashMap::new())
        }
    }

    /// Get migration status (if using SQLite)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_migration_status(&self) -> Result<Option<Vec<writemagic_shared::MigrationStatus>>> {
        if let Some(db_manager) = &self.database_manager {
            Ok(Some(db_manager.get_migration_status().await?))
        } else {
            Ok(None)
        }
    }

    /// Graceful shutdown of the core engine
    pub async fn shutdown(self) {
        log::info!("Shutting down WriteMagic CoreEngine");
        
        // Shutdown database connections
        if let Some(db_manager) = self.database_manager {
            log::info!("Closing database connections");
            db_manager.close().await;
        }
        
        // Shutdown tokio runtime (this happens automatically when dropped)
        log::info!("CoreEngine shutdown completed");
    }
    
    /// Initialize logging based on configuration
    pub fn init_logging(&self) -> Result<()> {
        let level = match self.config.logging.level.as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info,
        };

        #[cfg(target_os = "android")]
        {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(level)
                    .with_tag("WriteMagic"),
            );
        }

        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            env_logger::Builder::from_default_env()
                .filter_level(level)
                .init();
        }

        log::info!("Logging initialized with level: {}", self.config.logging.level);
        
        if self.config.logging.enable_tracing {
            log::info!("Tracing is enabled");
            // TODO: Initialize tracing subscriber
        }

        Ok(())
    }
    
    /// Validate configuration and return any issues
    pub fn validate_config(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Validate AI configuration
        #[cfg(feature = "ai")]
        if self.config.ai.claude_api_key.is_none() && self.config.ai.openai_api_key.is_none() {
            issues.push("No AI API keys configured - AI features will be disabled".to_string());
        }
        
        // Validate database configuration
        if !self.config.database.database_url.starts_with("sqlite:") {
            issues.push("Unsupported database type - only SQLite is currently supported".to_string());
        }
        
        // Validate security settings
        if !self.config.security.encrypt_at_rest && self.config.database.database_url != "sqlite::memory:" {
            issues.push("Encryption at rest is disabled for persistent storage".to_string());
        }
        
        issues
    }
}

/// Enhanced application builder for comprehensive configuration
pub struct ApplicationConfigBuilder {
    config: ApplicationConfig,
}

impl ApplicationConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ApplicationConfig::default(),
        }
    }

    /// Set database configuration
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_database_config(mut self, database_config: DatabaseConfig) -> Self {
        self.config.database = database_config;
        self
    }

    /// Use SQLite with default settings
    pub fn with_sqlite(mut self) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        { self.config.database = DatabaseConfig::default(); }
        self
    }

    /// Use SQLite in-memory database
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_sqlite_in_memory(mut self) -> Self {
        self.config.database = DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            enable_wal: false,
            enable_foreign_keys: true,
        };
        self
    }

    /// Set AI configuration
    #[cfg(feature = "ai")]
    pub fn with_ai_config(mut self, ai_config: AIConfig) -> Self {
        self.config.ai = ai_config;
        self
    }

    /// Configure Claude API key
    #[cfg(feature = "ai")]
    pub fn with_claude_key(mut self, api_key: String) -> Self {
        self.config.ai.claude_api_key = Some(api_key);
        self
    }

    /// Configure OpenAI API key  
    #[cfg(feature = "ai")]
    pub fn with_openai_key(mut self, api_key: String) -> Self {
        self.config.ai.openai_api_key = Some(api_key);
        self
    }

    /// Set default AI model
    #[cfg(feature = "ai")]
    pub fn with_default_model(mut self, model: String) -> Self {
        self.config.ai.default_model = model;
        self
    }

    /// Set maximum context length for AI
    #[cfg(feature = "ai")]
    pub fn with_max_context_length(mut self, length: usize) -> Self {
        self.config.ai.max_context_length = length;
        self
    }

    /// Enable or disable content filtering
    #[cfg(feature = "ai")]
    pub fn with_content_filtering(mut self, enabled: bool) -> Self {
        self.config.ai.enable_content_filtering = enabled;
        self
    }

    /// Set logging level
    pub fn with_log_level(mut self, level: String) -> Self {
        self.config.logging.level = level;
        self
    }

    /// Enable or disable tracing
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.config.logging.enable_tracing = enabled;
        self
    }

    /// Set security configuration
    pub fn with_security_config(mut self, security_config: SecurityConfig) -> Self {
        self.config.security = security_config;
        self
    }

    /// Enable or disable encryption at rest
    pub fn with_encryption_at_rest(mut self, enabled: bool) -> Self {
        self.config.security.encrypt_at_rest = enabled;
        self
    }

    /// Set API rate limit per hour
    pub fn with_api_rate_limit(mut self, limit: u32) -> Self {
        self.config.security.api_rate_limit_per_hour = limit;
        self
    }

    /// Build the core engine
    pub async fn build(self) -> Result<CoreEngine> {
        CoreEngine::new_with_config(self.config).await
    }

    /// Get the configuration (for validation before building)
    pub fn config(&self) -> &ApplicationConfig {
        &self.config
    }
}

impl Default for ApplicationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Core engine builder for backwards compatibility
pub struct CoreEngineBuilder {
    config: CoreEngineConfig,
}

impl CoreEngineBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: CoreEngineConfig::default(),
        }
    }

    /// Use in-memory storage
    pub fn with_in_memory(mut self) -> Self {
        self.config.use_in_memory = true;
        self.config.database_config = None;
        self
    }

    /// Use SQLite with default configuration
    pub fn with_sqlite(mut self) -> Self {
        self.config.use_in_memory = false;
        #[cfg(not(target_arch = "wasm32"))]
        { self.config.database_config = Some(DatabaseConfig::default()); }
        self
    }

    /// Use SQLite with custom configuration
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_sqlite_config(mut self, config: DatabaseConfig) -> Self {
        self.config.use_in_memory = false;
        self.config.database_config = Some(config);
        self
    }

    /// Use SQLite in-memory database
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_sqlite_in_memory(mut self) -> Self {
        self.config.use_in_memory = false;
        self.config.database_config = Some(DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            enable_wal: false,
            enable_foreign_keys: true,
        });
        self
    }

    /// Build the core engine
    pub async fn build(self) -> Result<CoreEngine> {
        CoreEngine::new(self.config).await
    }
}

impl Default for CoreEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// WASM-specific bindings for the core engine
#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::future_to_promise;
    use js_sys::Promise;
    use serde_wasm_bindgen::{to_value, from_value};
    use std::sync::Arc;
    use super::*;
    
    /// WASM wrapper for the CoreEngine
    #[wasm_bindgen]
    pub struct WasmCoreEngine {
        engine: Arc<CoreEngine>,
    }
    
    #[wasm_bindgen]
    impl WasmCoreEngine {
        /// Initialize the core engine for WASM
        #[wasm_bindgen(constructor)]
        pub fn new() -> Promise {
            future_to_promise(async move {
                match CoreEngine::new_indexeddb_default().await {
                    Ok(engine) => {
                        let wasm_engine = WasmCoreEngine {
                            engine: Arc::new(engine),
                        };
                        Ok(JsValue::from(wasm_engine))
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to initialize engine: {}", e))),
                }
            })
        }
        
        /// Create a new document
        #[wasm_bindgen(js_name = createDocument)]
        pub fn create_document(&self, title: &str, content: &str) -> Promise {
            let engine = self.engine.clone();
            let title = title.to_string();
            let content = content.to_string();
            
            future_to_promise(async move {
                match engine.create_document(title, content, None).await {
                    Ok(document) => {
                        match to_value(&document) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to create document: {}", e))),
                }
            })
        }
        
        /// Get a document by ID
        #[wasm_bindgen(js_name = getDocument)]
        pub fn get_document(&self, document_id: &str) -> Promise {
            let engine = self.engine.clone();
            let document_id = document_id.to_string();
            
            future_to_promise(async move {
                // Parse the document ID (in a real implementation, you'd have proper ID parsing)
                let id = match EntityId::new_from_string(&document_id) {
                    Ok(id) => id,
                    Err(e) => return Err(JsValue::from_str(&format!("Invalid document ID: {}", e))),
                };
                
                match engine.get_document(&id).await {
                    Ok(Some(document)) => {
                        match to_value(&document) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Ok(None) => Ok(JsValue::NULL),
                    Err(e) => Err(JsValue::from_str(&format!("Failed to get document: {}", e))),
                }
            })
        }
        
        /// Generate AI content
        #[wasm_bindgen(js_name = generateContent)]
        pub fn generate_content(&self, prompt: &str, model: Option<String>) -> Promise {
            let engine = self.engine.clone();
            let prompt = prompt.to_string();
            
            future_to_promise(async move {
                match engine.complete_text(prompt, model).await {
                    Ok(content) => Ok(JsValue::from_str(&content)),
                    Err(e) => Err(JsValue::from_str(&format!("Failed to generate content: {}", e))),
                }
            })
        }
        
        /// Create a project
        #[wasm_bindgen(js_name = createProject)]
        pub fn create_project(&self, name: &str, description: Option<String>) -> Promise {
            let engine = self.engine.clone();
            let name = name.to_string();
            
            future_to_promise(async move {
                match engine.create_project(name, description, None).await {
                    Ok(project) => {
                        match to_value(&project) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to create project: {}", e))),
                }
            })
        }
        
        /// Trigger an agent execution
        #[wasm_bindgen(js_name = triggerAgent)]
        pub fn trigger_agent(&self, agent_id: &str, context: JsValue) -> Promise {
            let engine = self.engine.clone();
            let agent_id = agent_id.to_string();
            
            future_to_promise(async move {
                // Parse the agent ID
                let id = match EntityId::new_from_string(&agent_id) {
                    Ok(id) => id,
                    Err(e) => return Err(JsValue::from_str(&format!("Invalid agent ID: {}", e))),
                };
                
                // Parse the context from JavaScript
                let context_map: std::collections::HashMap<String, String> = match from_value(context) {
                    Ok(map) => map,
                    Err(e) => return Err(JsValue::from_str(&format!("Invalid context: {}", e))),
                };
                
                // Use the agent orchestration service
                let agent_service = engine.agent_orchestration_service();
                
                // Convert the context to the required format for agent execution
                // This is a simplified conversion - in practice you'd have more sophisticated context handling
                // TODO: Agent integration - placeholder for future implementation
                let trigger_type = TriggerType::Manual;
                let strategy = ExecutionStrategy::Immediate;
                let context_json: std::collections::BTreeMap<String, serde_json::Value> = context_map
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                
                match agent_service.smart_trigger(&id, trigger_type, context_json, strategy).await {
                    Ok(result) => {
                        match to_value(&result) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to trigger agent: {}", e))),
                }
            })
        }
        
        /// Get system status
        #[wasm_bindgen(js_name = getSystemStatus)]
        pub fn get_system_status(&self) -> Promise {
            let engine = self.engine.clone();
            
            future_to_promise(async move {
                // Collect status from various services
                let agent_service = engine.agent_orchestration_service();
                
                match agent_service.get_comprehensive_status().await {
                    Ok(status) => {
                        match to_value(&status) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to get system status: {}", e))),
                }
            })
        }
        
        /// Access cross-domain coordination features
        #[wasm_bindgen(js_name = crossDomainCoordination)]
        pub fn cross_domain_coordination(&self) -> WasmCrossDomainCoordinator {
            WasmCrossDomainCoordinator {
                coordinator: self.engine.cross_domain_coordinator(),
                engine: self.engine.clone(),
            }
        }
    }
    
    /// WASM wrapper for cross-domain coordination
    #[wasm_bindgen]
    pub struct WasmCrossDomainCoordinator {
        coordinator: Arc<CrossDomainCoordinator>,
        engine: Arc<CoreEngine>,
    }
    
    #[wasm_bindgen]
    impl WasmCrossDomainCoordinator {
        /// Create document in project with AI generation
        #[wasm_bindgen(js_name = generateAndSaveDocument)]
        pub fn generate_and_save_document(&self, prompt: &str, project_id: Option<String>) -> Promise {
            let coordinator = self.coordinator.clone();
            let prompt = prompt.to_string();
            
            future_to_promise(async move {
                // Create AI generation request
                let request = writemagic_shared::services::AIGenerationRequest {
                    prompt,
                    max_tokens: Some(2000),
                    temperature: Some(0.7),
                    context: None,
                    style: None,
                };
                
                // Parse project ID if provided
                let project_entity_id = if let Some(project_id_str) = project_id {
                    match EntityId::new_from_string(&project_id_str) {
                        Ok(id) => Some(id),
                        Err(e) => return Err(JsValue::from_str(&format!("Invalid project ID: {}", e))),
                    }
                } else {
                    None
                };
                
                match coordinator.generate_and_save_document(request, project_entity_id.as_ref()).await {
                    Ok(document) => {
                        match to_value(&document) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to generate and save document: {}", e))),
                }
            })
        }
        
        /// Create analyzed commit
        #[wasm_bindgen(js_name = createAnalyzedCommit)]
        pub fn create_analyzed_commit(&self, document_id: &str, message: &str) -> Promise {
            let coordinator = self.coordinator.clone();
            let document_id = document_id.to_string();
            let message = message.to_string();
            
            future_to_promise(async move {
                // Parse document ID
                let id = match EntityId::new_from_string(&document_id) {
                    Ok(id) => id,
                    Err(e) => return Err(JsValue::from_str(&format!("Invalid document ID: {}", e))),
                };
                
                match coordinator.create_analyzed_commit(&id, message).await {
                    Ok(commit) => {
                        match to_value(&commit) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                        }
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to create analyzed commit: {}", e))),
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writemagic_shared::Pagination;
    use crate::entities::{Document, Project};
    use writemagic_shared::{ContentType, EntityId};

    #[tokio::test]
    async fn test_core_engine_in_memory() {
        let engine = CoreEngine::new_in_memory().await.unwrap();
        assert!(engine.is_in_memory());
        assert!(engine.database_manager().is_none());
    }

    #[tokio::test]
    async fn test_core_engine_sqlite_in_memory() {
        let engine = CoreEngine::new_sqlite_in_memory().await.unwrap();
        assert!(!engine.is_in_memory());
        assert!(engine.database_manager().is_some());
    }

    #[tokio::test]
    async fn test_document_operations_in_memory() {
        let engine = CoreEngine::new_in_memory().await.unwrap();
        let repo = engine.document_repository();

        // Create a document
        let doc = Document::new(
            "Test Document".to_string(),
            "Test content".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        let saved_doc = repo.save(&doc).await.unwrap();
        assert_eq!(saved_doc.id, doc.id);
        assert_eq!(saved_doc.title, "Test Document");

        // Find by ID
        let found_doc = repo.find_by_id(&doc.id).await.unwrap();
        assert!(found_doc.is_some());
        assert_eq!(found_doc.unwrap().title, "Test Document");

        // Count documents
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_document_operations_sqlite() {
        let engine = CoreEngine::new_sqlite_in_memory().await.unwrap();
        let repo = engine.document_repository();

        // Create a document
        let doc = Document::new(
            "Test Document".to_string(),
            "Test content".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        
        let saved_doc = repo.save(&doc).await.unwrap();
        assert_eq!(saved_doc.id, doc.id);
        assert_eq!(saved_doc.title, "Test Document");

        // Find by ID
        let found_doc = repo.find_by_id(&doc.id).await.unwrap();
        assert!(found_doc.is_some());
        assert_eq!(found_doc.unwrap().title, "Test Document");

        // Count documents
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_project_operations_sqlite() {
        let engine = CoreEngine::new_sqlite_in_memory().await.unwrap();
        let project_repo = engine.project_repository();
        let doc_repo = engine.document_repository();

        // Create a document first
        let doc = Document::new(
            "Test Document".to_string(),
            "Test content".to_string(),
            ContentType::Markdown,
            Some(EntityId::new()),
        );
        doc_repo.save(&doc).await.unwrap();

        // Create a project
        let mut project = Project::new(
            "Test Project".to_string(),
            Some("Test description".to_string()),
            Some(EntityId::new()),
        );
        project.add_document(doc.id, None);

        let saved_project = project_repo.save(&project).await.unwrap();
        assert_eq!(saved_project.id, project.id);
        assert_eq!(saved_project.name, "Test Project");
        assert_eq!(saved_project.document_ids.len(), 1);
        assert_eq!(saved_project.document_ids[0], doc.id);

        // Find by ID
        let found_project = project_repo.find_by_id(&project.id).await.unwrap();
        assert!(found_project.is_some());
        let found = found_project.unwrap();
        assert_eq!(found.name, "Test Project");
        assert_eq!(found.document_ids.len(), 1);
        assert_eq!(found.document_ids[0], doc.id);
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let engine = CoreEngineBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await
            .unwrap();

        assert!(!engine.is_in_memory());
        assert!(engine.database_manager().is_some());
    }

    #[tokio::test]
    async fn test_application_config_builder() {
        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .with_claude_key("test-key".to_string())
            .with_log_level("debug".to_string())
            .with_content_filtering(true)
            .build()
            .await
            .unwrap();

        assert!(!engine.is_in_memory());
        assert!(engine.database_manager().is_some());
        assert_eq!(engine.config().logging.level, "debug");
        assert!(engine.config().ai.enable_content_filtering);
        assert!(engine.config().ai.claude_api_key.is_some());
        assert!(engine.content_filtering_service().is_some());
    }

    #[tokio::test]
    async fn test_ai_integration_without_keys() {
        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .build()
            .await
            .unwrap();

        // AI service should not be available without API keys
        assert!(engine.ai_orchestration_service().is_none());
        
        // Test AI completion without keys (should fail)
        let result = engine.complete_text("Test prompt".to_string(), None).await;
        assert!(result.is_err());
        
        // Health check should return empty map
        let health = engine.check_ai_provider_health().await.unwrap();
        assert!(health.is_empty());
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let engine = ApplicationConfigBuilder::new()
            .with_sqlite_in_memory()
            .with_encryption_at_rest(false)
            .build()
            .await
            .unwrap();

        let issues = engine.validate_config();
        assert!(issues.contains(&"No AI API keys configured - AI features will be disabled".to_string()));
    }

    #[tokio::test]
    async fn test_context_management_service() {
        let engine = CoreEngine::new_in_memory().await.unwrap();
        let context_service = engine.context_management_service();
        
        // Test context management
        let messages = vec![
            writemagic_ai::Message::system("System message"),
            writemagic_ai::Message::user("User message"),
        ];
        
        let managed_context = context_service.manage_context(messages.clone(), "test-model").expect("Failed to manage context");
        assert_eq!(managed_context.len(), 2);
    }

    #[tokio::test]
    async fn test_provider_stats() {
        let engine = CoreEngine::new_in_memory().await.unwrap();
        
        // Should return empty stats when no AI service is configured
        let stats = engine.get_ai_provider_stats().await.unwrap();
        assert!(stats.is_empty());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let engine = CoreEngine::new_sqlite_in_memory().await.unwrap();
        assert!(engine.database_manager().is_some());
        
        // Test that shutdown completes without panicking
        engine.shutdown().await;
    }
}