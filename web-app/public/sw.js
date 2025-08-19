/**
 * WriteMagic Enhanced Service Worker
 * 
 * Provides enterprise-grade offline functionality with advanced caching strategies,
 * background sync, and intelligent cache management for the WriteMagic PWA.
 * 
 * Features:
 * - Multi-tier caching strategies (cache-first, network-first, stale-while-revalidate)
 * - Advanced WASM module caching with integrity checks
 * - AI response caching with smart invalidation
 * - Background sync for offline actions
 * - Intelligent cache cleanup and storage management
 * - Network detection and user feedback
 * - Cache analytics and debugging tools
 * - Push notification support for collaborative features
 */

// Cache Configuration
const CACHE_VERSION = '2.0.0';
const CACHE_PREFIX = 'writemagic';
const CACHE_NAMES = {
  STATIC: `${CACHE_PREFIX}-static-v${CACHE_VERSION}`,
  DYNAMIC: `${CACHE_PREFIX}-dynamic-v${CACHE_VERSION}`,
  WASM: `${CACHE_PREFIX}-wasm-v${CACHE_VERSION}`,
  AI_RESPONSES: `${CACHE_PREFIX}-ai-v${CACHE_VERSION}`,
  DOCUMENTS: `${CACHE_PREFIX}-docs-v${CACHE_VERSION}`,
  IMAGES: `${CACHE_PREFIX}-images-v${CACHE_VERSION}`,
  FONTS: `${CACHE_PREFIX}-fonts-v${CACHE_VERSION}`
};

// Core app shell resources (cache-first strategy)
const STATIC_CACHE_URLS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/offline.html', // Offline fallback page
  '/styles/reset.css',
  '/styles/variables.css', 
  '/styles/base.css',
  '/styles/components.css',
  '/styles/layout.css',
  '/styles/themes.css',
  '/styles/responsive.css',
  '/styles/accessibility.css',
  '/styles/performance.css',
  '/scripts/app.js',
  '/icons/favicon.svg',
  '/icons/icon-16x16.png',
  '/icons/icon-32x32.png',
  '/icons/icon-72x72.png',
  '/icons/icon-96x96.png',
  '/icons/icon-128x128.png',
  '/icons/icon-144x144.png',
  '/icons/icon-152x152.png',
  '/icons/icon-192x192.png',
  '/icons/icon-384x384.png',
  '/icons/icon-512x512.png'
];

// WASM resources (cache-first with integrity validation)
const WASM_CACHE_URLS = [
  '/src/js/index.js',
  '/src/js/document-manager.js',
  '/src/js/project-workspace.js',
  '/src/js/writing-analytics.js',
  '/src/js/content-utilities.js',
  '/src/js/writing-session.js',
  '/src/js/utils/event-emitter.js',
  '/src/js/utils/debounce.js',
  '/src/js/ai-proxy-integration.js',
  '/src/js/utils/wasm-loader.js',
  '/src/js/utils/progressive-loader.js', 
  '/src/js/utils/performance-monitor.js',
  '/src/js/performance-dashboard.js',
  '/core/wasm/pkg/writemagic_wasm.js',
  '/core/wasm/pkg/writemagic_wasm_bg.wasm',
  '/core/wasm/pkg/streaming_loader.js',
  '/core/wasm/pkg/module_manifest.json',
  '/core/wasm/pkg/integrity.json'
];

// Cache configuration with strategies and expiration
const CACHE_STRATEGIES = {
  CACHE_FIRST: 'cache-first',
  NETWORK_FIRST: 'network-first',
  STALE_WHILE_REVALIDATE: 'stale-while-revalidate',
  NETWORK_ONLY: 'network-only',
  CACHE_ONLY: 'cache-only'
};

const CACHE_EXPIRATION = {
  STATIC: 7 * 24 * 60 * 60 * 1000,      // 7 days
  DYNAMIC: 24 * 60 * 60 * 1000,          // 1 day
  WASM: 30 * 24 * 60 * 60 * 1000,       // 30 days
  AI_RESPONSES: 60 * 60 * 1000,          // 1 hour
  DOCUMENTS: 7 * 24 * 60 * 60 * 1000,   // 7 days
  IMAGES: 30 * 24 * 60 * 60 * 1000,     // 30 days
  FONTS: 90 * 24 * 60 * 60 * 1000       // 90 days
};

// Storage quotas (in bytes)
const STORAGE_QUOTAS = {
  MAX_TOTAL: 100 * 1024 * 1024,         // 100MB total
  MAX_AI_CACHE: 20 * 1024 * 1024,       // 20MB for AI responses
  MAX_DOCUMENTS: 50 * 1024 * 1024,      // 50MB for documents
  MAX_DYNAMIC: 30 * 1024 * 1024,        // 30MB for dynamic content
  WARNING_THRESHOLD: 80 * 1024 * 1024    // 80MB warning threshold
};

// Background sync queue names
const SYNC_QUEUES = {
  AI_REQUESTS: 'ai-requests',
  DOCUMENT_SAVES: 'document-saves',
  USER_ACTIONS: 'user-actions',
  ANALYTICS: 'analytics-data'
};

// Network detection
let isOnline = true;
let networkQuality = 'good'; // 'good', 'slow', 'offline'

// Enhanced installation event with multi-tier caching
self.addEventListener('install', event => {
  console.log('[SW] Installing WriteMagic Enhanced Service Worker v' + CACHE_VERSION);
  
  event.waitUntil(
    Promise.all([
      // Cache static resources
      caches.open(CACHE_NAMES.STATIC)
        .then(cache => {
          console.log('[SW] Caching static resources');
          return cache.addAll(STATIC_CACHE_URLS);
        }),
      
      // Cache WASM resources with integrity validation
      caches.open(CACHE_NAMES.WASM)
        .then(cache => {
          console.log('[SW] Caching WASM resources');
          return cacheWasmResourcesWithIntegrity(cache, WASM_CACHE_URLS);
        }),
      
      // Initialize other caches
      caches.open(CACHE_NAMES.DYNAMIC),
      caches.open(CACHE_NAMES.AI_RESPONSES),
      caches.open(CACHE_NAMES.DOCUMENTS),
      caches.open(CACHE_NAMES.IMAGES),
      caches.open(CACHE_NAMES.FONTS),
      
      // Initialize background sync queues
      initializeSyncQueues()
    ])
    .then(() => {
      console.log('[SW] Enhanced installation complete');
      return self.skipWaiting();
    })
    .catch(error => {
      console.error('[SW] Installation failed:', error);
      throw error;
    })
  );
});

// Enhanced activation event with intelligent cache cleanup
self.addEventListener('activate', event => {
  console.log('[SW] Activating WriteMagic Enhanced Service Worker...');
  
  event.waitUntil(
    Promise.all([
      // Clean up old caches
      cleanupOldCaches(),
      
      // Initialize storage management
      initializeStorageManagement(),
      
      // Set up network monitoring
      initializeNetworkMonitoring(),
      
      // Claim all clients
      self.clients.claim()
    ])
    .then(() => {
      console.log('[SW] Enhanced activation complete');
      // Post message to all clients about activation
      broadcastToClients({
        type: 'SW_ACTIVATED',
        version: CACHE_VERSION,
        caches: Object.keys(CACHE_NAMES),
        timestamp: Date.now()
      });
    })
    .catch(error => {
      console.error('[SW] Activation failed:', error);
    })
  );
});

// Enhanced fetch event with intelligent routing and caching strategies
self.addEventListener('fetch', event => {
  const { request } = event;
  const url = new URL(request.url);
  
  // Skip non-GET requests for caching (but handle for background sync)
  if (request.method !== 'GET') {
    if (shouldQueueForSync(request)) {
      event.respondWith(handleNonGetRequest(request));
    }
    return;
  }
  
  // Skip cross-origin requests (except for fonts and CDN resources)
  if (url.origin !== location.origin && !isCachableExternalResource(url)) {
    return;
  }
  
  // Route requests to appropriate handlers based on enhanced categorization
  if (isAppShell(url)) {
    event.respondWith(handleAppShellRequest(request));
  } else if (isStaticAsset(url)) {
    event.respondWith(handleStaticAssetRequest(request));
  } else if (isWasmResource(url)) {
    event.respondWith(handleWasmResourceRequest(request));
  } else if (isAIRequest(url)) {
    event.respondWith(handleAIRequest(request));
  } else if (isDocumentRequest(url)) {
    event.respondWith(handleDocumentRequest(request));
  } else if (isImageResource(url)) {
    event.respondWith(handleImageRequest(request));
  } else if (isFontResource(url)) {
    event.respondWith(handleFontRequest(request));
  } else if (isAnalyticsRequest(url)) {
    event.respondWith(handleAnalyticsRequest(request));
  } else {
    event.respondWith(handleDynamicRequest(request));
  }
});

// ==================== ENHANCED REQUEST HANDLERS ====================

// App shell handler: Cache-first with stale-while-revalidate
async function handleAppShellRequest(request) {
  const cacheName = CACHE_NAMES.STATIC;
  const url = new URL(request.url);
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse && !isExpired(cachedResponse, CACHE_EXPIRATION.STATIC)) {
      // Serve from cache and update in background if online
      if (isOnline) {
        updateCacheInBackground(request, cache, 'app-shell');
      }
      return cachedResponse;
    }
    
    // Try network first for fresh content
    if (isOnline) {
      try {
        const networkResponse = await fetchWithTimeout(request, 3000);
        if (networkResponse.ok) {
          await cache.put(request, networkResponse.clone());
          return networkResponse;
        }
      } catch (networkError) {
        console.warn('[SW] Network failed for app shell, falling back to cache');
      }
    }
    
    // Fallback to cached version even if expired
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Last resort: offline fallback page
    return await getOfflineFallback();
    
  } catch (error) {
    console.error('[SW] App shell request failed:', error);
    return await getOfflineFallback();
  }
}

// Static assets handler: Cache-first with long-term caching
async function handleStaticAssetRequest(request) {
  const cacheName = CACHE_NAMES.STATIC;
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    // Serve from cache if available (static assets change infrequently)
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Fetch from network and cache
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 5000);
      if (networkResponse.ok) {
        await cache.put(request, networkResponse.clone());
        return networkResponse;
      }
    }
    
    // Return 404 for missing static assets
    return new Response('Asset not found', { status: 404 });
    
  } catch (error) {
    console.warn('[SW] Static asset request failed:', error);
    return new Response('Asset unavailable', { status: 503 });
  }
}

// WASM resources handler: Cache-first with integrity validation
async function handleWasmResourceRequest(request) {
  const cacheName = CACHE_NAMES.WASM;
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    // WASM resources are critical - serve from cache if available
    if (cachedResponse && await validateWasmIntegrity(cachedResponse)) {
      return cachedResponse;
    }
    
    // Fetch fresh WASM resources
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 10000);
      if (networkResponse.ok) {
        // Validate and cache WASM resources
        const clonedResponse = networkResponse.clone();
        if (await validateWasmIntegrity(clonedResponse)) {
          await cache.put(request, networkResponse.clone());
          return networkResponse;
        } else {
          console.error('[SW] WASM integrity validation failed');
        }
      }
    }
    
    // If cached version exists but failed integrity, try to use it anyway
    if (cachedResponse) {
      console.warn('[SW] Using potentially corrupted WASM resource');
      return cachedResponse;
    }
    
    throw new Error('Critical WASM resource unavailable');
    
  } catch (error) {
    console.error('[SW] WASM resource request failed:', error);
    throw error; // Don't provide fallback for critical WASM resources
  }
}

// AI request handler: Network-first with intelligent caching
async function handleAIRequest(request) {
  const cacheName = CACHE_NAMES.AI_RESPONSES;
  const url = new URL(request.url);
  
  try {
    // For GET requests (queries), try cache first if recent
    if (request.method === 'GET') {
      const cache = await caches.open(cacheName);
      const cacheKey = generateAICacheKey(request);
      const cachedResponse = await cache.match(cacheKey);
      
      if (cachedResponse && !isExpired(cachedResponse, CACHE_EXPIRATION.AI_RESPONSES)) {
        return cachedResponse;
      }
    }
    
    // Always try network first for AI requests
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 30000);
      
      if (networkResponse.ok) {
        // Cache successful AI responses (GET only)
        if (request.method === 'GET') {
          const cache = await caches.open(cacheName);
          const cacheKey = generateAICacheKey(request);
          await cache.put(cacheKey, networkResponse.clone());
          
          // Clean up old AI cache entries
          await cleanupAICache();
        }
        
        return networkResponse;
      }
    }
    
    // For GET requests, try to serve stale cache
    if (request.method === 'GET') {
      const cache = await caches.open(cacheName);
      const cacheKey = generateAICacheKey(request);
      const cachedResponse = await cache.match(cacheKey);
      
      if (cachedResponse) {
        return cachedResponse;
      }
    }
    
    // Queue non-GET requests for background sync
    if (request.method !== 'GET') {
      await queueForBackgroundSync(SYNC_QUEUES.AI_REQUESTS, {
        request: await serializeRequest(request),
        timestamp: Date.now(),
        retryCount: 0
      });
    }
    
    return new Response(
      JSON.stringify({
        error: 'AI service temporarily unavailable',
        offline: !isOnline,
        queued: request.method !== 'GET',
        cached: false
      }),
      {
        status: 503,
        headers: { 'Content-Type': 'application/json' }
      }
    );
    
  } catch (error) {
    console.error('[SW] AI request failed:', error);
    return new Response(
      JSON.stringify({ error: 'AI request processing failed' }),
      {
        status: 500,
        headers: { 'Content-Type': 'application/json' }
      }
    );
  }
}

// Document request handler: Stale-while-revalidate with IndexedDB sync
async function handleDocumentRequest(request) {
  const cacheName = CACHE_NAMES.DOCUMENTS;
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    // Serve from cache immediately
    if (cachedResponse) {
      // Update in background if online
      if (isOnline) {
        updateCacheInBackground(request, cache, 'document');
      }
      return cachedResponse;
    }
    
    // Try network for fresh content
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 8000);
      if (networkResponse.ok) {
        await cache.put(request, networkResponse.clone());
        return networkResponse;
      }
    }
    
    // Queue document saves for background sync
    if (request.method !== 'GET') {
      await queueForBackgroundSync(SYNC_QUEUES.DOCUMENT_SAVES, {
        request: await serializeRequest(request),
        timestamp: Date.now(),
        retryCount: 0
      });
      
      return new Response(
        JSON.stringify({ success: true, queued: true }),
        { headers: { 'Content-Type': 'application/json' } }
      );
    }
    
    return new Response('Document not available offline', { status: 503 });
    
  } catch (error) {
    console.error('[SW] Document request failed:', error);
    return new Response('Document request processing failed', { status: 500 });
  }
}

// Image request handler: Cache-first with compression
async function handleImageRequest(request) {
  const cacheName = CACHE_NAMES.IMAGES;
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
      return cachedResponse;
    }
    
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 10000);
      if (networkResponse.ok) {
        await cache.put(request, networkResponse.clone());
        return networkResponse;
      }
    }
    
    return new Response('Image not available offline', { status: 503 });
    
  } catch (error) {
    console.warn('[SW] Image request failed:', error);
    return new Response('Image unavailable', { status: 503 });
  }
}

// Font request handler: Cache-first with long expiration
async function handleFontRequest(request) {
  const cacheName = CACHE_NAMES.FONTS;
  
  try {
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
      return cachedResponse;
    }
    
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 15000);
      if (networkResponse.ok) {
        await cache.put(request, networkResponse.clone());
        return networkResponse;
      }
    }
    
    // Fonts are not critical - return 404 if not available
    return new Response('Font not found', { status: 404 });
    
  } catch (error) {
    console.warn('[SW] Font request failed:', error);
    return new Response('Font unavailable', { status: 503 });
  }
}

// Analytics request handler: Queue for background sync
async function handleAnalyticsRequest(request) {
  try {
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 5000);
      if (networkResponse.ok) {
        return networkResponse;
      }
    }
    
    // Queue analytics for background sync
    await queueForBackgroundSync(SYNC_QUEUES.ANALYTICS, {
      request: await serializeRequest(request),
      timestamp: Date.now(),
      retryCount: 0
    });
    
    return new Response(
      JSON.stringify({ queued: true }),
      { headers: { 'Content-Type': 'application/json' } }
    );
    
  } catch (error) {
    console.warn('[SW] Analytics request failed:', error);
    return new Response('Analytics queued for sync', { status: 202 });
  }
}

// Dynamic request handler: Network-first with selective caching
async function handleDynamicRequest(request) {
  const cacheName = CACHE_NAMES.DYNAMIC;
  
  try {
    if (isOnline) {
      const networkResponse = await fetchWithTimeout(request, 8000);
      
      if (networkResponse.ok && shouldCacheDynamicContent(request, networkResponse)) {
        const cache = await caches.open(cacheName);
        await cache.put(request, networkResponse.clone());
        
        // Clean up dynamic cache periodically
        await cleanupDynamicCache();
      }
      
      return networkResponse;
    }
    
    // Try cache for offline access
    const cache = await caches.open(cacheName);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
      return cachedResponse;
    }
    
    return new Response('Content not available offline', { status: 503 });
    
  } catch (error) {
    console.warn('[SW] Dynamic request failed:', error);
    return new Response('Request processing failed', { status: 500 });
  }
}

// Non-GET request handler: Queue for background sync
async function handleNonGetRequest(request) {
  try {
    if (isOnline) {
      return await fetchWithTimeout(request, 10000);
    }
    
    // Queue for background sync based on request type
    const queueName = getQueueForRequest(request);
    await queueForBackgroundSync(queueName, {
      request: await serializeRequest(request),
      timestamp: Date.now(),
      retryCount: 0
    });
    
    return new Response(
      JSON.stringify({
        success: true,
        queued: true,
        message: 'Request queued for processing when online'
      }),
      {
        status: 202,
        headers: { 'Content-Type': 'application/json' }
      }
    );
    
  } catch (error) {
    console.error('[SW] Non-GET request failed:', error);
    return new Response(
      JSON.stringify({ error: 'Request processing failed' }),
      {
        status: 500,
        headers: { 'Content-Type': 'application/json' }
      }
    );
  }
}

// ==================== BACKGROUND SYNC HANDLERS ====================

// Enhanced background sync with queue-specific processing
self.addEventListener('sync', event => {
  const tags = Array.isArray(event.tag) ? event.tag : [event.tag];
  
  for (const tag of tags) {
    if (tag.startsWith('writemagic-')) {
      const queueName = tag.replace('writemagic-', '');
      event.waitUntil(processBackgroundSyncQueue(queueName));
    }
  }
});

// Process specific background sync queues
async function processBackgroundSyncQueue(queueName) {
  console.log(`[SW] Processing background sync queue: ${queueName}`);
  
  try {
    const queuedItems = await getQueuedItems(queueName);
    
    if (queuedItems.length === 0) {
      console.log(`[SW] No items in queue: ${queueName}`);
      return;
    }
    
    const results = await Promise.allSettled(
      queuedItems.map(item => processQueuedItem(item, queueName))
    );
    
    const successful = results.filter(r => r.status === 'fulfilled').length;
    const failed = results.filter(r => r.status === 'rejected').length;
    
    console.log(`[SW] Queue ${queueName}: ${successful} successful, ${failed} failed`);
    
    // Notify clients about sync completion
    broadcastToClients({
      type: 'BACKGROUND_SYNC_COMPLETE',
      queue: queueName,
      successful,
      failed,
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error(`[SW] Failed to process queue ${queueName}:`, error);
  }
}

// Process individual queued item
async function processQueuedItem(item, queueName) {
  try {
    const request = await deserializeRequest(item.request);
    const response = await fetch(request);
    
    if (response.ok) {
      // Remove successfully processed item
      await removeQueuedItem(queueName, item.id);
      return { success: true, item };
    } else {
      // Increment retry count
      item.retryCount = (item.retryCount || 0) + 1;
      
      // Remove after max retries
      if (item.retryCount >= 3) {
        await removeQueuedItem(queueName, item.id);
        console.warn(`[SW] Max retries reached for item in queue ${queueName}`);
      } else {
        await updateQueuedItem(queueName, item.id, item);
      }
      
      throw new Error(`Request failed with status: ${response.status}`);
    }
  } catch (error) {
    console.error('[SW] Failed to process queued item:', error);
    throw error;
  }
}

// ==================== PUSH NOTIFICATIONS ====================

// Enhanced push notification handling
self.addEventListener('push', event => {
  if (!event.data) {
    console.warn('[SW] Push event without data');
    return;
  }
  
  try {
    const data = event.data.json();
    const options = {
      body: data.body || 'New notification from WriteMagic',
      icon: '/icons/icon-192x192.png',
      badge: '/icons/icon-72x72.png',
      tag: data.tag || 'writemagic-general',
      data: data.data || {},
      actions: data.actions || [],
      requireInteraction: data.requireInteraction || false,
      silent: data.silent || false,
      vibrate: data.vibrate || [200, 100, 200]
    };
    
    event.waitUntil(
      self.registration.showNotification(data.title || 'WriteMagic', options)
    );
    
  } catch (error) {
    console.error('[SW] Push notification error:', error);
    
    // Fallback notification
    event.waitUntil(
      self.registration.showNotification('WriteMagic', {
        body: 'You have a new notification',
        icon: '/icons/icon-192x192.png'
      })
    );
  }
});

// Enhanced notification click handler
self.addEventListener('notificationclick', event => {
  event.notification.close();
  
  const notificationData = event.notification.data || {};
  const action = event.action;
  
  event.waitUntil(
    handleNotificationClick(notificationData, action)
  );
});

// Handle notification click actions
async function handleNotificationClick(data, action) {
  try {
    const clients = await self.clients.matchAll({ type: 'window' });
    
    // Handle specific actions
    if (action === 'open_document' && data.documentId) {
      const url = `/?document=${data.documentId}`;
      return await openOrFocusClient(url, clients);
    }
    
    if (action === 'dismiss') {
      return; // Just close the notification
    }
    
    // Default action: open or focus the main app
    return await openOrFocusClient('/', clients);
    
  } catch (error) {
    console.error('[SW] Notification click error:', error);
  }
}

// Open or focus existing client
async function openOrFocusClient(url, clients) {
  // Try to focus existing client with matching URL
  for (const client of clients) {
    if (client.url.includes(url.split('?')[0]) && 'focus' in client) {
      await client.focus();
      
      // Send message to update client state if needed
      if (url.includes('?')) {
        client.postMessage({
          type: 'NOTIFICATION_ACTION',
          url: url
        });
      }
      
      return;
    }
  }
  
  // Open new window if no matching client
  if (self.clients.openWindow) {
    return await self.clients.openWindow(url);
  }
}

// ==================== ENHANCED MESSAGE HANDLING ====================

// Enhanced message handling for client communication
self.addEventListener('message', event => {
  const { data } = event;
  
  if (!data || !data.type) {
    console.warn('[SW] Message without type:', data);
    return;
  }
  
  // Handle different message types
  switch (data.type) {
    case 'SKIP_WAITING':
      self.skipWaiting();
      break;
      
    case 'GET_VERSION':
      event.ports[0]?.postMessage({
        version: CACHE_VERSION,
        caches: Object.keys(CACHE_NAMES)
      });
      break;
      
    case 'GET_CACHE_STATUS':
      event.waitUntil(handleCacheStatusRequest(event));
      break;
      
    case 'CLEAR_CACHE':
      event.waitUntil(handleClearCacheRequest(data.cacheName, event));
      break;
      
    case 'GET_STORAGE_STATUS':
      event.waitUntil(handleStorageStatusRequest(event));
      break;
      
    case 'FORCE_SYNC':
      event.waitUntil(handleForceSyncRequest(data.queueName, event));
      break;
      
    case 'UPDATE_NETWORK_STATUS':
      handleNetworkStatusUpdate(data.isOnline, data.quality);
      break;
      
    case 'PRELOAD_RESOURCES':
      event.waitUntil(handlePreloadRequest(data.urls, event));
      break;
      
    case 'OPTIMIZE_CACHE_STRATEGY':
      event.waitUntil(handleCacheOptimization(event));
      break;
      
    case 'GET_PERFORMANCE_METRICS':
      event.waitUntil(handlePerformanceMetricsRequest(event));
      break;
      
    default:
      console.warn('[SW] Unknown message type:', data.type);
  }
});

// Handle cache status request
async function handleCacheStatusRequest(event) {
  try {
    const cacheStatus = {};
    
    for (const [name, cacheName] of Object.entries(CACHE_NAMES)) {
      const cache = await caches.open(cacheName);
      const keys = await cache.keys();
      cacheStatus[name] = {
        name: cacheName,
        size: keys.length,
        urls: keys.map(req => req.url)
      };
    }
    
    const storageEstimate = await navigator.storage?.estimate?.();
    
    event.ports[0]?.postMessage({
      cacheStatus,
      storageEstimate: storageEstimate || null,
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error('[SW] Cache status error:', error);
    event.ports[0]?.postMessage({ error: error.message });
  }
}

// Handle clear cache request
async function handleClearCacheRequest(cacheName, event) {
  try {
    if (cacheName && CACHE_NAMES[cacheName.toUpperCase()]) {
      const success = await caches.delete(CACHE_NAMES[cacheName.toUpperCase()]);
      event.ports[0]?.postMessage({ success, cacheName });
    } else {
      // Clear all caches
      const results = {};
      for (const [name, cache] of Object.entries(CACHE_NAMES)) {
        results[name] = await caches.delete(cache);
      }
      event.ports[0]?.postMessage({ success: true, results });
    }
  } catch (error) {
    console.error('[SW] Clear cache error:', error);
    event.ports[0]?.postMessage({ error: error.message });
  }
}

// Handle storage status request
async function handleStorageStatusRequest(event) {
  try {
    const storageEstimate = await navigator.storage?.estimate?.();
    const queueSizes = {};
    
    for (const queueName of Object.values(SYNC_QUEUES)) {
      const items = await getQueuedItems(queueName);
      queueSizes[queueName] = items.length;
    }
    
    event.ports[0]?.postMessage({
      storage: storageEstimate || null,
      queues: queueSizes,
      quotas: STORAGE_QUOTAS,
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error('[SW] Storage status error:', error);
    event.ports[0]?.postMessage({ error: error.message });
  }
}

// Handle force sync request
async function handleForceSyncRequest(queueName, event) {
  try {
    if (queueName) {
      await processBackgroundSyncQueue(queueName);
    } else {
      // Process all queues
      for (const queue of Object.values(SYNC_QUEUES)) {
        await processBackgroundSyncQueue(queue);
      }
    }
    
    event.ports[0]?.postMessage({ success: true, queueName });
    
  } catch (error) {
    console.error('[SW] Force sync error:', error);
    event.ports[0]?.postMessage({ error: error.message });
  }
}

// Handle preload resources request
async function handlePreloadRequest(urls, event) {
  try {
    const cache = await caches.open(CACHE_NAMES.DYNAMIC);
    const results = await Promise.allSettled(
      urls.map(url => 
        fetch(url)
          .then(response => response.ok ? cache.put(url, response) : null)
          .catch(error => console.warn(`[SW] Preload failed for ${url}:`, error))
      )
    );
    
    const successful = results.filter(r => r.status === 'fulfilled').length;
    
    event.ports[0]?.postMessage({
      success: true,
      total: urls.length,
      successful
    });
    
  } catch (error) {
    console.error('[SW] Preload error:', error);
    event.ports[0]?.postMessage({ error: error.message });
  }
}

// ==================== UTILITY FUNCTIONS ====================

// Enhanced resource classification functions
function isAppShell(url) {
  return url.pathname === '/' || 
         url.pathname === '/index.html' ||
         url.pathname.startsWith('/app/') ||
         url.pathname === '/offline.html';
}

function isStaticAsset(url) {
  return url.pathname.startsWith('/styles/') ||
         url.pathname.startsWith('/scripts/') ||
         url.pathname.startsWith('/icons/') ||
         /\.(css|js|svg|png|jpg|jpeg|gif|webp|ico)$/i.test(url.pathname);
}

function isWasmResource(url) {
  return url.pathname.includes('.wasm') ||
         url.pathname.includes('/wasm/') ||
         url.pathname.includes('writemagic_wasm') ||
         url.pathname.startsWith('/src/js/') ||
         url.pathname.startsWith('/core/wasm/');
}

function isAIRequest(url) {
  return url.pathname.startsWith('/api/ai/') ||
         url.pathname.startsWith('/ai-proxy/') ||
         url.pathname.includes('completion') ||
         url.pathname.includes('chat');
}

function isDocumentRequest(url) {
  return url.pathname.startsWith('/api/documents/') ||
         url.pathname.startsWith('/api/projects/') ||
         url.pathname.includes('save') ||
         url.pathname.includes('document');
}

function isImageResource(url) {
  return /\.(png|jpg|jpeg|gif|webp|svg|ico|bmp)$/i.test(url.pathname) ||
         url.pathname.startsWith('/images/') ||
         url.pathname.startsWith('/icons/');
}

function isFontResource(url) {
  return /\.(woff|woff2|ttf|otf|eot)$/i.test(url.pathname) ||
         url.hostname === 'fonts.googleapis.com' ||
         url.hostname === 'fonts.gstatic.com';
}

function isAnalyticsRequest(url) {
  return url.pathname.startsWith('/api/analytics/') ||
         url.pathname.includes('telemetry') ||
         url.pathname.includes('metrics');
}

function isCachableExternalResource(url) {
  return isFontResource(url) ||
         url.hostname === 'cdn.jsdelivr.net' ||
         url.hostname === 'unpkg.com';
}

function shouldQueueForSync(request) {
  const url = new URL(request.url);
  return isAIRequest(url) || 
         isDocumentRequest(url) || 
         isAnalyticsRequest(url) ||
         (request.method !== 'GET' && url.origin === location.origin);
}

function shouldCacheDynamicContent(request, response) {
  // Don't cache error responses
  if (!response.ok) return false;
  
  // Don't cache responses without cache-control headers that are too large
  const contentLength = response.headers.get('content-length');
  if (contentLength && parseInt(contentLength) > 1024 * 1024) return false; // 1MB limit
  
  // Cache based on content type
  const contentType = response.headers.get('content-type') || '';
  return contentType.includes('application/json') ||
         contentType.includes('text/html') ||
         contentType.includes('text/plain');
}

function getQueueForRequest(request) {
  const url = new URL(request.url);
  
  if (isAIRequest(url)) return SYNC_QUEUES.AI_REQUESTS;
  if (isDocumentRequest(url)) return SYNC_QUEUES.DOCUMENT_SAVES;
  if (isAnalyticsRequest(url)) return SYNC_QUEUES.ANALYTICS;
  
  return SYNC_QUEUES.USER_ACTIONS;
}

// ==================== CACHE MANAGEMENT UTILITIES ====================

// Check if cached response is expired
function isExpired(response, maxAge) {
  const dateHeader = response.headers.get('date');
  if (!dateHeader) return true;
  
  const cacheDate = new Date(dateHeader);
  const now = new Date();
  
  return (now.getTime() - cacheDate.getTime()) > maxAge;
}

// Fetch with timeout
function fetchWithTimeout(request, timeout) {
  return Promise.race([
    fetch(request),
    new Promise((_, reject) => 
      setTimeout(() => reject(new Error('Fetch timeout')), timeout)
    )
  ]);
}

// Update cache in background with categorization
async function updateCacheInBackground(request, cache, category = 'general') {
  try {
    console.debug(`[SW] Background updating ${category} cache for:`, request.url);
    const networkResponse = await fetchWithTimeout(request, 5000);
    
    if (networkResponse.ok) {
      await cache.put(request, networkResponse.clone());
      console.debug(`[SW] Background cache update successful for ${category}`);
    }
  } catch (error) {
    console.debug(`[SW] Background cache update failed for ${category}:`, error.message);
  }
}

// Get offline fallback page
async function getOfflineFallback() {
  try {
    const cache = await caches.open(CACHE_NAMES.STATIC);
    const fallback = await cache.match('/offline.html') || await cache.match('/');
    
    if (fallback) {
      return fallback;
    }
    
    // Create a basic offline page if none exists
    return new Response(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>WriteMagic - Offline</title>
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <style>
            body { 
              font-family: system-ui, -apple-system, sans-serif;
              text-align: center;
              padding: 2rem;
              background: #f8fafc;
            }
            .offline-container {
              max-width: 500px;
              margin: 2rem auto;
              padding: 2rem;
              background: white;
              border-radius: 8px;
              box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            }
            .offline-icon { font-size: 4rem; margin-bottom: 1rem; }
            h1 { color: #374151; margin-bottom: 1rem; }
            p { color: #6b7280; line-height: 1.5; }
          </style>
        </head>
        <body>
          <div class="offline-container">
            <div class="offline-icon">📝</div>
            <h1>You're Offline</h1>
            <p>WriteMagic is not available right now. Please check your connection and try again.</p>
            <button onclick="window.location.reload()">Try Again</button>
          </div>
        </body>
      </html>
    `, {
      headers: { 'Content-Type': 'text/html' }
    });
    
  } catch (error) {
    console.error('[SW] Failed to get offline fallback:', error);
    return new Response('Service unavailable', { status: 503 });
  }
}

// ==================== ADVANCED CACHE MANAGEMENT ====================

// Cache WASM resources with integrity validation
async function cacheWasmResourcesWithIntegrity(cache, urls) {
  const promises = urls.map(async (url) => {
    try {
      const response = await fetch(url);
      if (response.ok && await validateWasmIntegrity(response.clone())) {
        await cache.put(url, response);
        console.log(`[SW] Cached WASM resource with integrity: ${url}`);
      } else {
        console.warn(`[SW] WASM integrity validation failed for: ${url}`);
      }
    } catch (error) {
      console.error(`[SW] Failed to cache WASM resource ${url}:`, error);
    }
  });
  
  return Promise.allSettled(promises);
}

// Validate WASM resource integrity
async function validateWasmIntegrity(response) {
  try {
    const contentType = response.headers.get('content-type');
    const url = response.url;
    
    // Basic validation for WASM files
    if (url.endsWith('.wasm')) {
      const buffer = await response.arrayBuffer();
      
      // Check WASM magic number (0x6d736100)
      const view = new Uint8Array(buffer);
      if (view.length >= 4) {
        const magic = (view[0] << 24) | (view[1] << 16) | (view[2] << 8) | view[3];
        return magic === 0x0061736d; // WASM magic number in little endian
      }
    }
    
    // For JS WASM bindings, just check if it's valid JS
    if (url.includes('writemagic_wasm.js') || contentType?.includes('javascript')) {
      const text = await response.text();
      return text.length > 0 && !text.includes('error') && text.includes('wasm');
    }
    
    return true; // Default to valid for other resources
  } catch (error) {
    console.warn('[SW] WASM integrity validation error:', error);
    return false;
  }
}

// Generate AI cache key for consistent caching
function generateAICacheKey(request) {
  const url = new URL(request.url);
  const searchParams = new URLSearchParams(url.search);
  
  // Create a normalized cache key based on relevant parameters
  const relevantParams = ['prompt', 'model', 'temperature', 'max_tokens'];
  const normalizedParams = new URLSearchParams();
  
  for (const param of relevantParams) {
    if (searchParams.has(param)) {
      normalizedParams.set(param, searchParams.get(param));
    }
  }
  
  const baseUrl = `${url.origin}${url.pathname}`;
  const cacheUrl = normalizedParams.toString() 
    ? `${baseUrl}?${normalizedParams.toString()}`
    : baseUrl;
    
  return new Request(cacheUrl);
}

// Clean up old AI cache entries
async function cleanupAICache() {
  try {
    const cache = await caches.open(CACHE_NAMES.AI_RESPONSES);
    const requests = await cache.keys();
    
    let deletedCount = 0;
    const now = Date.now();
    
    for (const request of requests) {
      const response = await cache.match(request);
      if (response && isExpired(response, CACHE_EXPIRATION.AI_RESPONSES)) {
        await cache.delete(request);
        deletedCount++;
      }
    }
    
    // If still too many entries, delete oldest ones
    const remainingRequests = await cache.keys();
    if (remainingRequests.length > 100) { // Max 100 AI responses
      const sortedRequests = remainingRequests
        .map(req => ({ request: req, date: new Date(req.headers.get('date') || 0) }))
        .sort((a, b) => a.date - b.date);
        
      for (let i = 0; i < remainingRequests.length - 100; i++) {
        await cache.delete(sortedRequests[i].request);
        deletedCount++;
      }
    }
    
    if (deletedCount > 0) {
      console.log(`[SW] Cleaned up ${deletedCount} old AI cache entries`);
    }
    
  } catch (error) {
    console.warn('[SW] AI cache cleanup failed:', error);
  }
}

// Clean up dynamic cache entries
async function cleanupDynamicCache() {
  try {
    const cache = await caches.open(CACHE_NAMES.DYNAMIC);
    const requests = await cache.keys();
    
    if (requests.length < 200) return; // Only cleanup if we have too many entries
    
    let deletedCount = 0;
    
    // Sort by date and remove oldest entries
    const sortedRequests = [];
    for (const request of requests) {
      const response = await cache.match(request);
      if (response) {
        const date = new Date(response.headers.get('date') || 0);
        sortedRequests.push({ request, date });
      }
    }
    
    sortedRequests.sort((a, b) => a.date - b.date);
    
    // Keep only the newest 150 entries
    for (let i = 0; i < sortedRequests.length - 150; i++) {
      await cache.delete(sortedRequests[i].request);
      deletedCount++;
    }
    
    if (deletedCount > 0) {
      console.log(`[SW] Cleaned up ${deletedCount} old dynamic cache entries`);
    }
    
  } catch (error) {
    console.warn('[SW] Dynamic cache cleanup failed:', error);
  }
}

// Clean up old caches from previous versions
async function cleanupOldCaches() {
  try {
    const cacheNames = await caches.keys();
    const currentCaches = new Set(Object.values(CACHE_NAMES));
    
    const deletionPromises = cacheNames
      .filter(cacheName => 
        cacheName.startsWith(CACHE_PREFIX) && !currentCaches.has(cacheName)
      )
      .map(cacheName => {
        console.log(`[SW] Deleting old cache: ${cacheName}`);
        return caches.delete(cacheName);
      });
    
    const results = await Promise.allSettled(deletionPromises);
    const successful = results.filter(r => r.status === 'fulfilled').length;
    
    if (successful > 0) {
      console.log(`[SW] Cleaned up ${successful} old caches`);
    }
    
  } catch (error) {
    console.error('[SW] Old cache cleanup failed:', error);
  }
}

// ==================== STORAGE MANAGEMENT ====================

// Initialize storage management
async function initializeStorageManagement() {
  try {
    // Check storage quota and usage
    if (navigator.storage && navigator.storage.estimate) {
      const estimate = await navigator.storage.estimate();
      const usagePercent = estimate.usage / estimate.quota * 100;
      
      console.log(`[SW] Storage usage: ${Math.round(usagePercent)}% (${formatBytes(estimate.usage)} / ${formatBytes(estimate.quota)})`);
      
      if (usagePercent > 80) {
        console.warn('[SW] Storage usage high, initiating cleanup');
        await performStorageCleanup();
      }
    }
    
    // Set up periodic storage monitoring
    setInterval(monitorStorage, 5 * 60 * 1000); // Every 5 minutes
    
  } catch (error) {
    console.warn('[SW] Storage management initialization failed:', error);
  }
}

// Monitor storage usage
async function monitorStorage() {
  try {
    if (navigator.storage && navigator.storage.estimate) {
      const estimate = await navigator.storage.estimate();
      const usagePercent = estimate.usage / estimate.quota * 100;
      
      if (usagePercent > STORAGE_QUOTAS.WARNING_THRESHOLD / STORAGE_QUOTAS.MAX_TOTAL * 100) {
        console.warn(`[SW] Storage usage warning: ${Math.round(usagePercent)}%`);
        await performStorageCleanup();
        
        // Notify clients about storage pressure
        broadcastToClients({
          type: 'STORAGE_WARNING',
          usage: estimate.usage,
          quota: estimate.quota,
          percent: usagePercent,
          timestamp: Date.now()
        });
      }
    }
  } catch (error) {
    console.debug('[SW] Storage monitoring error:', error);
  }
}

// Perform storage cleanup
async function performStorageCleanup() {
  console.log('[SW] Performing storage cleanup');
  
  try {
    await Promise.all([
      cleanupAICache(),
      cleanupDynamicCache(),
      cleanupOldCaches(),
      cleanupBackgroundSyncQueues()
    ]);
    
    console.log('[SW] Storage cleanup completed');
  } catch (error) {
    console.error('[SW] Storage cleanup failed:', error);
  }
}

// Format bytes for display
function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// ==================== BACKGROUND SYNC QUEUE MANAGEMENT ====================

// Initialize sync queues
async function initializeSyncQueues() {
  try {
    // Initialize IndexedDB for queue storage
    await initializeQueueStorage();
    console.log('[SW] Background sync queues initialized');
  } catch (error) {
    console.error('[SW] Queue initialization failed:', error);
  }
}

// Initialize IndexedDB for queue storage
function initializeQueueStorage() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('WriteMagicSyncQueues', 1);
    
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
    
    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      
      // Create object stores for each queue
      for (const queueName of Object.values(SYNC_QUEUES)) {
        if (!db.objectStoreNames.contains(queueName)) {
          const store = db.createObjectStore(queueName, { keyPath: 'id', autoIncrement: true });
          store.createIndex('timestamp', 'timestamp', { unique: false });
          store.createIndex('retryCount', 'retryCount', { unique: false });
        }
      }
    };
  });
}

// Queue request for background sync
async function queueForBackgroundSync(queueName, item) {
  try {
    const db = await initializeQueueStorage();
    const transaction = db.transaction([queueName], 'readwrite');
    const store = transaction.objectStore(queueName);
    
    await new Promise((resolve, reject) => {
      const request = store.add(item);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    
    // Register for background sync
    if (self.registration && self.registration.sync) {
      await self.registration.sync.register(`writemagic-${queueName}`);
    }
    
    console.log(`[SW] Queued item for background sync: ${queueName}`);
  } catch (error) {
    console.error(`[SW] Failed to queue item for ${queueName}:`, error);
  }
}

// Get queued items from storage
async function getQueuedItems(queueName) {
  try {
    const db = await initializeQueueStorage();
    const transaction = db.transaction([queueName], 'readonly');
    const store = transaction.objectStore(queueName);
    
    return new Promise((resolve, reject) => {
      const request = store.getAll();
      request.onsuccess = () => resolve(request.result || []);
      request.onerror = () => reject(request.error);
    });
  } catch (error) {
    console.error(`[SW] Failed to get queued items for ${queueName}:`, error);
    return [];
  }
}

// Remove queued item
async function removeQueuedItem(queueName, itemId) {
  try {
    const db = await initializeQueueStorage();
    const transaction = db.transaction([queueName], 'readwrite');
    const store = transaction.objectStore(queueName);
    
    return new Promise((resolve, reject) => {
      const request = store.delete(itemId);
      request.onsuccess = () => resolve(true);
      request.onerror = () => reject(request.error);
    });
  } catch (error) {
    console.error(`[SW] Failed to remove queued item from ${queueName}:`, error);
    return false;
  }
}

// Update queued item
async function updateQueuedItem(queueName, itemId, updatedItem) {
  try {
    const db = await initializeQueueStorage();
    const transaction = db.transaction([queueName], 'readwrite');
    const store = transaction.objectStore(queueName);
    
    updatedItem.id = itemId;
    
    return new Promise((resolve, reject) => {
      const request = store.put(updatedItem);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  } catch (error) {
    console.error(`[SW] Failed to update queued item in ${queueName}:`, error);
    return false;
  }
}

// Cleanup background sync queues
async function cleanupBackgroundSyncQueues() {
  try {
    for (const queueName of Object.values(SYNC_QUEUES)) {
      const items = await getQueuedItems(queueName);
      const cutoffTime = Date.now() - (7 * 24 * 60 * 60 * 1000); // 7 days
      
      for (const item of items) {
        if (item.timestamp < cutoffTime || item.retryCount >= 5) {
          await removeQueuedItem(queueName, item.id);
        }
      }
    }
    
    console.log('[SW] Background sync queues cleaned up');
  } catch (error) {
    console.warn('[SW] Queue cleanup failed:', error);
  }
}

// ==================== REQUEST SERIALIZATION ====================

// Serialize request for storage
async function serializeRequest(request) {
  const serialized = {
    url: request.url,
    method: request.method,
    headers: {},
    body: null
  };
  
  // Serialize headers
  for (const [key, value] of request.headers.entries()) {
    serialized.headers[key] = value;
  }
  
  // Serialize body if present
  if (request.body && ['POST', 'PUT', 'PATCH'].includes(request.method)) {
    try {
      serialized.body = await request.text();
    } catch (error) {
      console.warn('[SW] Failed to serialize request body:', error);
    }
  }
  
  return serialized;
}

// Deserialize request from storage
async function deserializeRequest(serializedRequest) {
  const options = {
    method: serializedRequest.method,
    headers: serializedRequest.headers
  };
  
  if (serializedRequest.body) {
    options.body = serializedRequest.body;
  }
  
  return new Request(serializedRequest.url, options);
}

// ==================== NETWORK MONITORING ====================

// Initialize network monitoring
async function initializeNetworkMonitoring() {
  isOnline = navigator.onLine;
  
  // Set up network quality detection
  if ('connection' in navigator) {
    const connection = navigator.connection;
    networkQuality = getNetworkQuality(connection);
    
    connection.addEventListener('change', () => {
      networkQuality = getNetworkQuality(connection);
      broadcastNetworkStatus();
    });
  }
  
  // Set up online/offline detection
  self.addEventListener('online', () => {
    isOnline = true;
    networkQuality = navigator.connection ? getNetworkQuality(navigator.connection) : 'good';
    broadcastNetworkStatus();
  });
  
  self.addEventListener('offline', () => {
    isOnline = false;
    networkQuality = 'offline';
    broadcastNetworkStatus();
  });
  
  console.log(`[SW] Network monitoring initialized: ${isOnline ? 'online' : 'offline'}, quality: ${networkQuality}`);
}

// Get network quality based on connection
function getNetworkQuality(connection) {
  if (!connection) return 'good';
  
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

// Handle network status update from client
function handleNetworkStatusUpdate(online, quality) {
  isOnline = online;
  if (quality) {
    networkQuality = quality;
  }
  console.log(`[SW] Network status updated: ${online ? 'online' : 'offline'}, quality: ${networkQuality}`);
}

// Broadcast network status to clients
function broadcastNetworkStatus() {
  broadcastToClients({
    type: 'NETWORK_STATUS',
    isOnline,
    quality: networkQuality,
    timestamp: Date.now()
  });
}

// ==================== CLIENT COMMUNICATION ====================

// Broadcast message to all clients
async function broadcastToClients(message) {
  try {
    const clients = await self.clients.matchAll();
    for (const client of clients) {
      client.postMessage(message);
    }
  } catch (error) {
    console.warn('[SW] Failed to broadcast to clients:', error);
  }
}

// ==================== PERFORMANCE OPTIMIZATION HANDLERS ====================

// Handle cache optimization request
async function handleCacheOptimization(event) {
  try {
    console.log('[SW] Optimizing cache strategies...');
    
    // Implement aggressive cache cleanup
    await performStorageCleanup();
    
    // Optimize cache strategies for WASM resources
    await optimizeWasmCaching();
    
    // Preload critical resources based on usage patterns
    await preloadCriticalResources();
    
    event.ports[0]?.postMessage({ 
      success: true, 
      message: 'Cache optimization completed',
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error('[SW] Cache optimization failed:', error);
    event.ports[0]?.postMessage({ 
      error: error.message,
      timestamp: Date.now()
    });
  }
}

// Handle performance metrics request
async function handlePerformanceMetricsRequest(event) {
  try {
    const metrics = {
      cacheStats: await getCacheStatistics(),
      networkStats: getNetworkStatistics(),
      storageStats: await getStorageStatistics(),
      wasmStats: await getWasmStatistics(),
      timestamp: Date.now()
    };
    
    event.ports[0]?.postMessage({
      success: true,
      metrics,
      timestamp: Date.now()
    });
    
  } catch (error) {
    console.error('[SW] Performance metrics collection failed:', error);
    event.ports[0]?.postMessage({ 
      error: error.message,
      timestamp: Date.now()
    });
  }
}

// Optimize WASM caching strategies
async function optimizeWasmCaching() {
  try {
    const wasmCache = await caches.open(CACHE_NAMES.WASM);
    const wasmRequests = await wasmCache.keys();
    
    // Check and validate all WASM resources
    let optimizedCount = 0;
    
    for (const request of wasmRequests) {
      const response = await wasmCache.match(request);
      
      if (response) {
        // Check if WASM resource needs revalidation
        const shouldRevalidate = await shouldRevalidateWasmResource(response, request);
        
        if (shouldRevalidate) {
          try {
            const freshResponse = await fetch(request);
            if (freshResponse.ok) {
              await wasmCache.put(request, freshResponse.clone());
              optimizedCount++;
            }
          } catch (error) {
            console.debug('[SW] Failed to revalidate WASM resource:', request.url);
          }
        }
      }
    }
    
    console.log(`[SW] Optimized ${optimizedCount} WASM resources`);
    
  } catch (error) {
    console.error('[SW] WASM cache optimization failed:', error);
  }
}

// Check if WASM resource should be revalidated
async function shouldRevalidateWasmResource(response, request) {
  // Check age of cached resource
  const dateHeader = response.headers.get('date');
  if (dateHeader) {
    const cacheDate = new Date(dateHeader);
    const age = Date.now() - cacheDate.getTime();
    
    // Revalidate WASM resources older than 1 hour
    if (age > 60 * 60 * 1000) {
      return true;
    }
  }
  
  // Check if integrity validation fails
  if (request.url.includes('.wasm')) {
    try {
      const isValid = await validateWasmIntegrity(response.clone());
      return !isValid;
    } catch (error) {
      return true; // Revalidate on integrity check failure
    }
  }
  
  return false;
}

// Preload critical resources based on usage patterns
async function preloadCriticalResources() {
  const criticalResources = [
    '/styles/critical.css',
    '/scripts/core.js',
    '/src/js/utils/wasm-loader.js',
    '/src/js/utils/progressive-loader.js'
  ];
  
  const cache = await caches.open(CACHE_NAMES.DYNAMIC);
  let preloadedCount = 0;
  
  for (const resourceUrl of criticalResources) {
    try {
      // Check if already cached
      const cached = await cache.match(resourceUrl);
      if (!cached) {
        const response = await fetch(resourceUrl);
        if (response.ok) {
          await cache.put(resourceUrl, response);
          preloadedCount++;
        }
      }
    } catch (error) {
      console.debug(`[SW] Failed to preload ${resourceUrl}:`, error);
    }
  }
  
  console.log(`[SW] Preloaded ${preloadedCount} critical resources`);
}

// Get cache statistics
async function getCacheStatistics() {
  const stats = {};
  
  try {
    for (const [name, cacheName] of Object.entries(CACHE_NAMES)) {
      const cache = await caches.open(cacheName);
      const requests = await cache.keys();
      
      let totalSize = 0;
      let resourceCount = 0;
      
      for (const request of requests) {
        const response = await cache.match(request);
        if (response) {
          const size = parseInt(response.headers.get('content-length') || '0');
          totalSize += size;
          resourceCount++;
        }
      }
      
      stats[name.toLowerCase()] = {
        resourceCount,
        totalSize,
        cacheName
      };
    }
  } catch (error) {
    console.error('[SW] Cache statistics collection failed:', error);
  }
  
  return stats;
}

// Get network statistics
function getNetworkStatistics() {
  return {
    isOnline,
    networkQuality,
    effectiveType: navigator.connection?.effectiveType || 'unknown',
    downlink: navigator.connection?.downlink || 0,
    rtt: navigator.connection?.rtt || 0
  };
}

// Get storage statistics
async function getStorageStatistics() {
  try {
    const estimate = await navigator.storage?.estimate?.();
    
    return {
      quota: estimate?.quota || 0,
      usage: estimate?.usage || 0,
      usagePercent: estimate ? (estimate.usage / estimate.quota) * 100 : 0,
      available: estimate ? estimate.quota - estimate.usage : 0
    };
  } catch (error) {
    console.error('[SW] Storage statistics collection failed:', error);
    return {};
  }
}

// Get WASM-specific statistics
async function getWasmStatistics() {
  try {
    const wasmCache = await caches.open(CACHE_NAMES.WASM);
    const requests = await wasmCache.keys();
    
    let wasmFileCount = 0;
    let jsFileCount = 0;
    let totalWasmSize = 0;
    let totalJsSize = 0;
    
    for (const request of requests) {
      const response = await wasmCache.match(request);
      if (response) {
        const size = parseInt(response.headers.get('content-length') || '0');
        
        if (request.url.endsWith('.wasm')) {
          wasmFileCount++;
          totalWasmSize += size;
        } else if (request.url.endsWith('.js')) {
          jsFileCount++;
          totalJsSize += size;
        }
      }
    }
    
    return {
      wasmFiles: wasmFileCount,
      jsFiles: jsFileCount,
      totalWasmSize,
      totalJsSize,
      totalSize: totalWasmSize + totalJsSize,
      compressionRatio: totalWasmSize > 0 ? (totalWasmSize / (totalWasmSize + totalJsSize)) : 0
    };
    
  } catch (error) {
    console.error('[SW] WASM statistics collection failed:', error);
    return {};
  }
}

console.log('[SW] WriteMagic Enhanced Service Worker initialized v' + CACHE_VERSION);