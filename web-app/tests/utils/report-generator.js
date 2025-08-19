/**
 * Comprehensive test report generator for WriteMagic web application
 * Aggregates results from all test suites and generates detailed reports
 */

const fs = require('fs').promises;
const path = require('path');
const chalk = require('chalk');

// Import test runners
const PerformanceTestRunner = require('../performance/runner.js');
const OfflineTestRunner = require('../offline/runner.js');
const BuildValidationRunner = require('../build/build-validation.js');

class ComprehensiveReportGenerator {
  constructor() {
    this.reportData = {
      timestamp: new Date().toISOString(),
      version: '1.0.0',
      environment: {
        node: process.version,
        platform: process.platform,
        arch: process.arch,
        ci: !!process.env.CI
      },
      testSuites: {},
      overallSummary: {},
      recommendations: [],
      artifacts: []
    };
  }

  async generateComprehensiveReport() {
    try {
      console.log(chalk.blue('📊 Generating comprehensive test report...'));
      
      await this.runAllTestSuites();
      this.calculateOverallSummary();
      this.generateRecommendations();
      await this.generateArtifacts();
      
      console.log(chalk.green('✅ Comprehensive report generation completed!'));
      
    } catch (error) {
      console.error(chalk.red('❌ Report generation failed:'), error);
      throw error;
    }
  }

  async runAllTestSuites() {
    console.log(chalk.cyan('  Running all test suites...'));
    
    // Performance Tests
    try {
      console.log(chalk.gray('    Running performance tests...'));
      const performanceRunner = new PerformanceTestRunner();
      await performanceRunner.runAllTests();
      this.reportData.testSuites.performance = performanceRunner.results;
    } catch (error) {
      console.log(chalk.yellow(`    Performance tests failed: ${error.message}`));
      this.reportData.testSuites.performance = { error: error.message, status: 'FAILED' };
    }

    // Offline Tests
    try {
      console.log(chalk.gray('    Running offline functionality tests...'));
      const offlineRunner = new OfflineTestRunner();
      await offlineRunner.runAllTests();
      this.reportData.testSuites.offline = offlineRunner.results;
    } catch (error) {
      console.log(chalk.yellow(`    Offline tests failed: ${error.message}`));
      this.reportData.testSuites.offline = { error: error.message, status: 'FAILED' };
    }

    // Build Validation
    try {
      console.log(chalk.gray('    Running build validation...'));
      const buildRunner = new BuildValidationRunner();
      await buildRunner.runAllValidations();
      this.reportData.testSuites.build = buildRunner.results;
    } catch (error) {
      console.log(chalk.yellow(`    Build validation failed: ${error.message}`));
      this.reportData.testSuites.build = { error: error.message, status: 'FAILED' };
    }

    // Unit Tests (Jest results)
    await this.aggregateJestResults();
    
    // E2E Tests (Playwright results)
    await this.aggregatePlaywrightResults();
    
    // Browser Compatibility
    await this.aggregateBrowserCompatibilityResults();
  }

  async aggregateJestResults() {
    console.log(chalk.gray('    Aggregating Jest test results...'));
    
    try {
      const jestResultsPath = path.join(__dirname, '../coverage/unit/coverage-final.json');
      
      if (await fs.access(jestResultsPath).then(() => true).catch(() => false)) {
        const coverageData = JSON.parse(await fs.readFile(jestResultsPath, 'utf-8'));
        
        let totalStatements = 0;
        let coveredStatements = 0;
        let totalFunctions = 0;
        let coveredFunctions = 0;
        let totalBranches = 0;
        let coveredBranches = 0;
        let totalLines = 0;
        let coveredLines = 0;
        
        Object.values(coverageData).forEach(file => {
          totalStatements += file.s ? Object.keys(file.s).length : 0;
          coveredStatements += file.s ? Object.values(file.s).filter(v => v > 0).length : 0;
          
          totalFunctions += file.f ? Object.keys(file.f).length : 0;
          coveredFunctions += file.f ? Object.values(file.f).filter(v => v > 0).length : 0;
          
          totalBranches += file.b ? Object.keys(file.b).length : 0;
          coveredBranches += file.b ? Object.values(file.b).filter(b => b.some(v => v > 0)).length : 0;
          
          totalLines += file.l ? Object.keys(file.l).length : 0;
          coveredLines += file.l ? Object.values(file.l).filter(v => v > 0).length : 0;
        });
        
        this.reportData.testSuites.unit = {
          status: 'COMPLETED',
          coverage: {
            statements: { total: totalStatements, covered: coveredStatements, percentage: (coveredStatements / totalStatements) * 100 },
            functions: { total: totalFunctions, covered: coveredFunctions, percentage: (coveredFunctions / totalFunctions) * 100 },
            branches: { total: totalBranches, covered: coveredBranches, percentage: (coveredBranches / totalBranches) * 100 },
            lines: { total: totalLines, covered: coveredLines, percentage: (coveredLines / totalLines) * 100 }
          }
        };
      }
    } catch (error) {
      console.log(chalk.yellow(`    Jest results aggregation failed: ${error.message}`));
      this.reportData.testSuites.unit = { error: error.message, status: 'FAILED' };
    }
  }

  async aggregatePlaywrightResults() {
    console.log(chalk.gray('    Aggregating Playwright test results...'));
    
    try {
      const playwrightResultsPath = path.join(__dirname, '../test-results.json');
      
      if (await fs.access(playwrightResultsPath).then(() => true).catch(() => false)) {
        const playwrightData = JSON.parse(await fs.readFile(playwrightResultsPath, 'utf-8'));
        
        const summary = {
          totalTests: 0,
          passedTests: 0,
          failedTests: 0,
          skippedTests: 0,
          duration: 0,
          browsers: []
        };
        
        if (playwrightData.suites) {
          playwrightData.suites.forEach(suite => {
            if (suite.specs) {
              suite.specs.forEach(spec => {
                summary.totalTests++;
                if (spec.tests) {
                  spec.tests.forEach(test => {
                    if (test.results) {
                      test.results.forEach(result => {
                        if (result.status === 'passed') summary.passedTests++;
                        else if (result.status === 'failed') summary.failedTests++;
                        else if (result.status === 'skipped') summary.skippedTests++;
                        summary.duration += result.duration || 0;
                      });
                    }
                  });
                }
              });
            }
          });
        }
        
        this.reportData.testSuites.e2e = {
          status: summary.failedTests === 0 ? 'PASSED' : 'FAILED',
          summary,
          rawData: playwrightData
        };
      }
    } catch (error) {
      console.log(chalk.yellow(`    Playwright results aggregation failed: ${error.message}`));
      this.reportData.testSuites.e2e = { error: error.message, status: 'FAILED' };
    }
  }

  async aggregateBrowserCompatibilityResults() {
    console.log(chalk.gray('    Aggregating browser compatibility results...'));
    
    try {
      const browserResultsPath = path.join(__dirname, '../browser-test-results.json');
      
      if (await fs.access(browserResultsPath).then(() => true).catch(() => false)) {
        const browserData = JSON.parse(await fs.readFile(browserResultsPath, 'utf-8'));
        
        this.reportData.testSuites.browserCompatibility = {
          status: 'COMPLETED',
          results: browserData
        };
      }
    } catch (error) {
      console.log(chalk.yellow(`    Browser compatibility aggregation failed: ${error.message}`));
      this.reportData.testSuites.browserCompatibility = { error: error.message, status: 'FAILED' };
    }
  }

  calculateOverallSummary() {
    console.log(chalk.cyan('  Calculating overall summary...'));
    
    const summary = {
      totalTestSuites: 0,
      passedTestSuites: 0,
      failedTestSuites: 0,
      totalTests: 0,
      passedTests: 0,
      failedTests: 0,
      skippedTests: 0,
      overallDuration: 0,
      codeCoverage: null,
      performanceScore: null,
      securityScore: null,
      buildReady: false,
      deploymentReady: false
    };

    // Aggregate test suite results
    Object.entries(this.reportData.testSuites).forEach(([suiteName, suite]) => {
      summary.totalTestSuites++;
      
      if (suite.status === 'PASSED' || suite.status === 'COMPLETED' || suite.summary?.overallStatus === 'PASS') {
        summary.passedTestSuites++;
      } else if (suite.status === 'FAILED' || suite.summary?.overallStatus === 'FAIL') {
        summary.failedTestSuites++;
      }
      
      // Aggregate individual test counts
      if (suite.tests) {
        suite.tests.forEach(test => {
          summary.totalTests++;
          if (test.status === 'PASS') summary.passedTests++;
          else if (test.status === 'FAIL') summary.failedTests++;
          else if (test.status === 'SKIP') summary.skippedTests++;
          summary.overallDuration += test.duration || 0;
        });
      }
      
      if (suite.summary) {
        summary.totalTests += suite.summary.totalTests || 0;
        summary.passedTests += suite.summary.passedTests || 0;
        summary.failedTests += suite.summary.failedTests || 0;
        summary.skippedTests += suite.summary.skippedTests || 0;
      }
    });

    // Calculate code coverage
    if (this.reportData.testSuites.unit?.coverage) {
      const coverage = this.reportData.testSuites.unit.coverage;
      summary.codeCoverage = {
        overall: (coverage.statements.percentage + coverage.functions.percentage + 
                 coverage.branches.percentage + coverage.lines.percentage) / 4,
        statements: coverage.statements.percentage,
        functions: coverage.functions.percentage,
        branches: coverage.branches.percentage,
        lines: coverage.lines.percentage
      };
    }

    // Calculate performance score
    if (this.reportData.testSuites.performance?.tests) {
      const performanceTests = this.reportData.testSuites.performance.tests;
      const passedPerformanceTests = performanceTests.filter(test => test.status === 'PASS').length;
      summary.performanceScore = (passedPerformanceTests / performanceTests.length) * 100;
    }

    // Determine build readiness
    summary.buildReady = this.reportData.testSuites.build?.summary?.buildReady || false;
    
    // Determine deployment readiness
    summary.deploymentReady = summary.buildReady && 
                              summary.passedTestSuites >= summary.totalTestSuites * 0.8 &&
                              (summary.codeCoverage?.overall || 0) >= 70 &&
                              (summary.performanceScore || 0) >= 80;

    this.reportData.overallSummary = summary;
    
    // Log summary
    console.log(chalk.yellow('\n📊 Overall Test Summary:'));
    console.log(chalk.blue(`  Test Suites: ${summary.passedTestSuites}/${summary.totalTestSuites} passed`));
    console.log(chalk.blue(`  Individual Tests: ${summary.passedTests}/${summary.totalTests} passed`));
    if (summary.codeCoverage) {
      console.log(chalk.blue(`  Code Coverage: ${summary.codeCoverage.overall.toFixed(1)}%`));
    }
    if (summary.performanceScore) {
      console.log(chalk.blue(`  Performance Score: ${summary.performanceScore.toFixed(1)}%`));
    }
    console.log(chalk.blue(`  Build Ready: ${summary.buildReady ? 'YES' : 'NO'}`));
    console.log(chalk.blue(`  Deployment Ready: ${summary.deploymentReady ? 'YES' : 'NO'}`));
  }

  generateRecommendations() {
    console.log(chalk.cyan('  Generating recommendations...'));
    
    const recommendations = [];
    const summary = this.reportData.overallSummary;

    // Coverage recommendations
    if (summary.codeCoverage && summary.codeCoverage.overall < 80) {
      recommendations.push({
        category: 'Code Coverage',
        severity: 'HIGH',
        title: 'Improve code coverage',
        description: `Current coverage is ${summary.codeCoverage.overall.toFixed(1)}%. Target 80%+ for production.`,
        actions: [
          'Add more unit tests for uncovered functions',
          'Implement integration tests for complex workflows',
          'Add edge case testing for error conditions'
        ]
      });
    }

    // Performance recommendations
    if (summary.performanceScore && summary.performanceScore < 90) {
      recommendations.push({
        category: 'Performance',
        severity: 'MEDIUM',
        title: 'Optimize application performance',
        description: `Performance score is ${summary.performanceScore.toFixed(1)}%. Target 90%+ for optimal user experience.`,
        actions: [
          'Optimize WASM bundle size and loading time',
          'Implement code splitting for large JavaScript bundles',
          'Add performance monitoring and alerting',
          'Optimize critical rendering path'
        ]
      });
    }

    // Build recommendations
    if (!summary.buildReady) {
      recommendations.push({
        category: 'Build System',
        severity: 'HIGH',
        title: 'Fix build system issues',
        description: 'Build validation failed. Application is not ready for deployment.',
        actions: [
          'Fix all build validation errors',
          'Ensure all required dependencies are present',
          'Validate deployment scripts and configuration',
          'Test build process in clean environment'
        ]
      });
    }

    // Test stability recommendations
    const failureRate = summary.totalTests > 0 ? (summary.failedTests / summary.totalTests) * 100 : 0;
    if (failureRate > 5) {
      recommendations.push({
        category: 'Test Stability',
        severity: 'HIGH',
        title: 'Improve test stability',
        description: `${failureRate.toFixed(1)}% of tests are failing. Target <5% for stable CI/CD.`,
        actions: [
          'Fix failing tests immediately',
          'Add retry mechanisms for flaky tests',
          'Improve test isolation and cleanup',
          'Review test data and mocking strategies'
        ]
      });
    }

    // Browser compatibility recommendations
    if (this.reportData.testSuites.browserCompatibility?.status === 'FAILED') {
      recommendations.push({
        category: 'Browser Compatibility',
        severity: 'MEDIUM',
        title: 'Address browser compatibility issues',
        description: 'Some features may not work consistently across all browsers.',
        actions: [
          'Test and fix issues in failing browsers',
          'Add polyfills for missing features',
          'Implement progressive enhancement',
          'Update browser support matrix'
        ]
      });
    }

    // Security recommendations
    const securityTests = this.reportData.testSuites.build?.tests?.find(test => test.name === 'Security Headers Validation');
    if (securityTests && securityTests.status === 'FAIL') {
      recommendations.push({
        category: 'Security',
        severity: 'HIGH',
        title: 'Implement security headers',
        description: 'Missing security headers can expose application to attacks.',
        actions: [
          'Configure Content Security Policy (CSP)',
          'Add HTTPS redirect and HSTS headers',
          'Implement X-Frame-Options and X-Content-Type-Options',
          'Regular security audit and testing'
        ]
      });
    }

    // Offline functionality recommendations
    if (this.reportData.testSuites.offline?.summary?.overallStatus === 'FAIL') {
      recommendations.push({
        category: 'Offline Support',
        severity: 'MEDIUM',
        title: 'Improve offline functionality',
        description: 'Offline features are not working properly, affecting user experience.',
        actions: [
          'Fix Service Worker registration and caching',
          'Implement proper offline storage strategies',
          'Add background sync for offline operations',
          'Test offline scenarios thoroughly'
        ]
      });
    }

    this.reportData.recommendations = recommendations;
    
    // Log recommendations
    if (recommendations.length > 0) {
      console.log(chalk.yellow('\n💡 Recommendations:'));
      recommendations.forEach((rec, index) => {
        const severityColor = rec.severity === 'HIGH' ? chalk.red : rec.severity === 'MEDIUM' ? chalk.yellow : chalk.blue;
        console.log(severityColor(`  ${index + 1}. [${rec.severity}] ${rec.title}`));
        console.log(chalk.gray(`     ${rec.description}`));
      });
    } else {
      console.log(chalk.green('\n🎉 No recommendations - everything looks great!'));
    }
  }

  async generateArtifacts() {
    console.log(chalk.cyan('  Generating report artifacts...'));
    
    const reportDir = path.join(__dirname, '../reports');
    await fs.mkdir(reportDir, { recursive: true });
    
    const timestamp = Date.now();
    
    // JSON Report
    const jsonReportPath = path.join(reportDir, `comprehensive-report-${timestamp}.json`);
    await fs.writeFile(jsonReportPath, JSON.stringify(this.reportData, null, 2));
    this.reportData.artifacts.push({ type: 'json', path: jsonReportPath });
    
    // HTML Report
    const htmlReportPath = path.join(reportDir, `comprehensive-report-${timestamp}.html`);
    const htmlContent = this.generateHTMLReport();
    await fs.writeFile(htmlReportPath, htmlContent);
    this.reportData.artifacts.push({ type: 'html', path: htmlReportPath });
    
    // Markdown Summary
    const markdownReportPath = path.join(reportDir, `test-summary-${timestamp}.md`);
    const markdownContent = this.generateMarkdownSummary();
    await fs.writeFile(markdownReportPath, markdownContent);
    this.reportData.artifacts.push({ type: 'markdown', path: markdownReportPath });
    
    // CSV Data Export
    const csvReportPath = path.join(reportDir, `test-data-${timestamp}.csv`);
    const csvContent = this.generateCSVExport();
    await fs.writeFile(csvReportPath, csvContent);
    this.reportData.artifacts.push({ type: 'csv', path: csvReportPath });
    
    console.log(chalk.blue(`\n📄 Generated reports:`));
    this.reportData.artifacts.forEach(artifact => {
      console.log(chalk.blue(`  - ${artifact.type.toUpperCase()}: ${artifact.path}`));
    });
  }

  generateHTMLReport() {
    const summary = this.reportData.overallSummary;
    
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WriteMagic Test Report - ${new Date(this.reportData.timestamp).toLocaleDateString()}</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0; padding: 20px; background: #f8f9fa; color: #333; 
        }
        .header { 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white; padding: 30px; border-radius: 10px; margin-bottom: 30px;
            text-align: center;
        }
        .summary-grid { 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px; margin-bottom: 30px;
        }
        .metric-card { 
            background: white; padding: 20px; border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); text-align: center;
        }
        .metric-value { 
            font-size: 2.5em; font-weight: bold; margin: 10px 0;
        }
        .pass { color: #28a745; }
        .fail { color: #dc3545; }
        .warning { color: #ffc107; }
        .test-suite { 
            background: white; margin: 20px 0; padding: 20px;
            border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .recommendations { 
            background: white; padding: 20px; border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); margin: 20px 0;
        }
        .recommendation { 
            padding: 15px; margin: 10px 0; border-left: 4px solid;
            background: #f8f9fa; border-radius: 0 5px 5px 0;
        }
        .high { border-color: #dc3545; }
        .medium { border-color: #ffc107; }
        .low { border-color: #17a2b8; }
        .progress-bar { 
            width: 100%; height: 20px; background: #e9ecef;
            border-radius: 10px; overflow: hidden; margin: 10px 0;
        }
        .progress-fill { 
            height: 100%; transition: width 0.3s ease;
        }
        table { 
            width: 100%; border-collapse: collapse; margin: 20px 0;
            background: white; border-radius: 8px; overflow: hidden;
        }
        th, td { 
            padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6;
        }
        th { background: #f8f9fa; font-weight: 600; }
        .badge { 
            padding: 4px 8px; border-radius: 12px; font-size: 0.8em;
            font-weight: 600; text-transform: uppercase;
        }
        .badge.pass { background: #d4edda; color: #155724; }
        .badge.fail { background: #f8d7da; color: #721c24; }
        .badge.skip { background: #fff3cd; color: #856404; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🧪 WriteMagic Test Report</h1>
        <p>Generated on ${new Date(this.reportData.timestamp).toLocaleString()}</p>
        <p>Environment: ${this.reportData.environment.platform} | Node.js ${this.reportData.environment.node}</p>
    </div>

    <div class="summary-grid">
        <div class="metric-card">
            <div class="metric-value ${summary.deploymentReady ? 'pass' : 'fail'}">
                ${summary.deploymentReady ? '🚀' : '⚠️'}
            </div>
            <h3>Deployment Ready</h3>
            <p>${summary.deploymentReady ? 'YES' : 'NO'}</p>
        </div>
        
        <div class="metric-card">
            <div class="metric-value ${summary.passedTestSuites === summary.totalTestSuites ? 'pass' : 'fail'}">
                ${summary.passedTestSuites}/${summary.totalTestSuites}
            </div>
            <h3>Test Suites Passed</h3>
            <div class="progress-bar">
                <div class="progress-fill pass" style="width: ${(summary.passedTestSuites / summary.totalTestSuites) * 100}%; background: #28a745;"></div>
            </div>
        </div>
        
        <div class="metric-card">
            <div class="metric-value ${summary.passedTests / summary.totalTests > 0.95 ? 'pass' : summary.passedTests / summary.totalTests > 0.8 ? 'warning' : 'fail'}">
                ${summary.passedTests}/${summary.totalTests}
            </div>
            <h3>Individual Tests</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${(summary.passedTests / summary.totalTests) * 100}%; background: ${summary.passedTests / summary.totalTests > 0.95 ? '#28a745' : summary.passedTests / summary.totalTests > 0.8 ? '#ffc107' : '#dc3545'};"></div>
            </div>
        </div>
        
        ${summary.codeCoverage ? `
        <div class="metric-card">
            <div class="metric-value ${summary.codeCoverage.overall > 80 ? 'pass' : summary.codeCoverage.overall > 60 ? 'warning' : 'fail'}">
                ${summary.codeCoverage.overall.toFixed(1)}%
            </div>
            <h3>Code Coverage</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${summary.codeCoverage.overall}%; background: ${summary.codeCoverage.overall > 80 ? '#28a745' : summary.codeCoverage.overall > 60 ? '#ffc107' : '#dc3545'};"></div>
            </div>
        </div>
        ` : ''}
        
        ${summary.performanceScore ? `
        <div class="metric-card">
            <div class="metric-value ${summary.performanceScore > 90 ? 'pass' : summary.performanceScore > 70 ? 'warning' : 'fail'}">
                ${summary.performanceScore.toFixed(1)}%
            </div>
            <h3>Performance Score</h3>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${summary.performanceScore}%; background: ${summary.performanceScore > 90 ? '#28a745' : summary.performanceScore > 70 ? '#ffc107' : '#dc3545'};"></div>
            </div>
        </div>
        ` : ''}
    </div>

    <div class="test-suite">
        <h2>📋 Test Suite Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Suite</th>
                    <th>Status</th>
                    <th>Tests</th>
                    <th>Pass Rate</th>
                    <th>Duration</th>
                </tr>
            </thead>
            <tbody>
                ${Object.entries(this.reportData.testSuites).map(([name, suite]) => `
                <tr>
                    <td><strong>${name.charAt(0).toUpperCase() + name.slice(1)}</strong></td>
                    <td>
                        <span class="badge ${suite.status?.toLowerCase() === 'pass' || suite.status?.toLowerCase() === 'passed' || suite.status?.toLowerCase() === 'completed' ? 'pass' : 'fail'}">
                            ${suite.status || 'Unknown'}
                        </span>
                    </td>
                    <td>${suite.tests?.length || suite.summary?.totalTests || 'N/A'}</td>
                    <td>${suite.summary?.successRate ? suite.summary.successRate.toFixed(1) + '%' : 'N/A'}</td>
                    <td>${suite.summary?.totalDuration ? (suite.summary.totalDuration / 1000).toFixed(1) + 's' : 'N/A'}</td>
                </tr>
                `).join('')}
            </tbody>
        </table>
    </div>

    ${this.reportData.recommendations.length > 0 ? `
    <div class="recommendations">
        <h2>💡 Recommendations</h2>
        ${this.reportData.recommendations.map(rec => `
        <div class="recommendation ${rec.severity.toLowerCase()}">
            <h3>${rec.title}</h3>
            <p><strong>Severity:</strong> ${rec.severity}</p>
            <p>${rec.description}</p>
            <h4>Suggested Actions:</h4>
            <ul>
                ${rec.actions.map(action => `<li>${action}</li>`).join('')}
            </ul>
        </div>
        `).join('')}
    </div>
    ` : ''}

    <div style="text-align: center; margin-top: 50px; padding: 20px; color: #6c757d; border-top: 1px solid #dee2e6;">
        <p>Generated by WriteMagic Test Suite | ${this.reportData.timestamp}</p>
        <p>🚀 Ready for production: <strong>${summary.deploymentReady ? 'YES' : 'NO'}</strong></p>
    </div>
</body>
</html>`;
  }

  generateMarkdownSummary() {
    const summary = this.reportData.overallSummary;
    
    return `# 🧪 WriteMagic Test Report

**Generated:** ${new Date(this.reportData.timestamp).toLocaleString()}  
**Environment:** ${this.reportData.environment.platform} | Node.js ${this.reportData.environment.node}  
**Deployment Ready:** ${summary.deploymentReady ? '✅ YES' : '❌ NO'}

## 📊 Summary

| Metric | Value | Status |
|--------|-------|--------|
| Test Suites Passed | ${summary.passedTestSuites}/${summary.totalTestSuites} | ${summary.passedTestSuites === summary.totalTestSuites ? '✅' : '❌'} |
| Individual Tests | ${summary.passedTests}/${summary.totalTests} | ${summary.passedTests / summary.totalTests > 0.95 ? '✅' : summary.passedTests / summary.totalTests > 0.8 ? '⚠️' : '❌'} |
${summary.codeCoverage ? `| Code Coverage | ${summary.codeCoverage.overall.toFixed(1)}% | ${summary.codeCoverage.overall > 80 ? '✅' : summary.codeCoverage.overall > 60 ? '⚠️' : '❌'} |` : ''}
${summary.performanceScore ? `| Performance Score | ${summary.performanceScore.toFixed(1)}% | ${summary.performanceScore > 90 ? '✅' : summary.performanceScore > 70 ? '⚠️' : '❌'} |` : ''}
| Build Ready | ${summary.buildReady ? 'YES' : 'NO'} | ${summary.buildReady ? '✅' : '❌'} |

## 🧪 Test Suite Results

${Object.entries(this.reportData.testSuites).map(([name, suite]) => `
### ${name.charAt(0).toUpperCase() + name.slice(1)} Tests
- **Status:** ${suite.status || 'Unknown'}
- **Tests:** ${suite.tests?.length || suite.summary?.totalTests || 'N/A'}
- **Pass Rate:** ${suite.summary?.successRate ? suite.summary.successRate.toFixed(1) + '%' : 'N/A'}
- **Duration:** ${suite.summary?.totalDuration ? (suite.summary.totalDuration / 1000).toFixed(1) + 's' : 'N/A'}
`).join('')}

${this.reportData.recommendations.length > 0 ? `
## 💡 Recommendations

${this.reportData.recommendations.map((rec, index) => `
### ${index + 1}. ${rec.title} (${rec.severity})

${rec.description}

**Actions:**
${rec.actions.map(action => `- ${action}`).join('\n')}
`).join('')}
` : '## 🎉 No Recommendations\n\nEverything looks great!'}

## 🚀 Deployment Status

${summary.deploymentReady ? 
  '✅ **READY FOR DEPLOYMENT**\n\nAll checks passed. The application is ready for production deployment.' :
  '❌ **NOT READY FOR DEPLOYMENT**\n\nPlease address the failing tests and recommendations before deploying to production.'
}

---
*Report generated by WriteMagic Test Suite*`;
  }

  generateCSVExport() {
    const csvRows = [
      'Test Suite,Test Name,Status,Duration,Metrics'
    ];
    
    Object.entries(this.reportData.testSuites).forEach(([suiteName, suite]) => {
      if (suite.tests) {
        suite.tests.forEach(test => {
          const metrics = test.metrics ? JSON.stringify(test.metrics).replace(/"/g, '""') : '';
          csvRows.push(`"${suiteName}","${test.name}","${test.status}",${test.duration || 0},"${metrics}"`);
        });
      } else {
        csvRows.push(`"${suiteName}","Overall","${suite.status || 'Unknown'}",0,""`);
      }
    });
    
    return csvRows.join('\n');
  }
}

// Run report generation if called directly
if (require.main === module) {
  const generator = new ComprehensiveReportGenerator();
  
  generator.generateComprehensiveReport()
    .then(() => {
      const deploymentReady = generator.reportData.overallSummary.deploymentReady;
      console.log(chalk.blue(`\n🚀 Deployment Ready: ${deploymentReady ? 'YES' : 'NO'}`));
      process.exit(deploymentReady ? 0 : 1);
    })
    .catch((error) => {
      console.error(chalk.red('Report generation failed:'), error);
      process.exit(1);
    });
}

module.exports = ComprehensiveReportGenerator;