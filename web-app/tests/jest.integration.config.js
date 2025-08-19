module.exports = {
  displayName: 'Integration Tests',
  testEnvironment: 'jsdom',
  testMatch: ['<rootDir>/integration/**/*.test.js'],
  setupFilesAfterEnv: ['<rootDir>/setup/jest.setup.js'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/../src/js/$1',
    '^@public/(.*)$': '<rootDir>/../public/$1'
  },
  testTimeout: 10000,
  collectCoverageFrom: [
    '../src/js/**/*.js',
    '!../src/js/**/*.test.js'
  ],
  coverageDirectory: '<rootDir>/coverage/integration',
  coverageReporters: ['text', 'lcov'],
  testPathIgnorePatterns: [
    '/node_modules/',
    '/unit/',
    '/e2e/',
    '/performance/',
    '/offline/'
  ],
  verbose: true
};