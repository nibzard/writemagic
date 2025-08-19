/**
 * Progressive Loading Manager for WriteMagic
 * 
 * Provides intelligent preloading strategies, detailed progress indicators,
 * and user behavior-based optimization for the WriteMagic web application.
 * 
 * Features:
 * - Intelligent preloading based on user behavior patterns
 * - Detailed progress indicators with phase-based loading
 * - Network-aware loading strategies with fallback mechanisms
 * - Service Worker integration for cache optimization
 * - Real-time performance monitoring and optimization
 * - User preference learning and adaptation
 */

import { EventEmitter } from './event-emitter.js';
import WasmLoader from './wasm-loader.js';
import ServiceWorkerManager from './service-worker-manager.js';

/**
 * User behavior patterns for intelligent preloading
 */
const BEHAVIOR_PATTERNS = {
  // Common feature usage sequences
  WRITING_FLOW: [
    'document_create',
    'document_edit', 
    'ai_suggestions',
    'document_save'
  ],
  
  PROJECT_FLOW: [
    'project_create',
    'document_create',
    'document_edit',
    'project_organize'
  ],
  
  ANALYSIS_FLOW: [
    'document_edit',
    'writing_analytics',
    'content_analysis',
    'improvement_suggestions'
  ],
  
  COLLABORATION_FLOW: [
    'project_share',
    'version_control',
    'comment_system',
    'document_merge'
  ]
};

/**
 * Preloading priorities based on user context
 */
const PRELOAD_PRIORITIES = {
  IMMEDIATE: {
    weight: 1.0,
    timeout: 5000,
    critical: true
  },
  
  HIGH: {
    weight: 0.8,
    timeout: 10000,
    critical: false
  },
  
  MEDIUM: {
    weight: 0.6,
    timeout: 15000,
    critical: false
  },
  
  LOW: {
    weight: 0.4,
    timeout: 30000,
    critical: false
  },
  
  BACKGROUND: {
    weight: 0.2,
    timeout: 60000,
    critical: false
  }
};

/**
 * Loading phase configurations
 */
const LOADING_PHASES = {
  INITIALIZATION: {
    name: 'initialization',
    description: 'Initializing application core',
    weight: 0.1,
    estimatedTime: 500
  },
  
  CORE_ASSETS: {
    name: 'core_assets',
    description: 'Loading core application assets',
    weight: 0.3,
    estimatedTime: 2000
  },
  
  WASM_COMPILATION: {
    name: 'wasm_compilation',
    description: 'Compiling WebAssembly modules',
    weight: 0.4,
    estimatedTime: 3000
  },
  
  FEATURE_MODULES: {
    name: 'feature_modules',
    description: 'Loading feature-specific modules',
    weight: 0.15,
    estimatedTime: 1500
  },
  
  FINALIZATION: {
    name: 'finalization',
    description: 'Finalizing application setup',
    weight: 0.05,
    estimatedTime: 300
  }
};

/**
 * Progressive Loader with intelligent preloading and performance monitoring
 */
export class ProgressiveLoader extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.options = {
      enableIntelligentPreloading: true,
      enableUserBehaviorLearning: true,
      enablePerformanceOptimization: true,
      enableDetailedProgress: true,
      maxConcurrentRequests: 6,
      preloadThreshold: 0.7,
      adaptiveBehavior: true,
      ...options
    };
    
    // Core components
    this.wasmLoader = new WasmLoader({
      enableStreaming: true,
      enableProgressiveLoading: true,
      enableMemoryMonitoring: true
    });
    
    this.serviceWorkerManager = null;
    if (typeof ServiceWorkerManager !== 'undefined') {
      this.serviceWorkerManager = new ServiceWorkerManager();
    }
    
    // Loading state
    this.loadingState = {
      phase: null,
      overallProgress: 0,
      phaseProgress: 0,
      estimatedTimeRemaining: 0,
      currentTasks: [],
      completedTasks: [],
      failedTasks: [],
      startTime: null,
      isLoading: false,
      networkQuality: 'unknown'
    };
    
    // Performance metrics
    this.performanceMetrics = {
      loadTimes: new Map(),
      networkSpeeds: [],
      errorRates: new Map(),
      userInteractions: [],
      cacheHitRates: new Map(),
      memoryUsage: []
    };
    
    // User behavior learning
    this.behaviorLearning = {
      patterns: new Map(),
      preferences: new Map(),
      sessionData: {
        startTime: Date.now(),
        interactions: [],
        loadedResources: [],
        errors: []
      }
    };
    
    // Preloading queue
    this.preloadQueue = new Map();
    this.activePreloads = new Set();
    
    // Network monitoring
    this.networkMonitor = new NetworkQualityMonitor();
    
    // Initialize components
    this.initializeProgressiveLoader();
  }

  /**
   * Initialize the progressive loader
   */
  async initializeProgressiveLoader() {
    try {
      console.log('[ProgressiveLoader] Initializing progressive loading system...');
      
      // Load user preferences and behavior patterns
      await this.loadUserPreferences();
      
      // Initialize network monitoring
      await this.networkMonitor.initialize();
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Initialize Service Worker integration
      if (this.serviceWorkerManager) {
        await this.serviceWorkerManager.initialize();
      }
      
      console.log('[ProgressiveLoader] Progressive loading system initialized');
      this.emit('initialized');
      
    } catch (error) {
      console.error('[ProgressiveLoader] Initialization failed:', error);
      this.emit('initializationFailed', error);
    }
  }

  /**
   * Start progressive loading with intelligent preloading
   */
  async startProgressiveLoading(requiredFeatures = [], userContext = {}) {
    if (this.loadingState.isLoading) {
      console.warn('[ProgressiveLoader] Loading already in progress');
      return;
    }
    
    this.loadingState.isLoading = true;
    this.loadingState.startTime = performance.now();
    this.loadingState.phase = LOADING_PHASES.INITIALIZATION;
    
    console.log('[ProgressiveLoader] Starting progressive loading...', { requiredFeatures, userContext });
    this.emit('loadingStarted', { requiredFeatures, userContext, loadingState: this.loadingState });
    
    try {
      // Phase 1: Initialization
      await this.executePhase(LOADING_PHASES.INITIALIZATION, async () => {
        await this.initializeLoadingContext(requiredFeatures, userContext);
      });
      
      // Phase 2: Core Assets
      await this.executePhase(LOADING_PHASES.CORE_ASSETS, async () => {
        await this.loadCoreAssets();
      });
      
      // Phase 3: WASM Compilation (parallel with intelligent preloading)
      const wasmLoadingPromise = this.executePhase(LOADING_PHASES.WASM_COMPILATION, async () => {
        return await this.wasmLoader.loadModules(requiredFeatures);
      });
      
      // Start intelligent preloading in parallel
      const preloadingPromise = this.startIntelligentPreloading(userContext);
      
      // Wait for WASM loading to complete
      const wasmModules = await wasmLoadingPromise;
      
      // Phase 4: Feature Modules
      await this.executePhase(LOADING_PHASES.FEATURE_MODULES, async () => {
        await this.loadFeatureModules(requiredFeatures, userContext);
      });
      
      // Phase 5: Finalization
      await this.executePhase(LOADING_PHASES.FINALIZATION, async () => {
        await this.finalizeLoading(wasmModules);
      });
      
      // Wait for preloading to complete (or timeout)
      try {
        await Promise.race([
          preloadingPromise,
          new Promise(resolve => setTimeout(resolve, 10000)) // 10 second timeout
        ]);
      } catch (error) {
        console.warn('[ProgressiveLoader] Preloading timeout or error:', error);
      }
      
      const totalLoadTime = performance.now() - this.loadingState.startTime;
      
      console.log(`[ProgressiveLoader] Progressive loading completed in ${totalLoadTime.toFixed(2)}ms`);
      this.emit('loadingCompleted', {
        totalLoadTime,
        wasmModules,
        performanceMetrics: this.performanceMetrics,
        loadingState: this.loadingState
      });
      
      // Record performance metrics
      this.recordLoadingMetrics(totalLoadTime, requiredFeatures);
      
      // Learn from this loading session
      if (this.options.enableUserBehaviorLearning) {
        this.learnFromLoadingSession(requiredFeatures, userContext, totalLoadTime);
      }
      
      return {
        wasmModules,
        loadTime: totalLoadTime,
        performanceMetrics: this.performanceMetrics
      };
      
    } catch (error) {
      console.error('[ProgressiveLoader] Progressive loading failed:', error);
      this.emit('loadingFailed', error);
      throw error;
    } finally {
      this.loadingState.isLoading = false;
    }
  }

  /**
   * Execute a loading phase with progress tracking
   */
  async executePhase(phaseConfig, executor) {
    this.loadingState.phase = phaseConfig;
    this.loadingState.phaseProgress = 0;
    
    console.log(`[ProgressiveLoader] Starting phase: ${phaseConfig.name}`);
    this.emit('phaseStarted', { phase: phaseConfig, loadingState: this.loadingState });
    
    const phaseStartTime = performance.now();
    
    try {
      // Update progress periodically during phase
      const progressInterval = setInterval(() => {
        this.updatePhaseProgress(phaseConfig);
      }, 100);
      
      const result = await executor();
      
      clearInterval(progressInterval);
      
      const phaseEndTime = performance.now();
      const phaseLoadTime = phaseEndTime - phaseStartTime;
      
      // Complete phase
      this.loadingState.phaseProgress = 100;
      this.updateOverallProgress();
      
      console.log(`[ProgressiveLoader] Phase completed: ${phaseConfig.name} (${phaseLoadTime.toFixed(2)}ms)`);
      this.emit('phaseCompleted', { 
        phase: phaseConfig, 
        loadTime: phaseLoadTime,
        loadingState: this.loadingState
      });
      
      return result;
      
    } catch (error) {
      console.error(`[ProgressiveLoader] Phase failed: ${phaseConfig.name}`, error);
      this.emit('phaseFailed', { phase: phaseConfig, error });
      throw error;
    }
  }

  /**
   * Initialize loading context with user data
   */
  async initializeLoadingContext(requiredFeatures, userContext) {
    // Detect network quality
    this.loadingState.networkQuality = await this.networkMonitor.getNetworkQuality();
    
    // Load user behavior patterns
    if (this.options.enableUserBehaviorLearning) {
      await this.loadUserBehaviorPatterns(userContext);
    }
    
    // Prepare loading strategy based on network and user context
    this.optimizeLoadingStrategy(requiredFeatures, userContext);
    
    // Initialize progress tracking
    this.initializeProgressTracking(requiredFeatures);
  }

  /**
   * Load core application assets
   */
  async loadCoreAssets() {
    const coreAssets = [
      '/styles/critical.css',
      '/scripts/core.js',
      '/manifest.json'
    ];
    
    const loadPromises = coreAssets.map(async (asset, index) => {
      try {
        const response = await fetch(asset);
        if (!response.ok) {
          throw new Error(`Failed to load ${asset}: ${response.statusText}`);
        }
        
        this.updateTaskProgress(`core_asset_${index}`, 100);
        return { asset, success: true };
      } catch (error) {
        console.error(`[ProgressiveLoader] Failed to load core asset ${asset}:`, error);
        this.updateTaskProgress(`core_asset_${index}`, 0, error);
        return { asset, success: false, error };
      }
    });
    
    const results = await Promise.allSettled(loadPromises);
    const successful = results.filter(r => r.status === 'fulfilled' && r.value.success).length;
    
    console.log(`[ProgressiveLoader] Core assets loaded: ${successful}/${coreAssets.length}`);
    
    if (successful === 0) {
      throw new Error('Failed to load any core assets');
    }
  }

  /**
   * Load feature-specific modules
   */
  async loadFeatureModules(requiredFeatures, userContext) {
    // Determine which feature modules to load based on requirements and user patterns
    const featureModules = this.determineFeatureModules(requiredFeatures, userContext);
    
    console.log('[ProgressiveLoader] Loading feature modules:', featureModules);
    
    // Load modules with priority-based ordering
    const sortedModules = featureModules.sort((a, b) => b.priority - a.priority);
    
    for (const module of sortedModules) {
      try {
        await this.loadFeatureModule(module);
      } catch (error) {
        console.warn(`[ProgressiveLoader] Feature module ${module.name} failed to load:`, error);
        // Continue with other modules
      }
    }
  }

  /**
   * Load individual feature module
   */
  async loadFeatureModule(moduleConfig) {
    const { name, url, priority, optional } = moduleConfig;
    
    try {
      this.updateTaskProgress(`feature_${name}`, 0);
      
      if (url.endsWith('.js')) {
        await import(url);
      } else {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
      }
      
      this.updateTaskProgress(`feature_${name}`, 100);
      console.log(`[ProgressiveLoader] Feature module loaded: ${name}`);
      
    } catch (error) {
      this.updateTaskProgress(`feature_${name}`, 0, error);
      
      if (!optional) {
        throw error;
      }
      
      console.warn(`[ProgressiveLoader] Optional feature module failed: ${name}`, error);
    }
  }

  /**
   * Start intelligent preloading based on user behavior
   */
  async startIntelligentPreloading(userContext) {
    if (!this.options.enableIntelligentPreloading) {
      return;
    }
    
    console.log('[ProgressiveLoader] Starting intelligent preloading...');
    
    try {
      // Analyze user behavior patterns
      const predictedResources = this.predictResourceUsage(userContext);
      
      // Create preloading plan
      const preloadPlan = this.createPreloadPlan(predictedResources);
      
      // Execute preloading with priority queuing
      await this.executePreloadPlan(preloadPlan);
      
      console.log('[ProgressiveLoader] Intelligent preloading completed');
      this.emit('preloadingCompleted', { plan: preloadPlan });
      
    } catch (error) {
      console.warn('[ProgressiveLoader] Intelligent preloading failed:', error);
      this.emit('preloadingFailed', error);
    }
  }

  /**
   * Predict resource usage based on user behavior patterns
   */
  predictResourceUsage(userContext) {
    const predictions = new Map();
    
    // Analyze current context
    const timeOfDay = new Date().getHours();
    const dayOfWeek = new Date().getDay();
    const userType = userContext.userType || 'general';
    
    // Get historical patterns
    const userPatterns = this.behaviorLearning.patterns.get(userType) || new Map();
    
    // Predict based on common workflow patterns
    for (const [patternName, workflow] of Object.entries(BEHAVIOR_PATTERNS)) {
      const patternScore = this.calculatePatternScore(workflow, userContext, userPatterns);
      
      if (patternScore > this.options.preloadThreshold) {
        // Add resources from this workflow to predictions
        for (const step of workflow) {
          const resources = this.getResourcesForWorkflowStep(step);
          for (const resource of resources) {
            const currentScore = predictions.get(resource) || 0;
            predictions.set(resource, Math.max(currentScore, patternScore));
          }
        }
      }
    }
    
    // Factor in time-based patterns
    this.applyTimeBasedPredictions(predictions, timeOfDay, dayOfWeek);
    
    // Apply user-specific preferences
    this.applyUserPreferences(predictions, userContext);
    
    return predictions;
  }

  /**
   * Calculate pattern score based on user history and context
   */
  calculatePatternScore(workflow, userContext, userPatterns) {
    let baseScore = 0.5; // Default probability
    
    // Check recent user actions
    const recentActions = this.behaviorLearning.sessionData.interactions.slice(-10);
    const workflowMatches = workflow.filter(step => 
      recentActions.some(action => action.type === step)
    ).length;
    
    const sequenceScore = workflowMatches / workflow.length;
    
    // Get historical pattern strength
    const historicalScore = userPatterns.get(workflow.join('->')) || 0;
    
    // Combine scores with weighting
    const combinedScore = (
      sequenceScore * 0.4 +
      historicalScore * 0.4 +
      baseScore * 0.2
    );
    
    // Apply context modifiers
    let contextMultiplier = 1.0;
    
    if (userContext.currentProject) {
      contextMultiplier *= 1.2; // Higher chance of project-related workflows
    }
    
    if (userContext.collaborators && userContext.collaborators.length > 0) {
      contextMultiplier *= 1.1; // Higher chance of collaboration workflows
    }
    
    return Math.min(combinedScore * contextMultiplier, 1.0);
  }

  /**
   * Get resources needed for a specific workflow step
   */
  getResourcesForWorkflowStep(step) {
    const resourceMap = {
      'document_create': ['/api/templates', '/scripts/document-editor.js'],
      'document_edit': ['/scripts/editor-enhancements.js', '/scripts/auto-save.js'],
      'ai_suggestions': ['/scripts/ai-integration.js', '/api/ai/models'],
      'document_save': ['/api/documents', '/scripts/sync-manager.js'],
      'project_create': ['/scripts/project-manager.js', '/api/projects'],
      'project_organize': ['/scripts/file-tree.js', '/scripts/drag-drop.js'],
      'writing_analytics': ['/scripts/analytics.js', '/api/analytics'],
      'content_analysis': ['/scripts/content-analyzer.js'],
      'improvement_suggestions': ['/scripts/writing-assistant.js'],
      'project_share': ['/scripts/collaboration.js', '/api/sharing'],
      'version_control': ['/scripts/version-control.js', '/api/versions'],
      'comment_system': ['/scripts/comments.js', '/api/comments'],
      'document_merge': ['/scripts/merge-conflict-resolver.js']
    };
    
    return resourceMap[step] || [];
  }

  /**
   * Apply time-based prediction adjustments
   */
  applyTimeBasedPredictions(predictions, timeOfDay, dayOfWeek) {
    // Business hours (9-17) - higher chance of professional features
    if (timeOfDay >= 9 && timeOfDay <= 17 && dayOfWeek >= 1 && dayOfWeek <= 5) {
      this.boostPredictionScores(predictions, ['project_', 'collaboration_', 'analytics_'], 1.3);
    }
    
    // Evening hours - higher chance of personal writing
    if (timeOfDay >= 18 || timeOfDay <= 7) {
      this.boostPredictionScores(predictions, ['document_', 'ai_suggestions', 'writing_'], 1.2);
    }
    
    // Weekends - higher chance of creative writing features  
    if (dayOfWeek === 0 || dayOfWeek === 6) {
      this.boostPredictionScores(predictions, ['ai_', 'content_analysis'], 1.1);
    }
  }

  /**
   * Boost prediction scores for resources matching patterns
   */
  boostPredictionScores(predictions, patterns, multiplier) {
    for (const [resource, score] of predictions) {
      if (patterns.some(pattern => resource.includes(pattern))) {
        predictions.set(resource, Math.min(score * multiplier, 1.0));
      }
    }
  }

  /**
   * Apply user-specific preferences to predictions
   */
  applyUserPreferences(predictions, userContext) {
    const userPrefs = this.behaviorLearning.preferences;
    
    // Apply feature usage preferences
    for (const [feature, preference] of userPrefs) {
      const resources = this.getResourcesForWorkflowStep(feature);
      for (const resource of resources) {
        const currentScore = predictions.get(resource) || 0;
        predictions.set(resource, Math.min(currentScore * preference, 1.0));
      }
    }
    
    // Apply user-specific patterns
    if (userContext.userId) {
      const userPatterns = this.behaviorLearning.patterns.get(`user_${userContext.userId}`);
      if (userPatterns) {
        // Boost resources from user's common patterns
        for (const [pattern, frequency] of userPatterns) {
          const steps = pattern.split('->');
          for (const step of steps) {
            const resources = this.getResourcesForWorkflowStep(step);
            for (const resource of resources) {
              const currentScore = predictions.get(resource) || 0;
              predictions.set(resource, Math.min(currentScore + (frequency * 0.2), 1.0));
            }
          }
        }
      }
    }
  }

  /**
   * Create preload plan with priority ordering
   */
  createPreloadPlan(predictedResources) {
    const plan = {
      immediate: [],
      high: [],
      medium: [],
      low: [],
      background: []
    };
    
    // Sort resources by prediction score
    const sortedResources = Array.from(predictedResources.entries())
      .sort(([,a], [,b]) => b - a);
    
    // Categorize into priority buckets
    for (const [resource, score] of sortedResources) {
      if (score >= 0.9) {
        plan.immediate.push({ resource, score, priority: 'immediate' });
      } else if (score >= 0.7) {
        plan.high.push({ resource, score, priority: 'high' });
      } else if (score >= 0.5) {
        plan.medium.push({ resource, score, priority: 'medium' });
      } else if (score >= 0.3) {
        plan.low.push({ resource, score, priority: 'low' });
      } else {
        plan.background.push({ resource, score, priority: 'background' });
      }
    }
    
    return plan;
  }

  /**
   * Execute preload plan with priority queuing
   */
  async executePreloadPlan(plan) {
    const allTasks = [];
    
    // Process each priority level
    for (const [priority, resources] of Object.entries(plan)) {
      if (resources.length === 0) continue;
      
      const priorityConfig = PRELOAD_PRIORITIES[priority.toUpperCase()];
      
      for (const resourceConfig of resources) {
        const task = this.createPreloadTask(resourceConfig, priorityConfig);
        allTasks.push(task);
      }
    }
    
    // Execute tasks with concurrency control
    await this.executeConcurrentPreloads(allTasks);
  }

  /**
   * Create preload task with configuration
   */
  createPreloadTask(resourceConfig, priorityConfig) {
    return {
      ...resourceConfig,
      ...priorityConfig,
      id: `preload_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      startTime: null,
      endTime: null,
      success: false,
      error: null
    };
  }

  /**
   * Execute preload tasks with concurrency control
   */
  async executeConcurrentPreloads(tasks) {
    const activeTasks = [];
    const completedTasks = [];
    let taskIndex = 0;
    
    while (taskIndex < tasks.length || activeTasks.length > 0) {
      // Start new tasks up to concurrency limit
      while (activeTasks.length < this.options.maxConcurrentRequests && taskIndex < tasks.length) {
        const task = tasks[taskIndex++];
        const taskPromise = this.executePreloadTask(task);
        activeTasks.push(taskPromise);
      }
      
      // Wait for at least one task to complete
      if (activeTasks.length > 0) {
        const completedTask = await Promise.race(activeTasks);
        
        // Remove completed task from active tasks
        const completedIndex = activeTasks.findIndex(p => p === completedTask);
        if (completedIndex >= 0) {
          activeTasks.splice(completedIndex, 1);
        }
        
        completedTasks.push(completedTask);
      }
    }
    
    const successfulTasks = completedTasks.filter(task => task.success).length;
    console.log(`[ProgressiveLoader] Preloading completed: ${successfulTasks}/${tasks.length} successful`);
    
    return completedTasks;
  }

  /**
   * Execute individual preload task
   */
  async executePreloadTask(task) {
    task.startTime = performance.now();
    this.activePreloads.add(task.id);
    
    try {
      // Check if resource is already cached
      if (this.serviceWorkerManager) {
        const cached = await this.serviceWorkerManager.isCached(task.resource);
        if (cached) {
          task.success = true;
          task.cached = true;
          return task;
        }
      }
      
      // Preload the resource
      const response = await this.preloadResource(task.resource, task.timeout);
      
      if (response.ok) {
        task.success = true;
        this.emit('resourcePreloaded', { task, resource: task.resource });
      } else {
        task.error = new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
    } catch (error) {
      task.error = error;
      console.debug(`[ProgressiveLoader] Preload failed for ${task.resource}:`, error.message);
    } finally {
      task.endTime = performance.now();
      this.activePreloads.delete(task.id);
    }
    
    return task;
  }

  /**
   * Preload individual resource
   */
  async preloadResource(url, timeout = 10000) {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error('Preload timeout'));
      }, timeout);
      
      fetch(url, { 
        method: 'GET',
        cache: 'default',
        priority: 'low' // Use low priority to avoid blocking critical requests
      }).then(response => {
        clearTimeout(timeoutId);
        resolve(response);
      }).catch(error => {
        clearTimeout(timeoutId);
        reject(error);
      });
    });
  }

  /**
   * Finalize loading process
   */
  async finalizeLoading(wasmModules) {
    // Initialize final application state
    this.initializeFinalApplicationState(wasmModules);
    
    // Clean up temporary loading resources
    await this.cleanupLoadingResources();
    
    // Save session data for learning
    if (this.options.enableUserBehaviorLearning) {
      await this.saveSessionData();
    }
    
    // Update performance metrics
    this.updatePerformanceMetrics();
  }

  /**
   * Initialize final application state
   */
  initializeFinalApplicationState(wasmModules) {
    // Register global WASM modules
    if (typeof window !== 'undefined') {
      window.WriteMagicWasm = wasmModules;
      
      // Add utility methods
      window.WriteMagicLoader = {
        getMetrics: () => this.performanceMetrics,
        getLoadingState: () => this.loadingState,
        getBehaviorData: () => this.behaviorLearning,
        preloadResources: (urls) => this.preloadResourceList(urls),
        clearCache: () => this.clearApplicationCache()
      };
    }
  }

  /**
   * Clean up temporary loading resources
   */
  async cleanupLoadingResources() {
    try {
      // Clear loading state
      this.loadingState.currentTasks = [];
      
      // Clean up preload queue
      this.preloadQueue.clear();
      this.activePreloads.clear();
      
      // Trigger garbage collection if available
      if (typeof window !== 'undefined' && window.gc) {
        window.gc();
      }
      
    } catch (error) {
      console.warn('[ProgressiveLoader] Cleanup failed:', error);
    }
  }

  /**
   * Update progress tracking
   */
  updatePhaseProgress(phaseConfig) {
    // Simulate progress based on elapsed time and estimated duration
    const elapsed = performance.now() - this.loadingState.startTime;
    const phaseElapsed = elapsed - this.getPreviousPhasesTime();
    const estimatedPhaseTime = phaseConfig.estimatedTime;
    
    this.loadingState.phaseProgress = Math.min(
      (phaseElapsed / estimatedPhaseTime) * 100, 
      95 // Never show 100% until actually complete
    );
    
    this.updateOverallProgress();
    this.updateTimeRemaining();
    
    this.emit('progressUpdated', { loadingState: this.loadingState });
  }

  /**
   * Update overall progress based on phase weights
   */
  updateOverallProgress() {
    const phases = Object.values(LOADING_PHASES);
    const currentPhaseIndex = phases.findIndex(p => p.name === this.loadingState.phase?.name);
    
    if (currentPhaseIndex === -1) return;
    
    // Calculate progress from completed phases
    let overallProgress = 0;
    
    for (let i = 0; i < currentPhaseIndex; i++) {
      overallProgress += phases[i].weight * 100;
    }
    
    // Add current phase progress
    const currentPhaseWeight = phases[currentPhaseIndex].weight;
    overallProgress += (this.loadingState.phaseProgress / 100) * currentPhaseWeight * 100;
    
    this.loadingState.overallProgress = Math.min(overallProgress, 100);
  }

  /**
   * Update estimated time remaining
   */
  updateTimeRemaining() {
    if (!this.loadingState.startTime) return;
    
    const elapsed = performance.now() - this.loadingState.startTime;
    const progress = this.loadingState.overallProgress / 100;
    
    if (progress > 0) {
      const estimatedTotal = elapsed / progress;
      this.loadingState.estimatedTimeRemaining = Math.max(0, estimatedTotal - elapsed);
    }
  }

  /**
   * Get previous phases total time
   */
  getPreviousPhasesTime() {
    const phases = Object.values(LOADING_PHASES);
    const currentPhaseIndex = phases.findIndex(p => p.name === this.loadingState.phase?.name);
    
    if (currentPhaseIndex === -1) return 0;
    
    let previousTime = 0;
    for (let i = 0; i < currentPhaseIndex; i++) {
      previousTime += phases[i].estimatedTime;
    }
    
    return previousTime;
  }

  /**
   * Update task progress
   */
  updateTaskProgress(taskId, progress, error = null) {
    const existingTask = this.loadingState.currentTasks.find(t => t.id === taskId);
    
    if (existingTask) {
      existingTask.progress = progress;
      existingTask.error = error;
      
      if (progress === 100) {
        // Move to completed
        const taskIndex = this.loadingState.currentTasks.findIndex(t => t.id === taskId);
        if (taskIndex >= 0) {
          const completedTask = this.loadingState.currentTasks.splice(taskIndex, 1)[0];
          this.loadingState.completedTasks.push(completedTask);
        }
      } else if (error) {
        // Move to failed
        const taskIndex = this.loadingState.currentTasks.findIndex(t => t.id === taskId);
        if (taskIndex >= 0) {
          const failedTask = this.loadingState.currentTasks.splice(taskIndex, 1)[0];
          this.loadingState.failedTasks.push(failedTask);
        }
      }
    } else {
      // Create new task
      const newTask = {
        id: taskId,
        progress,
        error,
        startTime: performance.now()
      };
      
      if (progress === 100) {
        this.loadingState.completedTasks.push(newTask);
      } else if (error) {
        this.loadingState.failedTasks.push(newTask);
      } else {
        this.loadingState.currentTasks.push(newTask);
      }
    }
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    // WASM Loader events
    this.wasmLoader.on('moduleLoadStarted', (data) => {
      this.emit('taskStarted', { type: 'wasm_module', ...data });
    });
    
    this.wasmLoader.on('moduleLoadCompleted', (data) => {
      this.emit('taskCompleted', { type: 'wasm_module', ...data });
    });
    
    this.wasmLoader.on('moduleLoadFailed', (data) => {
      this.emit('taskFailed', { type: 'wasm_module', ...data });
    });
    
    // Network Monitor events
    this.networkMonitor.on('qualityChanged', (quality) => {
      this.loadingState.networkQuality = quality;
      this.adaptToNetworkChange(quality);
    });
    
    // Service Worker events
    if (this.serviceWorkerManager) {
      this.serviceWorkerManager.on('cacheUpdated', (data) => {
        this.updateCacheMetrics(data);
      });
    }
  }

  /**
   * Adapt to network quality changes
   */
  adaptToNetworkChange(newQuality) {
    if (!this.options.adaptiveBehavior) return;
    
    console.log(`[ProgressiveLoader] Network quality changed to: ${newQuality}`);
    
    // Adjust concurrent request limits based on network quality
    switch (newQuality) {
      case 'slow':
        this.options.maxConcurrentRequests = Math.min(this.options.maxConcurrentRequests, 2);
        break;
      case 'moderate':
        this.options.maxConcurrentRequests = Math.min(this.options.maxConcurrentRequests, 4);
        break;
      case 'fast':
        this.options.maxConcurrentRequests = Math.max(this.options.maxConcurrentRequests, 6);
        break;
    }
    
    // Adjust preload aggressiveness
    if (newQuality === 'slow') {
      // Reduce preloading on slow connections
      this.options.preloadThreshold = Math.min(this.options.preloadThreshold + 0.2, 0.9);
    } else if (newQuality === 'fast') {
      // Increase preloading on fast connections
      this.options.preloadThreshold = Math.max(this.options.preloadThreshold - 0.1, 0.5);
    }
    
    this.emit('adaptedToNetwork', { quality: newQuality, options: this.options });
  }

  /**
   * Record loading performance metrics
   */
  recordLoadingMetrics(totalLoadTime, requiredFeatures) {
    const sessionId = Date.now().toString();
    
    this.performanceMetrics.loadTimes.set(sessionId, {
      totalTime: totalLoadTime,
      features: requiredFeatures,
      networkQuality: this.loadingState.networkQuality,
      timestamp: Date.now(),
      phases: this.metrics?.loadingPhases || []
    });
    
    // Calculate averages
    const loadTimes = Array.from(this.performanceMetrics.loadTimes.values());
    const averageLoadTime = loadTimes.reduce((sum, entry) => sum + entry.totalTime, 0) / loadTimes.length;
    
    console.log(`[ProgressiveLoader] Performance: Current=${totalLoadTime.toFixed(2)}ms, Average=${averageLoadTime.toFixed(2)}ms`);
    
    // Clean up old metrics (keep only last 50 entries)
    if (this.performanceMetrics.loadTimes.size > 50) {
      const entries = Array.from(this.performanceMetrics.loadTimes.entries());
      const oldestEntries = entries
        .sort(([,a], [,b]) => a.timestamp - b.timestamp)
        .slice(0, entries.length - 50);
      
      for (const [sessionId] of oldestEntries) {
        this.performanceMetrics.loadTimes.delete(sessionId);
      }
    }
  }

  /**
   * Learn from loading session for future optimization
   */
  learnFromLoadingSession(requiredFeatures, userContext, loadTime) {
    const sessionData = this.behaviorLearning.sessionData;
    
    // Record successful feature combinations
    const featurePattern = requiredFeatures.sort().join('->');
    const userType = userContext.userType || 'general';
    
    if (!this.behaviorLearning.patterns.has(userType)) {
      this.behaviorLearning.patterns.set(userType, new Map());
    }
    
    const userPatterns = this.behaviorLearning.patterns.get(userType);
    const currentCount = userPatterns.get(featurePattern) || 0;
    userPatterns.set(featurePattern, currentCount + 1);
    
    // Update preferences based on load time satisfaction
    if (loadTime < 3000) { // Good performance
      for (const feature of requiredFeatures) {
        const currentPref = this.behaviorLearning.preferences.get(feature) || 1.0;
        this.behaviorLearning.preferences.set(feature, Math.min(currentPref + 0.1, 2.0));
      }
    } else if (loadTime > 8000) { // Poor performance
      for (const feature of requiredFeatures) {
        const currentPref = this.behaviorLearning.preferences.get(feature) || 1.0;
        this.behaviorLearning.preferences.set(feature, Math.max(currentPref - 0.05, 0.5));
      }
    }
    
    // Save learning data
    this.saveUserBehaviorData();
  }

  /**
   * Load user preferences and behavior patterns
   */
  async loadUserPreferences() {
    try {
      const behaviorData = localStorage.getItem('writemagic_behavior_learning');
      if (behaviorData) {
        const parsed = JSON.parse(behaviorData);
        
        // Restore patterns
        if (parsed.patterns) {
          for (const [userType, patterns] of Object.entries(parsed.patterns)) {
            this.behaviorLearning.patterns.set(userType, new Map(Object.entries(patterns)));
          }
        }
        
        // Restore preferences
        if (parsed.preferences) {
          this.behaviorLearning.preferences = new Map(Object.entries(parsed.preferences));
        }
      }
    } catch (error) {
      console.warn('[ProgressiveLoader] Failed to load user behavior data:', error);
    }
  }

  /**
   * Save user behavior learning data
   */
  saveUserBehaviorData() {
    try {
      const behaviorData = {
        patterns: {},
        preferences: Object.fromEntries(this.behaviorLearning.preferences),
        lastUpdated: Date.now()
      };
      
      // Convert patterns Map to object
      for (const [userType, patterns] of this.behaviorLearning.patterns) {
        behaviorData.patterns[userType] = Object.fromEntries(patterns);
      }
      
      localStorage.setItem('writemagic_behavior_learning', JSON.stringify(behaviorData));
    } catch (error) {
      console.warn('[ProgressiveLoader] Failed to save user behavior data:', error);
    }
  }

  /**
   * Save session data
   */
  async saveSessionData() {
    try {
      const sessionSummary = {
        duration: Date.now() - this.behaviorLearning.sessionData.startTime,
        interactions: this.behaviorLearning.sessionData.interactions.length,
        resourcesLoaded: this.behaviorLearning.sessionData.loadedResources.length,
        errors: this.behaviorLearning.sessionData.errors.length,
        networkQuality: this.loadingState.networkQuality,
        timestamp: Date.now()
      };
      
      // Save to indexed DB or send to analytics service
      console.log('[ProgressiveLoader] Session summary:', sessionSummary);
      
    } catch (error) {
      console.warn('[ProgressiveLoader] Failed to save session data:', error);
    }
  }

  // Additional utility methods...
  
  /**
   * Get current loading state
   */
  getLoadingState() {
    return { ...this.loadingState };
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    return { ...this.performanceMetrics };
  }

  /**
   * Force preload specific resources
   */
  async preloadResourceList(urls) {
    const tasks = urls.map(url => this.createPreloadTask(
      { resource: url, score: 1.0 },
      PRELOAD_PRIORITIES.HIGH
    ));
    
    return await this.executeConcurrentPreloads(tasks);
  }

  /**
   * Clear application cache
   */
  async clearApplicationCache() {
    if (this.serviceWorkerManager) {
      return await this.serviceWorkerManager.clearCache();
    }
    return false;
  }
}

/**
 * Network Quality Monitor
 */
class NetworkQualityMonitor extends EventEmitter {
  constructor() {
    super();
    this.currentQuality = 'unknown';
    this.measurements = [];
  }
  
  async initialize() {
    // Set up network monitoring
    if (navigator.connection) {
      this.updateQuality();
      navigator.connection.addEventListener('change', () => this.updateQuality());
    }
    
    // Set up periodic speed testing
    setInterval(() => this.measureNetworkSpeed(), 30000); // Every 30 seconds
  }
  
  updateQuality() {
    if (!navigator.connection) return;
    
    const connection = navigator.connection;
    const effectiveType = connection.effectiveType;
    const downlink = connection.downlink;
    
    let newQuality = 'moderate';
    
    if (effectiveType === 'slow-2g' || downlink < 0.5) {
      newQuality = 'slow';
    } else if (effectiveType === '4g' && downlink > 2) {
      newQuality = 'fast';
    }
    
    if (newQuality !== this.currentQuality) {
      this.currentQuality = newQuality;
      this.emit('qualityChanged', newQuality);
    }
  }
  
  async measureNetworkSpeed() {
    try {
      const startTime = performance.now();
      const response = await fetch('/api/health?_t=' + Date.now(), { 
        cache: 'no-cache' 
      });
      const endTime = performance.now();
      
      if (response.ok) {
        const roundTripTime = endTime - startTime;
        this.measurements.push({
          rtt: roundTripTime,
          timestamp: Date.now()
        });
        
        // Keep only recent measurements
        const cutoff = Date.now() - 300000; // 5 minutes
        this.measurements = this.measurements.filter(m => m.timestamp > cutoff);
      }
    } catch (error) {
      console.debug('[NetworkMonitor] Speed measurement failed:', error);
    }
  }
  
  async getNetworkQuality() {
    return this.currentQuality;
  }
  
  getAverageRTT() {
    if (this.measurements.length === 0) return null;
    
    const total = this.measurements.reduce((sum, m) => sum + m.rtt, 0);
    return total / this.measurements.length;
  }
}

export default ProgressiveLoader;