/**
 * Performance Monitor for WriteMagic Web Application
 * 
 * Provides comprehensive performance monitoring including WASM bundle size tracking,
 * runtime performance profiling, loading time optimization, and memory usage monitoring.
 * 
 * Features:
 * - Real-time WASM bundle size monitoring with alerts
 * - Critical path performance profiling
 * - Loading time optimization with <3s targets
 * - Memory usage tracking for WASM operations
 * - Performance regression detection
 * - User experience metrics collection
 * - Network condition adaptation
 * - Automated optimization suggestions
 */

import { EventEmitter } from './event-emitter.js';

/**
 * Performance thresholds and targets
 */
const PERFORMANCE_TARGETS = {
  // Loading time targets
  INITIAL_LOAD: 3000,     // 3 seconds for initial load
  WASM_COMPILATION: 2000,  // 2 seconds for WASM compilation
  FIRST_PAINT: 1000,      // 1 second for first paint
  FIRST_CONTENTFUL_PAINT: 1500, // 1.5 seconds for FCP
  
  // Bundle size targets
  WASM_BUNDLE_SIZE: 2 * 1024 * 1024,      // 2MB total WASM
  CRITICAL_JS_SIZE: 200 * 1024,           // 200KB critical JS
  TOTAL_BUNDLE_SIZE: 5 * 1024 * 1024,     // 5MB total application
  
  // Runtime performance targets
  FRAME_RATE: 60,         // 60 FPS
  INPUT_LATENCY: 100,     // 100ms maximum input latency
  MEMORY_GROWTH: 10 * 1024 * 1024, // 10MB/hour max memory growth
  
  // Network targets
  SLOW_NETWORK_TARGET: 8000,    // 8 seconds on slow 3G
  FAST_NETWORK_TARGET: 1500,    // 1.5 seconds on fast connections
};

/**
 * Performance metric categories
 */
const METRIC_CATEGORIES = {
  LOADING: 'loading',
  RUNTIME: 'runtime', 
  MEMORY: 'memory',
  NETWORK: 'network',
  USER_EXPERIENCE: 'user_experience',
  WASM: 'wasm'
};

/**
 * Performance alert severity levels
 */
const ALERT_LEVELS = {
  INFO: 'info',
  WARNING: 'warning',
  CRITICAL: 'critical',
  EMERGENCY: 'emergency'
};

/**
 * Comprehensive Performance Monitor
 */
export class PerformanceMonitor extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.options = {
      enableRealTimeMonitoring: true,
      enableAutoOptimization: true,
      enableRegressionDetection: true,
      enableUserExperienceTracking: true,
      monitoringInterval: 5000,
      alertThreshold: 0.8,
      retentionDays: 7,
      ...options
    };
    
    // Performance data stores
    this.metrics = new Map();
    this.alerts = [];
    this.regressions = [];
    this.optimizations = [];
    
    // Current monitoring state
    this.isMonitoring = false;
    this.startTime = null;
    this.lastMeasurement = null;
    
    // Specialized monitors
    this.wasmMonitor = new WasmPerformanceMonitor();
    this.loadingMonitor = new LoadingPerformanceMonitor();
    this.runtimeMonitor = new RuntimePerformanceMonitor();
    this.memoryMonitor = new MemoryPerformanceMonitor();
    this.networkMonitor = new NetworkPerformanceMonitor();
    this.uxMonitor = new UserExperienceMonitor();
    
    // Performance observer for browser metrics
    this.performanceObserver = null;
    
    // Initialize
    this.initializePerformanceMonitor();
  }

  /**
   * Initialize the performance monitoring system
   */
  async initializePerformanceMonitor() {
    try {
      console.log('[PerformanceMonitor] Initializing comprehensive performance monitoring...');
      
      // Initialize specialized monitors
      await this.wasmMonitor.initialize();
      await this.loadingMonitor.initialize();
      await this.runtimeMonitor.initialize();
      await this.memoryMonitor.initialize();
      await this.networkMonitor.initialize();
      await this.uxMonitor.initialize();
      
      // Set up Performance Observer API
      this.initializePerformanceObserver();
      
      // Load historical data
      await this.loadHistoricalData();
      
      // Start monitoring if enabled
      if (this.options.enableRealTimeMonitoring) {
        this.startMonitoring();
      }
      
      console.log('[PerformanceMonitor] Performance monitoring system initialized');
      this.emit('initialized');
      
    } catch (error) {
      console.error('[PerformanceMonitor] Initialization failed:', error);
      this.emit('initializationFailed', error);
    }
  }

  /**
   * Start comprehensive performance monitoring
   */
  startMonitoring() {
    if (this.isMonitoring) {
      console.warn('[PerformanceMonitor] Monitoring already active');
      return;
    }
    
    this.isMonitoring = true;
    this.startTime = performance.now();
    
    console.log('[PerformanceMonitor] Starting performance monitoring...');
    
    // Start all specialized monitors
    this.wasmMonitor.startMonitoring();
    this.loadingMonitor.startMonitoring();
    this.runtimeMonitor.startMonitoring();
    this.memoryMonitor.startMonitoring();
    this.networkMonitor.startMonitoring();
    this.uxMonitor.startMonitoring();
    
    // Set up periodic measurements
    this.monitoringInterval = setInterval(() => {
      this.collectMetrics();
    }, this.options.monitoringInterval);
    
    // Set up regression detection
    if (this.options.enableRegressionDetection) {
      this.regressionInterval = setInterval(() => {
        this.detectRegressions();
      }, 30000); // Check every 30 seconds
    }
    
    this.emit('monitoringStarted');
  }

  /**
   * Stop performance monitoring
   */
  stopMonitoring() {
    if (!this.isMonitoring) return;
    
    this.isMonitoring = false;
    
    // Stop all specialized monitors
    this.wasmMonitor.stopMonitoring();
    this.loadingMonitor.stopMonitoring();
    this.runtimeMonitor.stopMonitoring();
    this.memoryMonitor.stopMonitoring();
    this.networkMonitor.stopMonitoring();
    this.uxMonitor.stopMonitoring();
    
    // Clear intervals
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
    
    if (this.regressionInterval) {
      clearInterval(this.regressionInterval);
      this.regressionInterval = null;
    }
    
    console.log('[PerformanceMonitor] Performance monitoring stopped');
    this.emit('monitoringStopped');
  }

  /**
   * Collect comprehensive performance metrics
   */
  async collectMetrics() {
    const timestamp = Date.now();
    const metrics = {};
    
    try {
      // Collect from all specialized monitors
      metrics[METRIC_CATEGORIES.WASM] = await this.wasmMonitor.collectMetrics();
      metrics[METRIC_CATEGORIES.LOADING] = await this.loadingMonitor.collectMetrics();
      metrics[METRIC_CATEGORIES.RUNTIME] = await this.runtimeMonitor.collectMetrics();
      metrics[METRIC_CATEGORIES.MEMORY] = await this.memoryMonitor.collectMetrics();
      metrics[METRIC_CATEGORIES.NETWORK] = await this.networkMonitor.collectMetrics();
      metrics[METRIC_CATEGORIES.USER_EXPERIENCE] = await this.uxMonitor.collectMetrics();
      
      // Add browser performance metrics
      metrics.browser = this.collectBrowserMetrics();
      
      // Store metrics
      this.metrics.set(timestamp, metrics);
      
      // Clean up old metrics
      this.cleanupOldMetrics();
      
      // Analyze metrics for alerts
      this.analyzeMetricsForAlerts(metrics);
      
      // Trigger optimizations if enabled
      if (this.options.enableAutoOptimization) {
        this.triggerAutoOptimizations(metrics);
      }
      
      this.lastMeasurement = timestamp;
      this.emit('metricsCollected', { timestamp, metrics });
      
    } catch (error) {
      console.error('[PerformanceMonitor] Metrics collection failed:', error);
      this.emit('metricsCollectionFailed', error);
    }
  }

  /**
   * Collect browser-specific performance metrics
   */
  collectBrowserMetrics() {
    const metrics = {};
    
    try {
      // Navigation timing
      if (performance.navigation) {
        const nav = performance.getEntriesByType('navigation')[0];
        if (nav) {
          metrics.navigation = {
            domComplete: nav.domComplete,
            domContentLoaded: nav.domContentLoadedEventEnd - nav.domContentLoadedEventStart,
            loadComplete: nav.loadEventEnd - nav.loadEventStart,
            firstPaint: 0,
            firstContentfulPaint: 0
          };
          
          // Add paint timing
          const paintEntries = performance.getEntriesByType('paint');
          for (const entry of paintEntries) {
            if (entry.name === 'first-paint') {
              metrics.navigation.firstPaint = entry.startTime;
            } else if (entry.name === 'first-contentful-paint') {
              metrics.navigation.firstContentfulPaint = entry.startTime;
            }
          }
        }
      }
      
      // Resource timing
      const resources = performance.getEntriesByType('resource');
      metrics.resources = {
        totalRequests: resources.length,
        totalSize: resources.reduce((sum, r) => sum + (r.transferSize || 0), 0),
        averageLoadTime: resources.length > 0 ? 
          resources.reduce((sum, r) => sum + r.duration, 0) / resources.length : 0,
        cacheHitRate: resources.filter(r => r.transferSize === 0).length / resources.length
      };
      
      // Memory metrics (if available)
      if (performance.memory) {
        metrics.memory = {
          usedJSHeapSize: performance.memory.usedJSHeapSize,
          totalJSHeapSize: performance.memory.totalJSHeapSize,
          jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
          heapUsagePercent: (performance.memory.usedJSHeapSize / performance.memory.jsHeapSizeLimit) * 100
        };
      }
      
    } catch (error) {
      console.debug('[PerformanceMonitor] Browser metrics collection error:', error);
    }
    
    return metrics;
  }

  /**
   * Analyze metrics and generate alerts
   */
  analyzeMetricsForAlerts(metrics) {
    const alerts = [];
    
    // Check WASM performance
    if (metrics.wasm) {
      if (metrics.wasm.compilationTime > PERFORMANCE_TARGETS.WASM_COMPILATION) {
        alerts.push(this.createAlert(
          ALERT_LEVELS.WARNING,
          'WASM compilation time exceeds target',
          `Compilation took ${metrics.wasm.compilationTime}ms (target: ${PERFORMANCE_TARGETS.WASM_COMPILATION}ms)`,
          METRIC_CATEGORIES.WASM
        ));
      }
      
      if (metrics.wasm.bundleSize > PERFORMANCE_TARGETS.WASM_BUNDLE_SIZE) {
        alerts.push(this.createAlert(
          ALERT_LEVELS.CRITICAL,
          'WASM bundle size exceeds target',
          `Bundle size is ${this.formatBytes(metrics.wasm.bundleSize)} (target: ${this.formatBytes(PERFORMANCE_TARGETS.WASM_BUNDLE_SIZE)})`,
          METRIC_CATEGORIES.WASM
        ));
      }
    }
    
    // Check loading performance
    if (metrics.loading) {
      if (metrics.loading.initialLoadTime > PERFORMANCE_TARGETS.INITIAL_LOAD) {
        alerts.push(this.createAlert(
          ALERT_LEVELS.WARNING,
          'Initial load time exceeds target',
          `Load time: ${metrics.loading.initialLoadTime}ms (target: ${PERFORMANCE_TARGETS.INITIAL_LOAD}ms)`,
          METRIC_CATEGORIES.LOADING
        ));
      }
    }
    
    // Check memory usage
    if (metrics.memory) {
      if (metrics.memory.growthRate > PERFORMANCE_TARGETS.MEMORY_GROWTH) {
        alerts.push(this.createAlert(
          ALERT_LEVELS.WARNING,
          'Memory growth rate exceeds target',
          `Growth rate: ${this.formatBytes(metrics.memory.growthRate)}/hour`,
          METRIC_CATEGORIES.MEMORY
        ));
      }
    }
    
    // Check runtime performance
    if (metrics.runtime) {
      if (metrics.runtime.averageFrameRate < PERFORMANCE_TARGETS.FRAME_RATE * 0.8) {
        alerts.push(this.createAlert(
          ALERT_LEVELS.WARNING,
          'Frame rate below target',
          `Average FPS: ${metrics.runtime.averageFrameRate.toFixed(1)} (target: ${PERFORMANCE_TARGETS.FRAME_RATE})`,
          METRIC_CATEGORIES.RUNTIME
        ));
      }
    }
    
    // Add alerts and emit events
    for (const alert of alerts) {
      this.addAlert(alert);
    }
  }

  /**
   * Create performance alert
   */
  createAlert(level, title, message, category) {
    return {
      id: Date.now() + Math.random().toString(36),
      level,
      title,
      message,
      category,
      timestamp: Date.now(),
      acknowledged: false
    };
  }

  /**
   * Add alert to the system
   */
  addAlert(alert) {
    this.alerts.unshift(alert);
    
    // Limit alert history
    if (this.alerts.length > 100) {
      this.alerts = this.alerts.slice(0, 100);
    }
    
    console.log(`[PerformanceMonitor] ${alert.level.toUpperCase()} Alert: ${alert.title}`);
    this.emit('alertCreated', alert);
    
    // Auto-acknowledge info alerts
    if (alert.level === ALERT_LEVELS.INFO) {
      setTimeout(() => this.acknowledgeAlert(alert.id), 5000);
    }
  }

  /**
   * Acknowledge alert
   */
  acknowledgeAlert(alertId) {
    const alert = this.alerts.find(a => a.id === alertId);
    if (alert) {
      alert.acknowledged = true;
      this.emit('alertAcknowledged', alert);
    }
  }

  /**
   * Detect performance regressions
   */
  detectRegressions() {
    if (this.metrics.size < 10) return; // Need enough data points
    
    try {
      const recentMetrics = Array.from(this.metrics.entries()).slice(-10);
      const historicalMetrics = Array.from(this.metrics.entries()).slice(-50, -10);
      
      if (historicalMetrics.length < 10) return;
      
      // Calculate baselines
      const baselines = this.calculateBaselines(historicalMetrics);
      const currentAverages = this.calculateBaselines(recentMetrics);
      
      // Detect regressions
      const regressions = [];
      
      for (const [category, baseline] of Object.entries(baselines)) {
        const current = currentAverages[category];
        if (!current) continue;
        
        // Check for significant performance degradation
        const degradationThreshold = 1.2; // 20% degradation
        
        for (const [metric, baselineValue] of Object.entries(baseline)) {
          const currentValue = current[metric];
          if (typeof currentValue !== 'number' || typeof baselineValue !== 'number') continue;
          
          // For metrics where lower is better (loading times, etc.)
          const isWorsePerformance = this.isPerformanceMetricWorse(category, metric, currentValue, baselineValue);
          
          if (isWorsePerformance && currentValue > baselineValue * degradationThreshold) {
            regressions.push({
              category,
              metric,
              baselineValue,
              currentValue,
              degradationPercent: ((currentValue - baselineValue) / baselineValue) * 100,
              detectedAt: Date.now()
            });
          }
        }
      }
      
      // Process detected regressions
      for (const regression of regressions) {
        this.processRegression(regression);
      }
      
    } catch (error) {
      console.error('[PerformanceMonitor] Regression detection failed:', error);
    }
  }

  /**
   * Process detected performance regression
   */
  processRegression(regression) {
    // Check if this regression was already detected recently
    const recentRegression = this.regressions.find(r => 
      r.category === regression.category && 
      r.metric === regression.metric &&
      Date.now() - r.detectedAt < 300000 // 5 minutes
    );
    
    if (recentRegression) return;
    
    this.regressions.push(regression);
    
    // Create alert for regression
    const alert = this.createAlert(
      ALERT_LEVELS.WARNING,
      `Performance regression detected`,
      `${regression.category}.${regression.metric} degraded by ${regression.degradationPercent.toFixed(1)}%`,
      regression.category
    );
    
    this.addAlert(alert);
    
    console.warn('[PerformanceMonitor] Regression detected:', regression);
    this.emit('regressionDetected', regression);
  }

  /**
   * Determine if a performance metric value is worse than baseline
   */
  isPerformanceMetricWorse(category, metric, currentValue, baselineValue) {
    // Metrics where lower is better
    const lowerIsBetter = [
      'loadTime', 'compilationTime', 'responseTime', 'latency', 'duration',
      'bundleSize', 'memoryUsage', 'heapSize', 'gcTime'
    ];
    
    // Metrics where higher is better
    const higherIsBetter = [
      'frameRate', 'fps', 'throughput', 'cacheHitRate', 'successRate'
    ];
    
    const isLowerBetter = lowerIsBetter.some(term => metric.toLowerCase().includes(term));
    const isHigherBetter = higherIsBetter.some(term => metric.toLowerCase().includes(term));
    
    if (isLowerBetter) {
      return currentValue > baselineValue;
    } else if (isHigherBetter) {
      return currentValue < baselineValue;
    }
    
    // Default: assume lower is better for most performance metrics
    return currentValue > baselineValue;
  }

  /**
   * Calculate performance baselines from historical data
   */
  calculateBaselines(metricsEntries) {
    const baselines = {};
    
    for (const [timestamp, metrics] of metricsEntries) {
      for (const [category, categoryMetrics] of Object.entries(metrics)) {
        if (typeof categoryMetrics !== 'object' || !categoryMetrics) continue;
        
        if (!baselines[category]) baselines[category] = {};
        
        for (const [metric, value] of Object.entries(categoryMetrics)) {
          if (typeof value !== 'number') continue;
          
          if (!baselines[category][metric]) baselines[category][metric] = [];
          baselines[category][metric].push(value);
        }
      }
    }
    
    // Calculate averages
    for (const [category, categoryBaselines] of Object.entries(baselines)) {
      for (const [metric, values] of Object.entries(categoryBaselines)) {
        const average = values.reduce((sum, val) => sum + val, 0) / values.length;
        baselines[category][metric] = average;
      }
    }
    
    return baselines;
  }

  /**
   * Trigger automatic optimizations based on metrics
   */
  triggerAutoOptimizations(metrics) {
    const optimizations = [];
    
    // WASM bundle optimizations
    if (metrics.wasm?.bundleSize > PERFORMANCE_TARGETS.WASM_BUNDLE_SIZE * 0.8) {
      optimizations.push({
        type: 'wasm_compression',
        description: 'Enable aggressive WASM compression',
        impact: 'high',
        category: METRIC_CATEGORIES.WASM
      });
    }
    
    // Loading optimizations
    if (metrics.loading?.initialLoadTime > PERFORMANCE_TARGETS.INITIAL_LOAD * 0.8) {
      optimizations.push({
        type: 'resource_preloading',
        description: 'Implement intelligent resource preloading',
        impact: 'medium',
        category: METRIC_CATEGORIES.LOADING
      });
    }
    
    // Memory optimizations
    if (metrics.memory?.heapUsagePercent > 70) {
      optimizations.push({
        type: 'memory_cleanup',
        description: 'Trigger memory cleanup and garbage collection',
        impact: 'medium',
        category: METRIC_CATEGORIES.MEMORY
      });
    }
    
    // Network optimizations
    if (metrics.network?.cacheHitRate < 0.7) {
      optimizations.push({
        type: 'cache_optimization',
        description: 'Optimize caching strategies',
        impact: 'high',
        category: METRIC_CATEGORIES.NETWORK
      });
    }
    
    // Execute optimizations
    for (const optimization of optimizations) {
      this.executeOptimization(optimization);
    }
  }

  /**
   * Execute performance optimization
   */
  async executeOptimization(optimization) {
    try {
      console.log(`[PerformanceMonitor] Executing optimization: ${optimization.type}`);
      
      let result = false;
      
      switch (optimization.type) {
        case 'memory_cleanup':
          result = await this.executeMemoryCleanup();
          break;
          
        case 'cache_optimization':
          result = await this.executeCacheOptimization();
          break;
          
        case 'resource_preloading':
          result = await this.executeResourcePreloading();
          break;
          
        case 'wasm_compression':
          result = await this.executeWasmCompression();
          break;
          
        default:
          console.warn(`[PerformanceMonitor] Unknown optimization type: ${optimization.type}`);
      }
      
      if (result) {
        this.optimizations.push({
          ...optimization,
          executedAt: Date.now(),
          success: true
        });
        
        this.emit('optimizationExecuted', { optimization, success: true });
      }
      
    } catch (error) {
      console.error(`[PerformanceMonitor] Optimization failed: ${optimization.type}`, error);
      
      this.optimizations.push({
        ...optimization,
        executedAt: Date.now(),
        success: false,
        error: error.message
      });
      
      this.emit('optimizationExecuted', { optimization, success: false, error });
    }
  }

  /**
   * Execute memory cleanup optimization
   */
  async executeMemoryCleanup() {
    try {
      // Trigger garbage collection if available
      if (window.gc && typeof window.gc === 'function') {
        window.gc();
      }
      
      // Clear unnecessary caches
      if ('caches' in window) {
        const cacheNames = await caches.keys();
        const oldCaches = cacheNames.filter(name => 
          name.includes('old-') || name.includes('temp-')
        );
        
        for (const cacheName of oldCaches) {
          await caches.delete(cacheName);
        }
      }
      
      // Clean up large objects from memory monitor
      this.memoryMonitor.cleanup();
      
      console.log('[PerformanceMonitor] Memory cleanup executed');
      return true;
      
    } catch (error) {
      console.error('[PerformanceMonitor] Memory cleanup failed:', error);
      return false;
    }
  }

  /**
   * Execute cache optimization
   */
  async executeCacheOptimization() {
    try {
      // Optimize Service Worker cache strategies
      if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
        navigator.serviceWorker.controller.postMessage({
          type: 'OPTIMIZE_CACHE_STRATEGY'
        });
      }
      
      // Preload critical resources
      const criticalResources = [
        '/styles/critical.css',
        '/scripts/core.js',
        '/core/wasm/pkg/writemagic_wasm_bg.wasm'
      ];
      
      for (const resource of criticalResources) {
        try {
          fetch(resource, { cache: 'force-cache' });
        } catch (error) {
          console.debug(`[PerformanceMonitor] Failed to preload ${resource}:`, error);
        }
      }
      
      console.log('[PerformanceMonitor] Cache optimization executed');
      return true;
      
    } catch (error) {
      console.error('[PerformanceMonitor] Cache optimization failed:', error);
      return false;
    }
  }

  /**
   * Execute resource preloading optimization  
   */
  async executeResourcePreloading() {
    try {
      // Preload based on user behavior patterns
      const predictedResources = this.predictNextResources();
      
      for (const resource of predictedResources) {
        const link = document.createElement('link');
        link.rel = 'prefetch';
        link.href = resource;
        document.head.appendChild(link);
      }
      
      console.log(`[PerformanceMonitor] Preloading ${predictedResources.length} resources`);
      return true;
      
    } catch (error) {
      console.error('[PerformanceMonitor] Resource preloading failed:', error);
      return false;
    }
  }

  /**
   * Execute WASM compression optimization
   */
  async executeWasmCompression() {
    try {
      // Signal for WASM recompilation with better compression
      if (window.WriteMagicLoader) {
        window.WriteMagicLoader.requestRecompilation?.({
          optimizationLevel: 'aggressive',
          compressionEnabled: true
        });
      }
      
      console.log('[PerformanceMonitor] WASM compression optimization requested');
      return true;
      
    } catch (error) {
      console.error('[PerformanceMonitor] WASM compression failed:', error);
      return false;
    }
  }

  /**
   * Predict next resources to preload
   */
  predictNextResources() {
    // Simple prediction based on common usage patterns
    return [
      '/scripts/ai-integration.js',
      '/scripts/document-editor.js',
      '/scripts/project-manager.js',
      '/styles/editor.css'
    ];
  }

  /**
   * Initialize Performance Observer API
   */
  initializePerformanceObserver() {
    if (typeof PerformanceObserver === 'undefined') return;
    
    try {
      // Observe navigation timing
      this.performanceObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        
        for (const entry of entries) {
          this.processPerformanceEntry(entry);
        }
      });
      
      // Observe different entry types
      const entryTypes = ['navigation', 'paint', 'largest-contentful-paint', 'layout-shift'];
      
      for (const type of entryTypes) {
        try {
          this.performanceObserver.observe({ entryTypes: [type] });
        } catch (error) {
          console.debug(`[PerformanceMonitor] Cannot observe ${type}:`, error);
        }
      }
      
    } catch (error) {
      console.warn('[PerformanceMonitor] Performance Observer initialization failed:', error);
    }
  }

  /**
   * Process performance entry from observer
   */
  processPerformanceEntry(entry) {
    const timestamp = Date.now();
    
    if (entry.entryType === 'paint') {
      this.emit('paintTiming', {
        name: entry.name,
        startTime: entry.startTime,
        timestamp
      });
      
      // Check against targets
      if (entry.name === 'first-paint' && entry.startTime > PERFORMANCE_TARGETS.FIRST_PAINT) {
        this.addAlert(this.createAlert(
          ALERT_LEVELS.WARNING,
          'First paint time exceeds target',
          `First paint: ${entry.startTime.toFixed(1)}ms (target: ${PERFORMANCE_TARGETS.FIRST_PAINT}ms)`,
          METRIC_CATEGORIES.LOADING
        ));
      }
    }
    
    if (entry.entryType === 'largest-contentful-paint') {
      this.emit('lcpTiming', {
        startTime: entry.startTime,
        timestamp
      });
    }
    
    if (entry.entryType === 'layout-shift' && !entry.hadRecentInput) {
      this.emit('layoutShift', {
        value: entry.value,
        timestamp
      });
    }
  }

  /**
   * Clean up old metrics data
   */
  cleanupOldMetrics() {
    const retentionTime = this.options.retentionDays * 24 * 60 * 60 * 1000;
    const cutoffTime = Date.now() - retentionTime;
    
    for (const [timestamp] of this.metrics) {
      if (timestamp < cutoffTime) {
        this.metrics.delete(timestamp);
      }
    }
    
    // Clean up alerts
    this.alerts = this.alerts.filter(alert => 
      Date.now() - alert.timestamp < retentionTime
    );
    
    // Clean up regressions  
    this.regressions = this.regressions.filter(regression =>
      Date.now() - regression.detectedAt < retentionTime
    );
  }

  /**
   * Load historical performance data
   */
  async loadHistoricalData() {
    try {
      const stored = localStorage.getItem('writemagic_performance_data');
      if (stored) {
        const data = JSON.parse(stored);
        
        if (data.metrics) {
          // Convert stored metrics back to Map
          this.metrics = new Map(data.metrics);
        }
        
        if (data.alerts) {
          this.alerts = data.alerts;
        }
        
        if (data.regressions) {
          this.regressions = data.regressions;
        }
      }
    } catch (error) {
      console.warn('[PerformanceMonitor] Failed to load historical data:', error);
    }
  }

  /**
   * Save performance data
   */
  savePerformanceData() {
    try {
      const data = {
        metrics: Array.from(this.metrics.entries()),
        alerts: this.alerts.slice(-50), // Keep only recent alerts
        regressions: this.regressions.slice(-20), // Keep only recent regressions
        savedAt: Date.now()
      };
      
      localStorage.setItem('writemagic_performance_data', JSON.stringify(data));
    } catch (error) {
      console.warn('[PerformanceMonitor] Failed to save performance data:', error);
    }
  }

  /**
   * Get performance dashboard data
   */
  getDashboardData() {
    const recentMetrics = Array.from(this.metrics.entries()).slice(-20);
    const activeAlerts = this.alerts.filter(a => !a.acknowledged);
    const recentOptimizations = this.optimizations.slice(-10);
    
    return {
      overview: this.generateOverview(),
      recentMetrics: recentMetrics.map(([timestamp, metrics]) => ({
        timestamp,
        ...metrics
      })),
      activeAlerts,
      recentRegressions: this.regressions.slice(-5),
      recentOptimizations,
      targets: PERFORMANCE_TARGETS
    };
  }

  /**
   * Generate performance overview
   */
  generateOverview() {
    if (this.metrics.size === 0) {
      return {
        status: 'no_data',
        summary: 'No performance data available'
      };
    }
    
    const latestMetrics = Array.from(this.metrics.values()).pop();
    const criticalAlerts = this.alerts.filter(a => 
      !a.acknowledged && a.level === ALERT_LEVELS.CRITICAL
    ).length;
    
    let status = 'good';
    if (criticalAlerts > 0) {
      status = 'critical';
    } else if (this.regressions.filter(r => Date.now() - r.detectedAt < 300000).length > 0) {
      status = 'warning';
    }
    
    return {
      status,
      criticalAlerts,
      recentRegressions: this.regressions.filter(r => Date.now() - r.detectedAt < 300000).length,
      lastMeasurement: this.lastMeasurement,
      isMonitoring: this.isMonitoring,
      summary: this.generatePerformanceSummary(latestMetrics)
    };
  }

  /**
   * Generate performance summary text
   */
  generatePerformanceSummary(metrics) {
    if (!metrics) return 'No recent performance data';
    
    const summaryParts = [];
    
    if (metrics.loading?.initialLoadTime) {
      const loadTime = metrics.loading.initialLoadTime;
      const target = PERFORMANCE_TARGETS.INITIAL_LOAD;
      const status = loadTime <= target ? 'good' : 'needs improvement';
      summaryParts.push(`Load time: ${loadTime}ms (${status})`);
    }
    
    if (metrics.wasm?.bundleSize) {
      const size = this.formatBytes(metrics.wasm.bundleSize);
      const target = this.formatBytes(PERFORMANCE_TARGETS.WASM_BUNDLE_SIZE);
      const status = metrics.wasm.bundleSize <= PERFORMANCE_TARGETS.WASM_BUNDLE_SIZE ? 'good' : 'large';
      summaryParts.push(`WASM size: ${size} (${status})`);
    }
    
    if (metrics.runtime?.averageFrameRate) {
      const fps = metrics.runtime.averageFrameRate.toFixed(1);
      const status = metrics.runtime.averageFrameRate >= PERFORMANCE_TARGETS.FRAME_RATE * 0.8 ? 'smooth' : 'choppy';
      summaryParts.push(`Frame rate: ${fps}fps (${status})`);
    }
    
    return summaryParts.join(', ') || 'Performance data collected';
  }

  /**
   * Format bytes for display
   */
  formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  /**
   * Export performance data for analysis
   */
  exportPerformanceData(format = 'json') {
    const data = {
      metadata: {
        exportedAt: Date.now(),
        version: '1.0',
        timeRange: {
          start: Math.min(...Array.from(this.metrics.keys())),
          end: Math.max(...Array.from(this.metrics.keys()))
        }
      },
      metrics: Array.from(this.metrics.entries()),
      alerts: this.alerts,
      regressions: this.regressions,
      optimizations: this.optimizations
    };
    
    if (format === 'csv') {
      return this.convertToCSV(data);
    }
    
    return JSON.stringify(data, null, 2);
  }

  /**
   * Convert data to CSV format
   */
  convertToCSV(data) {
    // Simplified CSV conversion - could be enhanced based on needs
    let csv = 'timestamp,category,metric,value\n';
    
    for (const [timestamp, metrics] of data.metrics) {
      for (const [category, categoryMetrics] of Object.entries(metrics)) {
        if (typeof categoryMetrics === 'object') {
          for (const [metric, value] of Object.entries(categoryMetrics)) {
            if (typeof value === 'number') {
              csv += `${timestamp},${category},${metric},${value}\n`;
            }
          }
        }
      }
    }
    
    return csv;
  }

  /**
   * Cleanup and destroy monitor
   */
  destroy() {
    this.stopMonitoring();
    
    if (this.performanceObserver) {
      this.performanceObserver.disconnect();
    }
    
    // Save final data
    this.savePerformanceData();
    
    // Clean up specialized monitors
    this.wasmMonitor.destroy();
    this.loadingMonitor.destroy();
    this.runtimeMonitor.destroy();
    this.memoryMonitor.destroy();
    this.networkMonitor.destroy();
    this.uxMonitor.destroy();
    
    console.log('[PerformanceMonitor] Performance monitor destroyed');
    this.emit('destroyed');
  }
}

// Specialized monitor classes would be implemented here...
// For brevity, I'll include just the class stubs:

class WasmPerformanceMonitor {
  async initialize() { /* WASM-specific initialization */ }
  startMonitoring() { /* Start WASM monitoring */ }
  stopMonitoring() { /* Stop WASM monitoring */ }
  async collectMetrics() { return {}; /* WASM metrics collection */ }
  destroy() { /* Cleanup */ }
}

class LoadingPerformanceMonitor {
  async initialize() { /* Loading performance initialization */ }
  startMonitoring() { /* Start loading monitoring */ }
  stopMonitoring() { /* Stop loading monitoring */ }
  async collectMetrics() { return {}; /* Loading metrics collection */ }
  destroy() { /* Cleanup */ }
}

class RuntimePerformanceMonitor {
  async initialize() { /* Runtime performance initialization */ }
  startMonitoring() { /* Start runtime monitoring */ }
  stopMonitoring() { /* Stop runtime monitoring */ }
  async collectMetrics() { return {}; /* Runtime metrics collection */ }
  destroy() { /* Cleanup */ }
}

class MemoryPerformanceMonitor {
  async initialize() { /* Memory monitoring initialization */ }
  startMonitoring() { /* Start memory monitoring */ }
  stopMonitoring() { /* Stop memory monitoring */ }
  async collectMetrics() { return {}; /* Memory metrics collection */ }
  cleanup() { /* Memory cleanup */ }
  destroy() { /* Cleanup */ }
}

class NetworkPerformanceMonitor {
  async initialize() { /* Network monitoring initialization */ }
  startMonitoring() { /* Start network monitoring */ }
  stopMonitoring() { /* Stop network monitoring */ }
  async collectMetrics() { return {}; /* Network metrics collection */ }
  destroy() { /* Cleanup */ }
}

class UserExperienceMonitor {
  async initialize() { /* UX monitoring initialization */ }
  startMonitoring() { /* Start UX monitoring */ }
  stopMonitoring() { /* Stop UX monitoring */ }
  async collectMetrics() { return {}; /* UX metrics collection */ }
  destroy() { /* Cleanup */ }
}

export default PerformanceMonitor;