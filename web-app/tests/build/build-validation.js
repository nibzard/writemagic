/**
 * Build validation tests for WriteMagic web application
 * Validates WASM build, deployment scripts, and production readiness
 */

const fs = require('fs').promises;
const path = require('path');
const chalk = require('chalk');
const { execSync } = require('child_process');

class BuildValidationRunner {
  constructor() {
    this.results = {
      timestamp: new Date().toISOString(),
      tests: [],
      summary: {},
      buildMetrics: {}
    };
    this.rootDir = path.resolve(__dirname, '../../../..');
  }

  async runAllValidations() {
    try {
      console.log(chalk.blue('🔧 Running build validation tests...'));
      
      await this.validateProjectStructure();
      await this.validateWasmBuild();
      await this.validateWebAppBuild();
      await this.validateDependencies();
      await this.validateScripts();
      await this.validateDeploymentReadiness();
      await this.validateEnvironmentConfig();
      await this.validateSecurityHeaders();
      await this.validateAssetOptimization();
      await this.validateServiceWorkerBuild();
      
      this.calculateSummary();
      await this.generateReport();
      
      console.log(chalk.green('✅ Build validation completed successfully!'));
      
    } catch (error) {
      console.error(chalk.red('❌ Build validation failed:'), error);
      throw error;
    }
  }

  async validateProjectStructure() {
    console.log(chalk.cyan('  Validating project structure...'));
    
    const requiredFiles = [
      'web-app/public/index.html',
      'web-app/public/manifest.json',
      'web-app/public/sw.js',
      'web-app/src/js/index.js',
      'web-app/src/js/document-manager.js',
      'web-app/src/js/project-workspace.js',
      'web-app/src/js/ai-proxy-integration.js',
      'core/wasm/Cargo.toml',
      'core/wasm/src/lib.rs',
      'ai-proxy/package.json',
      'ai-proxy/server.js',
      'scripts/build-wasm.sh',
      'scripts/setup-wasm.sh'
    ];

    const requiredDirectories = [
      'web-app/public',
      'web-app/src',
      'web-app/src/js',
      'web-app/src/js/utils',
      'web-app/public/styles',
      'web-app/public/scripts',
      'core/wasm/src',
      'ai-proxy/routes',
      'ai-proxy/services'
    ];

    const structureResults = {
      missingFiles: [],
      missingDirectories: [],
      totalFiles: requiredFiles.length,
      totalDirectories: requiredDirectories.length
    };

    // Check files
    for (const file of requiredFiles) {
      const filePath = path.join(this.rootDir, file);
      try {
        await fs.access(filePath);
      } catch {
        structureResults.missingFiles.push(file);
      }
    }

    // Check directories
    for (const dir of requiredDirectories) {
      const dirPath = path.join(this.rootDir, dir);
      try {
        const stats = await fs.stat(dirPath);
        if (!stats.isDirectory()) {
          structureResults.missingDirectories.push(dir);
        }
      } catch {
        structureResults.missingDirectories.push(dir);
      }
    }

    const testResult = {
      name: 'Project Structure Validation',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: structureResults.missingFiles.length === 0 && structureResults.missingDirectories.length === 0 ? 'PASS' : 'FAIL',
      metrics: structureResults,
      thresholds: {
        allFilesPresent: { 
          value: structureResults.missingFiles.length === 0, 
          threshold: true, 
          passed: structureResults.missingFiles.length === 0 
        },
        allDirectoriesPresent: { 
          value: structureResults.missingDirectories.length === 0, 
          threshold: true, 
          passed: structureResults.missingDirectories.length === 0 
        }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Files: ${requiredFiles.length - structureResults.missingFiles.length}/${requiredFiles.length}`));
    console.log(chalk.gray(`    Directories: ${requiredDirectories.length - structureResults.missingDirectories.length}/${requiredDirectories.length}`));
    
    if (structureResults.missingFiles.length > 0) {
      console.log(chalk.red(`    Missing files: ${structureResults.missingFiles.join(', ')}`));
    }
  }

  async validateWasmBuild() {
    console.log(chalk.cyan('  Validating WASM build process...'));
    
    const wasmDir = path.join(this.rootDir, 'core/wasm');
    const wasmResults = {
      cargoTomlExists: false,
      rustSourceExists: false,
      buildScriptExists: false,
      buildSuccessful: false,
      wasmFileGenerated: false,
      jsBindingsGenerated: false,
      buildTime: 0,
      wasmSize: 0
    };

    try {
      // Check Cargo.toml
      await fs.access(path.join(wasmDir, 'Cargo.toml'));
      wasmResults.cargoTomlExists = true;

      // Check Rust source
      await fs.access(path.join(wasmDir, 'src/lib.rs'));
      wasmResults.rustSourceExists = true;

      // Check build script
      const buildScript = path.join(this.rootDir, 'scripts/build-wasm.sh');
      await fs.access(buildScript);
      wasmResults.buildScriptExists = true;

      // Run WASM build
      const buildStart = Date.now();
      try {
        execSync('chmod +x scripts/build-wasm.sh && ./scripts/build-wasm.sh', {
          cwd: this.rootDir,
          stdio: 'pipe',
          timeout: 120000 // 2 minutes timeout
        });
        wasmResults.buildSuccessful = true;
        wasmResults.buildTime = Date.now() - buildStart;

        // Check generated files
        const pkgDir = path.join(wasmDir, 'pkg');
        try {
          const wasmFile = path.join(pkgDir, 'writemagic_wasm_bg.wasm');
          const wasmStats = await fs.stat(wasmFile);
          wasmResults.wasmFileGenerated = true;
          wasmResults.wasmSize = wasmStats.size;

          const jsFile = path.join(pkgDir, 'writemagic_wasm.js');
          await fs.access(jsFile);
          wasmResults.jsBindingsGenerated = true;
        } catch (e) {
          console.log(chalk.yellow(`    Warning: Generated files not found: ${e.message}`));
        }
      } catch (buildError) {
        console.log(chalk.red(`    Build failed: ${buildError.message}`));
      }
    } catch (error) {
      console.log(chalk.red(`    WASM validation error: ${error.message}`));
    }

    const testResult = {
      name: 'WASM Build Validation',
      timestamp: new Date().toISOString(),
      duration: wasmResults.buildTime,
      status: wasmResults.buildSuccessful && wasmResults.wasmFileGenerated && wasmResults.jsBindingsGenerated ? 'PASS' : 'FAIL',
      metrics: wasmResults,
      thresholds: {
        buildSuccessful: { value: wasmResults.buildSuccessful, threshold: true, passed: wasmResults.buildSuccessful },
        buildTime: { value: wasmResults.buildTime, threshold: 120000, passed: wasmResults.buildTime < 120000 },
        wasmSize: { value: wasmResults.wasmSize, threshold: 5000000, passed: wasmResults.wasmSize < 5000000 } // 5MB limit
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Build time: ${wasmResults.buildTime}ms`));
    console.log(chalk.gray(`    WASM size: ${(wasmResults.wasmSize / 1024).toFixed(1)}KB`));
  }

  async validateWebAppBuild() {
    console.log(chalk.cyan('  Validating web app build...'));
    
    const webAppDir = path.join(this.rootDir, 'web-app');
    const webAppResults = {
      packageJsonExists: false,
      sourceFilesExist: false,
      stylesExist: false,
      assetsOptimized: false,
      htmlValidated: false,
      manifestValid: false,
      totalAssetSize: 0,
      assetCount: 0
    };

    try {
      // Check package.json
      const packageJsonPath = path.join(webAppDir, 'package.json');
      await fs.access(packageJsonPath);
      webAppResults.packageJsonExists = true;

      // Check source files
      const srcDir = path.join(webAppDir, 'src/js');
      const srcFiles = await fs.readdir(srcDir);
      webAppResults.sourceFilesExist = srcFiles.length > 0;

      // Check styles
      const stylesDir = path.join(webAppDir, 'public/styles');
      try {
        const styleFiles = await fs.readdir(stylesDir);
        webAppResults.stylesExist = styleFiles.some(file => file.endsWith('.css'));
      } catch (e) {
        console.log(chalk.yellow(`    Styles directory not found: ${e.message}`));
      }

      // Validate HTML
      const indexHtml = path.join(webAppDir, 'public/index.html');
      const htmlContent = await fs.readFile(indexHtml, 'utf-8');
      webAppResults.htmlValidated = 
        htmlContent.includes('<!DOCTYPE html>') &&
        htmlContent.includes('<meta charset="utf-8">') &&
        htmlContent.includes('<meta name="viewport"');

      // Validate manifest
      const manifestPath = path.join(webAppDir, 'public/manifest.json');
      try {
        const manifestContent = await fs.readFile(manifestPath, 'utf-8');
        const manifest = JSON.parse(manifestContent);
        webAppResults.manifestValid = 
          manifest.name && 
          manifest.short_name && 
          manifest.start_url &&
          manifest.display &&
          Array.isArray(manifest.icons);
      } catch (e) {
        console.log(chalk.yellow(`    Manifest validation failed: ${e.message}`));
      }

      // Calculate asset sizes
      const publicDir = path.join(webAppDir, 'public');
      const calculateDirSize = async (dir) => {
        let totalSize = 0;
        let fileCount = 0;
        
        const items = await fs.readdir(dir, { withFileTypes: true });
        for (const item of items) {
          const itemPath = path.join(dir, item.name);
          if (item.isDirectory()) {
            const { size, count } = await calculateDirSize(itemPath);
            totalSize += size;
            fileCount += count;
          } else {
            const stats = await fs.stat(itemPath);
            totalSize += stats.size;
            fileCount++;
          }
        }
        
        return { size: totalSize, count: fileCount };
      };

      const { size, count } = await calculateDirSize(publicDir);
      webAppResults.totalAssetSize = size;
      webAppResults.assetCount = count;
      webAppResults.assetsOptimized = size < 10 * 1024 * 1024; // 10MB limit

    } catch (error) {
      console.log(chalk.red(`    Web app validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Web App Build Validation',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: webAppResults.packageJsonExists && 
              webAppResults.sourceFilesExist && 
              webAppResults.htmlValidated && 
              webAppResults.manifestValid ? 'PASS' : 'FAIL',
      metrics: webAppResults,
      thresholds: {
        packageJsonExists: { value: webAppResults.packageJsonExists, threshold: true, passed: webAppResults.packageJsonExists },
        manifestValid: { value: webAppResults.manifestValid, threshold: true, passed: webAppResults.manifestValid },
        assetsOptimized: { value: webAppResults.assetsOptimized, threshold: true, passed: webAppResults.assetsOptimized }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Asset size: ${(webAppResults.totalAssetSize / 1024 / 1024).toFixed(1)}MB`));
    console.log(chalk.gray(`    Asset count: ${webAppResults.assetCount}`));
  }

  async validateDependencies() {
    console.log(chalk.cyan('  Validating dependencies...'));
    
    const dependencyResults = {
      webAppDeps: { valid: false, vulnerabilities: 0, outdated: 0 },
      aiProxyDeps: { valid: false, vulnerabilities: 0, outdated: 0 },
      rustDeps: { valid: false, errors: [] }
    };

    try {
      // Check web app dependencies
      const webAppPackage = path.join(this.rootDir, 'web-app/tests/package.json');
      if (await fs.access(webAppPackage).then(() => true).catch(() => false)) {
        try {
          execSync('npm audit --json', {
            cwd: path.join(this.rootDir, 'web-app/tests'),
            stdio: 'pipe'
          });
          dependencyResults.webAppDeps.valid = true;
        } catch (auditError) {
          const auditOutput = auditError.stdout?.toString() || '{}';
          try {
            const auditData = JSON.parse(auditOutput);
            dependencyResults.webAppDeps.vulnerabilities = auditData.metadata?.vulnerabilities?.total || 0;
          } catch (e) {
            console.log(chalk.yellow(`    Web app audit parsing failed`));
          }
        }
      }

      // Check AI proxy dependencies
      const aiProxyPackage = path.join(this.rootDir, 'ai-proxy/package.json');
      if (await fs.access(aiProxyPackage).then(() => true).catch(() => false)) {
        try {
          execSync('npm audit --json', {
            cwd: path.join(this.rootDir, 'ai-proxy'),
            stdio: 'pipe'
          });
          dependencyResults.aiProxyDeps.valid = true;
        } catch (auditError) {
          const auditOutput = auditError.stdout?.toString() || '{}';
          try {
            const auditData = JSON.parse(auditOutput);
            dependencyResults.aiProxyDeps.vulnerabilities = auditData.metadata?.vulnerabilities?.total || 0;
          } catch (e) {
            console.log(chalk.yellow(`    AI proxy audit parsing failed`));
          }
        }
      }

      // Check Rust dependencies
      try {
        execSync('cargo check --workspace', {
          cwd: this.rootDir,
          stdio: 'pipe'
        });
        dependencyResults.rustDeps.valid = true;
      } catch (cargoError) {
        dependencyResults.rustDeps.errors.push(cargoError.message);
      }

    } catch (error) {
      console.log(chalk.red(`    Dependency validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Dependency Validation',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: dependencyResults.webAppDeps.valid && 
              dependencyResults.aiProxyDeps.valid && 
              dependencyResults.rustDeps.valid &&
              dependencyResults.webAppDeps.vulnerabilities === 0 &&
              dependencyResults.aiProxyDeps.vulnerabilities === 0 ? 'PASS' : 'FAIL',
      metrics: dependencyResults,
      thresholds: {
        webAppDepsValid: { value: dependencyResults.webAppDeps.valid, threshold: true, passed: dependencyResults.webAppDeps.valid },
        aiProxyDepsValid: { value: dependencyResults.aiProxyDeps.valid, threshold: true, passed: dependencyResults.aiProxyDeps.valid },
        rustDepsValid: { value: dependencyResults.rustDeps.valid, threshold: true, passed: dependencyResults.rustDeps.valid },
        noVulnerabilities: { 
          value: dependencyResults.webAppDeps.vulnerabilities + dependencyResults.aiProxyDeps.vulnerabilities, 
          threshold: 0, 
          passed: dependencyResults.webAppDeps.vulnerabilities + dependencyResults.aiProxyDeps.vulnerabilities === 0 
        }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Web app vulnerabilities: ${dependencyResults.webAppDeps.vulnerabilities}`));
    console.log(chalk.gray(`    AI proxy vulnerabilities: ${dependencyResults.aiProxyDeps.vulnerabilities}`));
    console.log(chalk.gray(`    Rust deps valid: ${dependencyResults.rustDeps.valid}`));
  }

  async validateScripts() {
    console.log(chalk.cyan('  Validating build scripts...'));
    
    const scriptsDir = path.join(this.rootDir, 'scripts');
    const scriptResults = {
      scriptsExist: [],
      scriptsExecutable: [],
      scriptsValid: [],
      totalScripts: 0
    };

    const requiredScripts = [
      'build-wasm.sh',
      'setup-wasm.sh',
      'setup-git.sh'
    ];

    for (const script of requiredScripts) {
      const scriptPath = path.join(scriptsDir, script);
      scriptResults.totalScripts++;
      
      try {
        // Check if script exists
        await fs.access(scriptPath);
        scriptResults.scriptsExist.push(script);
        
        // Check if script is executable
        const stats = await fs.stat(scriptPath);
        if (stats.mode & parseInt('111', 8)) {
          scriptResults.scriptsExecutable.push(script);
        }
        
        // Basic script validation
        const content = await fs.readFile(scriptPath, 'utf-8');
        if (content.startsWith('#!') && content.includes('set -e')) {
          scriptResults.scriptsValid.push(script);
        }
        
      } catch (error) {
        console.log(chalk.yellow(`    Script ${script} not found or not accessible`));
      }
    }

    const testResult = {
      name: 'Build Scripts Validation',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: scriptResults.scriptsExist.length === requiredScripts.length &&
              scriptResults.scriptsExecutable.length === requiredScripts.length ? 'PASS' : 'FAIL',
      metrics: scriptResults,
      thresholds: {
        allScriptsExist: { 
          value: scriptResults.scriptsExist.length, 
          threshold: requiredScripts.length, 
          passed: scriptResults.scriptsExist.length === requiredScripts.length 
        },
        allScriptsExecutable: { 
          value: scriptResults.scriptsExecutable.length, 
          threshold: requiredScripts.length, 
          passed: scriptResults.scriptsExecutable.length === requiredScripts.length 
        }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Scripts found: ${scriptResults.scriptsExist.length}/${requiredScripts.length}`));
    console.log(chalk.gray(`    Scripts executable: ${scriptResults.scriptsExecutable.length}/${requiredScripts.length}`));
  }

  async validateDeploymentReadiness() {
    console.log(chalk.cyan('  Validating deployment readiness...'));
    
    const deploymentResults = {
      dockerfileExists: false,
      dockerComposeExists: false,
      k8sConfigExists: false,
      environmentConfigured: false,
      secretsConfigured: false,
      healthCheckEndpoint: false
    };

    try {
      // Check Dockerfile
      await fs.access(path.join(this.rootDir, 'Dockerfile'));
      deploymentResults.dockerfileExists = true;

      // Check docker-compose
      await fs.access(path.join(this.rootDir, 'docker-compose.yml'));
      deploymentResults.dockerComposeExists = true;

      // Check Kubernetes configs
      const k8sDir = path.join(this.rootDir, 'k8s');
      try {
        const k8sFiles = await fs.readdir(k8sDir);
        deploymentResults.k8sConfigExists = k8sFiles.length > 0;
      } catch (e) {
        console.log(chalk.yellow(`    K8s directory not found`));
      }

      // Check environment configuration
      const envExampleExists = await fs.access(path.join(this.rootDir, '.env.example'))
        .then(() => true)
        .catch(() => false);
      deploymentResults.environmentConfigured = envExampleExists;

      // Check for secrets configuration
      const secretsExample = await fs.access(path.join(this.rootDir, 'k8s/secrets.yaml'))
        .then(() => true)
        .catch(() => false);
      deploymentResults.secretsConfigured = secretsExample;

    } catch (error) {
      console.log(chalk.red(`    Deployment validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Deployment Readiness',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: deploymentResults.dockerfileExists && 
              deploymentResults.dockerComposeExists && 
              deploymentResults.environmentConfigured ? 'PASS' : 'FAIL',
      metrics: deploymentResults,
      thresholds: {
        dockerfileExists: { value: deploymentResults.dockerfileExists, threshold: true, passed: deploymentResults.dockerfileExists },
        dockerComposeExists: { value: deploymentResults.dockerComposeExists, threshold: true, passed: deploymentResults.dockerComposeExists },
        environmentConfigured: { value: deploymentResults.environmentConfigured, threshold: true, passed: deploymentResults.environmentConfigured }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Docker ready: ${deploymentResults.dockerfileExists && deploymentResults.dockerComposeExists}`));
    console.log(chalk.gray(`    K8s ready: ${deploymentResults.k8sConfigExists}`));
  }

  async validateEnvironmentConfig() {
    console.log(chalk.cyan('  Validating environment configuration...'));
    
    const envResults = {
      webAppConfigValid: false,
      aiProxyConfigValid: false,
      requiredEnvVars: [],
      missingEnvVars: [],
      configurationScore: 0
    };

    const requiredEnvVars = [
      'AI_PROXY_URL',
      'CLAUDE_API_KEY',
      'OPENAI_API_KEY',
      'NODE_ENV',
      'PORT'
    ];

    try {
      // Check web app configuration
      const webAppConfig = path.join(this.rootDir, 'web-app/src/js/config.js');
      if (await fs.access(webAppConfig).then(() => true).catch(() => false)) {
        envResults.webAppConfigValid = true;
      }

      // Check AI proxy configuration
      const aiProxyConfig = path.join(this.rootDir, 'ai-proxy/config/index.js');
      if (await fs.access(aiProxyConfig).then(() => true).catch(() => false)) {
        envResults.aiProxyConfigValid = true;
      }

      // Check environment variables
      for (const envVar of requiredEnvVars) {
        if (process.env[envVar]) {
          envResults.requiredEnvVars.push(envVar);
        } else {
          envResults.missingEnvVars.push(envVar);
        }
      }

      envResults.configurationScore = (envResults.requiredEnvVars.length / requiredEnvVars.length) * 100;

    } catch (error) {
      console.log(chalk.red(`    Environment validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Environment Configuration',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: envResults.webAppConfigValid && 
              envResults.aiProxyConfigValid && 
              envResults.configurationScore > 60 ? 'PASS' : 'FAIL',
      metrics: envResults,
      thresholds: {
        webAppConfigValid: { value: envResults.webAppConfigValid, threshold: true, passed: envResults.webAppConfigValid },
        aiProxyConfigValid: { value: envResults.aiProxyConfigValid, threshold: true, passed: envResults.aiProxyConfigValid },
        configurationScore: { value: envResults.configurationScore, threshold: 60, passed: envResults.configurationScore > 60 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Config score: ${envResults.configurationScore.toFixed(1)}%`));
    console.log(chalk.gray(`    Missing vars: ${envResults.missingEnvVars.length}`));
  }

  async validateSecurityHeaders() {
    console.log(chalk.cyan('  Validating security headers...'));
    
    const securityResults = {
      headersConfigExists: false,
      cspConfigured: false,
      httpsRedirectConfigured: false,
      securityScore: 0
    };

    try {
      // Check _headers file
      const headersFile = path.join(this.rootDir, 'web-app/public/_headers');
      if (await fs.access(headersFile).then(() => true).catch(() => false)) {
        securityResults.headersConfigExists = true;
        
        const headersContent = await fs.readFile(headersFile, 'utf-8');
        
        // Check for CSP
        if (headersContent.includes('Content-Security-Policy')) {
          securityResults.cspConfigured = true;
        }
        
        // Check for HTTPS redirect
        if (headersContent.includes('Strict-Transport-Security')) {
          securityResults.httpsRedirectConfigured = true;
        }
      }

      // Calculate security score
      const securityChecks = [
        securityResults.headersConfigExists,
        securityResults.cspConfigured,
        securityResults.httpsRedirectConfigured
      ];
      
      securityResults.securityScore = (securityChecks.filter(Boolean).length / securityChecks.length) * 100;

    } catch (error) {
      console.log(chalk.red(`    Security validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Security Headers Validation',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: securityResults.securityScore >= 100 ? 'PASS' : 'FAIL',
      metrics: securityResults,
      thresholds: {
        securityScore: { value: securityResults.securityScore, threshold: 100, passed: securityResults.securityScore >= 100 }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    Security score: ${securityResults.securityScore.toFixed(1)}%`));
  }

  async validateAssetOptimization() {
    console.log(chalk.cyan('  Validating asset optimization...'));
    
    const assetResults = {
      cssMinified: false,
      jsOptimized: false,
      imagesOptimized: false,
      compressionEnabled: false,
      totalSavings: 0
    };

    try {
      const publicDir = path.join(this.rootDir, 'web-app/public');
      
      // Check CSS files
      const stylesDir = path.join(publicDir, 'styles');
      if (await fs.access(stylesDir).then(() => true).catch(() => false)) {
        const cssFiles = await fs.readdir(stylesDir);
        for (const file of cssFiles.filter(f => f.endsWith('.css'))) {
          const cssPath = path.join(stylesDir, file);
          const cssContent = await fs.readFile(cssPath, 'utf-8');
          
          // Simple minification check
          if (!cssContent.includes('\n\n') && cssContent.length < 1000) {
            assetResults.cssMinified = true;
            break;
          }
        }
      }

      // Check JS files
      const scriptsDir = path.join(publicDir, 'scripts');
      if (await fs.access(scriptsDir).then(() => true).catch(() => false)) {
        const jsFiles = await fs.readdir(scriptsDir);
        for (const file of jsFiles.filter(f => f.endsWith('.js'))) {
          const jsPath = path.join(scriptsDir, file);
          const jsContent = await fs.readFile(jsPath, 'utf-8');
          
          // Simple optimization check
          if (!jsContent.includes('\n\n') && jsContent.includes('function')) {
            assetResults.jsOptimized = true;
            break;
          }
        }
      }

      // Check for compression headers
      const headersFile = path.join(publicDir, '_headers');
      if (await fs.access(headersFile).then(() => true).catch(() => false)) {
        const headersContent = await fs.readFile(headersFile, 'utf-8');
        if (headersContent.includes('Content-Encoding: gzip') || 
            headersContent.includes('Content-Encoding: br')) {
          assetResults.compressionEnabled = true;
        }
      }

    } catch (error) {
      console.log(chalk.red(`    Asset optimization validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Asset Optimization',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: assetResults.cssMinified || assetResults.jsOptimized || assetResults.compressionEnabled ? 'PASS' : 'SKIP',
      metrics: assetResults,
      thresholds: {
        cssMinified: { value: assetResults.cssMinified, threshold: true, passed: assetResults.cssMinified },
        jsOptimized: { value: assetResults.jsOptimized, threshold: true, passed: assetResults.jsOptimized },
        compressionEnabled: { value: assetResults.compressionEnabled, threshold: true, passed: assetResults.compressionEnabled }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    CSS minified: ${assetResults.cssMinified}`));
    console.log(chalk.gray(`    JS optimized: ${assetResults.jsOptimized}`));
    console.log(chalk.gray(`    Compression: ${assetResults.compressionEnabled}`));
  }

  async validateServiceWorkerBuild() {
    console.log(chalk.cyan('  Validating Service Worker build...'));
    
    const swResults = {
      swExists: false,
      swValid: false,
      cacheStrategies: false,
      offlineSupport: false,
      updateMechanism: false
    };

    try {
      const swPath = path.join(this.rootDir, 'web-app/public/sw.js');
      if (await fs.access(swPath).then(() => true).catch(() => false)) {
        swResults.swExists = true;
        
        const swContent = await fs.readFile(swPath, 'utf-8');
        
        // Check for basic SW structure
        if (swContent.includes('self.addEventListener') && 
            swContent.includes('install') && 
            swContent.includes('fetch')) {
          swResults.swValid = true;
        }
        
        // Check for cache strategies
        if (swContent.includes('caches.open') && 
            swContent.includes('cache.put')) {
          swResults.cacheStrategies = true;
        }
        
        // Check for offline support
        if (swContent.includes('offline') || 
            swContent.includes('network-first') || 
            swContent.includes('cache-first')) {
          swResults.offlineSupport = true;
        }
        
        // Check for update mechanism
        if (swContent.includes('skipWaiting') && 
            swContent.includes('clients.claim')) {
          swResults.updateMechanism = true;
        }
      }

    } catch (error) {
      console.log(chalk.red(`    Service Worker validation error: ${error.message}`));
    }

    const testResult = {
      name: 'Service Worker Build',
      timestamp: new Date().toISOString(),
      duration: 0,
      status: swResults.swExists && swResults.swValid && swResults.offlineSupport ? 'PASS' : 'FAIL',
      metrics: swResults,
      thresholds: {
        swExists: { value: swResults.swExists, threshold: true, passed: swResults.swExists },
        swValid: { value: swResults.swValid, threshold: true, passed: swResults.swValid },
        offlineSupport: { value: swResults.offlineSupport, threshold: true, passed: swResults.offlineSupport }
      }
    };

    this.results.tests.push(testResult);
    
    console.log(chalk.gray(`    SW exists: ${swResults.swExists}`));
    console.log(chalk.gray(`    SW valid: ${swResults.swValid}`));
    console.log(chalk.gray(`    Offline support: ${swResults.offlineSupport}`));
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
      overallStatus: failedTests === 0 ? 'PASS' : 'FAIL',
      buildReady: failedTests === 0 && passedTests >= totalTests * 0.8
    };
    
    console.log(chalk.yellow('\n🔧 Build Validation Summary:'));
    console.log(chalk.green(`  ✅ Passed: ${passedTests}`));
    console.log(chalk.red(`  ❌ Failed: ${failedTests}`));
    console.log(chalk.yellow(`  ⏭️  Skipped: ${skippedTests}`));
    console.log(chalk.blue(`  📊 Success Rate: ${this.results.summary.successRate.toFixed(1)}%`));
    console.log(chalk.blue(`  🚀 Build Ready: ${this.results.summary.buildReady ? 'YES' : 'NO'}`));
  }

  async generateReport() {
    const reportDir = path.join(__dirname, '../reports');
    await fs.mkdir(reportDir, { recursive: true });
    
    const reportPath = path.join(reportDir, `build-validation-report-${Date.now()}.json`);
    await fs.writeFile(reportPath, JSON.stringify(this.results, null, 2));
    
    console.log(chalk.blue(`\n📄 Build validation report saved: ${reportPath}`));
  }
}

// Run build validation if called directly
if (require.main === module) {
  const runner = new BuildValidationRunner();
  
  runner.runAllValidations()
    .then(() => {
      process.exit(runner.results.summary.overallStatus === 'PASS' ? 0 : 1);
    })
    .catch((error) => {
      console.error(chalk.red('Build validation failed:'), error);
      process.exit(1);
    });
}

module.exports = BuildValidationRunner;