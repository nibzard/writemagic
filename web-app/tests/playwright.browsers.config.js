// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './browser-compatibility',
  outputDir: './browser-results',
  
  timeout: 60 * 1000, // Longer timeout for browser compatibility tests
  
  expect: {
    timeout: 10000
  },
  
  fullyParallel: false, // Run browser tests sequentially for stability
  retries: 1,
  workers: 1,
  
  reporter: [
    ['html', { outputFolder: 'browser-compatibility-report' }],
    ['json', { outputFile: 'browser-test-results.json' }]
  ],
  
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on',
    screenshot: 'on',
    video: 'on'
  },

  // Comprehensive browser testing
  projects: [
    // Desktop browsers
    {
      name: 'Chrome Stable',
      use: { ...devices['Desktop Chrome'] }
    },
    {
      name: 'Chrome Beta',
      use: { 
        ...devices['Desktop Chrome'],
        channel: 'chrome-beta'
      }
    },
    {
      name: 'Firefox Stable',
      use: { ...devices['Desktop Firefox'] }
    },
    {
      name: 'Safari',
      use: { ...devices['Desktop Safari'] }
    },
    {
      name: 'Edge',
      use: { 
        ...devices['Desktop Edge'],
        channel: 'msedge'
      }
    },
    
    // Mobile browsers
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] }
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] }
    },
    {
      name: 'Mobile Firefox',
      use: { 
        ...devices['Pixel 5'],
        // Override to use Firefox mobile
        browserName: 'firefox'
      }
    },
    
    // Tablet testing
    {
      name: 'iPad',
      use: { ...devices['iPad Pro'] }
    },
    {
      name: 'Android Tablet',
      use: { ...devices['Galaxy Tab S4'] }
    },
    
    // Different screen sizes
    {
      name: '4K Desktop',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 3840, height: 2160 }
      }
    },
    {
      name: 'Small Desktop',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1024, height: 768 }
      }
    }
  ],

  webServer: {
    command: 'npm run serve:test',
    port: 8080,
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000
  }
});