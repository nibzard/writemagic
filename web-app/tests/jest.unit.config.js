module.exports = {
  displayName: 'Unit Tests',
  testEnvironment: 'jsdom',
  testMatch: ['<rootDir>/unit/**/*.test.js'],
  setupFilesAfterEnv: ['<rootDir>/setup/jest.setup.js'],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/../src/js/$1',
    '^@public/(.*)$': '<rootDir>/../public/$1'
  },
  transform: {
    '^.+\\.js$': ['babel-jest', { presets: [['@babel/preset-env', { targets: { node: 'current' } }]] }]
  },
  extensionsToTreatAsEsm: ['.js'],
  clearMocks: true,
  resetMocks: true,
  collectCoverageFrom: [
    '../src/js/**/*.js',
    '!../src/js/**/*.test.js',
    '!../src/js/example-usage.js'
  ],
  coverageDirectory: '<rootDir>/coverage/unit',
  coverageReporters: ['text', 'lcov', 'html'],
  testPathIgnorePatterns: [
    '/node_modules/',
    '/e2e/',
    '/integration/',
    '/performance/',
    '/offline/'
  ],
  verbose: true,
  bail: false,
  errorOnDeprecated: true
};