// Jest setup configuration for all test environments
import '@testing-library/jest-dom';

// Mock WASM module for tests
const mockWasm = {
  Document: {
    new: jest.fn(),
    get_content: jest.fn(),
    set_content: jest.fn(),
    get_metadata: jest.fn()
  },
  Project: {
    new: jest.fn(),
    add_document: jest.fn(),
    get_documents: jest.fn(),
    get_metadata: jest.fn()
  },
  WritingSession: {
    new: jest.fn(),
    start: jest.fn(),
    pause: jest.fn(),
    get_analytics: jest.fn()
  }
};

// Mock IndexedDB for unit tests
const mockIndexedDB = {
  open: jest.fn(),
  deleteDatabase: jest.fn()
};

// Mock Service Worker for tests
const mockServiceWorker = {
  register: jest.fn(),
  getRegistration: jest.fn(),
  ready: Promise.resolve({
    active: { postMessage: jest.fn() }
  })
};

// Setup global mocks
global.writemagic_wasm = mockWasm;
global.indexedDB = mockIndexedDB;
global.navigator = {
  ...global.navigator,
  serviceWorker: mockServiceWorker,
  onLine: true,
  storage: {
    estimate: jest.fn().mockResolvedValue({
      usage: 1000000,
      quota: 10000000
    })
  }
};

// Mock fetch for network requests
global.fetch = jest.fn();

// Mock performance API
global.performance = {
  ...global.performance,
  mark: jest.fn(),
  measure: jest.fn(),
  now: jest.fn(() => Date.now()),
  getEntriesByType: jest.fn(() => []),
  getEntriesByName: jest.fn(() => [])
};

// Mock localStorage and sessionStorage
const mockStorage = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
  length: 0,
  key: jest.fn()
};

global.localStorage = mockStorage;
global.sessionStorage = mockStorage;

// Mock WebGL context for potential canvas operations
HTMLCanvasElement.prototype.getContext = jest.fn();

// Mock console methods to reduce test noise
const originalConsole = global.console;
global.console = {
  ...originalConsole,
  warn: jest.fn(),
  error: jest.fn(),
  debug: jest.fn()
};

// Cleanup after each test
afterEach(() => {
  jest.clearAllMocks();
  
  // Reset WASM mock state
  Object.values(mockWasm).forEach(module => {
    if (typeof module === 'object') {
      Object.values(module).forEach(method => {
        if (typeof method === 'function' && method.mockClear) {
          method.mockClear();
        }
      });
    }
  });
  
  // Reset DOM
  document.body.innerHTML = '';
  document.head.innerHTML = '';
});

// Global test utilities
global.testUtils = {
  // Create a mock document element with common properties
  createMockDocument: () => ({
    id: 'test-doc-1',
    title: 'Test Document',
    content: 'Test content',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString()
  }),
  
  // Create a mock project element
  createMockProject: () => ({
    id: 'test-project-1',
    name: 'Test Project',
    documents: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString()
  }),
  
  // Wait for async operations in tests
  waitFor: (callback, timeout = 1000) => {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();
      const checkCondition = () => {
        if (callback()) {
          resolve();
        } else if (Date.now() - startTime > timeout) {
          reject(new Error('Timeout waiting for condition'));
        } else {
          setTimeout(checkCondition, 10);
        }
      };
      checkCondition();
    });
  }
};