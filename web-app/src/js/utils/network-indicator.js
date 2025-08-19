/**
 * Network Status Indicator
 * 
 * Provides visual feedback about network connectivity, sync status,
 * and offline capabilities for the WriteMagic PWA.
 */

export class NetworkIndicator {
    constructor(container, serviceWorkerManager) {
        this.container = container;
        this.swManager = serviceWorkerManager;
        this.element = null;
        
        this.state = {
            isOnline: navigator.onLine,
            quality: 'unknown',
            syncStatus: 'idle', // 'idle', 'syncing', 'error'
            queuedItems: 0,
            offlineReady: false
        };
        
        this.init();
    }
    
    /**
     * Initialize the network indicator
     */
    init() {
        this.createElement();
        this.setupEventListeners();
        this.updateDisplay();
        
        // Check offline readiness
        this.checkOfflineReadiness();
    }
    
    /**
     * Create the indicator element
     */
    createElement() {
        this.element = document.createElement('div');
        this.element.className = 'network-indicator';
        this.element.innerHTML = `
            <div class="indicator-content">
                <div class="status-icon">
                    <div class="connection-dot"></div>
                    <div class="sync-spinner" style="display: none;">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" opacity="0.3"/>
                            <path d="M12 2a10 10 0 0110 10" stroke="currentColor" stroke-width="4"/>
                        </svg>
                    </div>
                </div>
                <div class="status-text">
                    <span class="status-label">Online</span>
                    <span class="status-detail">Good connection</span>
                </div>
                <div class="queue-badge" style="display: none;">
                    <span class="queue-count">0</span>
                </div>
            </div>
            <div class="indicator-dropdown" style="display: none;">
                <div class="dropdown-content">
                    <div class="network-info">
                        <h4>Connection Status</h4>
                        <div class="info-row">
                            <span class="info-label">Status:</span>
                            <span class="info-value online-status">Online</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Quality:</span>
                            <span class="info-value quality-status">Good</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Type:</span>
                            <span class="info-value connection-type">4G</span>
                        </div>
                    </div>
                    
                    <div class="sync-info">
                        <h4>Background Sync</h4>
                        <div class="info-row">
                            <span class="info-label">Status:</span>
                            <span class="info-value sync-status-text">Up to date</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Queued:</span>
                            <span class="info-value queued-count">0 items</span>
                        </div>
                        <button class="force-sync-btn" onclick="this.parentElement.parentElement.parentElement.parentElement.__networkIndicator.forceSync()">
                            Force Sync
                        </button>
                    </div>
                    
                    <div class="offline-info">
                        <h4>Offline Status</h4>
                        <div class="info-row">
                            <span class="info-label">Ready:</span>
                            <span class="info-value offline-ready-status">Checking...</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Cached:</span>
                            <span class="info-value cached-resources">0 resources</span>
                        </div>
                    </div>
                    
                    <div class="actions">
                        <button class="action-btn" onclick="this.parentElement.parentElement.parentElement.parentElement.__networkIndicator.showDiagnostics()">
                            View Diagnostics
                        </button>
                        <button class="action-btn secondary" onclick="this.parentElement.parentElement.parentElement.parentElement.__networkIndicator.clearCaches()">
                            Clear Caches
                        </button>
                    </div>
                </div>
            </div>
        `;
        
        // Add reference for event handlers
        this.element.__networkIndicator = this;
        
        this.container.appendChild(this.element);
        
        // Add styles
        this.addStyles();
    }
    
    /**
     * Add CSS styles for the indicator
     */
    addStyles() {
        if (document.getElementById('network-indicator-styles')) return;
        
        const styles = document.createElement('style');
        styles.id = 'network-indicator-styles';
        styles.textContent = `
            .network-indicator {
                position: relative;
                display: inline-flex;
                align-items: center;
                background: rgba(255, 255, 255, 0.9);
                border: 1px solid rgba(0, 0, 0, 0.1);
                border-radius: 20px;
                padding: 4px 12px;
                font-size: 0.8rem;
                cursor: pointer;
                transition: all 0.2s ease;
                backdrop-filter: blur(10px);
                user-select: none;
                z-index: 1000;
            }
            
            .network-indicator:hover {
                background: rgba(255, 255, 255, 0.95);
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                transform: translateY(-1px);
            }
            
            .indicator-content {
                display: flex;
                align-items: center;
                gap: 6px;
            }
            
            .status-icon {
                position: relative;
                width: 16px;
                height: 16px;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            
            .connection-dot {
                width: 8px;
                height: 8px;
                border-radius: 50%;
                background: #10b981;
                transition: background-color 0.2s ease;
            }
            
            .connection-dot.offline {
                background: #ef4444;
                animation: pulse-red 2s infinite;
            }
            
            .connection-dot.slow {
                background: #f59e0b;
                animation: pulse-yellow 2s infinite;
            }
            
            .connection-dot.moderate {
                background: #3b82f6;
            }
            
            @keyframes pulse-red {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.5; }
            }
            
            @keyframes pulse-yellow {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
            
            .sync-spinner {
                position: absolute;
                top: 0;
                left: 0;
                width: 16px;
                height: 16px;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            
            .sync-spinner svg {
                animation: spin 1s linear infinite;
            }
            
            @keyframes spin {
                from { transform: rotate(0deg); }
                to { transform: rotate(360deg); }
            }
            
            .status-text {
                display: flex;
                flex-direction: column;
                line-height: 1.2;
            }
            
            .status-label {
                font-weight: 500;
                color: #374151;
            }
            
            .status-detail {
                font-size: 0.7rem;
                color: #6b7280;
            }
            
            .queue-badge {
                background: #ef4444;
                color: white;
                border-radius: 10px;
                padding: 2px 6px;
                font-size: 0.7rem;
                font-weight: 500;
                min-width: 16px;
                text-align: center;
                margin-left: 4px;
                animation: bounce 0.5s ease-in-out;
            }
            
            @keyframes bounce {
                0%, 100% { transform: scale(1); }
                50% { transform: scale(1.2); }
            }
            
            .indicator-dropdown {
                position: absolute;
                top: 100%;
                right: 0;
                margin-top: 4px;
                background: white;
                border: 1px solid rgba(0, 0, 0, 0.1);
                border-radius: 8px;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
                min-width: 280px;
                z-index: 1001;
                animation: dropdownSlide 0.2s ease-out;
            }
            
            @keyframes dropdownSlide {
                from {
                    opacity: 0;
                    transform: translateY(-10px);
                }
                to {
                    opacity: 1;
                    transform: translateY(0);
                }
            }
            
            .dropdown-content {
                padding: 16px;
            }
            
            .dropdown-content h4 {
                margin: 0 0 8px 0;
                font-size: 0.9rem;
                font-weight: 600;
                color: #374151;
                border-bottom: 1px solid #e5e7eb;
                padding-bottom: 4px;
            }
            
            .dropdown-content h4:not(:first-child) {
                margin-top: 16px;
            }
            
            .info-row {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 4px;
                font-size: 0.8rem;
            }
            
            .info-label {
                color: #6b7280;
            }
            
            .info-value {
                color: #374151;
                font-weight: 500;
            }
            
            .info-value.online-status.offline {
                color: #ef4444;
            }
            
            .info-value.quality-status.slow {
                color: #f59e0b;
            }
            
            .info-value.quality-status.moderate {
                color: #3b82f6;
            }
            
            .info-value.quality-status.good {
                color: #10b981;
            }
            
            .actions {
                margin-top: 16px;
                padding-top: 12px;
                border-top: 1px solid #e5e7eb;
                display: flex;
                gap: 8px;
                flex-wrap: wrap;
            }
            
            .action-btn, .force-sync-btn {
                background: #4f46e5;
                color: white;
                border: none;
                border-radius: 4px;
                padding: 6px 12px;
                font-size: 0.8rem;
                cursor: pointer;
                transition: background-color 0.2s ease;
                flex: 1;
                min-width: 80px;
            }
            
            .action-btn:hover, .force-sync-btn:hover {
                background: #4338ca;
            }
            
            .action-btn.secondary {
                background: #6b7280;
            }
            
            .action-btn.secondary:hover {
                background: #4b5563;
            }
            
            .force-sync-btn {
                background: #059669;
                margin-top: 8px;
                width: 100%;
                flex: none;
            }
            
            .force-sync-btn:hover {
                background: #047857;
            }
            
            .force-sync-btn:disabled {
                background: #9ca3af;
                cursor: not-allowed;
            }
            
            /* Dark theme support */
            @media (prefers-color-scheme: dark) {
                .network-indicator {
                    background: rgba(31, 41, 55, 0.9);
                    border-color: rgba(255, 255, 255, 0.1);
                }
                
                .network-indicator:hover {
                    background: rgba(31, 41, 55, 0.95);
                }
                
                .status-label {
                    color: #f3f4f6;
                }
                
                .status-detail {
                    color: #9ca3af;
                }
                
                .indicator-dropdown {
                    background: #1f2937;
                    border-color: rgba(255, 255, 255, 0.1);
                }
                
                .dropdown-content h4 {
                    color: #f3f4f6;
                    border-color: #374151;
                }
                
                .info-label {
                    color: #9ca3af;
                }
                
                .info-value {
                    color: #f3f4f6;
                }
                
                .actions {
                    border-color: #374151;
                }
            }
        `;
        
        document.head.appendChild(styles);
    }
    
    /**
     * Setup event listeners
     */
    setupEventListeners() {
        // Network status changes
        window.addEventListener('online', () => this.handleNetworkChange({ isOnline: true }));
        window.addEventListener('offline', () => this.handleNetworkChange({ isOnline: false }));
        
        // Service Worker events
        this.swManager.on('network_change', data => this.handleNetworkChange(data));
        this.swManager.on('sync_complete', data => this.handleSyncComplete(data));
        this.swManager.on('storage_warning', data => this.handleStorageWarning(data));
        
        // Click to toggle dropdown
        this.element.addEventListener('click', () => this.toggleDropdown());
        
        // Close dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!this.element.contains(e.target)) {
                this.hideDropdown();
            }
        });
    }
    
    /**
     * Handle network status changes
     */
    handleNetworkChange(data) {
        this.state.isOnline = data.isOnline;
        this.state.quality = data.quality || 'unknown';
        this.updateDisplay();
    }
    
    /**
     * Handle sync completion
     */
    handleSyncComplete(data) {
        this.state.syncStatus = 'idle';
        this.state.queuedItems = Math.max(0, this.state.queuedItems - (data.successful || 0));
        this.updateDisplay();
        
        // Show brief success indication
        this.showSyncSuccess(data.successful, data.failed);
    }
    
    /**
     * Handle storage warnings
     */
    handleStorageWarning(data) {
        this.showNotification('Storage space is running low. Consider clearing old caches.', 'warning');
    }
    
    /**
     * Update the display based on current state
     */
    updateDisplay() {
        const dot = this.element.querySelector('.connection-dot');
        const label = this.element.querySelector('.status-label');
        const detail = this.element.querySelector('.status-detail');
        const queueBadge = this.element.querySelector('.queue-badge');
        const queueCount = this.element.querySelector('.queue-count');
        const syncSpinner = this.element.querySelector('.sync-spinner');
        
        // Update connection status
        dot.className = 'connection-dot';
        if (!this.state.isOnline) {
            dot.classList.add('offline');
            label.textContent = 'Offline';
            detail.textContent = this.state.offlineReady ? 'Ready for offline use' : 'Limited functionality';
        } else {
            if (this.state.quality === 'slow') {
                dot.classList.add('slow');
                detail.textContent = 'Slow connection';
            } else if (this.state.quality === 'moderate') {
                dot.classList.add('moderate');
                detail.textContent = 'Moderate connection';
            } else {
                detail.textContent = 'Good connection';
            }
            
            label.textContent = 'Online';
        }
        
        // Update sync status
        if (this.state.syncStatus === 'syncing') {
            syncSpinner.style.display = 'flex';
            dot.style.display = 'none';
        } else {
            syncSpinner.style.display = 'none';
            dot.style.display = 'block';
        }
        
        // Update queue badge
        if (this.state.queuedItems > 0) {
            queueBadge.style.display = 'block';
            queueCount.textContent = this.state.queuedItems;
        } else {
            queueBadge.style.display = 'none';
        }
        
        // Update dropdown content if visible
        if (this.element.querySelector('.indicator-dropdown').style.display !== 'none') {
            this.updateDropdownContent();
        }
    }
    
    /**
     * Update dropdown content
     */
    updateDropdownContent() {
        const networkInfo = this.swManager.getNetworkInfo();
        
        // Update network info
        this.element.querySelector('.online-status').textContent = this.state.isOnline ? 'Online' : 'Offline';
        this.element.querySelector('.online-status').className = `info-value online-status ${this.state.isOnline ? '' : 'offline'}`;
        
        this.element.querySelector('.quality-status').textContent = 
            this.state.quality.charAt(0).toUpperCase() + this.state.quality.slice(1);
        this.element.querySelector('.quality-status').className = `info-value quality-status ${this.state.quality}`;
        
        this.element.querySelector('.connection-type').textContent = 
            networkInfo.effectiveType?.toUpperCase() || 'Unknown';
        
        // Update sync info
        this.element.querySelector('.sync-status-text').textContent = 
            this.state.syncStatus === 'syncing' ? 'Syncing...' : 
            this.state.queuedItems > 0 ? `${this.state.queuedItems} items queued` : 'Up to date';
            
        this.element.querySelector('.queued-count').textContent = `${this.state.queuedItems} items`;
        
        // Update offline status
        this.element.querySelector('.offline-ready-status').textContent = 
            this.state.offlineReady ? 'Yes' : 'No';
    }
    
    /**
     * Toggle dropdown visibility
     */
    toggleDropdown() {
        const dropdown = this.element.querySelector('.indicator-dropdown');
        const isVisible = dropdown.style.display !== 'none';
        
        if (isVisible) {
            this.hideDropdown();
        } else {
            this.showDropdown();
        }
    }
    
    /**
     * Show dropdown
     */
    showDropdown() {
        const dropdown = this.element.querySelector('.indicator-dropdown');
        dropdown.style.display = 'block';
        this.updateDropdownContent();
    }
    
    /**
     * Hide dropdown
     */
    hideDropdown() {
        const dropdown = this.element.querySelector('.indicator-dropdown');
        dropdown.style.display = 'none';
    }
    
    /**
     * Check offline readiness
     */
    async checkOfflineReadiness() {
        try {
            this.state.offlineReady = await this.swManager.isOfflineReady();
            this.updateDisplay();
        } catch (error) {
            console.warn('[Network Indicator] Failed to check offline readiness:', error);
        }
    }
    
    /**
     * Force sync
     */
    async forceSync() {
        const button = this.element.querySelector('.force-sync-btn');
        button.disabled = true;
        button.textContent = 'Syncing...';
        
        this.state.syncStatus = 'syncing';
        this.updateDisplay();
        
        try {
            await this.swManager.forceSync();
            this.showNotification('Sync completed successfully', 'success');
        } catch (error) {
            this.showNotification('Sync failed: ' + error.message, 'error');
        } finally {
            this.state.syncStatus = 'idle';
            button.disabled = false;
            button.textContent = 'Force Sync';
            this.updateDisplay();
        }
    }
    
    /**
     * Show diagnostics
     */
    async showDiagnostics() {
        try {
            const diagnostics = await this.swManager.getDiagnostics();
            console.log('[Network Indicator] Diagnostics:', diagnostics);
            
            // Create a simple modal or alert with diagnostics
            const summary = [
                `Service Worker: ${diagnostics.isSupported ? 'Supported' : 'Not supported'}`,
                `Status: ${diagnostics.hasController ? 'Active' : 'Inactive'}`,
                `Network: ${diagnostics.network.isOnline ? 'Online' : 'Offline'} (${diagnostics.network.quality})`,
                `Storage: ${diagnostics.storage?.usage || 'Unknown'} / ${diagnostics.storage?.quota || 'Unknown'}`,
                `Version: ${diagnostics.version?.version || 'Unknown'}`
            ].join('\n');
            
            alert('WriteMagic Diagnostics:\n\n' + summary + '\n\nCheck console for detailed information.');
        } catch (error) {
            this.showNotification('Failed to get diagnostics: ' + error.message, 'error');
        }
    }
    
    /**
     * Clear caches
     */
    async clearCaches() {
        if (!confirm('Are you sure you want to clear all caches? This will remove offline content.')) {
            return;
        }
        
        try {
            await this.swManager.clearCache();
            this.showNotification('Caches cleared successfully', 'success');
            await this.checkOfflineReadiness();
        } catch (error) {
            this.showNotification('Failed to clear caches: ' + error.message, 'error');
        }
    }
    
    /**
     * Show sync success animation
     */
    showSyncSuccess(successful, failed) {
        const dot = this.element.querySelector('.connection-dot');
        const originalClass = dot.className;
        
        dot.classList.add('success-pulse');
        
        setTimeout(() => {
            dot.className = originalClass;
        }, 1000);
        
        if (successful > 0) {
            this.showNotification(`Successfully synced ${successful} item${successful > 1 ? 's' : ''}`, 'success');
        }
        
        if (failed > 0) {
            this.showNotification(`Failed to sync ${failed} item${failed > 1 ? 's' : ''}`, 'warning');
        }
    }
    
    /**
     * Show temporary notification
     */
    showNotification(message, type = 'info') {
        // Create a simple toast notification
        const notification = document.createElement('div');
        notification.className = `network-notification ${type}`;
        notification.textContent = message;
        
        // Add notification styles if not present
        if (!document.getElementById('network-notification-styles')) {
            const styles = document.createElement('style');
            styles.id = 'network-notification-styles';
            styles.textContent = `
                .network-notification {
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    background: #1f2937;
                    color: white;
                    padding: 12px 16px;
                    border-radius: 8px;
                    font-size: 0.9rem;
                    max-width: 300px;
                    z-index: 10000;
                    animation: slideInRight 0.3s ease-out, slideOutRight 0.3s ease-in 2.7s;
                    animation-fill-mode: both;
                }
                
                .network-notification.success {
                    background: #059669;
                }
                
                .network-notification.warning {
                    background: #d97706;
                }
                
                .network-notification.error {
                    background: #dc2626;
                }
                
                @keyframes slideInRight {
                    from {
                        transform: translateX(100%);
                        opacity: 0;
                    }
                    to {
                        transform: translateX(0);
                        opacity: 1;
                    }
                }
                
                @keyframes slideOutRight {
                    from {
                        transform: translateX(0);
                        opacity: 1;
                    }
                    to {
                        transform: translateX(100%);
                        opacity: 0;
                    }
                }
            `;
            document.head.appendChild(styles);
        }
        
        document.body.appendChild(notification);
        
        // Remove after animation
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 3000);
    }
    
    /**
     * Update queued items count
     */
    updateQueuedItems(count) {
        this.state.queuedItems = count;
        this.updateDisplay();
    }
    
    /**
     * Clean up
     */
    destroy() {
        if (this.element && this.element.parentNode) {
            this.element.parentNode.removeChild(this.element);
        }
    }
}

export default NetworkIndicator;