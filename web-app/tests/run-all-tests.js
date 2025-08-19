#!/usr/bin/env node

/**
 * Main test runner for WriteMagic web application
 * Orchestrates all test suites and generates comprehensive reports
 */

const { spawn, execSync } = require('child_process');
const path = require('path');
const chalk = require('chalk');
const fs = require('fs').promises;

// Import the comprehensive report generator
const ComprehensiveReportGenerator = require('./utils/report-generator.js');

class MainTestRunner {
  constructor() {
    this.results = {
      unit: null,
      integration: null,
      e2e: null,
      performance: null,
      offline: null,
      ai: null,
      browserCompatibility: null,
      buildValidation: null
    };
    
    this.startTime = Date.now();
  }

  async runAllTests() {
    console.log(chalk.blue('🚀 Starting WriteMagic comprehensive test suite...'));
    console.log(chalk.gray(`Started at: ${new Date().toLocaleString()}\n`));

    try {
      // Check if dependencies are installed
      await this.checkDependencies();
      
      // Run tests in optimal order
      await this.runUnitTests();
      await this.runIntegrationTests();
      await this.runAITests();
      await this.runPerformanceTests();
      await this.runOfflineTests();
      await this.runE2ETests();
      await this.runBrowserCompatibilityTests();
      await this.runBuildValidation();
      
      // Generate comprehensive report
      await this.generateComprehensiveReport();
      
      this.printFinalSummary();
      
    } catch (error) {
      console.error(chalk.red('❌ Test suite execution failed:'), error);
      process.exit(1);
    }
  }

  async checkDependencies() {
    console.log(chalk.cyan('📦 Checking dependencies...'));
    
    try {
      // Check if node_modules exists
      await fs.access(path.join(__dirname, 'node_modules'));
    } catch {
      console.log(chalk.yellow('📦 Installing test dependencies...'));
      execSync('npm install', { cwd: __dirname, stdio: 'inherit' });
    }
    
    // Check for required binaries
    const requiredBinaries = ['npx', 'node'];
    for (const binary of requiredBinaries) {
      try {
        execSync(`which ${binary}`, { stdio: 'pipe' });
      } catch {
        throw new Error(`Required binary not found: ${binary}`);
      }
    }
    
    console.log(chalk.green('✅ Dependencies verified\n'));
  }

  async runUnitTests() {
    console.log(chalk.cyan('🧪 Running unit tests...'));
    
    try {
      const result = await this.runCommand('npm', ['run', 'test:unit'], {
        cwd: __dirname,
        timeout: 120000 // 2 minutes
      });
      
      this.results.unit = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ Unit tests completed\n'));
      
    } catch (error) {
      this.results.unit = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Unit tests failed\n'));
    }
  }

  async runIntegrationTests() {
    console.log(chalk.cyan('🔗 Running integration tests...'));
    
    try {
      const result = await this.runCommand('npm', ['run', 'test:integration'], {
        cwd: __dirname,
        timeout: 180000 // 3 minutes
      });
      
      this.results.integration = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ Integration tests completed\n'));
      
    } catch (error) {
      this.results.integration = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Integration tests failed\n'));
    }
  }

  async runAITests() {
    console.log(chalk.cyan('🤖 Running AI integration tests...'));
    
    try {
      const result = await this.runCommand('npm', ['run', 'test:ai'], {
        cwd: __dirname,
        timeout: 300000 // 5 minutes (AI tests may take longer)
      });
      
      this.results.ai = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ AI integration tests completed\n'));
      
    } catch (error) {
      this.results.ai = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ AI integration tests failed\n'));
    }
  }

  async runPerformanceTests() {
    console.log(chalk.cyan('⚡ Running performance tests...'));
    
    try {
      const result = await this.runCommand('npm', ['run', 'test:performance'], {
        cwd: __dirname,
        timeout: 600000 // 10 minutes
      });
      
      this.results.performance = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ Performance tests completed\n'));
      
    } catch (error) {
      this.results.performance = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Performance tests failed\n'));
    }
  }

  async runOfflineTests() {
    console.log(chalk.cyan('🔌 Running offline functionality tests...'));
    
    try {
      const result = await this.runCommand('npm', ['run', 'test:offline'], {
        cwd: __dirname,
        timeout: 300000 // 5 minutes
      });
      
      this.results.offline = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ Offline tests completed\n'));
      
    } catch (error) {
      this.results.offline = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Offline tests failed\n'));
    }
  }

  async runE2ETests() {
    console.log(chalk.cyan('🎭 Running end-to-end tests...'));
    
    try {
      // Start the test server in background
      console.log(chalk.gray('  Starting test server...'));
      const serverProcess = spawn('npm', ['run', 'serve:test'], {
        cwd: __dirname,
        stdio: 'pipe'
      });
      
      // Wait for server to start
      await this.waitForServer('http://localhost:8080', 30000);
      
      try {
        const result = await this.runCommand('npx', ['playwright', 'test'], {
          cwd: __dirname,
          timeout: 600000 // 10 minutes
        });
        
        this.results.e2e = { status: 'PASS', duration: result.duration };
        console.log(chalk.green('✅ E2E tests completed\n'));
        
      } finally {
        // Clean up server
        serverProcess.kill('SIGTERM');
      }
      
    } catch (error) {
      this.results.e2e = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ E2E tests failed\n'));
    }
  }

  async runBrowserCompatibilityTests() {
    console.log(chalk.cyan('🌐 Running browser compatibility tests...'));
    
    try {
      // Start the test server in background
      const serverProcess = spawn('npm', ['run', 'serve:test'], {
        cwd: __dirname,
        stdio: 'pipe'
      });
      
      // Wait for server to start
      await this.waitForServer('http://localhost:8080', 30000);
      
      try {
        const result = await this.runCommand('npx', ['playwright', 'test', '--config=playwright.browsers.config.js'], {
          cwd: __dirname,
          timeout: 900000 // 15 minutes (browser tests take longer)
        });
        
        this.results.browserCompatibility = { status: 'PASS', duration: result.duration };
        console.log(chalk.green('✅ Browser compatibility tests completed\n'));
        
      } finally {
        serverProcess.kill('SIGTERM');
      }
      
    } catch (error) {
      this.results.browserCompatibility = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Browser compatibility tests failed\n'));
    }
  }

  async runBuildValidation() {
    console.log(chalk.cyan('🔧 Running build validation...'));
    
    try {
      const result = await this.runCommand('node', ['build/build-validation.js'], {
        cwd: __dirname,
        timeout: 300000 // 5 minutes
      });
      
      this.results.buildValidation = { status: 'PASS', duration: result.duration };
      console.log(chalk.green('✅ Build validation completed\n'));
      
    } catch (error) {
      this.results.buildValidation = { status: 'FAIL', error: error.message };
      console.log(chalk.red('❌ Build validation failed\n'));
    }
  }

  async generateComprehensiveReport() {
    console.log(chalk.cyan('📊 Generating comprehensive report...'));
    
    try {
      const generator = new ComprehensiveReportGenerator();
      await generator.generateComprehensiveReport();
      
      console.log(chalk.green('✅ Comprehensive report generated\n'));
      
    } catch (error) {
      console.log(chalk.red('❌ Report generation failed:'), error.message);
    }
  }

  async runCommand(command, args, options = {}) {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();
      
      const child = spawn(command, args, {
        stdio: process.env.CI ? 'inherit' : 'pipe',
        cwd: options.cwd || process.cwd(),
        env: { ...process.env, ...options.env }
      });
      
      let stdout = '';
      let stderr = '';
      
      if (child.stdout) {
        child.stdout.on('data', (data) => {
          stdout += data.toString();
          if (!process.env.CI) {
            process.stdout.write(chalk.gray(data.toString()));
          }
        });
      }
      
      if (child.stderr) {
        child.stderr.on('data', (data) => {
          stderr += data.toString();
          if (!process.env.CI) {
            process.stderr.write(chalk.red(data.toString()));
          }
        });
      }
      
      const timeout = options.timeout || 600000; // 10 minutes default
      const timer = setTimeout(() => {
        child.kill('SIGTERM');
        reject(new Error(`Command timed out after ${timeout}ms: ${command} ${args.join(' ')}`));
      }, timeout);
      
      child.on('close', (code) => {
        clearTimeout(timer);
        const duration = Date.now() - startTime;
        
        if (code === 0) {
          resolve({ code, stdout, stderr, duration });
        } else {
          reject(new Error(`Command failed with code ${code}: ${command} ${args.join(' ')}\n${stderr}`));
        }
      });
      
      child.on('error', (error) => {
        clearTimeout(timer);
        reject(error);
      });
    });
  }

  async waitForServer(url, timeout = 30000) {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      try {
        const response = await fetch(url);
        if (response.ok) {
          console.log(chalk.green('  ✅ Test server ready'));
          return;
        }
      } catch (error) {
        // Server not ready yet
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new Error('Test server failed to start within timeout period');
  }

  printFinalSummary() {
    const totalDuration = Date.now() - this.startTime;
    const testSuites = Object.keys(this.results);
    const passedSuites = testSuites.filter(suite => this.results[suite]?.status === 'PASS');
    const failedSuites = testSuites.filter(suite => this.results[suite]?.status === 'FAIL');
    
    console.log(chalk.blue('=' .repeat(60)));
    console.log(chalk.blue('🏁 WriteMagic Test Suite Complete'));
    console.log(chalk.blue('=' .repeat(60)));
    console.log(chalk.blue(`Total Duration: ${(totalDuration / 1000 / 60).toFixed(1)} minutes`));
    console.log(chalk.blue(`Test Suites: ${passedSuites.length}/${testSuites.length} passed`));
    
    console.log(chalk.green('\n✅ Passed Suites:'));
    passedSuites.forEach(suite => {
      const duration = this.results[suite].duration ? ` (${(this.results[suite].duration / 1000).toFixed(1)}s)` : '';
      console.log(chalk.green(`  - ${suite}${duration}`));
    });
    
    if (failedSuites.length > 0) {
      console.log(chalk.red('\n❌ Failed Suites:'));
      failedSuites.forEach(suite => {
        console.log(chalk.red(`  - ${suite}: ${this.results[suite].error}`));
      });
    }
    
    const allPassed = failedSuites.length === 0;
    
    console.log(chalk.blue('\n🚀 Final Status:'));
    if (allPassed) {
      console.log(chalk.green('✅ ALL TESTS PASSED - READY FOR DEPLOYMENT!'));
    } else {
      console.log(chalk.red('❌ SOME TESTS FAILED - NOT READY FOR DEPLOYMENT'));
      console.log(chalk.yellow('Please address the failing tests before deploying to production.'));
    }
    
    console.log(chalk.blue('\n📊 View detailed reports in the reports/ directory'));
    console.log(chalk.blue('=' .repeat(60)));
    
    process.exit(allPassed ? 0 : 1);
  }
}

// Run if called directly
if (require.main === module) {
  // Handle command line arguments
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
WriteMagic Test Suite Runner

Usage: node run-all-tests.js [options]

Options:
  --help, -h          Show this help message
  --unit-only         Run only unit tests
  --e2e-only          Run only e2e tests
  --performance-only  Run only performance tests
  --offline-only      Run only offline tests
  --ai-only           Run only AI integration tests
  --browser-only      Run only browser compatibility tests
  --build-only        Run only build validation
  --no-report         Skip comprehensive report generation

Environment Variables:
  CI=true             Run in CI mode (less verbose output)
  TEST_TIMEOUT=600000 Override default test timeout in milliseconds
    `);
    process.exit(0);
  }
  
  const runner = new MainTestRunner();
  
  // Handle specific test suite runs
  if (args.includes('--unit-only')) {
    runner.runUnitTests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--e2e-only')) {
    runner.runE2ETests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--performance-only')) {
    runner.runPerformanceTests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--offline-only')) {
    runner.runOfflineTests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--ai-only')) {
    runner.runAITests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--browser-only')) {
    runner.runBrowserCompatibilityTests().then(() => process.exit(0)).catch(() => process.exit(1));
  } else if (args.includes('--build-only')) {
    runner.runBuildValidation().then(() => process.exit(0)).catch(() => process.exit(1));
  } else {
    // Run all tests
    runner.runAllTests();
  }
}

module.exports = MainTestRunner;