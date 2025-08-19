module.exports = {
  displayName: 'AI Integration Tests',
  testEnvironment: 'jsdom',
  testMatch: ['<rootDir>/ai/**/*.test.js'],
  setupFilesAfterEnv: ['<rootDir>/setup/jest.setup.js'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/../src/js/$1',
    '^@public/(.*)$': '<rootDir>/../public/$1'
  },
  testTimeout: 30000, // AI tests may take longer
  collectCoverageFrom: [
    '../src/js/ai-proxy-integration.js',
    '../src/js/writing-session.js'
  ],
  coverageDirectory: '<rootDir>/coverage/ai',
  coverageReporters: ['text', 'lcov'],
  testPathIgnorePatterns: [
    '/node_modules/',
    '/unit/',
    '/integration/',
    '/e2e/',
    '/performance/',
    '/offline/'
  ],
  verbose: true,
  // Mock network requests by default
  clearMocks: true,
  resetMocks: true
};