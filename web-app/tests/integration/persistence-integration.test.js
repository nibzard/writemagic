/**
 * Integration tests for persistence layer
 * Tests IndexedDB integration with document and project management
 */

import { DocumentManager } from '@/document-manager.js';
import { ProjectWorkspace } from '@/project-workspace.js';

// Mock IndexedDB for consistent testing
let mockDB;

const createMockIndexedDB = () => {
  const stores = new Map();
  
  const mockObjectStore = (name) => ({
    add: jest.fn((data) => {
      const id = data.id || `${name}-${Date.now()}`;
      stores.set(name, stores.get(name) || new Map());
      stores.get(name).set(id, { ...data, id });
      return Promise.resolve(id);
    }),
    put: jest.fn((data) => {
      const id = data.id;
      stores.set(name, stores.get(name) || new Map());
      stores.get(name).set(id, data);
      return Promise.resolve(id);
    }),
    get: jest.fn((id) => {
      const store = stores.get(name);
      return Promise.resolve(store ? store.get(id) : undefined);
    }),
    getAll: jest.fn(() => {
      const store = stores.get(name);
      return Promise.resolve(store ? Array.from(store.values()) : []);
    }),
    delete: jest.fn((id) => {
      const store = stores.get(name);
      if (store) {
        store.delete(id);
      }
      return Promise.resolve();
    }),
    clear: jest.fn(() => {
      stores.delete(name);
      return Promise.resolve();
    }),
    count: jest.fn(() => {
      const store = stores.get(name);
      return Promise.resolve(store ? store.size : 0);
    }),
    index: jest.fn(() => ({
      getAll: jest.fn(() => Promise.resolve([]))
    }))
  });

  return {
    transaction: jest.fn((storeNames, mode) => ({
      objectStore: jest.fn((name) => mockObjectStore(name)),
      complete: Promise.resolve(),
      abort: jest.fn(),
      error: null
    })),
    close: jest.fn(),
    version: 1,
    objectStoreNames: ['documents', 'projects', 'sessions'],
    createObjectStore: jest.fn(),
    deleteObjectStore: jest.fn()
  };
};

describe('Persistence Integration Tests', () => {
  let documentManager;
  let projectWorkspace;

  beforeEach(() => {
    // Reset and setup mock IndexedDB
    mockDB = createMockIndexedDB();
    
    global.indexedDB = {
      open: jest.fn().mockResolvedValue(mockDB),
      deleteDatabase: jest.fn().mockResolvedValue(undefined)
    };

    // Setup WASM mocks
    global.writemagic_wasm = {
      Document: {
        new: jest.fn(() => ({
          get_content: jest.fn().mockReturnValue(''),
          set_content: jest.fn(),
          get_metadata: jest.fn().mockReturnValue('{}'),
          free: jest.fn()
        }))
      },
      Project: {
        new: jest.fn(() => ({
          add_document: jest.fn(),
          get_documents: jest.fn().mockReturnValue([]),
          get_metadata: jest.fn().mockReturnValue('{}'),
          free: jest.fn()
        }))
      }
    };

    documentManager = new DocumentManager();
    projectWorkspace = new ProjectWorkspace();
  });

  describe('Document Persistence', () => {
    test('should save documents to IndexedDB automatically', async () => {
      // Enable persistence
      documentManager.enablePersistence(true);
      
      const document = await documentManager.createDocument('Persistent Document', 'Content to persist');
      
      // Verify document was saved to IndexedDB
      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      // Should have been called during document creation
      expect(store.add).toHaveBeenCalledWith(
        expect.objectContaining({
          id: document.id,
          title: 'Persistent Document',
          content: 'Content to persist'
        })
      );
    });

    test('should load documents from IndexedDB on initialization', async () => {
      // Pre-populate IndexedDB with documents
      const mockDocuments = [
        { id: 'doc-1', title: 'Loaded Doc 1', content: 'Content 1', createdAt: new Date().toISOString() },
        { id: 'doc-2', title: 'Loaded Doc 2', content: 'Content 2', createdAt: new Date().toISOString() }
      ];

      // Mock the getAll call to return our test data
      const transaction = mockDB.transaction(['documents'], 'readonly');
      const store = transaction.objectStore('documents');
      store.getAll.mockResolvedValue(mockDocuments);

      // Create new document manager to trigger loading
      const newDocumentManager = new DocumentManager();
      await newDocumentManager.loadFromPersistence();

      expect(store.getAll).toHaveBeenCalled();
      expect(newDocumentManager.getDocuments()).toHaveLength(2);
      expect(newDocumentManager.getDocument('doc-1').title).toBe('Loaded Doc 1');
    });

    test('should update documents in IndexedDB on modification', async () => {
      documentManager.enablePersistence(true);
      
      const document = await documentManager.createDocument('Update Test Doc');
      
      // Update the document
      await documentManager.updateDocument(document.id, {
        title: 'Updated Title',
        content: 'Updated content'
      });

      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      // Should have been called during update
      expect(store.put).toHaveBeenCalledWith(
        expect.objectContaining({
          id: document.id,
          title: 'Updated Title',
          content: 'Updated content'
        })
      );
    });

    test('should delete documents from IndexedDB', async () => {
      documentManager.enablePersistence(true);
      
      const document = await documentManager.createDocument('Delete Test Doc');
      
      // Delete the document
      await documentManager.deleteDocument(document.id);

      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      expect(store.delete).toHaveBeenCalledWith(document.id);
    });

    test('should handle auto-save with debouncing', async () => {
      documentManager.enablePersistence(true);
      documentManager.enableAutoSave('auto-save-doc', 100); // 100ms debounce
      
      const document = await documentManager.createDocument('Auto Save Doc');
      
      // Make multiple rapid updates
      await documentManager.updateDocument(document.id, { content: 'Update 1' });
      await documentManager.updateDocument(document.id, { content: 'Update 2' });
      await documentManager.updateDocument(document.id, { content: 'Update 3' });

      // Wait for debounce period
      await new Promise(resolve => setTimeout(resolve, 150));

      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      // Should only save the final state due to debouncing
      expect(store.put).toHaveBeenCalledWith(
        expect.objectContaining({
          content: 'Update 3'
        })
      );
    });
  });

  describe('Project Persistence', () => {
    test('should save projects with document references', async () => {
      projectWorkspace.enablePersistence(true);
      
      const project = await projectWorkspace.createProject('Persistent Project', 'Test description');
      const document = await documentManager.createDocument('Project Document');
      
      await projectWorkspace.addDocumentToProject(project.id, document);

      const transaction = mockDB.transaction(['projects'], 'readwrite');
      const store = transaction.objectStore('projects');
      
      expect(store.put).toHaveBeenCalledWith(
        expect.objectContaining({
          id: project.id,
          name: 'Persistent Project',
          description: 'Test description',
          documents: expect.arrayContaining([document])
        })
      );
    });

    test('should maintain referential integrity between projects and documents', async () => {
      projectWorkspace.enablePersistence(true);
      documentManager.enablePersistence(true);
      
      // Create project and document
      const project = await projectWorkspace.createProject('Integrity Test Project');
      const document = await documentManager.createDocument('Integrity Test Document');
      
      await projectWorkspace.addDocumentToProject(project.id, document);
      
      // Delete document
      await documentManager.deleteDocument(document.id);
      
      // Project should be updated to remove the deleted document reference
      const updatedProject = projectWorkspace.getProject(project.id);
      expect(updatedProject.documents.find(doc => doc.id === document.id)).toBeUndefined();

      // Verify persistence layer is updated
      const transaction = mockDB.transaction(['projects'], 'readwrite');
      const store = transaction.objectStore('projects');
      
      expect(store.put).toHaveBeenCalledWith(
        expect.objectContaining({
          documents: expect.not.arrayContaining([
            expect.objectContaining({ id: document.id })
          ])
        })
      );
    });

    test('should handle project workspace state persistence', async () => {
      projectWorkspace.enablePersistence(true);
      
      const project = await projectWorkspace.createProject('Workspace State Project');
      const doc1 = await documentManager.createDocument('Pane Document 1');
      const doc2 = await documentManager.createDocument('Pane Document 2');
      
      await projectWorkspace.addDocumentToProject(project.id, doc1);
      await projectWorkspace.addDocumentToProject(project.id, doc2);
      
      // Set up workspace layout
      projectWorkspace.setLayout('split');
      projectWorkspace.setPaneContent(0, doc1);
      projectWorkspace.setPaneContent(1, doc2);
      
      // Save workspace state
      const workspaceState = await projectWorkspace.saveState();

      const transaction = mockDB.transaction(['workspace'], 'readwrite');
      const store = transaction.objectStore('workspace');
      
      expect(store.put).toHaveBeenCalledWith(
        expect.objectContaining({
          projects: expect.arrayContaining([project]),
          currentLayout: 'split',
          panes: expect.arrayContaining([doc1, doc2])
        })
      );
    });
  });

  describe('Offline Synchronization', () => {
    test('should queue operations when offline', async () => {
      documentManager.enablePersistence(true);
      
      // Simulate offline state
      Object.defineProperty(navigator, 'onLine', {
        writable: true,
        value: false
      });

      const document = await documentManager.createDocument('Offline Document');
      
      // Operations should be queued
      expect(documentManager.operationQueue).toHaveLength(1);
      expect(documentManager.operationQueue[0]).toMatchObject({
        type: 'create',
        data: expect.objectContaining({ id: document.id })
      });
    });

    test('should sync queued operations when back online', async () => {
      documentManager.enablePersistence(true);
      
      // Start offline
      Object.defineProperty(navigator, 'onLine', {
        writable: true,
        value: false
      });

      // Perform operations while offline
      const doc1 = await documentManager.createDocument('Offline Doc 1');
      const doc2 = await documentManager.createDocument('Offline Doc 2');
      await documentManager.updateDocument(doc1.id, { content: 'Offline update' });

      expect(documentManager.operationQueue).toHaveLength(3);

      // Go back online
      Object.defineProperty(navigator, 'onLine', {
        writable: true,
        value: true
      });

      // Trigger sync
      await documentManager.syncOfflineOperations();

      // All operations should be processed
      expect(documentManager.operationQueue).toHaveLength(0);

      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      // Verify all documents were saved
      expect(store.add).toHaveBeenCalledTimes(2);
      expect(store.put).toHaveBeenCalledTimes(1);
    });

    test('should handle conflicts during sync', async () => {
      documentManager.enablePersistence(true);
      
      const document = await documentManager.createDocument('Conflict Test Doc', 'Original content');
      
      // Simulate document being modified by another client
      const conflictingDocument = {
        ...document,
        content: 'Modified by another client',
        updatedAt: new Date(Date.now() + 1000).toISOString() // Later timestamp
      };

      const transaction = mockDB.transaction(['documents'], 'readonly');
      const store = transaction.objectStore('documents');
      store.get.mockResolvedValue(conflictingDocument);

      // Try to update with older timestamp
      Object.defineProperty(navigator, 'onLine', {
        writable: true,
        value: false
      });

      await documentManager.updateDocument(document.id, { content: 'Local modification' });

      Object.defineProperty(navigator, 'onLine', {
        writable: true,
        value: true
      });

      // Sync should detect conflict and trigger resolution
      const conflictHandler = jest.fn();
      documentManager.on('conflict', conflictHandler);
      
      await documentManager.syncOfflineOperations();

      expect(conflictHandler).toHaveBeenCalledWith({
        documentId: document.id,
        localVersion: expect.objectContaining({ content: 'Local modification' }),
        remoteVersion: expect.objectContaining({ content: 'Modified by another client' })
      });
    });
  });

  describe('Storage Management', () => {
    test('should monitor storage usage', async () => {
      documentManager.enablePersistence(true);
      
      // Mock storage estimate
      navigator.storage.estimate = jest.fn().mockResolvedValue({
        usage: 50000000, // 50MB
        quota: 100000000 // 100MB
      });

      const storageInfo = await documentManager.getStorageInfo();
      
      expect(storageInfo.usage).toBe(50000000);
      expect(storageInfo.quota).toBe(100000000);
      expect(storageInfo.percentage).toBe(50);
    });

    test('should trigger cleanup when storage is low', async () => {
      documentManager.enablePersistence(true);
      
      // Mock low storage
      navigator.storage.estimate = jest.fn().mockResolvedValue({
        usage: 95000000, // 95MB
        quota: 100000000 // 100MB
      });

      const cleanupHandler = jest.fn();
      documentManager.on('storageCleanupNeeded', cleanupHandler);
      
      await documentManager.checkStorageUsage();
      
      expect(cleanupHandler).toHaveBeenCalledWith({
        usage: 95000000,
        quota: 100000000,
        percentage: 95
      });
    });

    test('should implement LRU cache for document cleanup', async () => {
      documentManager.enablePersistence(true);
      documentManager.setMaxDocuments(3); // Limit to 3 documents
      
      // Create documents beyond limit
      const doc1 = await documentManager.createDocument('Doc 1');
      const doc2 = await documentManager.createDocument('Doc 2');
      const doc3 = await documentManager.createDocument('Doc 3');
      const doc4 = await documentManager.createDocument('Doc 4'); // Should trigger cleanup

      // Access doc1 to make it recently used
      documentManager.getDocument(doc1.id);
      
      const doc5 = await documentManager.createDocument('Doc 5'); // Should remove doc2 (LRU)

      // Verify cleanup occurred
      expect(documentManager.getDocuments()).toHaveLength(3);
      expect(documentManager.getDocument(doc2.id)).toBeUndefined(); // Should be removed
      expect(documentManager.getDocument(doc1.id)).toBeDefined(); // Should be kept (recently accessed)

      const transaction = mockDB.transaction(['documents'], 'readwrite');
      const store = transaction.objectStore('documents');
      
      expect(store.delete).toHaveBeenCalledWith(doc2.id);
    });
  });

  describe('Data Migration', () => {
    test('should handle database schema upgrades', async () => {
      const upgradeHandler = jest.fn();
      
      // Mock database with old version
      const oldDB = {
        ...mockDB,
        version: 1,
        createObjectStore: jest.fn(),
        transaction: jest.fn()
      };

      global.indexedDB.open = jest.fn().mockImplementation((name, version) => {
        const request = {
          result: oldDB,
          onupgradeneeded: null,
          onsuccess: null,
          onerror: null
        };

        setTimeout(() => {
          if (version > oldDB.version) {
            request.onupgradeneeded({ target: request, oldVersion: 1, newVersion: version });
          }
          request.onsuccess({ target: request });
        }, 0);

        return request;
      });

      documentManager.onDatabaseUpgrade(upgradeHandler);
      await documentManager.initializePersistence();

      expect(upgradeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          oldVersion: 1,
          newVersion: expect.any(Number)
        })
      );
    });

    test('should migrate data between schema versions', async () => {
      const migrationData = [
        { id: 'doc-1', title: 'Old Doc 1', body: 'Old content 1' }, // Old schema
        { id: 'doc-2', title: 'Old Doc 2', body: 'Old content 2' }
      ];

      // Mock old schema data retrieval
      const transaction = mockDB.transaction(['documents'], 'readonly');
      const store = transaction.objectStore('documents');
      store.getAll.mockResolvedValue(migrationData);

      const migratedDocs = await documentManager.migrateDocuments(migrationData);

      // Should convert 'body' to 'content'
      expect(migratedDocs).toEqual([
        { id: 'doc-1', title: 'Old Doc 1', content: 'Old content 1', version: 2 },
        { id: 'doc-2', title: 'Old Doc 2', content: 'Old content 2', version: 2 }
      ]);
    });
  });

  describe('Error Handling and Recovery', () => {
    test('should handle IndexedDB connection failures', async () => {
      global.indexedDB.open = jest.fn().mockRejectedValue(new Error('IndexedDB not available'));
      
      const errorHandler = jest.fn();
      documentManager.on('persistenceError', errorHandler);
      
      const result = await documentManager.enablePersistence(true);
      
      expect(result).toBe(false);
      expect(errorHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'connection_failed',
          error: expect.any(Error)
        })
      );
    });

    test('should gracefully degrade when persistence fails', async () => {
      // Mock transaction failure
      mockDB.transaction = jest.fn().mockImplementation(() => {
        throw new Error('Transaction failed');
      });

      documentManager.enablePersistence(true);
      
      // Should still create document in memory
      const document = await documentManager.createDocument('Fallback Document');
      
      expect(document).toBeDefined();
      expect(documentManager.getDocument(document.id)).toBeDefined();
      
      // Should emit warning about persistence failure
      const warningHandler = jest.fn();
      documentManager.on('persistenceWarning', warningHandler);
      
      expect(warningHandler).toHaveBeenCalledWith({
        message: 'Persistence temporarily unavailable, using in-memory storage'
      });
    });

    test('should implement data backup and restore', async () => {
      documentManager.enablePersistence(true);
      
      // Create test data
      const doc1 = await documentManager.createDocument('Backup Doc 1', 'Content 1');
      const doc2 = await documentManager.createDocument('Backup Doc 2', 'Content 2');

      // Create backup
      const backup = await documentManager.createBackup();
      
      expect(backup).toMatchObject({
        version: expect.any(String),
        timestamp: expect.any(String),
        documents: expect.arrayContaining([
          expect.objectContaining({ id: doc1.id }),
          expect.objectContaining({ id: doc2.id })
        ])
      });

      // Clear all data
      await documentManager.clearAllData();
      expect(documentManager.getDocuments()).toHaveLength(0);

      // Restore from backup
      const result = await documentManager.restoreFromBackup(backup);
      
      expect(result).toBe(true);
      expect(documentManager.getDocuments()).toHaveLength(2);
      expect(documentManager.getDocument(doc1.id)).toBeDefined();
      expect(documentManager.getDocument(doc2.id)).toBeDefined();
    });
  });
});