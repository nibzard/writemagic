module.exports = {
  displayName: 'Integration Tests',
  testEnvironment: 'jsdom',
  testMatch: ['<rootDir>/integration/**/*.test.js'],
  setupFilesAfterEnv: ['<rootDir>/setup/jest.setup.js'],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/../src/js/$1',
    '^@public/(.*)$': '<rootDir>/../public/$1'
  },
  transform: {
    '^.+\\.js$': ['babel-jest', { presets: [['@babel/preset-env', { targets: { node: 'current' } }]] }]
  },
  extensionsToTreatAsEsm: ['.js'],
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