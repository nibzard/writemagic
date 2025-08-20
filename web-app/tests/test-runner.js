#!/usr/bin/env node

/**
 * WriteMagic Test Runner
 * Runs tests with better error handling and reporting
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

// Colors for console output
const colors = {
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  reset: '\x1b[0m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function checkPrerequisites() {
  log('🔍 Checking test prerequisites...', 'blue');
  
  // Check if package.json exists
  if (!fs.existsSync('package.json')) {
    log('❌ package.json not found', 'red');
    return false;
  }
  
  // Check if node_modules exists
  if (!fs.existsSync('node_modules')) {
    log('📦 node_modules not found, installing dependencies...', 'yellow');
    try {
      execSync('npm install', { stdio: 'inherit' });
      log('✅ Dependencies installed', 'green');
    } catch (error) {
      log('❌ Failed to install dependencies', 'red');
      return false;
    }
  }
  
  return true;
}

function runTests(testType = 'unit') {
  if (!checkPrerequisites()) {
    process.exit(1);
  }
  
  log(`🧪 Running ${testType} tests...`, 'blue');
  
  try {
    const command = `npx jest --config=jest.${testType}.config.js --verbose`;
    log(`Running: ${command}`, 'blue');
    
    execSync(command, { 
      stdio: 'inherit',
      cwd: process.cwd()
    });
    
    log(`✅ ${testType} tests completed successfully`, 'green');
    
  } catch (error) {
    log(`❌ ${testType} tests failed`, 'red');
    
    // Show helpful error information
    if (error.message.includes('moduleNameMapper')) {
      log('💡 Hint: This might be a Jest configuration issue', 'yellow');
    }
    
    if (error.message.includes('SyntaxError')) {
      log('💡 Hint: This might be a Babel/ES modules issue', 'yellow');
    }
    
    if (error.message.includes('@testing-library/jest-dom')) {
      log('💡 Hint: This might be a Jest setup issue', 'yellow');
    }
    
    process.exit(1);
  }
}

function runPlaywrightTests() {
  if (!checkPrerequisites()) {
    process.exit(1);
  }
  
  log('🎭 Running Playwright tests...', 'blue');
  
  try {
    // Check if Playwright browsers are installed
    try {
      execSync('npx playwright --version', { stdio: 'pipe' });
    } catch (e) {
      log('📦 Playwright not found, installing...', 'yellow');
      execSync('npm install @playwright/test', { stdio: 'inherit' });
    }
    
    // Try to install browsers (this might fail, but that's ok for now)
    try {
      log('🌐 Installing Playwright browsers (this might take a while)...', 'yellow');
      execSync('npx playwright install --with-deps chromium', { 
        stdio: 'inherit',
        timeout: 300000 // 5 minutes
      });
      log('✅ Browsers installed successfully', 'green');
    } catch (e) {
      log('⚠️  Browser installation failed, tests may not work properly', 'yellow');
      log('💡 Run "npx playwright install" manually if needed', 'yellow');
    }
    
    const command = 'npx playwright test';
    log(`Running: ${command}`, 'blue');
    
    execSync(command, { 
      stdio: 'inherit',
      cwd: process.cwd()
    });
    
    log('✅ Playwright tests completed successfully', 'green');
    
  } catch (error) {
    log('❌ Playwright tests failed', 'red');
    
    if (error.message.includes('Browser')) {
      log('💡 Hint: Try running "npx playwright install" first', 'yellow');
    }
    
    process.exit(1);
  }
}

// Parse command line arguments
const args = process.argv.slice(2);
const testType = args[0] || 'unit';

switch (testType) {
  case 'unit':
  case 'integration':
  case 'ai':
    runTests(testType);
    break;
  case 'e2e':
  case 'playwright':
    runPlaywrightTests();
    break;
  case 'all':
    log('🚀 Running all tests...', 'blue');
    runTests('unit');
    runTests('integration');
    runTests('ai');
    runPlaywrightTests();
    log('🎉 All tests completed!', 'green');
    break;
  default:
    log('Usage: node test-runner.js [unit|integration|ai|e2e|playwright|all]', 'blue');
    log('Available test types:', 'blue');
    log('  unit        - Run unit tests', 'blue');
    log('  integration - Run integration tests', 'blue');
    log('  ai          - Run AI integration tests', 'blue');
    log('  e2e         - Run end-to-end tests', 'blue');
    log('  playwright  - Run Playwright tests', 'blue');
    log('  all         - Run all test suites', 'blue');
    break;
}