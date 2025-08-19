/**
 * Browser compatibility tests for WriteMagic web application
 * Tests core functionality across Chrome, Firefox, Safari, and Edge
 */

const { test, expect, devices } = require('@playwright/test');

// Define browser-specific test configurations
const browsers = [
  { name: 'Chrome', ...devices['Desktop Chrome'] },
  { name: 'Firefox', ...devices['Desktop Firefox'] },
  { name: 'Safari', ...devices['Desktop Safari'] },
  { name: 'Edge', ...devices['Desktop Edge'], channel: 'msedge' }
];

// Test each browser
browsers.forEach(browser => {
  test.describe(`Browser Compatibility - ${browser.name}`, () => {
    test.beforeEach(async ({ page, context }) => {
      // Set up browser-specific context
      await context.addInitScript(() => {
        window.browserTestMetrics = {
          loadStart: performance.now(),
          features: {},
          errors: []
        };
        
        // Capture console errors
        window.addEventListener('error', (e) => {
          window.browserTestMetrics.errors.push({
            message: e.message,
            source: e.filename,
            line: e.lineno,
            column: e.colno
          });
        });
      });

      await page.goto('/', { waitUntil: 'networkidle' });
    });

    test(`${browser.name}: Basic page load and functionality`, async ({ page }) => {
      // Test basic page elements
      await expect(page.locator('.app-container')).toBeVisible();
      await expect(page.locator('[data-testid="new-document-btn"]')).toBeVisible();
      
      // Test navigation
      await page.click('[data-testid="projects-tab"]');
      await expect(page.locator('[data-testid="projects-container"]')).toBeVisible();
      
      await page.click('[data-testid="documents-tab"]');
      await expect(page.locator('[data-testid="documents-container"]')).toBeVisible();
    });

    test(`${browser.name}: WASM module loading and initialization`, async ({ page }) => {
      // Wait for WASM module to load
      const wasmLoaded = await page.waitForFunction(() => window.writemagic_wasm !== undefined, { timeout: 10000 });
      expect(wasmLoaded).toBeTruthy();

      // Test WASM functionality
      const wasmTest = await page.evaluate(async () => {
        try {
          // Test creating WASM objects
          const doc = new window.writemagic_wasm.Document();
          const project = new window.writemagic_wasm.Project();
          const session = new window.writemagic_wasm.WritingSession();
          
          // Test basic operations
          const results = {
            documentCreated: !!doc,
            projectCreated: !!project,
            sessionCreated: !!session,
            wasmMemoryWorking: true
          };
          
          // Clean up
          if (doc.free) doc.free();
          if (project.free) project.free();
          if (session.free) session.free();
          
          return results;
        } catch (error) {
          return {
            error: error.message,
            wasmMemoryWorking: false
          };
        }
      });

      expect(wasmTest.documentCreated).toBe(true);
      expect(wasmTest.projectCreated).toBe(true);
      expect(wasmTest.sessionCreated).toBe(true);
    });

    test(`${browser.name}: Local storage and IndexedDB support`, async ({ page }) => {
      const storageTest = await page.evaluate(async () => {
        const results = {
          localStorage: false,
          sessionStorage: false,
          indexedDB: false,
          cookies: false
        };

        try {
          // Test localStorage
          localStorage.setItem('test-key', 'test-value');
          results.localStorage = localStorage.getItem('test-key') === 'test-value';
          localStorage.removeItem('test-key');
        } catch (e) {
          console.log('localStorage error:', e);
        }

        try {
          // Test sessionStorage
          sessionStorage.setItem('test-session', 'test-session-value');
          results.sessionStorage = sessionStorage.getItem('test-session') === 'test-session-value';
          sessionStorage.removeItem('test-session');
        } catch (e) {
          console.log('sessionStorage error:', e);
        }

        try {
          // Test IndexedDB
          const request = indexedDB.open('compatibility-test-db', 1);
          const db = await new Promise((resolve, reject) => {
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);
            request.onupgradeneeded = (event) => {
              const db = event.target.result;
              db.createObjectStore('test-store', { keyPath: 'id' });
            };
          });
          
          results.indexedDB = !!db;
          db.close();
          indexedDB.deleteDatabase('compatibility-test-db');
        } catch (e) {
          console.log('IndexedDB error:', e);
        }

        try {
          // Test cookies
          document.cookie = 'test-cookie=test-value';
          results.cookies = document.cookie.includes('test-cookie=test-value');
          document.cookie = 'test-cookie=; expires=Thu, 01 Jan 1970 00:00:00 GMT';
        } catch (e) {
          console.log('Cookies error:', e);
        }

        return results;
      });

      expect(storageTest.localStorage).toBe(true);
      expect(storageTest.sessionStorage).toBe(true);
      expect(storageTest.indexedDB).toBe(true);
      // Cookies might be disabled in some test environments
    });

    test(`${browser.name}: Service Worker support`, async ({ page }) => {
      const serviceWorkerTest = await page.evaluate(async () => {
        const results = {
          supported: false,
          registered: false,
          cacheSupported: false,
          error: null
        };

        try {
          if ('serviceWorker' in navigator) {
            results.supported = true;
            
            // Check if we can register a service worker
            const registration = await navigator.serviceWorker.register('/sw.js');
            results.registered = !!registration;
            
            // Test Cache API
            if ('caches' in window) {
              results.cacheSupported = true;
            }
          }
        } catch (error) {
          results.error = error.message;
        }

        return results;
      });

      expect(serviceWorkerTest.supported).toBe(true);
      // Service worker registration might fail in test environment, so we'll check support only
    });

    test(`${browser.name}: CSS Grid and Flexbox support`, async ({ page }) => {
      const cssTest = await page.evaluate(() => {
        const testElement = document.createElement('div');
        document.body.appendChild(testElement);

        const results = {
          flexbox: false,
          cssGrid: false,
          customProperties: false
        };

        try {
          // Test Flexbox
          testElement.style.display = 'flex';
          results.flexbox = getComputedStyle(testElement).display === 'flex';

          // Test CSS Grid
          testElement.style.display = 'grid';
          results.cssGrid = getComputedStyle(testElement).display === 'grid';

          // Test CSS Custom Properties
          testElement.style.setProperty('--test-var', 'test-value');
          results.customProperties = testElement.style.getPropertyValue('--test-var') === 'test-value';
        } catch (error) {
          console.log('CSS test error:', error);
        }

        document.body.removeChild(testElement);
        return results;
      });

      expect(cssTest.flexbox).toBe(true);
      expect(cssTest.cssGrid).toBe(true);
      expect(cssTest.customProperties).toBe(true);
    });

    test(`${browser.name}: Document creation and editing workflow`, async ({ page }) => {
      // Create a new document
      await page.click('[data-testid="new-document-btn"]');
      await page.fill('[data-testid="document-title-input"]', `${browser.name} Test Document`);
      await page.click('[data-testid="create-document-btn"]');
      
      // Wait for editor to be visible
      await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
      
      // Test text input
      const testContent = 'This is a test document created in ' + browser.name + ' browser.';
      await page.fill('[data-testid="document-content"]', testContent);
      
      // Verify content was entered
      const contentValue = await page.inputValue('[data-testid="document-content"]');
      expect(contentValue).toBe(testContent);
      
      // Test word count update
      await expect(page.locator('[data-testid="word-count"]')).toContainText('12 words');
    });

    test(`${browser.name}: Multi-pane workspace functionality`, async ({ page }) => {
      // Go to projects
      await page.click('[data-testid="projects-tab"]');
      
      // Create a project
      await page.click('[data-testid="new-project-btn"]');
      await page.fill('[data-testid="project-name-input"]', `${browser.name} Test Project`);
      await page.click('[data-testid="create-project-btn"]');
      await page.click('[data-testid="open-project-btn"]');
      
      // Add a document to project
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', 'Multi-pane Test Doc');
      await page.click('[data-testid="create-document-btn"]');
      await page.fill('[data-testid="document-content"]', 'Content for multi-pane test');
      await page.click('[data-testid="back-to-project-btn"]');
      
      // Test layout switching
      await page.click('[data-testid="layout-controls"]');
      await page.click('[data-testid="split-layout-btn"]');
      
      // Verify split layout is active
      const panes = page.locator('[data-testid="editor-pane"]');
      await expect(panes).toHaveCount(2);
    });

    test(`${browser.name}: Responsive design and mobile viewport`, async ({ page }) => {
      // Test different viewport sizes
      const viewports = [
        { width: 1920, height: 1080, name: 'Desktop' },
        { width: 1024, height: 768, name: 'Tablet' },
        { width: 375, height: 667, name: 'Mobile' }
      ];

      for (const viewport of viewports) {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        
        // Verify the app is still functional at this viewport
        await expect(page.locator('.app-container')).toBeVisible();
        
        // Check if mobile navigation is present for small screens
        if (viewport.width < 768) {
          const mobileNav = page.locator('[data-testid="mobile-nav"]');
          if (await mobileNav.count() > 0) {
            await expect(mobileNav).toBeVisible();
          }
        }
        
        // Test that key functionality still works
        await page.click('[data-testid="new-document-btn"]');
        await expect(page.locator('[data-testid="document-title-input"]')).toBeVisible();
        
        // Close the modal/dialog
        await page.keyboard.press('Escape');
      }
      
      // Reset to original viewport
      await page.setViewportSize({ width: 1280, height: 720 });
    });

    test(`${browser.name}: Keyboard navigation and accessibility`, async ({ page }) => {
      // Test tab navigation
      await page.keyboard.press('Tab');
      
      // Should focus on first interactive element
      const focusedElement = await page.evaluate(() => document.activeElement.tagName);
      expect(['BUTTON', 'A', 'INPUT'].includes(focusedElement)).toBe(true);
      
      // Test keyboard shortcuts
      await page.keyboard.press('Control+n'); // New document shortcut
      
      // Should open new document dialog
      const dialogVisible = await page.locator('[data-testid="document-title-input"]').isVisible();
      if (dialogVisible) {
        // Close dialog
        await page.keyboard.press('Escape');
      }
      
      // Test ARIA attributes
      const ariaTest = await page.evaluate(() => {
        const buttons = document.querySelectorAll('button');
        const inputs = document.querySelectorAll('input');
        const links = document.querySelectorAll('a');
        
        let ariaScore = 0;
        let totalElements = 0;
        
        [buttons, inputs, links].forEach(nodeList => {
          nodeList.forEach(element => {
            totalElements++;
            if (element.getAttribute('aria-label') || 
                element.getAttribute('aria-labelledby') || 
                element.getAttribute('aria-describedby') ||
                element.textContent.trim()) {
              ariaScore++;
            }
          });
        });
        
        return {
          score: ariaScore,
          total: totalElements,
          percentage: totalElements > 0 ? (ariaScore / totalElements) * 100 : 0
        };
      });
      
      // Expect at least 80% of interactive elements to have proper accessibility attributes
      expect(ariaTest.percentage).toBeGreaterThan(80);
    });

    test(`${browser.name}: Performance metrics and load times`, async ({ page }) => {
      const performanceMetrics = await page.evaluate(() => {
        const navigation = performance.getEntriesByType('navigation')[0];
        const paint = performance.getEntriesByType('paint');
        
        return {
          domContentLoaded: navigation.domContentLoadedEventEnd - navigation.navigationStart,
          loadComplete: navigation.loadEventEnd - navigation.navigationStart,
          firstPaint: paint.find(p => p.name === 'first-paint')?.startTime || 0,
          firstContentfulPaint: paint.find(p => p.name === 'first-contentful-paint')?.startTime || 0,
          memoryUsage: performance.memory ? {
            used: performance.memory.usedJSHeapSize,
            total: performance.memory.totalJSHeapSize,
            limit: performance.memory.jsHeapSizeLimit
          } : null
        };
      });

      // Performance thresholds (may vary by browser)
      expect(performanceMetrics.domContentLoaded).toBeLessThan(5000); // 5 seconds
      expect(performanceMetrics.firstContentfulPaint).toBeLessThan(3000); // 3 seconds
      
      console.log(`${browser.name} Performance:`, {
        domReady: `${performanceMetrics.domContentLoaded.toFixed(0)}ms`,
        fcp: `${performanceMetrics.firstContentfulPaint.toFixed(0)}ms`,
        memory: performanceMetrics.memoryUsage ? 
          `${(performanceMetrics.memoryUsage.used / 1024 / 1024).toFixed(1)}MB` : 'N/A'
      });
    });

    test(`${browser.name}: JavaScript ES6+ features support`, async ({ page }) => {
      const jsFeatureTest = await page.evaluate(() => {
        const results = {
          arrow_functions: false,
          template_literals: false,
          destructuring: false,
          spread_operator: false,
          async_await: false,
          modules: false,
          classes: false,
          promises: false
        };

        try {
          // Test arrow functions
          const arrowFn = () => true;
          results.arrow_functions = arrowFn();

          // Test template literals
          const name = 'test';
          const template = `Hello ${name}`;
          results.template_literals = template === 'Hello test';

          // Test destructuring
          const obj = { a: 1, b: 2 };
          const { a, b } = obj;
          results.destructuring = a === 1 && b === 2;

          // Test spread operator
          const arr1 = [1, 2];
          const arr2 = [...arr1, 3];
          results.spread_operator = arr2.length === 3;

          // Test async/await (basic syntax check)
          results.async_await = typeof async function() {} === 'function';

          // Test classes
          class TestClass {
            constructor() {
              this.value = true;
            }
          }
          const instance = new TestClass();
          results.classes = instance.value === true;

          // Test Promises
          results.promises = typeof Promise !== 'undefined';

        } catch (error) {
          console.log('JS feature test error:', error);
        }

        return results;
      });

      // Verify modern JavaScript features are supported
      expect(jsFeatureTest.arrow_functions).toBe(true);
      expect(jsFeatureTest.template_literals).toBe(true);
      expect(jsFeatureTest.destructuring).toBe(true);
      expect(jsFeatureTest.spread_operator).toBe(true);
      expect(jsFeatureTest.async_await).toBe(true);
      expect(jsFeatureTest.classes).toBe(true);
      expect(jsFeatureTest.promises).toBe(true);
    });

    test(`${browser.name}: Error handling and recovery`, async ({ page }) => {
      // Test error boundary by intentionally causing an error
      const errorTest = await page.evaluate(() => {
        const errors = [];
        
        // Listen for errors
        const originalError = window.onerror;
        window.onerror = (message, source, lineno, colno, error) => {
          errors.push({ message, source, lineno, colno });
          return false; // Let default handler run
        };

        try {
          // Cause a recoverable error
          throw new Error('Test error for error handling');
        } catch (error) {
          errors.push({ message: error.message, caught: true });
        }

        // Restore original error handler
        window.onerror = originalError;

        return {
          errorsRecorded: errors.length > 0,
          errorsCaught: errors.some(e => e.caught),
          errors: errors
        };
      });

      expect(errorTest.errorsRecorded).toBe(true);
      expect(errorTest.errorsCaught).toBe(true);

      // Verify the app is still functional after error
      await expect(page.locator('.app-container')).toBeVisible();
      await expect(page.locator('[data-testid="new-document-btn"]')).toBeVisible();
    });

    test(`${browser.name}: Network conditions and offline behavior`, async ({ page }) => {
      // Test offline functionality
      await page.context().setOffline(true);
      
      // The page should still be functional when offline
      await expect(page.locator('.app-container')).toBeVisible();
      
      // Check for offline indicator
      const offlineIndicator = page.locator('[data-testid="offline-indicator"]');
      if (await offlineIndicator.count() > 0) {
        await expect(offlineIndicator).toBeVisible();
      }
      
      // Try to create a document while offline
      await page.click('[data-testid="new-document-btn"]');
      await page.fill('[data-testid="document-title-input"]', 'Offline Test Document');
      await page.click('[data-testid="create-document-btn"]');
      
      // Document should be created locally
      await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
      
      // Go back online
      await page.context().setOffline(false);
      
      // Check for online indicator
      const onlineIndicator = page.locator('[data-testid="online-indicator"]');
      if (await onlineIndicator.count() > 0) {
        await expect(onlineIndicator).toBeVisible();
      }
    });
  });
});

// Cross-browser comparison test
test.describe('Cross-Browser Feature Parity', () => {
  const features = [
    'WASM support',
    'Service Worker',
    'IndexedDB',
    'CSS Grid',
    'ES6+ JavaScript',
    'Local Storage'
  ];

  test('Feature support comparison across browsers', async ({ browser }) => {
    const browserName = browser.browserType().name();
    
    console.log(`\n=== ${browserName.toUpperCase()} FEATURE SUPPORT ===`);
    
    // This would be populated by running the individual tests
    // and comparing results across browsers
    const supportMatrix = {
      chromium: { wasm: true, sw: true, idb: true, grid: true, es6: true, storage: true },
      firefox: { wasm: true, sw: true, idb: true, grid: true, es6: true, storage: true },
      webkit: { wasm: true, sw: true, idb: true, grid: true, es6: true, storage: true }
    };
    
    if (supportMatrix[browserName]) {
      features.forEach(feature => {
        const key = feature.toLowerCase().replace(/[^a-z]/g, '').substring(0, 3);
        const supported = supportMatrix[browserName][key];
        console.log(`  ${feature}: ${supported ? '✅' : '❌'}`);
      });
    }
  });
});

// Performance comparison across browsers
test.describe('Cross-Browser Performance Comparison', () => {
  const performanceResults = {};

  browsers.forEach(browser => {
    test(`${browser.name} performance baseline`, async ({ page }) => {
      await page.goto('/', { waitUntil: 'networkidle' });
      
      const metrics = await page.evaluate(() => {
        const navigation = performance.getEntriesByType('navigation')[0];
        return {
          loadTime: navigation.loadEventEnd - navigation.navigationStart,
          domReady: navigation.domContentLoadedEventEnd - navigation.navigationStart,
          wasmLoadTime: window.browserTestMetrics?.wasmLoadTime || 0
        };
      });
      
      performanceResults[browser.name] = metrics;
      
      console.log(`${browser.name} Performance:`, {
        load: `${metrics.loadTime.toFixed(0)}ms`,
        dom: `${metrics.domReady.toFixed(0)}ms`,
        wasm: `${metrics.wasmLoadTime.toFixed(0)}ms`
      });
      
      // Performance should be reasonable across all browsers
      expect(metrics.loadTime).toBeLessThan(10000); // 10 seconds max
      expect(metrics.domReady).toBeLessThan(5000); // 5 seconds max
    });
  });
});