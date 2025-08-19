/**
 * Enhanced WASM Loader with Streaming Compilation and Progressive Loading
 * 
 * Provides optimized WASM loading with streaming compilation, feature-based code splitting,
 * and intelligent fallback strategies for WriteMagic web application.
 * 
 * Features:
 * - WebAssembly.compileStreaming for faster initialization
 * - Dynamic module loading for non-critical features  
 * - Bundle size optimization with feature flags
 * - Progressive loading with detailed progress indicators
 * - Network-aware loading strategies
 * - Memory usage monitoring
 */

import { EventEmitter } from './event-emitter.js';

/**
 * WASM module configuration for feature-based loading
 */
const WASM_MODULES = {
  // Core module - always loaded
  CORE: {
    name: 'writemagic_wasm',
    url: '/core/wasm/pkg/writemagic_wasm_bg.wasm',
    jsUrl: '/core/wasm/pkg/writemagic_wasm.js',
    priority: 'high',
    required: true,
    features: ['document', 'project', 'ai_basic'],
    estimatedSize: 1024 * 1024 // 1MB estimate
  },
  
  // AI features module - loaded on demand
  AI_ADVANCED: {
    name: 'writemagic_ai',
    url: '/core/wasm/pkg/writemagic_ai_bg.wasm',
    jsUrl: '/core/wasm/pkg/writemagic_ai.js',
    priority: 'medium',
    required: false,
    features: ['ai_suggestions', 'content_analysis', 'writing_insights'],
    estimatedSize: 512 * 1024 // 512KB estimate
  },
  
  // Version control module - loaded when needed
  VERSION_CONTROL: {
    name: 'writemagic_version',
    url: '/core/wasm/pkg/writemagic_version_bg.wasm',
    jsUrl: '/core/wasm/pkg/writemagic_version.js',
    priority: 'low',
    required: false,
    features: ['git_integration', 'timeline', 'diff_generation'],
    estimatedSize: 256 * 1024 // 256KB estimate
  },
  
  // Analytics module - loaded for premium features
  ANALYTICS: {
    name: 'writemagic_analytics',
    url: '/core/wasm/pkg/writemagic_analytics_bg.wasm',
    jsUrl: '/core/wasm/pkg/writemagic_analytics.js',
    priority: 'low',
    required: false,
    features: ['advanced_analytics', 'writing_patterns', 'productivity_insights'],
    estimatedSize: 384 * 1024 // 384KB estimate
  }
};

/**
 * Loading strategies based on network conditions
 */
const LOADING_STRATEGIES = {
  FAST: {
    name: 'fast',
    maxConcurrent: 4,
    timeout: 30000,
    retryAttempts: 3,
    useStreaming: true,
    preloadNonCritical: true
  },
  
  MODERATE: {
    name: 'moderate', 
    maxConcurrent: 2,
    timeout: 45000,
    retryAttempts: 2,
    useStreaming: true,
    preloadNonCritical: false
  },
  
  SLOW: {
    name: 'slow',
    maxConcurrent: 1,
    timeout: 60000,
    retryAttempts: 1,
    useStreaming: false,
    preloadNonCritical: false
  }
};

/**
 * Enhanced WASM Loader with streaming compilation and progressive loading
 */
export class WasmLoader extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.options = {
      enableStreaming: true,
      enableProgressiveLoading: true,
      enableFeatureSplitting: true,
      enableMemoryMonitoring: true,
      fallbackTimeout: 10000,
      maxRetries: 3,
      ...options
    };
    
    // Loading state
    this.loadedModules = new Map();
    this.loadingPromises = new Map();
    this.compiledModules = new Map();
    this.loadingStrategy = this.determineLoadingStrategy();
    
    // Performance metrics
    this.metrics = {
      loadStartTime: null,
      loadEndTime: null,
      totalSize: 0,
      streamingSupported: this.isStreamingSupported(),
      networkSpeed: 'unknown',
      loadingPhases: []
    };
    
    // Progress tracking
    this.progress = {
      totalModules: 0,
      loadedModules: 0,
      totalSize: 0,
      loadedSize: 0,
      currentPhase: 'initializing'
    };
    
    // Memory monitoring
    if (this.options.enableMemoryMonitoring) {
      this.memoryMonitor = new WasmMemoryMonitor();
    }
    
    // Initialize Service Worker integration if available
    this.serviceWorkerIntegration = null;
    if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
      this.initializeServiceWorkerIntegration();
    }
  }

  /**
   * Load WASM modules with progressive loading and streaming compilation
   */
  async loadModules(requiredFeatures = ['document', 'project', 'ai_basic']) {
    this.metrics.loadStartTime = performance.now();
    this.progress.currentPhase = 'planning';
    
    try {
      console.log('[WasmLoader] Starting progressive WASM loading...');
      this.emit('loadingStarted', { requiredFeatures, strategy: this.loadingStrategy.name });
      
      // Phase 1: Plan loading strategy
      const loadingPlan = this.createLoadingPlan(requiredFeatures);
      this.progress.totalModules = loadingPlan.phases.reduce((total, phase) => total + phase.modules.length, 0);
      this.progress.totalSize = loadingPlan.estimatedSize;
      
      console.log('[WasmLoader] Loading plan:', loadingPlan);
      this.emit('loadingPlanCreated', loadingPlan);
      
      // Phase 2: Load critical modules first
      this.progress.currentPhase = 'loading-critical';
      await this.loadPhase(loadingPlan.phases[0], 'critical');
      
      // Phase 3: Load important modules
      if (loadingPlan.phases[1]?.modules.length > 0) {
        this.progress.currentPhase = 'loading-important';
        await this.loadPhase(loadingPlan.phases[1], 'important');
      }
      
      // Phase 4: Load optional modules in background
      if (loadingPlan.phases[2]?.modules.length > 0 && this.loadingStrategy.preloadNonCritical) {
        this.progress.currentPhase = 'loading-optional';
        this.loadPhase(loadingPlan.phases[2], 'optional').catch(error => {
          console.warn('[WasmLoader] Optional modules loading failed:', error);
        });
      }
      
      this.metrics.loadEndTime = performance.now();
      this.progress.currentPhase = 'complete';
      
      const loadTime = this.metrics.loadEndTime - this.metrics.loadStartTime;
      console.log(`[WasmLoader] Progressive loading completed in ${loadTime.toFixed(2)}ms`);
      
      this.emit('loadingCompleted', {
        loadTime,
        modules: Array.from(this.loadedModules.keys()),
        metrics: this.metrics,
        memoryUsage: this.memoryMonitor?.getCurrentUsage()
      });
      
      return this.createModuleProxy();
      
    } catch (error) {
      this.progress.currentPhase = 'error';
      console.error('[WasmLoader] Progressive loading failed:', error);
      this.emit('loadingFailed', error);
      
      // Try fallback loading strategy
      return await this.fallbackLoading(requiredFeatures);
    }
  }

  /**
   * Create loading plan with phased approach
   */
  createLoadingPlan(requiredFeatures) {
    const plan = {
      phases: [
        { name: 'critical', modules: [], priority: 'high' },
        { name: 'important', modules: [], priority: 'medium' },
        { name: 'optional', modules: [], priority: 'low' }
      ],
      estimatedSize: 0,
      estimatedTime: 0
    };
    
    // Categorize modules based on required features and priority
    for (const [moduleId, module] of Object.entries(WASM_MODULES)) {
      const hasRequiredFeature = requiredFeatures.some(feature => 
        module.features.includes(feature)
      );
      
      if (module.required || hasRequiredFeature) {
        if (module.priority === 'high') {
          plan.phases[0].modules.push({ id: moduleId, ...module });
        } else if (module.priority === 'medium') {
          plan.phases[1].modules.push({ id: moduleId, ...module });
        } else {
          plan.phases[2].modules.push({ id: moduleId, ...module });
        }
      }
    }
    
    // Calculate estimated sizes
    plan.estimatedSize = plan.phases.reduce((total, phase) => 
      total + phase.modules.reduce((phaseTotal, module) => 
        phaseTotal + module.estimatedSize, 0
      ), 0
    );
    
    // Estimate loading time based on network conditions
    const bytesPerSecond = this.estimateNetworkSpeed();
    plan.estimatedTime = plan.estimatedSize / bytesPerSecond * 1000;
    
    return plan;
  }

  /**
   * Load a specific phase of modules
   */
  async loadPhase(phase, phaseName) {
    console.log(`[WasmLoader] Loading ${phaseName} phase (${phase.modules.length} modules)`);
    this.emit('phaseStarted', { phase: phaseName, modules: phase.modules.length });
    
    const phaseStartTime = performance.now();
    
    try {
      // Load modules concurrently within phase limits
      const concurrentLimit = Math.min(phase.modules.length, this.loadingStrategy.maxConcurrent);
      const batches = this.createBatches(phase.modules, concurrentLimit);
      
      for (const batch of batches) {
        const batchPromises = batch.map(module => this.loadModule(module));
        await Promise.all(batchPromises);
      }
      
      const phaseEndTime = performance.now();
      const phaseLoadTime = phaseEndTime - phaseStartTime;
      
      console.log(`[WasmLoader] ${phaseName} phase completed in ${phaseLoadTime.toFixed(2)}ms`);
      this.emit('phaseCompleted', { 
        phase: phaseName, 
        loadTime: phaseLoadTime,
        modules: phase.modules.map(m => m.id)
      });
      
      this.metrics.loadingPhases.push({
        name: phaseName,
        loadTime: phaseLoadTime,
        modules: phase.modules.length
      });
      
    } catch (error) {
      console.error(`[WasmLoader] ${phaseName} phase failed:`, error);
      throw error;
    }
  }

  /**
   * Load individual WASM module with streaming compilation
   */
  async loadModule(moduleConfig) {
    const moduleId = moduleConfig.id;
    
    // Return cached module if already loaded
    if (this.loadedModules.has(moduleId)) {
      return this.loadedModules.get(moduleId);
    }
    
    // Return in-progress loading promise
    if (this.loadingPromises.has(moduleId)) {
      return this.loadingPromises.get(moduleId);
    }
    
    console.log(`[WasmLoader] Loading module: ${moduleId}`);
    this.emit('moduleLoadStarted', { moduleId, config: moduleConfig });
    
    const loadingPromise = this.loadModuleInternal(moduleConfig);
    this.loadingPromises.set(moduleId, loadingPromise);
    
    try {
      const module = await loadingPromise;
      this.loadedModules.set(moduleId, module);
      this.progress.loadedModules++;
      this.progress.loadedSize += moduleConfig.estimatedSize;
      
      console.log(`[WasmLoader] Module loaded successfully: ${moduleId}`);
      this.emit('moduleLoadCompleted', { 
        moduleId, 
        module,
        progress: { ...this.progress }
      });
      
      return module;
      
    } catch (error) {
      console.error(`[WasmLoader] Module loading failed: ${moduleId}`, error);
      this.emit('moduleLoadFailed', { moduleId, error });
      throw error;
    } finally {
      this.loadingPromises.delete(moduleId);
    }
  }

  /**
   * Internal module loading with streaming compilation
   */
  async loadModuleInternal(moduleConfig) {
    const { name, url, jsUrl, estimatedSize } = moduleConfig;
    let retryCount = 0;
    
    while (retryCount < this.loadingStrategy.retryAttempts) {
      try {
        // Create progress tracking for this module
        const progressTracker = new ModuleLoadProgressTracker(name, estimatedSize);
        
        // Load JavaScript bindings first
        const jsModule = await this.loadJavaScriptModule(jsUrl, progressTracker);
        
        // Load and compile WASM module
        let wasmModule;
        if (this.options.enableStreaming && this.metrics.streamingSupported) {
          wasmModule = await this.loadWasmWithStreaming(url, jsModule, progressTracker);
        } else {
          wasmModule = await this.loadWasmTraditional(url, jsModule, progressTracker);
        }
        
        // Initialize the module
        const initializedModule = await this.initializeModule(wasmModule, jsModule, progressTracker);
        
        // Monitor memory usage
        if (this.memoryMonitor) {
          this.memoryMonitor.trackModule(name, initializedModule);
        }
        
        return {
          id: name,
          wasmModule,
          jsModule,
          initialized: initializedModule,
          loadedAt: Date.now(),
          memoryUsage: this.memoryMonitor?.getModuleUsage(name)
        };
        
      } catch (error) {
        retryCount++;
        console.warn(`[WasmLoader] Module loading attempt ${retryCount} failed for ${name}:`, error);
        
        if (retryCount >= this.loadingStrategy.retryAttempts) {
          throw new Error(`Failed to load module ${name} after ${retryCount} attempts: ${error.message}`);
        }
        
        // Exponential backoff for retries
        const delay = Math.min(1000 * Math.pow(2, retryCount - 1), 5000);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  /**
   * Load JavaScript module with import()
   */
  async loadJavaScriptModule(jsUrl, progressTracker) {
    progressTracker.updateProgress('js_loading', 0);
    
    try {
      const module = await import(jsUrl);
      progressTracker.updateProgress('js_complete', 25);
      return module;
    } catch (error) {
      progressTracker.updateProgress('js_error', 0);
      throw new Error(`Failed to load JavaScript module from ${jsUrl}: ${error.message}`);
    }
  }

  /**
   * Load WASM with streaming compilation (preferred method)
   */
  async loadWasmWithStreaming(wasmUrl, jsModule, progressTracker) {
    progressTracker.updateProgress('wasm_streaming', 25);
    
    try {
      // Use WebAssembly.compileStreaming for better performance
      const response = await fetch(wasmUrl);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM: ${response.status} ${response.statusText}`);
      }
      
      // Track download progress
      const reader = response.body?.getReader();
      const contentLength = parseInt(response.headers.get('content-length') || '0');
      
      if (reader && contentLength > 0) {
        let receivedLength = 0;
        const chunks = [];
        
        while (true) {
          const { done, value } = await reader.read();
          
          if (done) break;
          
          chunks.push(value);
          receivedLength += value.length;
          
          const progress = Math.min((receivedLength / contentLength) * 0.5, 0.5);
          progressTracker.updateProgress('wasm_downloading', 25 + progress * 100);
        }
        
        // Reconstruct response for WebAssembly.compileStreaming
        const fullBody = new Uint8Array(receivedLength);
        let position = 0;
        for (const chunk of chunks) {
          fullBody.set(chunk, position);
          position += chunk.length;
        }
        
        const streamResponse = new Response(fullBody, {
          headers: { 'Content-Type': 'application/wasm' }
        });
        
        progressTracker.updateProgress('wasm_compiling', 75);
        const wasmModule = await WebAssembly.compileStreaming(streamResponse);
        progressTracker.updateProgress('wasm_complete', 100);
        
        return wasmModule;
      } else {
        // Fallback if no reader or content-length
        progressTracker.updateProgress('wasm_compiling', 50);
        const wasmModule = await WebAssembly.compileStreaming(response);
        progressTracker.updateProgress('wasm_complete', 100);
        
        return wasmModule;
      }
      
    } catch (error) {
      progressTracker.updateProgress('wasm_error', 25);
      console.warn('[WasmLoader] Streaming compilation failed, falling back to traditional loading');
      return await this.loadWasmTraditional(wasmUrl, jsModule, progressTracker);
    }
  }

  /**
   * Load WASM with traditional fetch and compile (fallback method)
   */
  async loadWasmTraditional(wasmUrl, jsModule, progressTracker) {
    progressTracker.updateProgress('wasm_traditional', 25);
    
    try {
      const response = await fetch(wasmUrl);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM: ${response.status} ${response.statusText}`);
      }
      
      progressTracker.updateProgress('wasm_fetching', 50);
      const wasmBytes = await response.arrayBuffer();
      
      progressTracker.updateProgress('wasm_compiling', 75);
      const wasmModule = await WebAssembly.compile(wasmBytes);
      
      progressTracker.updateProgress('wasm_complete', 100);
      return wasmModule;
      
    } catch (error) {
      progressTracker.updateProgress('wasm_error', 25);
      throw new Error(`Traditional WASM loading failed: ${error.message}`);
    }
  }

  /**
   * Initialize compiled WASM module
   */
  async initializeModule(wasmModule, jsModule, progressTracker) {
    progressTracker.updateProgress('initializing', 90);
    
    try {
      // Create WebAssembly instance
      const wasmInstance = await WebAssembly.instantiate(wasmModule);
      
      // Initialize JavaScript bindings with WASM instance
      const initialized = await jsModule.default(wasmInstance);
      
      progressTracker.updateProgress('complete', 100);
      return {
        instance: wasmInstance,
        exports: wasmInstance.exports,
        jsBindings: initialized,
        module: wasmModule
      };
      
    } catch (error) {
      progressTracker.updateProgress('init_error', 90);
      throw new Error(`Module initialization failed: ${error.message}`);
    }
  }

  /**
   * Create module proxy for unified API access
   */
  createModuleProxy() {
    const proxy = {};
    
    // Combine all loaded module exports
    for (const [moduleId, module] of this.loadedModules) {
      proxy[moduleId] = {
        ...module.initialized.exports,
        ...module.initialized.jsBindings
      };
    }
    
    // Add utility methods
    proxy._wasmLoader = {
      getLoadedModules: () => Array.from(this.loadedModules.keys()),
      getModuleInfo: (moduleId) => this.loadedModules.get(moduleId),
      getMetrics: () => this.metrics,
      getMemoryUsage: () => this.memoryMonitor?.getTotalUsage(),
      loadOptionalModule: (moduleId) => this.loadOptionalModule(moduleId),
      unloadModule: (moduleId) => this.unloadModule(moduleId)
    };
    
    return proxy;
  }

  /**
   * Load optional module on demand
   */
  async loadOptionalModule(moduleId) {
    const moduleConfig = WASM_MODULES[moduleId];
    
    if (!moduleConfig) {
      throw new Error(`Unknown module: ${moduleId}`);
    }
    
    if (this.loadedModules.has(moduleId)) {
      return this.loadedModules.get(moduleId);
    }
    
    console.log(`[WasmLoader] Loading optional module on demand: ${moduleId}`);
    this.emit('optionalModuleRequested', { moduleId });
    
    try {
      const module = await this.loadModule({ id: moduleId, ...moduleConfig });
      this.emit('optionalModuleLoaded', { moduleId, module });
      return module;
    } catch (error) {
      this.emit('optionalModuleFailed', { moduleId, error });
      throw error;
    }
  }

  /**
   * Unload module and free memory
   */
  async unloadModule(moduleId) {
    const module = this.loadedModules.get(moduleId);
    
    if (!module) {
      console.warn(`[WasmLoader] Cannot unload non-existent module: ${moduleId}`);
      return false;
    }
    
    try {
      // Call module cleanup if available
      if (module.initialized.exports.cleanup) {
        module.initialized.exports.cleanup();
      }
      
      // Remove from memory monitor
      if (this.memoryMonitor) {
        this.memoryMonitor.untrackModule(moduleId);
      }
      
      // Remove from loaded modules
      this.loadedModules.delete(moduleId);
      
      console.log(`[WasmLoader] Module unloaded: ${moduleId}`);
      this.emit('moduleUnloaded', { moduleId });
      
      // Force garbage collection if available
      if (window.gc) {
        window.gc();
      }
      
      return true;
      
    } catch (error) {
      console.error(`[WasmLoader] Failed to unload module ${moduleId}:`, error);
      return false;
    }
  }

  /**
   * Fallback loading strategy for critical failures
   */
  async fallbackLoading(requiredFeatures) {
    console.log('[WasmLoader] Attempting fallback loading strategy...');
    this.emit('fallbackStarted', { requiredFeatures });
    
    try {
      // Load only the core module with basic retry logic
      const coreModule = WASM_MODULES.CORE;
      
      let retryCount = 0;
      while (retryCount < this.options.maxRetries) {
        try {
          const module = await this.loadModuleInternal({ id: 'CORE', ...coreModule });
          this.loadedModules.set('CORE', module);
          
          console.log('[WasmLoader] Fallback loading successful');
          this.emit('fallbackSucceeded', { module });
          
          return this.createModuleProxy();
          
        } catch (error) {
          retryCount++;
          if (retryCount >= this.options.maxRetries) {
            throw error;
          }
          
          console.warn(`[WasmLoader] Fallback attempt ${retryCount} failed, retrying...`);
          await new Promise(resolve => setTimeout(resolve, 1000 * retryCount));
        }
      }
      
    } catch (error) {
      console.error('[WasmLoader] All fallback attempts failed:', error);
      this.emit('fallbackFailed', { error });
      throw new Error(`Complete loading failure: ${error.message}`);
    }
  }

  /**
   * Determine optimal loading strategy based on network conditions
   */
  determineLoadingStrategy() {
    // Check network connection if available
    if (navigator.connection) {
      const connection = navigator.connection;
      const effectiveType = connection.effectiveType;
      const downlink = connection.downlink;
      
      if (effectiveType === 'slow-2g' || downlink < 0.5) {
        return LOADING_STRATEGIES.SLOW;
      } else if (effectiveType === '2g' || effectiveType === '3g' || downlink < 1.5) {
        return LOADING_STRATEGIES.MODERATE;
      } else {
        return LOADING_STRATEGIES.FAST;
      }
    }
    
    // Fallback to moderate strategy
    return LOADING_STRATEGIES.MODERATE;
  }

  /**
   * Check if streaming compilation is supported
   */
  isStreamingSupported() {
    return typeof WebAssembly.compileStreaming === 'function' && 
           typeof ReadableStream !== 'undefined';
  }

  /**
   * Estimate network speed for loading time calculations
   */
  estimateNetworkSpeed() {
    if (navigator.connection) {
      const connection = navigator.connection;
      if (connection.downlink) {
        return connection.downlink * 1024 * 1024 / 8; // Convert Mbps to bytes per second
      }
    }
    
    // Default conservative estimate (1 Mbps)
    return 1024 * 1024 / 8;
  }

  /**
   * Create batches for concurrent loading
   */
  createBatches(items, batchSize) {
    const batches = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  /**
   * Initialize Service Worker integration for caching
   */
  async initializeServiceWorkerIntegration() {
    try {
      this.serviceWorkerIntegration = {
        controller: navigator.serviceWorker.controller,
        preloadModules: async (modules) => {
          const urls = modules.flatMap(m => [m.url, m.jsUrl]);
          await this.postMessageToSW('PRELOAD_RESOURCES', { urls });
        },
        getCacheStatus: async () => {
          return await this.postMessageToSW('GET_CACHE_STATUS');
        }
      };
    } catch (error) {
      console.warn('[WasmLoader] Service Worker integration failed:', error);
    }
  }

  /**
   * Post message to Service Worker
   */
  async postMessageToSW(type, data = {}) {
    return new Promise((resolve, reject) => {
      const channel = new MessageChannel();
      
      channel.port1.onmessage = (event) => {
        if (event.data.error) {
          reject(new Error(event.data.error));
        } else {
          resolve(event.data);
        }
      };
      
      navigator.serviceWorker.controller.postMessage(
        { type, ...data },
        [channel.port2]
      );
      
      // Timeout after 10 seconds
      setTimeout(() => reject(new Error('Service Worker message timeout')), 10000);
    });
  }
}

/**
 * Module load progress tracker
 */
class ModuleLoadProgressTracker {
  constructor(moduleName, estimatedSize) {
    this.moduleName = moduleName;
    this.estimatedSize = estimatedSize;
    this.currentPhase = 'initializing';
    this.progress = 0;
    this.startTime = performance.now();
  }
  
  updateProgress(phase, progress) {
    this.currentPhase = phase;
    this.progress = Math.min(Math.max(progress, 0), 100);
    
    console.debug(`[WasmLoader] ${this.moduleName} - ${phase}: ${this.progress.toFixed(1)}%`);
  }
  
  getElapsedTime() {
    return performance.now() - this.startTime;
  }
}

/**
 * WASM Memory Monitor for tracking memory usage
 */
class WasmMemoryMonitor {
  constructor() {
    this.moduleUsage = new Map();
    this.totalUsage = {
      heapSize: 0,
      usedHeapSize: 0,
      modules: 0
    };
  }
  
  trackModule(moduleName, module) {
    try {
      const memory = module.instance.exports.memory;
      const usage = {
        heapSize: memory ? memory.buffer.byteLength : 0,
        timestamp: Date.now()
      };
      
      this.moduleUsage.set(moduleName, usage);
      this.updateTotalUsage();
      
      console.debug(`[WasmMemoryMonitor] Tracking ${moduleName}: ${this.formatBytes(usage.heapSize)}`);
    } catch (error) {
      console.warn(`[WasmMemoryMonitor] Failed to track module ${moduleName}:`, error);
    }
  }
  
  untrackModule(moduleName) {
    this.moduleUsage.delete(moduleName);
    this.updateTotalUsage();
  }
  
  updateTotalUsage() {
    this.totalUsage = {
      heapSize: Array.from(this.moduleUsage.values()).reduce((total, usage) => total + usage.heapSize, 0),
      modules: this.moduleUsage.size,
      lastUpdated: Date.now()
    };
    
    // Add JavaScript heap usage if available
    if (performance.memory) {
      this.totalUsage.jsHeapSize = performance.memory.usedJSHeapSize;
      this.totalUsage.totalHeapSize = performance.memory.totalJSHeapSize;
    }
  }
  
  getCurrentUsage() {
    this.updateTotalUsage();
    return { ...this.totalUsage };
  }
  
  getModuleUsage(moduleName) {
    return this.moduleUsage.get(moduleName);
  }
  
  getTotalUsage() {
    return this.getCurrentUsage();
  }
  
  formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
}

export default WasmLoader;