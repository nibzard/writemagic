/**
 * Service Worker Manager
 * 
 * Provides a high-level interface for interacting with the WriteMagic Service Worker,
 * including cache management, background sync monitoring, and offline capabilities.
 */

export class ServiceWorkerManager {
    constructor(config = {}) {
        this.config = {
            enableCacheDebugging: config.enableCacheDebugging || false,
            enableNetworkMonitoring: config.enableNetworkMonitoring || true,
            enableStorageMonitoring: config.enableStorageMonitoring || true,
            syncPollingInterval: config.syncPollingInterval || 30000, // 30 seconds
            ...config
        };
        
        this.registration = null;
        this.isOnline = navigator.onLine;
        this.networkQuality = 'unknown';
        this.storageStatus = null;
        this.cacheStatus = null;
        
        // Event listeners
        this.listeners = new Map();
        
        this.initialize();
    }
    
    /**
     * Initialize the Service Worker manager
     */
    async initialize() {
        try {
            if (!('serviceWorker' in navigator)) {
                console.warn('[SW Manager] Service Workers not supported');
                return;
            }
            
            // Register Service Worker
            await this.registerServiceWorker();
            
            // Set up message handling
            this.setupMessageHandling();
            
            // Set up network monitoring
            if (this.config.enableNetworkMonitoring) {
                this.setupNetworkMonitoring();
            }
            
            // Set up storage monitoring
            if (this.config.enableStorageMonitoring) {
                this.setupStorageMonitoring();
            }
            
            console.log('[SW Manager] Initialized successfully');
            
        } catch (error) {
            console.error('[SW Manager] Initialization failed:', error);
        }
    }
    
    /**
     * Register the Service Worker
     */
    async registerServiceWorker() {
        try {
            this.registration = await navigator.serviceWorker.register('/sw.js', {
                scope: '/',
                updateViaCache: 'none'
            });
            
            console.log('[SW Manager] Service Worker registered:', this.registration.scope);
            
            // Handle updates
            this.registration.addEventListener('updatefound', () => {
                const newWorker = this.registration.installing;
                
                newWorker.addEventListener('statechange', () => {
                    if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                        this.emit('update_available', { registration: this.registration });
                    }
                });
            });
            
            // Auto-update when new worker is waiting
            if (this.registration.waiting) {
                this.emit('update_available', { registration: this.registration });
            }
            
        } catch (error) {
            console.error('[SW Manager] Registration failed:', error);
            throw error;
        }
    }
    
    /**
     * Set up message handling with Service Worker
     */
    setupMessageHandling() {
        navigator.serviceWorker.addEventListener('message', event => {
            const { data } = event;
            
            if (!data || !data.type) return;
            
            switch (data.type) {
                case 'SW_ACTIVATED':
                    console.log('[SW Manager] Service Worker activated:', data.version);
                    this.emit('activated', data);
                    break;
                    
                case 'NETWORK_STATUS':
                    this.isOnline = data.isOnline;
                    this.networkQuality = data.quality;
                    this.emit('network_change', data);
                    break;
                    
                case 'STORAGE_WARNING':
                    console.warn('[SW Manager] Storage warning:', data);
                    this.emit('storage_warning', data);
                    break;
                    
                case 'BACKGROUND_SYNC_COMPLETE':
                    console.log('[SW Manager] Background sync complete:', data);
                    this.emit('sync_complete', data);
                    break;
                    
                case 'NOTIFICATION_ACTION':
                    this.emit('notification_action', data);
                    break;
                    
                default:
                    console.debug('[SW Manager] Unknown message type:', data.type);
            }
        });
    }
    
    /**
     * Set up network monitoring
     */
    setupNetworkMonitoring() {
        // Update Service Worker with network status
        const updateNetworkStatus = () => {
            this.isOnline = navigator.onLine;
            
            // Get network quality if available
            if ('connection' in navigator && navigator.connection) {
                const connection = navigator.connection;
                this.networkQuality = this.getNetworkQuality(connection);
                
                connection.addEventListener('change', updateNetworkStatus);
            }
            
            // Send update to Service Worker
            this.sendMessage({
                type: 'UPDATE_NETWORK_STATUS',
                isOnline: this.isOnline,
                quality: this.networkQuality
            });
        };
        
        window.addEventListener('online', updateNetworkStatus);
        window.addEventListener('offline', updateNetworkStatus);
        
        // Initial update
        updateNetworkStatus();
    }
    
    /**
     * Set up storage monitoring
     */
    setupStorageMonitoring() {
        const checkStorage = async () => {
            try {
                this.storageStatus = await this.getStorageStatus();
                this.emit('storage_update', this.storageStatus);
            } catch (error) {
                console.warn('[SW Manager] Storage monitoring error:', error);
            }
        };
        
        // Check storage periodically
        setInterval(checkStorage, this.config.syncPollingInterval);
        
        // Initial check
        checkStorage();
    }
    
    /**
     * Get network quality assessment
     */
    getNetworkQuality(connection) {
        const effectiveType = connection.effectiveType;
        const downlink = connection.downlink;
        
        if (effectiveType === 'slow-2g' || downlink < 0.5) {
            return 'slow';
        } else if (effectiveType === '2g' || downlink < 1.5) {
            return 'moderate';
        } else {
            return 'good';
        }
    }
    
    /**
     * Send message to Service Worker
     */
    sendMessage(message) {
        return new Promise((resolve, reject) => {
            if (!navigator.serviceWorker.controller) {
                reject(new Error('No active Service Worker'));
                return;
            }
            
            const messageChannel = new MessageChannel();
            
            messageChannel.port1.onmessage = event => {
                if (event.data && event.data.error) {
                    reject(new Error(event.data.error));
                } else {
                    resolve(event.data);
                }
            };
            
            navigator.serviceWorker.controller.postMessage(message, [messageChannel.port2]);
        });
    }
    
    // ==================== PUBLIC API ====================
    
    /**
     * Force update the Service Worker
     */
    async updateServiceWorker() {
        if (this.registration && this.registration.waiting) {
            this.registration.waiting.postMessage({ type: 'SKIP_WAITING' });
            
            // Reload page after update
            navigator.serviceWorker.addEventListener('controllerchange', () => {
                window.location.reload();
            });
        }
    }
    
    /**
     * Get current cache status
     */
    async getCacheStatus() {
        try {
            this.cacheStatus = await this.sendMessage({ type: 'GET_CACHE_STATUS' });
            return this.cacheStatus;
        } catch (error) {
            console.error('[SW Manager] Failed to get cache status:', error);
            return null;
        }
    }
    
    /**
     * Clear specific cache or all caches
     */
    async clearCache(cacheName = null) {
        try {
            const result = await this.sendMessage({
                type: 'CLEAR_CACHE',
                cacheName
            });
            
            this.emit('cache_cleared', { cacheName, result });
            return result;
        } catch (error) {
            console.error('[SW Manager] Failed to clear cache:', error);
            return { success: false, error: error.message };
        }
    }
    
    /**
     * Get storage status and quotas
     */
    async getStorageStatus() {
        try {
            const result = await this.sendMessage({ type: 'GET_STORAGE_STATUS' });
            this.storageStatus = result;
            return result;
        } catch (error) {
            console.error('[SW Manager] Failed to get storage status:', error);
            return null;
        }
    }
    
    /**
     * Force background sync for specific queue
     */
    async forceSync(queueName = null) {
        try {
            const result = await this.sendMessage({
                type: 'FORCE_SYNC',
                queueName
            });
            
            this.emit('sync_forced', { queueName, result });
            return result;
        } catch (error) {
            console.error('[SW Manager] Failed to force sync:', error);
            return { success: false, error: error.message };
        }
    }
    
    /**
     * Preload resources for caching
     */
    async preloadResources(urls) {
        try {
            const result = await this.sendMessage({
                type: 'PRELOAD_RESOURCES',
                urls
            });
            
            this.emit('resources_preloaded', result);
            return result;
        } catch (error) {
            console.error('[SW Manager] Failed to preload resources:', error);
            return { success: false, error: error.message };
        }
    }
    
    /**
     * Get Service Worker version
     */
    async getVersion() {
        try {
            return await this.sendMessage({ type: 'GET_VERSION' });
        } catch (error) {
            console.error('[SW Manager] Failed to get version:', error);
            return null;
        }
    }
    
    /**
     * Check if app is ready for offline use
     */
    async isOfflineReady() {
        try {
            const cacheStatus = await this.getCacheStatus();
            
            if (!cacheStatus) return false;
            
            // Check if critical caches are available
            const requiredCaches = ['STATIC', 'WASM'];
            
            for (const cacheType of requiredCaches) {
                if (!cacheStatus.cacheStatus[cacheType] || 
                    cacheStatus.cacheStatus[cacheType].size === 0) {
                    return false;
                }
            }
            
            return true;
        } catch (error) {
            console.error('[SW Manager] Failed to check offline readiness:', error);
            return false;
        }
    }
    
    /**
     * Get formatted storage usage information
     */
    getFormattedStorageInfo() {
        if (!this.storageStatus || !this.storageStatus.storage) {
            return null;
        }
        
        const storage = this.storageStatus.storage;
        const usagePercent = (storage.usage / storage.quota) * 100;
        
        return {
            usage: this.formatBytes(storage.usage),
            quota: this.formatBytes(storage.quota),
            available: this.formatBytes(storage.quota - storage.usage),
            usagePercent: Math.round(usagePercent),
            isNearLimit: usagePercent > 80
        };
    }
    
    /**
     * Get network status information
     */
    getNetworkInfo() {
        return {
            isOnline: this.isOnline,
            quality: this.networkQuality,
            effectiveType: navigator.connection?.effectiveType || 'unknown',
            downlink: navigator.connection?.downlink || 0
        };
    }
    
    // ==================== EVENT SYSTEM ====================
    
    /**
     * Add event listener
     */
    on(event, callback) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, new Set());
        }
        this.listeners.get(event).add(callback);
        return this;
    }
    
    /**
     * Remove event listener
     */
    off(event, callback) {
        if (this.listeners.has(event)) {
            this.listeners.get(event).delete(callback);
        }
        return this;
    }
    
    /**
     * Emit event
     */
    emit(event, data) {
        if (this.listeners.has(event)) {
            for (const callback of this.listeners.get(event)) {
                try {
                    callback(data);
                } catch (error) {
                    console.error(`[SW Manager] Error in event callback for '${event}':`, error);
                }
            }
        }
    }
    
    // ==================== UTILITIES ====================
    
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
     * Check if Service Worker is supported and active
     */
    isSupported() {
        return 'serviceWorker' in navigator && !!navigator.serviceWorker.controller;
    }
    
    /**
     * Get diagnostics information
     */
    async getDiagnostics() {
        const diagnostics = {
            timestamp: Date.now(),
            isSupported: this.isSupported(),
            registration: !!this.registration,
            hasController: !!navigator.serviceWorker.controller,
            network: this.getNetworkInfo(),
            storage: this.getFormattedStorageInfo(),
            cache: await this.getCacheStatus(),
            version: await this.getVersion()
        };
        
        return diagnostics;
    }
    
    /**
     * Clean up resources
     */
    destroy() {
        this.listeners.clear();
        
        // Clear any intervals
        if (this.storageMonitorInterval) {
            clearInterval(this.storageMonitorInterval);
        }
        
        console.log('[SW Manager] Destroyed');
    }
}

export default ServiceWorkerManager;