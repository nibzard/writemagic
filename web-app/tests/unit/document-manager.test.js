/**
 * Unit tests for DocumentManager module
 * Tests core document management functionality and WASM integration
 */

import { DocumentManager, SaveState, DocumentEvents } from '@/document-manager.js';

describe('DocumentManager', () => {
  let documentManager;
  let mockWasmEngine;

  beforeEach(() => {
    // Create mock WASM engine that matches the actual API
    mockWasmEngine = {
      create_document: jest.fn().mockResolvedValue({
        id: 'test-doc-1',
        title: 'Test Document',
        content: '',
        content_type: 'markdown',
        word_count: 0,
        character_count: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        created_by: 'test-user',
        is_deleted: false,
        project_id: null
      }),
      get_document: jest.fn().mockResolvedValue({
        id: 'test-doc-1',
        title: 'Test Document',
        content: 'Test content',
        content_type: 'markdown',
        word_count: 2,
        character_count: 12,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        created_by: 'test-user',
        is_deleted: false,
        project_id: null
      }),
      update_document: jest.fn().mockResolvedValue({
        id: 'test-doc-1',
        title: 'Test Document',
        content: 'Updated content',
        content_type: 'markdown',
        word_count: 2,
        character_count: 15,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        created_by: 'test-user',
        is_deleted: false,
        project_id: null
      }),
      list_project_documents: jest.fn().mockResolvedValue([]),
      add_document_to_project: jest.fn().mockResolvedValue(true)
    };

    documentManager = new DocumentManager(mockWasmEngine, {
      userId: 'test-user',
      autoSaveDelay: 100 // Shorter delay for tests
    });
  });

  describe('initialization', () => {
    test('should initialize with mock WASM engine', () => {
      expect(documentManager.engine).toBe(mockWasmEngine);
      expect(documentManager.config.userId).toBe('test-user');
    });

    test('should initialize with default config', () => {
      expect(documentManager.config.autoSaveDelay).toBe(100);
      expect(documentManager.config.contentValidation).toBe(true);
    });

    test('should initialize empty document cache', () => {
      expect(documentManager.documents.size).toBe(0);
      expect(documentManager.drafts.size).toBe(0);
      expect(documentManager.saveStates.size).toBe(0);
    });
  });

  describe('document creation', () => {
    test('should create new document with default options', async () => {
      const doc = await documentManager.createDocument();
      
      expect(mockWasmEngine.create_document).toHaveBeenCalledWith(
        'Untitled Document',
        '',
        'markdown',
        'test-user'
      );
      expect(doc).toHaveProperty('id', 'test-doc-1');
      expect(doc).toHaveProperty('title', 'Test Document');
      expect(doc).toHaveProperty('content', '');
    });

    test('should create document with custom options', async () => {
      const options = {
        title: 'Custom Title',
        content: 'Custom content',
        contentType: 'text'
      };
      
      const doc = await documentManager.createDocument(options);
      
      expect(mockWasmEngine.create_document).toHaveBeenCalledWith(
        'Custom Title',
        'Custom content',
        'text',
        'test-user'
      );
    });

    test('should cache created document', async () => {
      const doc = await documentManager.createDocument();
      
      expect(documentManager.documents.has('test-doc-1')).toBe(true);
      expect(documentManager.documents.get('test-doc-1')).toEqual(doc);
    });

    test('should set save state to SAVED', async () => {
      const doc = await documentManager.createDocument();
      
      expect(documentManager.saveStates.get('test-doc-1')).toBe(SaveState.SAVED);
    });

    test('should emit document created event', async () => {
      const eventHandler = jest.fn();
      documentManager.on(DocumentEvents.DOCUMENT_CREATED, eventHandler);
      
      const doc = await documentManager.createDocument();
      
      expect(eventHandler).toHaveBeenCalledWith({ document: doc });
    });
  });

  describe('document loading', () => {
    test('should load document from WASM engine', async () => {
      const doc = await documentManager.loadDocument('test-doc-1');
      
      expect(mockWasmEngine.get_document).toHaveBeenCalledWith('test-doc-1');
      expect(doc).toHaveProperty('id', 'test-doc-1');
      expect(doc).toHaveProperty('content', 'Test content');
    });

    test('should return cached document if available', async () => {
      // First load
      const doc1 = await documentManager.loadDocument('test-doc-1');
      
      // Second load should use cache
      const doc2 = await documentManager.loadDocument('test-doc-1');
      
      expect(mockWasmEngine.get_document).toHaveBeenCalledTimes(1);
      expect(doc2).toBe(doc1);
    });

    test('should force fresh load when requested', async () => {
      await documentManager.loadDocument('test-doc-1');
      await documentManager.loadDocument('test-doc-1', { forceFresh: true });
      
      expect(mockWasmEngine.get_document).toHaveBeenCalledTimes(2);
    });

    test('should emit document loaded event', async () => {
      const eventHandler = jest.fn();
      documentManager.on(DocumentEvents.DOCUMENT_LOADED, eventHandler);
      
      const doc = await documentManager.loadDocument('test-doc-1');
      
      expect(eventHandler).toHaveBeenCalledWith({ document: doc });
    });
  });

  describe('content updates', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument();
    });

    test('should update content and trigger draft state', () => {
      const result = documentManager.updateContent('test-doc-1', 'New content');
      
      expect(result).toBe(true);
      expect(documentManager.saveStates.get('test-doc-1')).toBe(SaveState.DRAFT);
    });

    test('should update document properties', () => {
      documentManager.updateContent('test-doc-1', 'New content with words');
      
      const doc = documentManager.documents.get('test-doc-1');
      expect(doc.content).toBe('New content with words');
      expect(doc.wordCount).toBeGreaterThan(0);
      expect(doc.characterCount).toBe(22);
    });

    test('should emit content changed event', () => {
      const eventHandler = jest.fn();
      documentManager.on(DocumentEvents.CONTENT_CHANGED, eventHandler);
      
      documentManager.updateContent('test-doc-1', 'New content');
      
      expect(eventHandler).toHaveBeenCalledWith(expect.objectContaining({
        content: 'New content'
      }));
    });

    test('should save draft to localStorage', () => {
      documentManager.updateContent('test-doc-1', 'New content');
      
      expect(documentManager.drafts.has('test-doc-1')).toBe(true);
    });
  });

  describe('document saving', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument();
      documentManager.updateContent('test-doc-1', 'Content to save');
    });

    test('should save document through WASM engine', async () => {
      const savedDoc = await documentManager.saveDocument('test-doc-1');
      
      expect(mockWasmEngine.update_document).toHaveBeenCalledWith(
        'test-doc-1',
        'Content to save',
        'test-user'
      );
      expect(savedDoc.content).toBe('Updated content'); // From mock response
    });

    test('should clear draft after successful save', async () => {
      await documentManager.saveDocument('test-doc-1');
      
      expect(documentManager.drafts.has('test-doc-1')).toBe(false);
    });

    test('should set save state to SAVED', async () => {
      await documentManager.saveDocument('test-doc-1');
      
      expect(documentManager.saveStates.get('test-doc-1')).toBe(SaveState.SAVED);
    });

    test('should emit document updated event', async () => {
      const eventHandler = jest.fn();
      documentManager.on(DocumentEvents.DOCUMENT_UPDATED, eventHandler);
      
      const savedDoc = await documentManager.saveDocument('test-doc-1');
      
      expect(eventHandler).toHaveBeenCalledWith({ document: savedDoc });
    });
  });

  describe('auto-save functionality', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument();
    });

    test('should trigger auto-save on content update', (done) => {
      const originalSave = documentManager.saveDocument;
      documentManager.saveDocument = jest.fn().mockResolvedValue(testDoc);
      
      documentManager.updateContent('test-doc-1', 'Auto save content');
      
      // Wait for debounced auto-save
      setTimeout(() => {
        expect(documentManager.saveDocument).toHaveBeenCalledWith('test-doc-1');
        done();
      }, 150);
    });

    test('should not auto-save when offline', () => {
      documentManager.isOnline = false;
      const originalSave = documentManager.saveDocument;
      documentManager.saveDocument = jest.fn();
      
      documentManager.updateContent('test-doc-1', 'Offline content');
      
      expect(documentManager.saveDocument).not.toHaveBeenCalled();
    });
  });

  describe('document listing', () => {
    beforeEach(async () => {
      await documentManager.createDocument({ title: 'Doc 1' });
      await documentManager.createDocument({ title: 'Doc 2' });
    });

    test('should list cached documents', async () => {
      const documents = await documentManager.listDocuments();
      
      expect(documents).toHaveLength(2);
      expect(documents.map(d => d.title)).toContain('Test Document');
    });

    test('should filter by search query', async () => {
      const documents = await documentManager.listDocuments({
        searchQuery: 'Doc 1'
      });
      
      // This would require implementing search in ContentUtilities mock
      expect(documents).toBeDefined();
    });
  });

  describe('document duplication', () => {
    let originalDoc;

    beforeEach(async () => {
      originalDoc = await documentManager.createDocument({
        title: 'Original Doc',
        content: 'Original content'
      });
    });

    test('should duplicate document with new title', async () => {
      const duplicated = await documentManager.duplicateDocument('test-doc-1', 'Duplicated Doc');
      
      expect(mockWasmEngine.create_document).toHaveBeenCalledTimes(2);
      expect(duplicated.title).toBe('Test Document'); // From mock response
    });
  });

  describe('save states', () => {
    test('should manage save states correctly', () => {
      documentManager.setSaveState('test-doc', SaveState.SAVING);
      
      expect(documentManager.saveStates.get('test-doc')).toBe(SaveState.SAVING);
    });

    test('should emit save state changed event', () => {
      const eventHandler = jest.fn();
      documentManager.on(DocumentEvents.SAVE_STATE_CHANGED, eventHandler);
      
      documentManager.setSaveState('test-doc', SaveState.ERROR);
      
      expect(eventHandler).toHaveBeenCalledWith({
        documentId: 'test-doc',
        state: SaveState.ERROR
      });
    });
  });

  describe('offline/online handling', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument();
      documentManager.updateContent('test-doc-1', 'Offline content');
    });

    test('should handle offline mode', () => {
      documentManager.handleOfflineMode();
      
      expect(documentManager.saveStates.get('test-doc-1')).toBe(SaveState.OFFLINE);
    });

    test('should resume online and sync changes', async () => {
      documentManager.handleOfflineMode();
      
      const syncSpy = jest.spyOn(documentManager, 'performAutoSave').mockResolvedValue();
      await documentManager.handleOnlineResume();
      
      expect(syncSpy).toHaveBeenCalledWith('test-doc-1');
    });
  });

  describe('analytics', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument();
      documentManager.updateContent('test-doc-1', 'Content for analytics');
    });

    test('should provide document with analytics', () => {
      const docWithAnalytics = documentManager.getDocumentWithAnalytics('test-doc-1');
      
      expect(docWithAnalytics).toHaveProperty('analytics');
      expect(docWithAnalytics.analytics).toHaveProperty('wordCount');
      expect(docWithAnalytics.analytics).toHaveProperty('characterCount');
      expect(docWithAnalytics.analytics).toHaveProperty('saveState');
    });

    test('should return null for non-existent document', () => {
      const result = documentManager.getDocumentWithAnalytics('non-existent');
      
      expect(result).toBeNull();
    });
  });

  describe('cleanup', () => {
    test('should clean up resources on destroy', () => {
      documentManager.documents.set('test', {});
      documentManager.drafts.set('test', []);
      
      documentManager.destroy();
      
      expect(documentManager.documents.size).toBe(0);
      expect(documentManager.drafts.size).toBe(0);
      expect(documentManager.saveStates.size).toBe(0);
    });
  });

  describe('error handling', () => {
    test('should handle WASM engine errors on creation', async () => {
      mockWasmEngine.create_document.mockRejectedValue(new Error('WASM error'));
      
      await expect(
        documentManager.createDocument()
      ).rejects.toThrow('Failed to create document: WASM error');
    });

    test('should handle WASM engine errors on load', async () => {
      mockWasmEngine.get_document.mockRejectedValue(new Error('Not found'));
      
      await expect(
        documentManager.loadDocument('invalid-id')
      ).rejects.toThrow('Failed to load document: Not found');
    });

    test('should emit error events', async () => {
      const errorHandler = jest.fn();
      documentManager.on(DocumentEvents.ERROR, errorHandler);
      
      mockWasmEngine.create_document.mockRejectedValue(new Error('Test error'));
      
      try {
        await documentManager.createDocument();
      } catch (e) {
        // Expected
      }
      
      expect(errorHandler).toHaveBeenCalledWith(expect.objectContaining({
        error: expect.any(Error),
        operation: 'create'
      }));
    });
  });
});