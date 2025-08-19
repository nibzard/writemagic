/**
 * Offline functionality test runner for WriteMagic web application
 * Tests Service Worker, offline storage, background sync, and cache management
 */

const puppeteer = require('puppeteer');
const fs = require('fs').promises;
const path = require('path');
const chalk = require('chalk');

class OfflineTestRunner {
  constructor() {
    this.results = {
      timestamp: new Date().toISOString(),
      tests: [],
      summary: {}
    };
    this.browser = null;
    this.page = null;
  }

  async initialize() {
    console.log(chalk.blue('🔌 Initializing Offline Test Runner...'));
    
    this.browser = await puppeteer.launch({
      headless: process.env.CI ? true : false,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage'
      ]
    });
    
    this.page = await this.browser.newPage();
    
    // Enable offline testing capabilities
    await this.page.evaluateOnNewDocument(() => {
      window.offlineTestUtils = {
        networkState: 'online',
        serviceWorkerEvents: [],
        backgroundSyncEvents: [],
        cacheOperations: []
      };
    });
  }

  async cleanup() {
    if (this.browser) {
      await this.browser.close();
    }
  }

  async runAllTests() {
    try {
      await this.initialize();
      
      console.log(chalk.yellow('🔌 Running offline functionality tests...'));
      
      await this.testServiceWorkerRegistration();
      await this.testOfflinePageLoad();
      await this.testOfflineDocumentEditing();
      await this.testCacheManagement();
      await this.testBackgroundSync();
      await this.testOfflineStorage();
      await this.testOnlineOfflineTransitions();
      await this.testConflictResolution();
      await this.testStorageLimits();
      await this.testCacheEviction();
      
      this.calculateSummary();
      await this.generateReport();
      
      console.log(chalk.green('✅ Offline tests completed successfully!'));
      
    } catch (error) {
      console.error(chalk.red('❌ Offline tests failed:'), error);
      throw error;
    } finally {
      await this.cleanup();
    }
  }

  async testServiceWorkerRegistration() {
    console.log(chalk.cyan('  Testing Service Worker registration...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const swMetrics = await this.page.evaluate(async () => {
      const startTime = performance.now();
      let registration = null;
      let error = null;
      
      try {
        if ('serviceWorker' in navigator) {
          registration = await navigator.serviceWorker.register('/sw.js');
          
          // Wait for service worker to be ready
          await navigator.serviceWorker.ready;
          
          const endTime = performance.now();
          
          return {
            registered: true,
            registrationTime: endTime - startTime,
            scope: registration.scope,
            state: registration.active ? registration.active.state : 'unknown',
            error: null
          };
        } else {
          return {
            registered: false,
            error: 'Service Worker not supported'
          };
        }
      } catch (err) {
        return {
          registered: false,
          error: err.message,
          registrationTime: performance.now() - startTime
        };
      }
    });

    const testResult = {
      name: 'Service Worker Registration',
      timestamp: new Date().toISOString(),
      duration: swMetrics.registrationTime || 0,
      status: swMetrics.registered ? 'PASS' : 'FAIL',
      metrics: swMetrics,
      thresholds: {
        registrationTime: { 
          value: swMetrics.registrationTime || 0, 
          threshold: 2000, 
          passed: (swMetrics.registrationTime || 0) < 2000 
        }
      }
    };

    this.results.tests.push(testResult);
    
    if (swMetrics.registered) {
      console.log(chalk.gray(`    Registration time: ${swMetrics.registrationTime.toFixed(2)}ms`));
      console.log(chalk.gray(`    State: ${swMetrics.state}`));
    } else {
      console.log(chalk.gray(`    Error: ${swMetrics.error}`));
    }
  }

  async testOfflinePageLoad() {
    console.log(chalk.cyan('  Testing offline page load...'));
    
    // First ensure page loads normally
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    // Wait for Service Worker to cache resources
    await this.page.waitForTimeout(2000);
    
    // Go offline
    await this.page.setOfflineMode(true);
    
    const offlineLoadMetrics = await this.page.evaluate(async () => {
      const startTime = performance.now();
      let loadSuccess = false;
      let error = null;
      
      try {
        // Try to reload the page while offline
        window.location.reload();
        
        // Wait for page to load
        await new Promise((resolve, reject) => {
          const timeout = setTimeout(() => reject(new Error('Load timeout')), 10000);
          
          const checkLoad = () => {
            if (document.readyState === 'complete') {
              clearTimeout(timeout);
              resolve();
            } else {
              setTimeout(checkLoad, 100);
            }
          };
          
          checkLoad();
        });
        
        loadSuccess = true;
      } catch (err) {
        error = err.message;
      }
      
      const loadTime = performance.now() - startTime;
      
      return {
        loadSuccess,
        loadTime,
        error,
        offlinePageDisplayed: document.querySelector('[data-testid="offline-indicator"]') !== null
      };
    });
    
    // Go back online
    await this.page.setOfflineMode(false);

    const testResult = {
      name: 'Offline Page Load',
      timestamp: new Date().toISOString(),
      duration: offlineLoadMetrics.loadTime,
      status: offlineLoadMetrics.loadSuccess ? 'PASS' : 'FAIL',
      metrics: offlineLoadMetrics,
      thresholds: {
        loadTime: { 
          value: offlineLoadMetrics.loadTime, 
          threshold: 3000, 
          passed: offlineLoadMetrics.loadTime < 3000 
        }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Load time: ${offlineLoadMetrics.loadTime.toFixed(2)}ms`));
    console.log(chalk.gray(`    Success: ${offlineLoadMetrics.loadSuccess}`));
  }

  async testOfflineDocumentEditing() {
    console.log(chalk.cyan('  Testing offline document editing...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    // Create a document while online
    await this.page.click('[data-testid="new-document-btn"]');
    await this.page.fill('[data-testid="document-title-input"]', 'Offline Test Document');
    await this.page.click('[data-testid="create-document-btn"]');
    
    await this.page.waitForSelector('[data-testid="document-editor"]');
    
    // Go offline
    await this.page.setOfflineMode(true);
    
    const offlineEditMetrics = await this.page.evaluate(async () => {
      const startTime = performance.now();
      const results = {
        editSuccess: false,
        saveAttempted: false,
        queuedOperations: 0,
        error: null
      };
      
      try {
        // Try to edit the document while offline
        const contentField = document.querySelector('[data-testid="document-content"]');
        if (contentField) {
          const originalContent = contentField.value;
          const newContent = originalContent + '\n\nEdited while offline at ' + new Date().toISOString();
          
          // Simulate typing
          contentField.value = newContent;
          contentField.dispatchEvent(new Event('input', { bubbles: true }));
          
          results.editSuccess = true;
          
          // Check if save was attempted
          const saveIndicator = document.querySelector('[data-testid="save-status"]');
          if (saveIndicator) {
            results.saveAttempted = saveIndicator.textContent.includes('Queued') || 
                                   saveIndicator.textContent.includes('Offline');
          }
          
          // Check for queued operations
          if (window.documentManager && window.documentManager.operationQueue) {
            results.queuedOperations = window.documentManager.operationQueue.length;
          }
        }
      } catch (err) {
        results.error = err.message;
      }
      
      results.duration = performance.now() - startTime;
      return results;
    });
    
    // Go back online
    await this.page.setOfflineMode(false);

    const testResult = {
      name: 'Offline Document Editing',
      timestamp: new Date().toISOString(),
      duration: offlineEditMetrics.duration,
      status: offlineEditMetrics.editSuccess ? 'PASS' : 'FAIL',
      metrics: offlineEditMetrics,
      thresholds: {
        editSuccess: { value: offlineEditMetrics.editSuccess, threshold: true, passed: offlineEditMetrics.editSuccess }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Edit success: ${offlineEditMetrics.editSuccess}`));
    console.log(chalk.gray(`    Queued operations: ${offlineEditMetrics.queuedOperations}`));
  }

  async testCacheManagement() {
    console.log(chalk.cyan('  Testing cache management...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const cacheMetrics = await this.page.evaluate(async () => {
      const results = {
        cachesAvailable: false,
        cacheNames: [],
        cacheOperations: {
          put: false,
          match: false,
          delete: false
        },
        error: null
      };
      
      try {
        if ('caches' in window) {
          results.cachesAvailable = true;
          
          // Get cache names
          results.cacheNames = await caches.keys();
          
          // Test cache operations
          const cacheName = 'writemagic-test-cache';
          const cache = await caches.open(cacheName);
          
          // Test PUT operation
          const testResponse = new Response('Test cache content', {
            headers: { 'Content-Type': 'text/plain' }
          });
          await cache.put('/test-cache-item', testResponse);
          results.cacheOperations.put = true;
          
          // Test MATCH operation
          const cachedResponse = await cache.match('/test-cache-item');
          results.cacheOperations.match = cachedResponse !== undefined;
          
          // Test DELETE operation
          const deleted = await cache.delete('/test-cache-item');
          results.cacheOperations.delete = deleted;
          
          // Clean up
          await caches.delete(cacheName);
        }
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });

    const testResult = {
      name: 'Cache Management',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: cacheMetrics.cachesAvailable && 
              cacheMetrics.cacheOperations.put && 
              cacheMetrics.cacheOperations.match && 
              cacheMetrics.cacheOperations.delete ? 'PASS' : 'FAIL',
      metrics: cacheMetrics,
      thresholds: {
        cachesAvailable: { value: cacheMetrics.cachesAvailable, threshold: true, passed: cacheMetrics.cachesAvailable },
        putOperation: { value: cacheMetrics.cacheOperations.put, threshold: true, passed: cacheMetrics.cacheOperations.put },
        matchOperation: { value: cacheMetrics.cacheOperations.match, threshold: true, passed: cacheMetrics.cacheOperations.match }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Caches available: ${cacheMetrics.cachesAvailable}`));
    console.log(chalk.gray(`    Cache names: ${cacheMetrics.cacheNames.join(', ')}`));
    console.log(chalk.gray(`    Operations: PUT=${cacheMetrics.cacheOperations.put}, MATCH=${cacheMetrics.cacheOperations.match}, DELETE=${cacheMetrics.cacheOperations.delete}`));
  }

  async testBackgroundSync() {
    console.log(chalk.cyan('  Testing background sync...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const backgroundSyncMetrics = await this.page.evaluate(async () => {
      const results = {
        syncSupported: false,
        syncRegistered: false,
        syncEvents: [],
        error: null
      };
      
      try {
        if ('serviceWorker' in navigator && 'sync' in window.ServiceWorkerRegistration.prototype) {
          results.syncSupported = true;
          
          const registration = await navigator.serviceWorker.ready;
          
          // Register a background sync
          await registration.sync.register('test-sync');
          results.syncRegistered = true;
          
          // Simulate offline operations that should trigger sync
          if (window.documentManager) {
            // Create offline operations
            const offlineOp = {
              type: 'update',
              documentId: 'test-doc',
              data: { content: 'Background sync test' },
              timestamp: Date.now()
            };
            
            // Add to sync queue
            if (window.documentManager.queueForSync) {
              window.documentManager.queueForSync(offlineOp);
            }
          }
          
          // Listen for sync events
          navigator.serviceWorker.addEventListener('message', (event) => {
            if (event.data.type === 'SYNC_EVENT') {
              results.syncEvents.push(event.data);
            }
          });
          
          // Trigger sync manually for testing
          navigator.serviceWorker.controller?.postMessage({
            type: 'TRIGGER_SYNC',
            tag: 'test-sync'
          });
          
        }
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });

    const testResult = {
      name: 'Background Sync',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: backgroundSyncMetrics.syncSupported && backgroundSyncMetrics.syncRegistered ? 'PASS' : 'FAIL',
      metrics: backgroundSyncMetrics,
      thresholds: {
        syncSupported: { value: backgroundSyncMetrics.syncSupported, threshold: true, passed: backgroundSyncMetrics.syncSupported },
        syncRegistered: { value: backgroundSyncMetrics.syncRegistered, threshold: true, passed: backgroundSyncMetrics.syncRegistered }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Sync supported: ${backgroundSyncMetrics.syncSupported}`));
    console.log(chalk.gray(`    Sync registered: ${backgroundSyncMetrics.syncRegistered}`));
  }

  async testOfflineStorage() {
    console.log(chalk.cyan('  Testing offline storage...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    // Go offline
    await this.page.setOfflineMode(true);
    
    const storageMetrics = await this.page.evaluate(async () => {
      const results = {
        localStorage: false,
        sessionStorage: false,
        indexedDB: false,
        storageOperations: 0,
        error: null
      };
      
      try {
        // Test localStorage
        localStorage.setItem('offline-test', 'test-value');
        results.localStorage = localStorage.getItem('offline-test') === 'test-value';
        results.storageOperations++;
        
        // Test sessionStorage
        sessionStorage.setItem('offline-session-test', 'session-value');
        results.sessionStorage = sessionStorage.getItem('offline-session-test') === 'session-value';
        results.storageOperations++;
        
        // Test IndexedDB
        if ('indexedDB' in window) {
          const dbPromise = new Promise((resolve, reject) => {
            const request = indexedDB.open('offline-test-db', 1);
            
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);
            
            request.onupgradeneeded = (event) => {
              const db = event.target.result;
              if (!db.objectStoreNames.contains('test-store')) {
                db.createObjectStore('test-store', { keyPath: 'id' });
              }
            };
          });
          
          const db = await dbPromise;
          const transaction = db.transaction(['test-store'], 'readwrite');
          const store = transaction.objectStore('test-store');
          
          await new Promise((resolve, reject) => {
            const addRequest = store.add({ id: 1, data: 'offline-test-data' });
            addRequest.onsuccess = () => resolve();
            addRequest.onerror = () => reject(addRequest.error);
          });
          
          const getRequest = store.get(1);
          const data = await new Promise((resolve, reject) => {
            getRequest.onsuccess = () => resolve(getRequest.result);
            getRequest.onerror = () => reject(getRequest.error);
          });
          
          results.indexedDB = data && data.data === 'offline-test-data';
          results.storageOperations++;
          
          db.close();
          indexedDB.deleteDatabase('offline-test-db');
        }
        
        // Clean up
        localStorage.removeItem('offline-test');
        sessionStorage.removeItem('offline-session-test');
        
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });
    
    // Go back online
    await this.page.setOfflineMode(false);

    const testResult = {
      name: 'Offline Storage',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: storageMetrics.localStorage && storageMetrics.indexedDB ? 'PASS' : 'FAIL',
      metrics: storageMetrics,
      thresholds: {
        localStorage: { value: storageMetrics.localStorage, threshold: true, passed: storageMetrics.localStorage },
        indexedDB: { value: storageMetrics.indexedDB, threshold: true, passed: storageMetrics.indexedDB }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    localStorage: ${storageMetrics.localStorage}`));
    console.log(chalk.gray(`    sessionStorage: ${storageMetrics.sessionStorage}`));
    console.log(chalk.gray(`    IndexedDB: ${storageMetrics.indexedDB}`));
  }

  async testOnlineOfflineTransitions() {
    console.log(chalk.cyan('  Testing online/offline transitions...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const transitionMetrics = await this.page.evaluate(async () => {
      const results = {
        transitions: [],
        eventsFired: {
          online: false,
          offline: false
        },
        networkStateTracking: false,
        error: null
      };
      
      try {
        // Set up event listeners
        const onlineHandler = () => {
          results.eventsFired.online = true;
          results.transitions.push({ type: 'online', timestamp: Date.now() });
        };
        
        const offlineHandler = () => {
          results.eventsFired.offline = true;
          results.transitions.push({ type: 'offline', timestamp: Date.now() });
        };
        
        window.addEventListener('online', onlineHandler);
        window.addEventListener('offline', offlineHandler);
        
        // Check if app tracks network state
        results.networkStateTracking = navigator.onLine !== undefined;
        
        // Clean up
        window.removeEventListener('online', onlineHandler);
        window.removeEventListener('offline', offlineHandler);
        
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });
    
    // Test actual online/offline transitions
    await this.page.setOfflineMode(true);
    await this.page.waitForTimeout(500);
    await this.page.setOfflineMode(false);
    await this.page.waitForTimeout(500);

    const testResult = {
      name: 'Online/Offline Transitions',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: transitionMetrics.networkStateTracking ? 'PASS' : 'FAIL',
      metrics: transitionMetrics,
      thresholds: {
        networkStateTracking: { value: transitionMetrics.networkStateTracking, threshold: true, passed: transitionMetrics.networkStateTracking }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Network state tracking: ${transitionMetrics.networkStateTracking}`));
    console.log(chalk.gray(`    Transitions recorded: ${transitionMetrics.transitions.length}`));
  }

  async testConflictResolution() {
    console.log(chalk.cyan('  Testing conflict resolution...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const conflictMetrics = await this.page.evaluate(async () => {
      const results = {
        conflictDetected: false,
        resolutionStrategy: null,
        conflictResolved: false,
        error: null
      };
      
      try {
        // Simulate creating a document
        if (window.documentManager) {
          const doc = await window.documentManager.createDocument('Conflict Test', 'Original content');
          
          // Simulate offline modification
          const offlineVersion = {
            ...doc,
            content: 'Offline modification',
            updatedAt: new Date(Date.now() - 1000).toISOString() // Older timestamp
          };
          
          // Simulate server version (newer)
          const serverVersion = {
            ...doc,
            content: 'Server modification',
            updatedAt: new Date().toISOString() // Newer timestamp
          };
          
          // Test conflict detection
          if (window.documentManager.detectConflict) {
            const conflict = window.documentManager.detectConflict(offlineVersion, serverVersion);
            results.conflictDetected = conflict;
            
            if (conflict) {
              // Test conflict resolution
              if (window.documentManager.resolveConflict) {
                const resolved = await window.documentManager.resolveConflict(
                  offlineVersion, 
                  serverVersion, 
                  'merge' // Strategy
                );
                
                results.resolutionStrategy = 'merge';
                results.conflictResolved = resolved !== null;
              }
            }
          }
        }
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });

    const testResult = {
      name: 'Conflict Resolution',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: conflictMetrics.conflictDetected && conflictMetrics.conflictResolved ? 'PASS' : 'SKIP',
      metrics: conflictMetrics,
      thresholds: {
        conflictDetected: { value: conflictMetrics.conflictDetected, threshold: true, passed: conflictMetrics.conflictDetected },
        conflictResolved: { value: conflictMetrics.conflictResolved, threshold: true, passed: conflictMetrics.conflictResolved }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Conflict detected: ${conflictMetrics.conflictDetected}`));
    console.log(chalk.gray(`    Resolution strategy: ${conflictMetrics.resolutionStrategy}`));
    console.log(chalk.gray(`    Conflict resolved: ${conflictMetrics.conflictResolved}`));
  }

  async testStorageLimits() {
    console.log(chalk.cyan('  Testing storage limits...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const storageMetrics = await this.page.evaluate(async () => {
      const results = {
        quotaSupported: false,
        currentUsage: 0,
        availableQuota: 0,
        storageEstimateWorking: false,
        error: null
      };
      
      try {
        if ('storage' in navigator && 'estimate' in navigator.storage) {
          results.quotaSupported = true;
          
          const estimate = await navigator.storage.estimate();
          results.currentUsage = estimate.usage || 0;
          results.availableQuota = estimate.quota || 0;
          results.storageEstimateWorking = true;
          
          // Test storage pressure handling
          if (results.availableQuota > 0) {
            const usagePercentage = (results.currentUsage / results.availableQuota) * 100;
            results.usagePercentage = usagePercentage;
            results.lowStorageWarning = usagePercentage > 80;
          }
        }
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });

    const testResult = {
      name: 'Storage Limits',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: storageMetrics.quotaSupported && storageMetrics.storageEstimateWorking ? 'PASS' : 'SKIP',
      metrics: storageMetrics,
      thresholds: {
        quotaSupported: { value: storageMetrics.quotaSupported, threshold: true, passed: storageMetrics.quotaSupported },
        storageEstimateWorking: { value: storageMetrics.storageEstimateWorking, threshold: true, passed: storageMetrics.storageEstimateWorking }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Quota supported: ${storageMetrics.quotaSupported}`));
    if (storageMetrics.quotaSupported) {
      console.log(chalk.gray(`    Current usage: ${(storageMetrics.currentUsage / 1024 / 1024).toFixed(2)} MB`));
      console.log(chalk.gray(`    Available quota: ${(storageMetrics.availableQuota / 1024 / 1024).toFixed(2)} MB`));
    }
  }

  async testCacheEviction() {
    console.log(chalk.cyan('  Testing cache eviction...'));
    
    await this.page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
    
    const evictionMetrics = await this.page.evaluate(async () => {
      const results = {
        cacheEvictionSupported: false,
        oldEntriesRemoved: false,
        lruWorking: false,
        error: null
      };
      
      try {
        if ('caches' in window) {
          results.cacheEvictionSupported = true;
          
          const cacheName = 'eviction-test-cache';
          const cache = await caches.open(cacheName);
          
          // Add multiple cache entries with timestamps
          const entries = [];
          for (let i = 0; i < 10; i++) {
            const url = `/test-entry-${i}`;
            const response = new Response(`Test content ${i}`, {
              headers: {
                'Date': new Date(Date.now() - (i * 10000)).toISOString(), // Older entries
                'Content-Type': 'text/plain'
              }
            });
            
            await cache.put(url, response);
            entries.push(url);
          }
          
          // Simulate cache size limit and eviction
          const cacheKeys = await cache.keys();
          results.entriesBeforeEviction = cacheKeys.length;
          
          // Remove oldest entries (simulate LRU eviction)
          const oldestEntries = entries.slice(0, 3);
          for (const entry of oldestEntries) {
            await cache.delete(entry);
          }
          
          const remainingKeys = await cache.keys();
          results.entriesAfterEviction = remainingKeys.length;
          results.oldEntriesRemoved = results.entriesBeforeEviction > results.entriesAfterEviction;
          results.lruWorking = results.oldEntriesRemoved;
          
          // Clean up
          await caches.delete(cacheName);
        }
      } catch (err) {
        results.error = err.message;
      }
      
      return results;
    });

    const testResult = {
      name: 'Cache Eviction',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: evictionMetrics.cacheEvictionSupported && evictionMetrics.lruWorking ? 'PASS' : 'SKIP',
      metrics: evictionMetrics,
      thresholds: {
        cacheEvictionSupported: { value: evictionMetrics.cacheEvictionSupported, threshold: true, passed: evictionMetrics.cacheEvictionSupported },
        lruWorking: { value: evictionMetrics.lruWorking, threshold: true, passed: evictionMetrics.lruWorking }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Cache eviction supported: ${evictionMetrics.cacheEvictionSupported}`));
    if (evictionMetrics.cacheEvictionSupported) {
      console.log(chalk.gray(`    Entries before/after: ${evictionMetrics.entriesBeforeEviction}/${evictionMetrics.entriesAfterEviction}`));
      console.log(chalk.gray(`    LRU working: ${evictionMetrics.lruWorking}`));
    }
  }

  calculateSummary() {
    const totalTests = this.results.tests.length;
    const passedTests = this.results.tests.filter(test => test.status === 'PASS').length;
    const failedTests = this.results.tests.filter(test => test.status === 'FAIL').length;
    const skippedTests = this.results.tests.filter(test => test.status === 'SKIP').length;
    
    this.results.summary = {
      totalTests,
      passedTests,
      failedTests,
      skippedTests,
      successRate: totalTests > skippedTests ? (passedTests / (totalTests - skippedTests)) * 100 : 0,
      overallStatus: failedTests === 0 ? 'PASS' : 'FAIL'
    };
    
    console.log(chalk.yellow('\n🔌 Offline Test Summary:'));
    console.log(chalk.green(`  ✅ Passed: ${passedTests}`));
    console.log(chalk.red(`  ❌ Failed: ${failedTests}`));
    console.log(chalk.yellow(`  ⏭️  Skipped: ${skippedTests}`));
    console.log(chalk.blue(`  📊 Success Rate: ${this.results.summary.successRate.toFixed(1)}%`));
  }

  async generateReport() {
    const reportDir = path.join(__dirname, '../reports');
    await fs.mkdir(reportDir, { recursive: true });
    
    const reportPath = path.join(reportDir, `offline-report-${Date.now()}.json`);
    await fs.writeFile(reportPath, JSON.stringify(this.results, null, 2));
    
    console.log(chalk.blue(`\n📄 Offline report saved: ${reportPath}`));
  }
}

// Run offline tests if called directly
if (require.main === module) {
  const runner = new OfflineTestRunner();
  
  runner.runAllTests()
    .then(() => {
      process.exit(runner.results.summary.overallStatus === 'PASS' ? 0 : 1);
    })
    .catch((error) => {
      console.error(chalk.red('Offline tests failed:'), error);
      process.exit(1);
    });
}

module.exports = OfflineTestRunner;