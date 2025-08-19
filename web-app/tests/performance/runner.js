/**
 * Performance test runner for WriteMagic web application
 * Tests load times, memory usage, WASM initialization, and responsiveness
 */

const puppeteer = require('puppeteer');
const fs = require('fs').promises;
const path = require('path');
const chalk = require('chalk');

class PerformanceTestRunner {
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
    console.log(chalk.blue('🚀 Initializing Performance Test Runner...'));
    
    this.browser = await puppeteer.launch({
      headless: process.env.CI ? true : false,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--enable-precise-memory-info'
      ]
    });
    
    this.page = await this.browser.newPage();
    
    // Enable performance monitoring
    await this.page.evaluateOnNewDocument(() => {
      window.performanceMetrics = {
        marks: [],
        measures: [],
        memoryUsage: [],
        wasmLoadTime: null,
        firstPaint: null,
        firstContentfulPaint: null
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
      
      console.log(chalk.yellow('📊 Running performance tests...'));
      
      await this.testPageLoadPerformance();
      await this.testWasmInitializationTime();
      await this.testDocumentCreationPerformance();
      await this.testLargeDocumentHandling();
      await this.testMultiPanePerformance();
      await this.testMemoryUsage();
      await this.testIndexedDBPerformance();
      await this.testServiceWorkerPerformance();
      await this.testUIResponsiveness();
      await this.testScrollPerformance();
      
      this.calculateSummary();
      await this.generateReport();
      
      console.log(chalk.green('✅ Performance tests completed successfully!'));
      
    } catch (error) {
      console.error(chalk.red('❌ Performance tests failed:'), error);
      throw error;
    } finally {
      await this.cleanup();
    }
  }

  async testPageLoadPerformance() {
    console.log(chalk.cyan('  Testing page load performance...'));
    
    const startTime = performance.now();
    
    // Navigate to the application
    const response = await this.page.goto('http://localhost:8080', {
      waitUntil: 'networkidle2'
    });
    
    const loadTime = performance.now() - startTime;
    
    // Get detailed performance metrics
    const performanceMetrics = await this.page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0];
      const paint = performance.getEntriesByType('paint');
      
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.navigationStart,
        loadComplete: navigation.loadEventEnd - navigation.navigationStart,
        dnsLookup: navigation.domainLookupEnd - navigation.domainLookupStart,
        tcpConnect: navigation.connectEnd - navigation.connectStart,
        serverResponse: navigation.responseEnd - navigation.requestStart,
        domParsing: navigation.domComplete - navigation.responseEnd,
        firstPaint: paint.find(p => p.name === 'first-paint')?.startTime || 0,
        firstContentfulPaint: paint.find(p => p.name === 'first-contentful-paint')?.startTime || 0
      };
    });

    const testResult = {
      name: 'Page Load Performance',
      timestamp: new Date().toISOString(),
      duration: loadTime,
      status: loadTime < 3000 ? 'PASS' : 'FAIL', // 3 second threshold
      metrics: {
        totalLoadTime: loadTime,
        httpStatus: response.status(),
        ...performanceMetrics
      },
      thresholds: {
        totalLoadTime: { value: loadTime, threshold: 3000, passed: loadTime < 3000 },
        domContentLoaded: { value: performanceMetrics.domContentLoaded, threshold: 2000, passed: performanceMetrics.domContentLoaded < 2000 },
        firstContentfulPaint: { value: performanceMetrics.firstContentfulPaint, threshold: 1500, passed: performanceMetrics.firstContentfulPaint < 1500 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Load time: ${loadTime.toFixed(2)}ms`));
    console.log(chalk.gray(`    DOM ready: ${performanceMetrics.domContentLoaded.toFixed(2)}ms`));
    console.log(chalk.gray(`    FCP: ${performanceMetrics.firstContentfulPaint.toFixed(2)}ms`));
  }

  async testWasmInitializationTime() {
    console.log(chalk.cyan('  Testing WASM initialization time...'));
    
    await this.page.reload({ waitUntil: 'networkidle2' });
    
    const wasmMetrics = await this.page.evaluate(async () => {
      const startTime = performance.now();
      
      // Wait for WASM module to be available
      while (!window.writemagic_wasm) {
        await new Promise(resolve => setTimeout(resolve, 10));
      }
      
      const wasmLoadTime = performance.now() - startTime;
      
      // Test WASM function performance
      const functionTestStart = performance.now();
      
      try {
        // Test creating WASM objects
        const doc = new window.writemagic_wasm.Document();
        const project = new window.writemagic_wasm.Project();
        const session = new window.writemagic_wasm.WritingSession();
        
        // Clean up
        if (doc.free) doc.free();
        if (project.free) project.free();
        if (session.free) session.free();
        
        const functionTestTime = performance.now() - functionTestStart;
        
        return {
          wasmLoadTime,
          functionTestTime,
          success: true
        };
      } catch (error) {
        return {
          wasmLoadTime,
          functionTestTime: performance.now() - functionTestStart,
          success: false,
          error: error.message
        };
      }
    });

    const testResult = {
      name: 'WASM Initialization Performance',
      timestamp: new Date().toISOString(),
      duration: wasmMetrics.wasmLoadTime,
      status: wasmMetrics.success && wasmMetrics.wasmLoadTime < 1000 ? 'PASS' : 'FAIL',
      metrics: wasmMetrics,
      thresholds: {
        wasmLoadTime: { value: wasmMetrics.wasmLoadTime, threshold: 1000, passed: wasmMetrics.wasmLoadTime < 1000 },
        functionTestTime: { value: wasmMetrics.functionTestTime, threshold: 100, passed: wasmMetrics.functionTestTime < 100 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    WASM load: ${wasmMetrics.wasmLoadTime.toFixed(2)}ms`));
    console.log(chalk.gray(`    Function test: ${wasmMetrics.functionTestTime.toFixed(2)}ms`));
  }

  async testDocumentCreationPerformance() {
    console.log(chalk.cyan('  Testing document creation performance...'));
    
    const creationMetrics = await this.page.evaluate(async () => {
      const iterations = 100;
      const times = [];
      
      for (let i = 0; i < iterations; i++) {
        const start = performance.now();
        
        // Simulate document creation through the app
        const event = new CustomEvent('createDocument', {
          detail: {
            title: `Performance Test Document ${i}`,
            content: 'Sample content for performance testing'
          }
        });
        
        window.dispatchEvent(event);
        
        const end = performance.now();
        times.push(end - start);
      }
      
      return {
        iterations,
        times,
        average: times.reduce((a, b) => a + b, 0) / times.length,
        min: Math.min(...times),
        max: Math.max(...times),
        median: times.sort((a, b) => a - b)[Math.floor(times.length / 2)]
      };
    });

    const testResult = {
      name: 'Document Creation Performance',
      timestamp: new Date().toISOString(),
      duration: creationMetrics.average,
      status: creationMetrics.average < 50 ? 'PASS' : 'FAIL', // 50ms threshold
      metrics: creationMetrics,
      thresholds: {
        averageTime: { value: creationMetrics.average, threshold: 50, passed: creationMetrics.average < 50 },
        maxTime: { value: creationMetrics.max, threshold: 200, passed: creationMetrics.max < 200 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Average: ${creationMetrics.average.toFixed(2)}ms`));
    console.log(chalk.gray(`    Min/Max: ${creationMetrics.min.toFixed(2)}/${creationMetrics.max.toFixed(2)}ms`));
  }

  async testLargeDocumentHandling() {
    console.log(chalk.cyan('  Testing large document handling...'));
    
    const largeDocMetrics = await this.page.evaluate(async () => {
      const sizes = [10000, 50000, 100000, 500000]; // Character counts
      const results = {};
      
      for (const size of sizes) {
        const content = 'Lorem ipsum dolor sit amet. '.repeat(Math.ceil(size / 27));
        const actualSize = content.length;
        
        const startTime = performance.now();
        
        // Create large document
        const event = new CustomEvent('createDocument', {
          detail: {
            title: `Large Document ${size} chars`,
            content: content.substring(0, size)
          }
        });
        
        window.dispatchEvent(event);
        
        // Test editing performance
        const editStart = performance.now();
        const editEvent = new CustomEvent('updateDocument', {
          detail: {
            content: content.substring(0, size) + ' Additional content.'
          }
        });
        
        window.dispatchEvent(editEvent);
        const editTime = performance.now() - editStart;
        
        const totalTime = performance.now() - startTime;
        
        results[size] = {
          actualSize,
          creationTime: totalTime - editTime,
          editTime,
          totalTime
        };
      }
      
      return results;
    });

    const testResult = {
      name: 'Large Document Handling',
      timestamp: new Date().toISOString(),
      duration: Math.max(...Object.values(largeDocMetrics).map(m => m.totalTime)),
      status: Object.values(largeDocMetrics).every(m => m.totalTime < 2000) ? 'PASS' : 'FAIL',
      metrics: largeDocMetrics,
      thresholds: {
        small10k: { value: largeDocMetrics[10000]?.totalTime || 0, threshold: 100, passed: (largeDocMetrics[10000]?.totalTime || 0) < 100 },
        medium50k: { value: largeDocMetrics[50000]?.totalTime || 0, threshold: 500, passed: (largeDocMetrics[50000]?.totalTime || 0) < 500 },
        large100k: { value: largeDocMetrics[100000]?.totalTime || 0, threshold: 1000, passed: (largeDocMetrics[100000]?.totalTime || 0) < 1000 },
        xlarge500k: { value: largeDocMetrics[500000]?.totalTime || 0, threshold: 2000, passed: (largeDocMetrics[500000]?.totalTime || 0) < 2000 }
      }
    };

    this.results.tests.push(testResult);
    
    Object.entries(largeDocMetrics).forEach(([size, metrics]) => {
      console.log(chalk.gray(`    ${size} chars: ${metrics.totalTime.toFixed(2)}ms (edit: ${metrics.editTime.toFixed(2)}ms)`));
    });
  }

  async testMultiPanePerformance() {
    console.log(chalk.cyan('  Testing multi-pane performance...'));
    
    const multiPaneMetrics = await this.page.evaluate(async () => {
      const layouts = ['single', 'split', 'triple', 'quad'];
      const results = {};
      
      for (const layout of layouts) {
        const startTime = performance.now();
        
        // Switch to layout
        const layoutEvent = new CustomEvent('changeLayout', {
          detail: { layout }
        });
        window.dispatchEvent(layoutEvent);
        
        // Load documents in panes
        for (let i = 0; i < (layout === 'single' ? 1 : layout === 'split' ? 2 : layout === 'triple' ? 3 : 4); i++) {
          const loadEvent = new CustomEvent('loadDocumentInPane', {
            detail: {
              paneIndex: i,
              document: {
                id: `perf-test-doc-${i}`,
                title: `Performance Test Doc ${i}`,
                content: 'Content '.repeat(1000) // Moderate content size
              }
            }
          });
          window.dispatchEvent(loadEvent);
        }
        
        const endTime = performance.now();
        results[layout] = endTime - startTime;
      }
      
      return results;
    });

    const testResult = {
      name: 'Multi-Pane Performance',
      timestamp: new Date().toISOString(),
      duration: Math.max(...Object.values(multiPaneMetrics)),
      status: Object.values(multiPaneMetrics).every(time => time < 1000) ? 'PASS' : 'FAIL',
      metrics: multiPaneMetrics,
      thresholds: {
        single: { value: multiPaneMetrics.single, threshold: 200, passed: multiPaneMetrics.single < 200 },
        split: { value: multiPaneMetrics.split, threshold: 400, passed: multiPaneMetrics.split < 400 },
        triple: { value: multiPaneMetrics.triple, threshold: 600, passed: multiPaneMetrics.triple < 600 },
        quad: { value: multiPaneMetrics.quad, threshold: 1000, passed: multiPaneMetrics.quad < 1000 }
      }
    };

    this.results.tests.push(testResult);
    
    Object.entries(multiPaneMetrics).forEach(([layout, time]) => {
      console.log(chalk.gray(`    ${layout}: ${time.toFixed(2)}ms`));
    });
  }

  async testMemoryUsage() {
    console.log(chalk.cyan('  Testing memory usage...'));
    
    const memoryMetrics = await this.page.evaluate(async () => {
      const measurements = [];
      
      // Initial memory
      const initialMemory = performance.memory ? {
        used: performance.memory.usedJSHeapSize,
        total: performance.memory.totalJSHeapSize,
        limit: performance.memory.jsHeapSizeLimit
      } : null;
      
      measurements.push({ stage: 'initial', memory: initialMemory });
      
      // Create many documents
      for (let i = 0; i < 100; i++) {
        const event = new CustomEvent('createDocument', {
          detail: {
            title: `Memory Test Doc ${i}`,
            content: 'Memory test content '.repeat(100)
          }
        });
        window.dispatchEvent(event);
        
        if (i % 25 === 0 && performance.memory) {
          measurements.push({
            stage: `after_${i}_docs`,
            memory: {
              used: performance.memory.usedJSHeapSize,
              total: performance.memory.totalJSHeapSize,
              limit: performance.memory.jsHeapSizeLimit
            }
          });
        }
      }
      
      // Force garbage collection if available
      if (window.gc) {
        window.gc();
        
        if (performance.memory) {
          measurements.push({
            stage: 'after_gc',
            memory: {
              used: performance.memory.usedJSHeapSize,
              total: performance.memory.totalJSHeapSize,
              limit: performance.memory.jsHeapSizeLimit
            }
          });
        }
      }
      
      return {
        measurements,
        memoryGrowth: initialMemory && measurements.length > 1 ? 
          measurements[measurements.length - 1].memory.used - initialMemory.used : null
      };
    });

    const testResult = {
      name: 'Memory Usage',
      timestamp: new Date().toISOString(),
      duration: 0, // Not time-based
      status: memoryMetrics.memoryGrowth ? (memoryMetrics.memoryGrowth < 50 * 1024 * 1024 ? 'PASS' : 'FAIL') : 'SKIP', // 50MB threshold
      metrics: memoryMetrics,
      thresholds: memoryMetrics.memoryGrowth ? {
        memoryGrowth: { 
          value: memoryMetrics.memoryGrowth, 
          threshold: 50 * 1024 * 1024, 
          passed: memoryMetrics.memoryGrowth < 50 * 1024 * 1024 
        }
      } : {}
    };

    this.results.tests.push(testResult);
    
    if (memoryMetrics.memoryGrowth) {
      console.log(chalk.gray(`    Memory growth: ${(memoryMetrics.memoryGrowth / 1024 / 1024).toFixed(2)} MB`));
    } else {
      console.log(chalk.gray('    Memory info not available'));
    }
  }

  async testIndexedDBPerformance() {
    console.log(chalk.cyan('  Testing IndexedDB performance...'));
    
    const dbMetrics = await this.page.evaluate(async () => {
      const results = {};
      
      // Test document writes
      const writeStart = performance.now();
      for (let i = 0; i < 50; i++) {
        const event = new CustomEvent('saveDocumentToDB', {
          detail: {
            document: {
              id: `db-test-doc-${i}`,
              title: `DB Test Document ${i}`,
              content: 'Database test content '.repeat(50),
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString()
            }
          }
        });
        window.dispatchEvent(event);
      }
      const writeTime = performance.now() - writeStart;
      
      // Test document reads
      const readStart = performance.now();
      for (let i = 0; i < 50; i++) {
        const event = new CustomEvent('loadDocumentFromDB', {
          detail: { documentId: `db-test-doc-${i}` }
        });
        window.dispatchEvent(event);
      }
      const readTime = performance.now() - readStart;
      
      // Test bulk operations
      const bulkStart = performance.now();
      const bulkEvent = new CustomEvent('bulkLoadDocuments');
      window.dispatchEvent(bulkEvent);
      const bulkTime = performance.now() - bulkStart;
      
      return {
        writeTime,
        readTime,
        bulkTime,
        averageWrite: writeTime / 50,
        averageRead: readTime / 50
      };
    });

    const testResult = {
      name: 'IndexedDB Performance',
      timestamp: new Date().toISOString(),
      duration: dbMetrics.writeTime + dbMetrics.readTime + dbMetrics.bulkTime,
      status: dbMetrics.averageWrite < 50 && dbMetrics.averageRead < 20 ? 'PASS' : 'FAIL',
      metrics: dbMetrics,
      thresholds: {
        averageWrite: { value: dbMetrics.averageWrite, threshold: 50, passed: dbMetrics.averageWrite < 50 },
        averageRead: { value: dbMetrics.averageRead, threshold: 20, passed: dbMetrics.averageRead < 20 },
        bulkLoad: { value: dbMetrics.bulkTime, threshold: 500, passed: dbMetrics.bulkTime < 500 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Avg write: ${dbMetrics.averageWrite.toFixed(2)}ms`));
    console.log(chalk.gray(`    Avg read: ${dbMetrics.averageRead.toFixed(2)}ms`));
    console.log(chalk.gray(`    Bulk load: ${dbMetrics.bulkTime.toFixed(2)}ms`));
  }

  async testServiceWorkerPerformance() {
    console.log(chalk.cyan('  Testing Service Worker performance...'));
    
    const swMetrics = await this.page.evaluate(async () => {
      let swRegistered = false;
      let swReady = false;
      const startTime = performance.now();
      
      try {
        if ('serviceWorker' in navigator) {
          const registration = await navigator.serviceWorker.register('/sw.js');
          swRegistered = true;
          
          await navigator.serviceWorker.ready;
          swReady = true;
        }
      } catch (error) {
        // Service worker not available or failed
      }
      
      const registrationTime = performance.now() - startTime;
      
      // Test cache performance
      let cacheTestTime = 0;
      if (swRegistered) {
        const cacheStart = performance.now();
        
        // Simulate cache operations
        const event = new CustomEvent('testServiceWorkerCache');
        window.dispatchEvent(event);
        
        cacheTestTime = performance.now() - cacheStart;
      }
      
      return {
        swRegistered,
        swReady,
        registrationTime,
        cacheTestTime
      };
    });

    const testResult = {
      name: 'Service Worker Performance',
      timestamp: new Date().toISOString(),
      duration: swMetrics.registrationTime,
      status: swMetrics.swRegistered && swMetrics.registrationTime < 1000 ? 'PASS' : 'FAIL',
      metrics: swMetrics,
      thresholds: {
        registrationTime: { value: swMetrics.registrationTime, threshold: 1000, passed: swMetrics.registrationTime < 1000 },
        cacheTestTime: { value: swMetrics.cacheTestTime, threshold: 200, passed: swMetrics.cacheTestTime < 200 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Registration: ${swMetrics.registrationTime.toFixed(2)}ms`));
    if (swMetrics.swRegistered) {
      console.log(chalk.gray(`    Cache test: ${swMetrics.cacheTestTime.toFixed(2)}ms`));
    }
  }

  async testUIResponsiveness() {
    console.log(chalk.cyan('  Testing UI responsiveness...'));
    
    const responsivenessMetrics = await this.page.evaluate(async () => {
      const interactions = [];
      
      // Test button clicks
      const clickStart = performance.now();
      const clickEvent = new CustomEvent('click', { bubbles: true });
      document.body.dispatchEvent(clickEvent);
      const clickTime = performance.now() - clickStart;
      
      interactions.push({ type: 'click', time: clickTime });
      
      // Test input typing
      const typeStart = performance.now();
      const inputEvent = new CustomEvent('input', { bubbles: true });
      document.body.dispatchEvent(inputEvent);
      const typeTime = performance.now() - typeStart;
      
      interactions.push({ type: 'input', time: typeTime });
      
      // Test scroll performance
      const scrollStart = performance.now();
      const scrollEvent = new CustomEvent('scroll', { bubbles: true });
      window.dispatchEvent(scrollEvent);
      const scrollTime = performance.now() - scrollStart;
      
      interactions.push({ type: 'scroll', time: scrollTime });
      
      return {
        interactions,
        averageResponseTime: interactions.reduce((sum, int) => sum + int.time, 0) / interactions.length
      };
    });

    const testResult = {
      name: 'UI Responsiveness',
      timestamp: new Date().toISOString(),
      duration: responsivenessMetrics.averageResponseTime,
      status: responsivenessMetrics.averageResponseTime < 16 ? 'PASS' : 'FAIL', // 16ms = 60fps
      metrics: responsivenessMetrics,
      thresholds: {
        averageResponseTime: { value: responsivenessMetrics.averageResponseTime, threshold: 16, passed: responsivenessMetrics.averageResponseTime < 16 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Average response: ${responsivenessMetrics.averageResponseTime.toFixed(2)}ms`));
  }

  async testScrollPerformance() {
    console.log(chalk.cyan('  Testing scroll performance...'));
    
    const scrollMetrics = await this.page.evaluate(async () => {
      // Create a long scrollable content
      const container = document.createElement('div');
      container.style.height = '10000px';
      container.style.background = 'linear-gradient(to bottom, red, blue)';
      document.body.appendChild(container);
      
      const scrollTimes = [];
      const scrollEvents = 50;
      
      for (let i = 0; i < scrollEvents; i++) {
        const start = performance.now();
        
        window.scrollTo(0, i * 200);
        
        // Wait for scroll to complete
        await new Promise(resolve => setTimeout(resolve, 10));
        
        const end = performance.now();
        scrollTimes.push(end - start);
      }
      
      // Clean up
      document.body.removeChild(container);
      
      return {
        scrollEvents,
        times: scrollTimes,
        averageTime: scrollTimes.reduce((a, b) => a + b, 0) / scrollTimes.length,
        maxTime: Math.max(...scrollTimes),
        minTime: Math.min(...scrollTimes)
      };
    });

    const testResult = {
      name: 'Scroll Performance',
      timestamp: new Date().toISOString(),
      duration: scrollMetrics.averageTime,
      status: scrollMetrics.averageTime < 16 ? 'PASS' : 'FAIL', // 16ms for smooth scrolling
      metrics: scrollMetrics,
      thresholds: {
        averageTime: { value: scrollMetrics.averageTime, threshold: 16, passed: scrollMetrics.averageTime < 16 },
        maxTime: { value: scrollMetrics.maxTime, threshold: 50, passed: scrollMetrics.maxTime < 50 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Average: ${scrollMetrics.averageTime.toFixed(2)}ms`));
    console.log(chalk.gray(`    Max: ${scrollMetrics.maxTime.toFixed(2)}ms`));
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
      successRate: (passedTests / (totalTests - skippedTests)) * 100,
      overallStatus: failedTests === 0 ? 'PASS' : 'FAIL'
    };
    
    console.log(chalk.yellow('\n📈 Performance Test Summary:'));
    console.log(chalk.green(`  ✅ Passed: ${passedTests}`));
    console.log(chalk.red(`  ❌ Failed: ${failedTests}`));
    console.log(chalk.yellow(`  ⏭️  Skipped: ${skippedTests}`));
    console.log(chalk.blue(`  📊 Success Rate: ${this.results.summary.successRate.toFixed(1)}%`));
  }

  async generateReport() {
    const reportDir = path.join(__dirname, '../reports');
    await fs.mkdir(reportDir, { recursive: true });
    
    const reportPath = path.join(reportDir, `performance-report-${Date.now()}.json`);
    await fs.writeFile(reportPath, JSON.stringify(this.results, null, 2));
    
    console.log(chalk.blue(`\n📄 Performance report saved: ${reportPath}`));
    
    // Generate HTML report
    const htmlReport = this.generateHTMLReport();
    const htmlPath = path.join(reportDir, `performance-report-${Date.now()}.html`);
    await fs.writeFile(htmlPath, htmlReport);
    
    console.log(chalk.blue(`📄 HTML report saved: ${htmlPath}`));
  }

  generateHTMLReport() {
    return `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WriteMagic Performance Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f8f9fa; padding: 20px; border-radius: 5px; }
        .summary { display: flex; gap: 20px; margin: 20px 0; }
        .metric { background: #e9ecef; padding: 15px; border-radius: 5px; text-align: center; }
        .test-result { margin: 15px 0; padding: 15px; border-radius: 5px; }
        .pass { background: #d4edda; border: 1px solid #c3e6cb; }
        .fail { background: #f8d7da; border: 1px solid #f5c6cb; }
        .skip { background: #fff3cd; border: 1px solid #ffeaa7; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }
        .metric-item { background: #f8f9fa; padding: 10px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>WriteMagic Performance Test Report</h1>
        <p>Generated: ${this.results.timestamp}</p>
    </div>
    
    <div class="summary">
        <div class="metric">
            <h3>${this.results.summary.totalTests}</h3>
            <p>Total Tests</p>
        </div>
        <div class="metric">
            <h3 style="color: green">${this.results.summary.passedTests}</h3>
            <p>Passed</p>
        </div>
        <div class="metric">
            <h3 style="color: red">${this.results.summary.failedTests}</h3>
            <p>Failed</p>
        </div>
        <div class="metric">
            <h3>${this.results.summary.successRate.toFixed(1)}%</h3>
            <p>Success Rate</p>
        </div>
    </div>
    
    <div class="tests">
        ${this.results.tests.map(test => `
            <div class="test-result ${test.status.toLowerCase()}">
                <h3>${test.name} - ${test.status}</h3>
                <p>Duration: ${test.duration.toFixed(2)}ms</p>
                <div class="metrics">
                    ${Object.entries(test.metrics).map(([key, value]) => `
                        <div class="metric-item">
                            <strong>${key}:</strong> ${typeof value === 'object' ? JSON.stringify(value, null, 2) : value}
                        </div>
                    `).join('')}
                </div>
            </div>
        `).join('')}
    </div>
</body>
</html>`;
  }
}

// Run performance tests if called directly
if (require.main === module) {
  const runner = new PerformanceTestRunner();
  
  runner.runAllTests()
    .then(() => {
      process.exit(runner.results.summary.overallStatus === 'PASS' ? 0 : 1);
    })
    .catch((error) => {
      console.error(chalk.red('Performance tests failed:'), error);
      process.exit(1);
    });
}

module.exports = PerformanceTestRunner;