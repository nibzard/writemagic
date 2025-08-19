/**
 * Performance Dashboard for WriteMagic
 * 
 * Provides a comprehensive dashboard for monitoring WASM bundle sizes,
 * loading performance, memory usage, and user experience metrics.
 * 
 * Features:
 * - Real-time performance metrics visualization
 * - WASM bundle size tracking with alerts
 * - Loading time optimization monitoring
 * - Memory usage trends and analysis
 * - Network performance adaptation
 * - User experience impact measurement
 * - Performance regression detection
 * - Automated optimization recommendations
 */

import PerformanceMonitor from './utils/performance-monitor.js';
import { EventEmitter } from './utils/event-emitter.js';

/**
 * Dashboard configuration
 */
const DASHBOARD_CONFIG = {
  updateInterval: 2000,
  maxDataPoints: 100,
  alertThresholds: {
    loadTime: 3000,
    wasmSize: 2 * 1024 * 1024,
    memoryGrowth: 10 * 1024 * 1024,
    frameRate: 45
  },
  colors: {
    primary: '#4f46e5',
    success: '#059669',
    warning: '#d97706',
    error: '#dc2626',
    info: '#3b82f6'
  }
};

/**
 * Performance Dashboard
 */
export class PerformanceDashboard extends EventEmitter {
  constructor(container, options = {}) {
    super();
    
    this.container = typeof container === 'string' ? 
      document.querySelector(container) : container;
    
    if (!this.container) {
      throw new Error('Dashboard container not found');
    }
    
    this.options = {
      ...DASHBOARD_CONFIG,
      ...options
    };
    
    // Performance monitor integration
    this.performanceMonitor = null;
    this.isVisible = false;
    this.updateTimer = null;
    
    // Chart instances
    this.charts = {
      loadTime: null,
      bundleSize: null,
      memoryUsage: null,
      frameRate: null,
      networkQuality: null
    };
    
    // Dashboard state
    this.dashboardData = {
      metrics: [],
      alerts: [],
      recommendations: [],
      summary: {}
    };
    
    this.initializeDashboard();
  }

  /**
   * Initialize the performance dashboard
   */
  async initializeDashboard() {
    try {
      console.log('[PerformanceDashboard] Initializing performance dashboard...');
      
      // Create dashboard UI
      this.createDashboardUI();
      
      // Initialize performance monitor if not provided
      if (!this.performanceMonitor) {
        this.performanceMonitor = new PerformanceMonitor({
          enableRealTimeMonitoring: true,
          enableAutoOptimization: true,
          enableRegressionDetection: true
        });
      }
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Initialize charts
      this.initializeCharts();
      
      // Start data collection
      this.startDataCollection();
      
      console.log('[PerformanceDashboard] Dashboard initialized successfully');
      this.emit('initialized');
      
    } catch (error) {
      console.error('[PerformanceDashboard] Initialization failed:', error);
      this.emit('initializationFailed', error);
    }
  }

  /**
   * Create the dashboard UI structure
   */
  createDashboardUI() {
    this.container.innerHTML = `
      <div class="performance-dashboard">
        <!-- Dashboard Header -->
        <div class="dashboard-header">
          <div class="header-title">
            <h2>Performance Monitor</h2>
            <div class="status-indicator" id="dashboard-status">
              <span class="status-dot"></span>
              <span class="status-text">Monitoring</span>
            </div>
          </div>
          
          <div class="header-controls">
            <button class="control-btn" id="dashboard-pause" title="Pause monitoring">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <rect x="6" y="4" width="1.5" height="8"/>
                <rect x="8.5" y="4" width="1.5" height="8"/>
              </svg>
            </button>
            <button class="control-btn" id="dashboard-export" title="Export data">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M8.5 1.5V11h-1V1.5L6 3l-.7-.7L8 .6l2.7 1.7L10 3 8.5 1.5zM14 8v6a1 1 0 01-1 1H3a1 1 0 01-1-1V8h2v5h8V8h2z"/>
              </svg>
            </button>
            <button class="control-btn" id="dashboard-close" title="Close dashboard">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M4 4l8 8M12 4l-8 8"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Summary Cards -->
        <div class="dashboard-summary">
          <div class="summary-card" id="load-time-card">
            <div class="card-header">
              <span class="card-title">Load Time</span>
              <span class="card-trend" id="load-time-trend"></span>
            </div>
            <div class="card-value" id="load-time-value">-</div>
            <div class="card-target">Target: &lt;3s</div>
          </div>

          <div class="summary-card" id="wasm-size-card">
            <div class="card-header">
              <span class="card-title">WASM Bundle</span>
              <span class="card-trend" id="wasm-size-trend"></span>
            </div>
            <div class="card-value" id="wasm-size-value">-</div>
            <div class="card-target">Target: &lt;2MB</div>
          </div>

          <div class="summary-card" id="memory-usage-card">
            <div class="card-header">
              <span class="card-title">Memory Usage</span>
              <span class="card-trend" id="memory-trend"></span>
            </div>
            <div class="card-value" id="memory-value">-</div>
            <div class="card-target">Growth: &lt;10MB/h</div>
          </div>

          <div class="summary-card" id="frame-rate-card">
            <div class="card-header">
              <span class="card-title">Frame Rate</span>
              <span class="card-trend" id="frame-rate-trend"></span>
            </div>
            <div class="card-value" id="frame-rate-value">-</div>
            <div class="card-target">Target: 60 FPS</div>
          </div>
        </div>

        <!-- Charts Section -->
        <div class="dashboard-charts">
          <div class="chart-container">
            <div class="chart-header">
              <h3>Loading Performance</h3>
              <div class="chart-controls">
                <select id="load-chart-timerange">
                  <option value="1h">Last Hour</option>
                  <option value="6h">Last 6 Hours</option>
                  <option value="24h" selected>Last 24 Hours</option>
                  <option value="7d">Last 7 Days</option>
                </select>
              </div>
            </div>
            <canvas id="load-time-chart" width="400" height="200"></canvas>
          </div>

          <div class="chart-container">
            <div class="chart-header">
              <h3>WASM Bundle Analysis</h3>
              <div class="chart-controls">
                <button class="chart-btn active" data-metric="size">Size</button>
                <button class="chart-btn" data-metric="compilation">Compilation</button>
              </div>
            </div>
            <canvas id="wasm-chart" width="400" height="200"></canvas>
          </div>

          <div class="chart-container">
            <div class="chart-header">
              <h3>Memory Performance</h3>
              <div class="chart-controls">
                <button class="chart-btn active" data-metric="heap">JS Heap</button>
                <button class="chart-btn" data-metric="wasm">WASM Memory</button>
              </div>
            </div>
            <canvas id="memory-chart" width="400" height="200"></canvas>
          </div>

          <div class="chart-container">
            <div class="chart-header">
              <h3>Runtime Performance</h3>
              <div class="chart-controls">
                <button class="chart-btn active" data-metric="fps">Frame Rate</button>
                <button class="chart-btn" data-metric="latency">Input Latency</button>
              </div>
            </div>
            <canvas id="runtime-chart" width="400" height="200"></canvas>
          </div>
        </div>

        <!-- Alerts and Recommendations -->
        <div class="dashboard-insights">
          <div class="insights-section">
            <div class="section-header">
              <h3>Active Alerts</h3>
              <span class="alert-count" id="alert-count">0</span>
            </div>
            <div class="alerts-list" id="alerts-list">
              <div class="no-alerts">No active alerts</div>
            </div>
          </div>

          <div class="insights-section">
            <div class="section-header">
              <h3>Optimization Recommendations</h3>
              <button class="apply-all-btn" id="apply-recommendations">Apply All</button>
            </div>
            <div class="recommendations-list" id="recommendations-list">
              <div class="no-recommendations">All optimizations applied</div>
            </div>
          </div>
        </div>

        <!-- Detailed Metrics Table -->
        <div class="dashboard-details">
          <div class="details-header">
            <h3>Detailed Metrics</h3>
            <div class="table-controls">
              <input type="search" placeholder="Filter metrics..." id="metrics-filter">
              <button class="control-btn" id="refresh-metrics">Refresh</button>
            </div>
          </div>
          <div class="metrics-table-container">
            <table class="metrics-table" id="metrics-table">
              <thead>
                <tr>
                  <th>Metric</th>
                  <th>Current</th>
                  <th>Target</th>
                  <th>Trend</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody id="metrics-table-body">
                <!-- Populated dynamically -->
              </tbody>
            </table>
          </div>
        </div>
      </div>
    `;

    // Add dashboard styles
    this.addDashboardStyles();
  }

  /**
   * Add dashboard-specific styles
   */
  addDashboardStyles() {
    const styleId = 'performance-dashboard-styles';
    
    if (document.getElementById(styleId)) return;
    
    const styles = document.createElement('style');
    styles.id = styleId;
    styles.textContent = `
      .performance-dashboard {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: #f8fafc;
        border-radius: 8px;
        padding: 1rem;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        max-width: 1200px;
        margin: 0 auto;
      }

      .dashboard-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
        padding-bottom: 1rem;
        border-bottom: 1px solid #e5e7eb;
      }

      .header-title {
        display: flex;
        align-items: center;
        gap: 1rem;
      }

      .header-title h2 {
        margin: 0;
        font-size: 1.5rem;
        font-weight: 600;
        color: #1f2937;
      }

      .status-indicator {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.875rem;
        color: #6b7280;
      }

      .status-dot {
        width: 8px;
        height: 8px;
        background: ${DASHBOARD_CONFIG.colors.success};
        border-radius: 50%;
        animation: pulse 2s infinite;
      }

      .header-controls {
        display: flex;
        gap: 0.5rem;
      }

      .control-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border: 1px solid #d1d5db;
        background: white;
        border-radius: 6px;
        cursor: pointer;
        color: #6b7280;
        transition: all 0.2s;
      }

      .control-btn:hover {
        background: #f3f4f6;
        border-color: #9ca3af;
        color: #374151;
      }

      .dashboard-summary {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
        gap: 1rem;
        margin-bottom: 2rem;
      }

      .summary-card {
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        border: 1px solid #e5e7eb;
        transition: all 0.2s;
      }

      .summary-card:hover {
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
        transform: translateY(-1px);
      }

      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
      }

      .card-title {
        font-size: 0.875rem;
        font-weight: 500;
        color: #6b7280;
      }

      .card-trend {
        font-size: 0.75rem;
        font-weight: 500;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        background: #f3f4f6;
        color: #6b7280;
      }

      .card-trend.positive {
        background: #dcfce7;
        color: #166534;
      }

      .card-trend.negative {
        background: #fef2f2;
        color: #991b1b;
      }

      .card-value {
        font-size: 2rem;
        font-weight: 700;
        color: #1f2937;
        margin-bottom: 0.25rem;
      }

      .card-target {
        font-size: 0.75rem;
        color: #9ca3af;
      }

      .dashboard-charts {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
        gap: 1.5rem;
        margin-bottom: 2rem;
      }

      .chart-container {
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        border: 1px solid #e5e7eb;
      }

      .chart-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
      }

      .chart-header h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
        color: #1f2937;
      }

      .chart-controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
      }

      .chart-btn {
        padding: 0.25rem 0.75rem;
        font-size: 0.75rem;
        border: 1px solid #d1d5db;
        background: white;
        border-radius: 4px;
        cursor: pointer;
        transition: all 0.2s;
      }

      .chart-btn.active,
      .chart-btn:hover {
        background: ${DASHBOARD_CONFIG.colors.primary};
        color: white;
        border-color: ${DASHBOARD_CONFIG.colors.primary};
      }

      .dashboard-insights {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
        margin-bottom: 2rem;
      }

      .insights-section {
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        border: 1px solid #e5e7eb;
      }

      .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
      }

      .section-header h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
        color: #1f2937;
      }

      .alert-count {
        background: ${DASHBOARD_CONFIG.colors.error};
        color: white;
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0.25rem 0.5rem;
        border-radius: 12px;
        min-width: 20px;
        text-align: center;
      }

      .apply-all-btn {
        padding: 0.5rem 1rem;
        font-size: 0.875rem;
        background: ${DASHBOARD_CONFIG.colors.primary};
        color: white;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s;
      }

      .apply-all-btn:hover {
        background: #4338ca;
      }

      .alerts-list,
      .recommendations-list {
        max-height: 200px;
        overflow-y: auto;
      }

      .alert-item,
      .recommendation-item {
        padding: 0.75rem;
        margin-bottom: 0.5rem;
        border-radius: 6px;
        border-left: 4px solid transparent;
      }

      .alert-item {
        background: #fef2f2;
        border-left-color: ${DASHBOARD_CONFIG.colors.error};
      }

      .alert-item.warning {
        background: #fefbf2;
        border-left-color: ${DASHBOARD_CONFIG.colors.warning};
      }

      .recommendation-item {
        background: #f0f9ff;
        border-left-color: ${DASHBOARD_CONFIG.colors.info};
        cursor: pointer;
        transition: all 0.2s;
      }

      .recommendation-item:hover {
        background: #e0f2fe;
        transform: translateX(2px);
      }

      .no-alerts,
      .no-recommendations {
        text-align: center;
        color: #9ca3af;
        font-style: italic;
        padding: 2rem;
      }

      .dashboard-details {
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        border: 1px solid #e5e7eb;
      }

      .details-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
      }

      .details-header h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
        color: #1f2937;
      }

      .table-controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
      }

      .table-controls input {
        padding: 0.5rem;
        border: 1px solid #d1d5db;
        border-radius: 6px;
        font-size: 0.875rem;
      }

      .metrics-table-container {
        overflow-x: auto;
      }

      .metrics-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.875rem;
      }

      .metrics-table th,
      .metrics-table td {
        text-align: left;
        padding: 0.75rem;
        border-bottom: 1px solid #e5e7eb;
      }

      .metrics-table th {
        font-weight: 600;
        color: #374151;
        background: #f9fafb;
      }

      .metric-status {
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.75rem;
        font-weight: 500;
      }

      .metric-status.good {
        background: #dcfce7;
        color: #166534;
      }

      .metric-status.warning {
        background: #fefbf2;
        color: #92400e;
      }

      .metric-status.critical {
        background: #fef2f2;
        color: #991b1b;
      }

      @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
      }

      @media (max-width: 768px) {
        .dashboard-summary {
          grid-template-columns: 1fr;
        }
        
        .dashboard-charts {
          grid-template-columns: 1fr;
        }
        
        .dashboard-insights {
          grid-template-columns: 1fr;
        }
        
        .chart-container {
          padding: 1rem;
        }
      }
    `;
    
    document.head.appendChild(styles);
  }

  /**
   * Initialize chart components
   */
  initializeCharts() {
    // This would integrate with a charting library like Chart.js
    // For now, we'll create placeholders that can be enhanced
    
    this.charts.loadTime = this.createChart('load-time-chart', 'line');
    this.charts.bundleSize = this.createChart('wasm-chart', 'bar');
    this.charts.memoryUsage = this.createChart('memory-chart', 'area');
    this.charts.frameRate = this.createChart('runtime-chart', 'line');
    
    console.log('[PerformanceDashboard] Charts initialized');
  }

  /**
   * Create a chart instance (placeholder implementation)
   */
  createChart(canvasId, type) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return null;
    
    // This would use Chart.js or similar library
    // For now, return a mock chart object
    return {
      canvas,
      type,
      data: [],
      update: (newData) => {
        // Update chart with new data
        console.debug(`[Chart:${canvasId}] Updating with`, newData);
      },
      render: () => {
        // Render chart
        this.renderChart(canvas, type);
      }
    };
  }

  /**
   * Basic chart rendering (placeholder)
   */
  renderChart(canvas, type) {
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Draw a simple placeholder chart
    ctx.strokeStyle = DASHBOARD_CONFIG.colors.primary;
    ctx.lineWidth = 2;
    ctx.beginPath();
    
    const width = canvas.width;
    const height = canvas.height;
    const points = 20;
    
    for (let i = 0; i < points; i++) {
      const x = (i / points) * width;
      const y = height/2 + Math.sin(i * 0.5) * (height/4);
      
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    
    ctx.stroke();
    
    // Add chart label
    ctx.fillStyle = '#6b7280';
    ctx.font = '12px -apple-system, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(`${type.toUpperCase()} Chart - Live Data Coming Soon`, width/2, height - 10);
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    // Performance monitor events
    if (this.performanceMonitor) {
      this.performanceMonitor.on('metricsCollected', (data) => {
        this.updateDashboardData(data);
      });
      
      this.performanceMonitor.on('alertCreated', (alert) => {
        this.addAlert(alert);
      });
      
      this.performanceMonitor.on('regressionDetected', (regression) => {
        this.handleRegression(regression);
      });
      
      this.performanceMonitor.on('optimizationExecuted', (data) => {
        this.updateOptimizationStatus(data);
      });
    }
    
    // Dashboard control events
    document.getElementById('dashboard-pause')?.addEventListener('click', () => {
      this.togglePause();
    });
    
    document.getElementById('dashboard-export')?.addEventListener('click', () => {
      this.exportData();
    });
    
    document.getElementById('dashboard-close')?.addEventListener('click', () => {
      this.hide();
    });
    
    document.getElementById('apply-recommendations')?.addEventListener('click', () => {
      this.applyAllRecommendations();
    });
    
    document.getElementById('refresh-metrics')?.addEventListener('click', () => {
      this.refreshMetrics();
    });
    
    // Chart control events
    const chartButtons = document.querySelectorAll('.chart-btn');
    chartButtons.forEach(btn => {
      btn.addEventListener('click', (e) => {
        this.switchChartMetric(e.target);
      });
    });
    
    console.log('[PerformanceDashboard] Event listeners setup complete');
  }

  /**
   * Start data collection and updates
   */
  startDataCollection() {
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
    }
    
    this.updateTimer = setInterval(() => {
      if (this.isVisible && !this.isPaused) {
        this.collectAndUpdateData();
      }
    }, this.options.updateInterval);
    
    // Initial data collection
    this.collectAndUpdateData();
    
    console.log('[PerformanceDashboard] Data collection started');
  }

  /**
   * Collect and update dashboard data
   */
  async collectAndUpdateData() {
    try {
      if (this.performanceMonitor) {
        await this.performanceMonitor.collectMetrics();
        const dashboardData = this.performanceMonitor.getDashboardData();
        this.updateUI(dashboardData);
      }
    } catch (error) {
      console.error('[PerformanceDashboard] Data collection failed:', error);
    }
  }

  /**
   * Update dashboard UI with new data
   */
  updateUI(data) {
    // Update summary cards
    this.updateSummaryCards(data);
    
    // Update charts
    this.updateCharts(data);
    
    // Update alerts
    this.updateAlerts(data.activeAlerts || []);
    
    // Update recommendations
    this.updateRecommendations(data.recentOptimizations || []);
    
    // Update metrics table
    this.updateMetricsTable(data.recentMetrics || []);
    
    // Update overall status
    this.updateStatus(data.overview || {});
  }

  /**
   * Update summary cards
   */
  updateSummaryCards(data) {
    const latest = data.recentMetrics?.[data.recentMetrics.length - 1];
    if (!latest) return;
    
    // Load time card
    if (latest.loading?.initialLoadTime) {
      const loadTime = latest.loading.initialLoadTime;
      const loadTimeElement = document.getElementById('load-time-value');
      const trendElement = document.getElementById('load-time-trend');
      
      if (loadTimeElement) {
        loadTimeElement.textContent = `${(loadTime / 1000).toFixed(2)}s`;
        
        const card = document.getElementById('load-time-card');
        if (loadTime > this.options.alertThresholds.loadTime) {
          card?.classList.add('warning');
          trendElement.textContent = 'SLOW';
          trendElement.className = 'card-trend negative';
        } else {
          card?.classList.remove('warning');
          trendElement.textContent = 'GOOD';
          trendElement.className = 'card-trend positive';
        }
      }
    }
    
    // WASM size card
    if (latest.wasm?.bundleSize) {
      const wasmSize = latest.wasm.bundleSize;
      const sizeElement = document.getElementById('wasm-size-value');
      const trendElement = document.getElementById('wasm-size-trend');
      
      if (sizeElement) {
        sizeElement.textContent = this.formatBytes(wasmSize);
        
        const card = document.getElementById('wasm-size-card');
        if (wasmSize > this.options.alertThresholds.wasmSize) {
          card?.classList.add('warning');
          trendElement.textContent = 'LARGE';
          trendElement.className = 'card-trend negative';
        } else {
          card?.classList.remove('warning');
          trendElement.textContent = 'GOOD';
          trendElement.className = 'card-trend positive';
        }
      }
    }
    
    // Memory usage card
    if (latest.memory?.heapUsagePercent) {
      const memoryPercent = latest.memory.heapUsagePercent;
      const memoryElement = document.getElementById('memory-value');
      const trendElement = document.getElementById('memory-trend');
      
      if (memoryElement) {
        memoryElement.textContent = `${memoryPercent.toFixed(1)}%`;
        
        const card = document.getElementById('memory-usage-card');
        if (memoryPercent > 70) {
          card?.classList.add('warning');
          trendElement.textContent = 'HIGH';
          trendElement.className = 'card-trend negative';
        } else {
          card?.classList.remove('warning');
          trendElement.textContent = 'GOOD';
          trendElement.className = 'card-trend positive';
        }
      }
    }
    
    // Frame rate card
    if (latest.runtime?.averageFrameRate) {
      const fps = latest.runtime.averageFrameRate;
      const fpsElement = document.getElementById('frame-rate-value');
      const trendElement = document.getElementById('frame-rate-trend');
      
      if (fpsElement) {
        fpsElement.textContent = `${fps.toFixed(1)}`;
        
        const card = document.getElementById('frame-rate-card');
        if (fps < this.options.alertThresholds.frameRate) {
          card?.classList.add('warning');
          trendElement.textContent = 'LOW';
          trendElement.className = 'card-trend negative';
        } else {
          card?.classList.remove('warning');
          trendElement.textContent = 'SMOOTH';
          trendElement.className = 'card-trend positive';
        }
      }
    }
  }

  /**
   * Update charts with new data
   */
  updateCharts(data) {
    if (!data.recentMetrics) return;
    
    // Update each chart with relevant data
    Object.values(this.charts).forEach(chart => {
      if (chart && chart.update) {
        chart.update(data.recentMetrics);
        chart.render();
      }
    });
  }

  /**
   * Update alerts list
   */
  updateAlerts(alerts) {
    const alertsList = document.getElementById('alerts-list');
    const alertCount = document.getElementById('alert-count');
    
    if (!alertsList || !alertCount) return;
    
    alertCount.textContent = alerts.length.toString();
    
    if (alerts.length === 0) {
      alertsList.innerHTML = '<div class="no-alerts">No active alerts</div>';
      return;
    }
    
    alertsList.innerHTML = alerts.map(alert => `
      <div class="alert-item ${alert.level}" data-alert-id="${alert.id}">
        <div class="alert-title">${alert.title}</div>
        <div class="alert-message">${alert.message}</div>
        <div class="alert-time">${this.formatTime(alert.timestamp)}</div>
      </div>
    `).join('');
  }

  /**
   * Update recommendations list
   */
  updateRecommendations(optimizations) {
    const recommendationsList = document.getElementById('recommendations-list');
    if (!recommendationsList) return;
    
    const pendingOptimizations = this.generateRecommendations();
    
    if (pendingOptimizations.length === 0) {
      recommendationsList.innerHTML = '<div class="no-recommendations">All optimizations applied</div>';
      return;
    }
    
    recommendationsList.innerHTML = pendingOptimizations.map(rec => `
      <div class="recommendation-item" data-recommendation="${rec.type}">
        <div class="recommendation-title">${rec.title}</div>
        <div class="recommendation-description">${rec.description}</div>
        <div class="recommendation-impact">Impact: ${rec.impact}</div>
      </div>
    `).join('');
  }

  /**
   * Generate optimization recommendations
   */
  generateRecommendations() {
    // This would analyze current performance and generate recommendations
    return [
      {
        type: 'wasm_compression',
        title: 'Enable WASM Compression',
        description: 'Reduce bundle size by 30-40% with aggressive compression',
        impact: 'High'
      },
      {
        type: 'preload_optimization',
        title: 'Optimize Resource Preloading',
        description: 'Implement intelligent preloading based on user behavior',
        impact: 'Medium'
      },
      {
        type: 'memory_management',
        title: 'Improve Memory Management',
        description: 'Implement more aggressive garbage collection',
        impact: 'Low'
      }
    ];
  }

  /**
   * Update metrics table
   */
  updateMetricsTable(metrics) {
    const tableBody = document.getElementById('metrics-table-body');
    if (!tableBody || !metrics.length) return;
    
    const latest = metrics[metrics.length - 1];
    const metricRows = [];
    
    // Flatten metrics for table display
    Object.entries(latest).forEach(([category, categoryMetrics]) => {
      if (typeof categoryMetrics === 'object' && categoryMetrics) {
        Object.entries(categoryMetrics).forEach(([metric, value]) => {
          if (typeof value === 'number') {
            metricRows.push({
              name: `${category}.${metric}`,
              current: this.formatMetricValue(metric, value),
              target: this.getMetricTarget(metric),
              trend: this.calculateTrend(metrics, category, metric),
              status: this.getMetricStatus(metric, value)
            });
          }
        });
      }
    });
    
    tableBody.innerHTML = metricRows.map(row => `
      <tr>
        <td>${row.name}</td>
        <td>${row.current}</td>
        <td>${row.target}</td>
        <td>${row.trend}</td>
        <td><span class="metric-status ${row.status.level}">${row.status.text}</span></td>
      </tr>
    `).join('');
  }

  /**
   * Format metric value for display
   */
  formatMetricValue(metric, value) {
    const lowerMetric = metric.toLowerCase();
    
    if (lowerMetric.includes('time') || lowerMetric.includes('latency')) {
      return `${value.toFixed(1)}ms`;
    } else if (lowerMetric.includes('size') || lowerMetric.includes('memory')) {
      return this.formatBytes(value);
    } else if (lowerMetric.includes('rate') || lowerMetric.includes('fps')) {
      return `${value.toFixed(1)}`;
    } else if (lowerMetric.includes('percent')) {
      return `${value.toFixed(1)}%`;
    }
    
    return value.toString();
  }

  /**
   * Get metric target value
   */
  getMetricTarget(metric) {
    const targets = {
      'initialLoadTime': '< 3000ms',
      'bundleSize': '< 2MB',
      'heapUsagePercent': '< 70%',
      'averageFrameRate': '> 45 FPS',
      'compilationTime': '< 2000ms'
    };
    
    return targets[metric] || '-';
  }

  /**
   * Calculate trend for metric
   */
  calculateTrend(metrics, category, metric) {
    if (metrics.length < 2) return '-';
    
    const recent = metrics.slice(-5);
    const values = recent.map(m => m[category]?.[metric]).filter(v => typeof v === 'number');
    
    if (values.length < 2) return '-';
    
    const first = values[0];
    const last = values[values.length - 1];
    const change = ((last - first) / first) * 100;
    
    if (Math.abs(change) < 1) return 'Stable';
    
    return change > 0 ? `+${change.toFixed(1)}%` : `${change.toFixed(1)}%`;
  }

  /**
   * Get metric status
   */
  getMetricStatus(metric, value) {
    // This would implement logic to determine if a metric value is good, warning, or critical
    // Based on the thresholds and metric type
    
    const thresholds = this.options.alertThresholds;
    
    if (metric === 'initialLoadTime') {
      if (value <= thresholds.loadTime) return { level: 'good', text: 'Good' };
      if (value <= thresholds.loadTime * 1.5) return { level: 'warning', text: 'Slow' };
      return { level: 'critical', text: 'Critical' };
    }
    
    if (metric === 'bundleSize') {
      if (value <= thresholds.wasmSize) return { level: 'good', text: 'Good' };
      if (value <= thresholds.wasmSize * 1.2) return { level: 'warning', text: 'Large' };
      return { level: 'critical', text: 'Too Large' };
    }
    
    if (metric === 'averageFrameRate') {
      if (value >= thresholds.frameRate) return { level: 'good', text: 'Smooth' };
      if (value >= thresholds.frameRate * 0.7) return { level: 'warning', text: 'Choppy' };
      return { level: 'critical', text: 'Poor' };
    }
    
    return { level: 'good', text: 'OK' };
  }

  /**
   * Show dashboard
   */
  show() {
    this.container.style.display = 'block';
    this.isVisible = true;
    
    if (!this.updateTimer) {
      this.startDataCollection();
    }
    
    this.emit('shown');
  }

  /**
   * Hide dashboard
   */
  hide() {
    this.container.style.display = 'none';
    this.isVisible = false;
    
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
      this.updateTimer = null;
    }
    
    this.emit('hidden');
  }

  /**
   * Toggle dashboard visibility
   */
  toggle() {
    if (this.isVisible) {
      this.hide();
    } else {
      this.show();
    }
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
   * Format time for display
   */
  formatTime(timestamp) {
    return new Date(timestamp).toLocaleTimeString();
  }

  /**
   * Export dashboard data
   */
  exportData() {
    if (this.performanceMonitor) {
      const data = this.performanceMonitor.exportPerformanceData('json');
      
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      
      const a = document.createElement('a');
      a.href = url;
      a.download = `writemagic-performance-${Date.now()}.json`;
      a.click();
      
      URL.revokeObjectURL(url);
    }
  }

  /**
   * Apply all recommendations
   */
  async applyAllRecommendations() {
    const recommendations = this.generateRecommendations();
    
    for (const rec of recommendations) {
      try {
        if (this.performanceMonitor) {
          await this.performanceMonitor.executeOptimization({
            type: rec.type,
            description: rec.description,
            impact: rec.impact.toLowerCase()
          });
        }
      } catch (error) {
        console.error(`Failed to apply optimization ${rec.type}:`, error);
      }
    }
    
    this.showNotification('All recommendations applied', 'success');
  }

  /**
   * Show notification
   */
  showNotification(message, type = 'info') {
    // Create and show a temporary notification
    const notification = document.createElement('div');
    notification.className = `dashboard-notification ${type}`;
    notification.textContent = message;
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 12px 20px;
      background: ${DASHBOARD_CONFIG.colors[type] || DASHBOARD_CONFIG.colors.info};
      color: white;
      border-radius: 6px;
      font-size: 14px;
      z-index: 10000;
      animation: slideIn 0.3s ease-out;
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
      notification.style.animation = 'slideOut 0.3s ease-in';
      setTimeout(() => notification.remove(), 300);
    }, 3000);
  }

  /**
   * Destroy dashboard
   */
  destroy() {
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
    }
    
    // Remove event listeners
    // Clean up charts
    Object.values(this.charts).forEach(chart => {
      if (chart && chart.destroy) {
        chart.destroy();
      }
    });
    
    // Clear container
    this.container.innerHTML = '';
    
    console.log('[PerformanceDashboard] Dashboard destroyed');
    this.emit('destroyed');
  }
}

export default PerformanceDashboard;