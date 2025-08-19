/**
 * Unit tests for DocumentManager module
 * Tests core document management functionality and WASM integration
 */

import { DocumentManager } from '@/document-manager.js';

describe('DocumentManager', () => {
  let documentManager;

  beforeEach(() => {
    // Reset WASM mocks
    global.writemagic_wasm.Document.new.mockReturnValue({
      get_content: jest.fn().mockReturnValue(''),
      set_content: jest.fn(),
      get_metadata: jest.fn().mockReturnValue('{}'),
      free: jest.fn()
    });

    documentManager = new DocumentManager();
  });

  describe('initialization', () => {
    test('should initialize with empty document list', () => {
      expect(documentManager.getDocuments()).toEqual([]);
    });

    test('should initialize WASM module correctly', () => {
      expect(documentManager.wasmModule).toBeDefined();
    });
  });

  describe('document creation', () => {
    test('should create new document with default properties', async () => {
      const doc = await documentManager.createDocument('Test Title');
      
      expect(doc).toHaveProperty('id');
      expect(doc).toHaveProperty('title', 'Test Title');
      expect(doc).toHaveProperty('content', '');
      expect(doc).toHaveProperty('createdAt');
      expect(doc).toHaveProperty('updatedAt');
      expect(global.writemagic_wasm.Document.new).toHaveBeenCalled();
    });

    test('should create document with custom content', async () => {
      const content = 'Custom test content';
      const doc = await documentManager.createDocument('Test Title', content);
      
      expect(doc.content).toBe(content);
    });

    test('should assign unique IDs to documents', async () => {
      const doc1 = await documentManager.createDocument('Doc 1');
      const doc2 = await documentManager.createDocument('Doc 2');
      
      expect(doc1.id).not.toBe(doc2.id);
    });

    test('should add new document to document list', async () => {
      const doc = await documentManager.createDocument('Test Doc');
      const documents = documentManager.getDocuments();
      
      expect(documents).toContain(doc);
      expect(documents.length).toBe(1);
    });
  });

  describe('document retrieval', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument('Test Document', 'Test content');
    });

    test('should retrieve document by ID', () => {
      const retrieved = documentManager.getDocument(testDoc.id);
      expect(retrieved).toEqual(testDoc);
    });

    test('should return undefined for non-existent document', () => {
      const retrieved = documentManager.getDocument('non-existent-id');
      expect(retrieved).toBeUndefined();
    });

    test('should retrieve all documents', () => {
      const documents = documentManager.getDocuments();
      expect(documents).toEqual([testDoc]);
    });
  });

  describe('document updates', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument('Test Document', 'Original content');
    });

    test('should update document content', async () => {
      const newContent = 'Updated content';
      const updated = await documentManager.updateDocument(testDoc.id, { content: newContent });
      
      expect(updated.content).toBe(newContent);
      expect(updated.updatedAt).not.toBe(testDoc.updatedAt);
    });

    test('should update document title', async () => {
      const newTitle = 'Updated Title';
      const updated = await documentManager.updateDocument(testDoc.id, { title: newTitle });
      
      expect(updated.title).toBe(newTitle);
    });

    test('should update multiple properties', async () => {
      const updates = {
        title: 'New Title',
        content: 'New Content'
      };
      const updated = await documentManager.updateDocument(testDoc.id, updates);
      
      expect(updated.title).toBe(updates.title);
      expect(updated.content).toBe(updates.content);
    });

    test('should throw error for non-existent document', async () => {
      await expect(
        documentManager.updateDocument('non-existent-id', { content: 'test' })
      ).rejects.toThrow('Document not found');
    });

    test('should call WASM set_content method', async () => {
      const mockWasmDoc = { set_content: jest.fn() };
      testDoc.wasmInstance = mockWasmDoc;
      
      await documentManager.updateDocument(testDoc.id, { content: 'new content' });
      
      expect(mockWasmDoc.set_content).toHaveBeenCalledWith('new content');
    });
  });

  describe('document deletion', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument('Test Document');
    });

    test('should delete document by ID', async () => {
      const result = await documentManager.deleteDocument(testDoc.id);
      
      expect(result).toBe(true);
      expect(documentManager.getDocument(testDoc.id)).toBeUndefined();
      expect(documentManager.getDocuments().length).toBe(0);
    });

    test('should return false for non-existent document', async () => {
      const result = await documentManager.deleteDocument('non-existent-id');
      expect(result).toBe(false);
    });

    test('should free WASM memory on deletion', async () => {
      const mockWasmDoc = { free: jest.fn() };
      testDoc.wasmInstance = mockWasmDoc;
      
      await documentManager.deleteDocument(testDoc.id);
      
      expect(mockWasmDoc.free).toHaveBeenCalled();
    });
  });

  describe('auto-save functionality', () => {
    let testDoc;

    beforeEach(async () => {
      testDoc = await documentManager.createDocument('Test Document');
      // Mock the persistence layer
      documentManager.persistence = {
        saveDocument: jest.fn().mockResolvedValue(true),
        loadDocuments: jest.fn().mockResolvedValue([])
      };
    });

    test('should enable auto-save for document', () => {
      documentManager.enableAutoSave(testDoc.id, 100); // 100ms interval
      
      expect(documentManager.autoSaveIntervals.has(testDoc.id)).toBe(true);
    });

    test('should disable auto-save for document', () => {
      documentManager.enableAutoSave(testDoc.id, 100);
      documentManager.disableAutoSave(testDoc.id);
      
      expect(documentManager.autoSaveIntervals.has(testDoc.id)).toBe(false);
    });

    test('should trigger save on content change', async () => {
      documentManager.enableAutoSave(testDoc.id, 50);
      
      await documentManager.updateDocument(testDoc.id, { content: 'new content' });
      
      // Wait for auto-save interval
      await new Promise(resolve => setTimeout(resolve, 100));
      
      expect(documentManager.persistence.saveDocument).toHaveBeenCalledWith(
        expect.objectContaining({
          id: testDoc.id,
          content: 'new content'
        })
      );
    });
  });

  describe('search functionality', () => {
    beforeEach(async () => {
      await documentManager.createDocument('First Document', 'This is the first test document with unique content.');
      await documentManager.createDocument('Second Document', 'This is the second test document with different content.');
      await documentManager.createDocument('Third Document', 'This document contains special keywords and terms.');
    });

    test('should search documents by title', () => {
      const results = documentManager.searchDocuments('First');
      expect(results).toHaveLength(1);
      expect(results[0].title).toBe('First Document');
    });

    test('should search documents by content', () => {
      const results = documentManager.searchDocuments('unique');
      expect(results).toHaveLength(1);
      expect(results[0].title).toBe('First Document');
    });

    test('should return empty array for no matches', () => {
      const results = documentManager.searchDocuments('nonexistent');
      expect(results).toHaveLength(0);
    });

    test('should be case insensitive', () => {
      const results = documentManager.searchDocuments('SPECIAL');
      expect(results).toHaveLength(1);
      expect(results[0].title).toBe('Third Document');
    });

    test('should search across multiple fields', () => {
      const results = documentManager.searchDocuments('document');
      expect(results).toHaveLength(3); // All documents contain 'document'
    });
  });

  describe('event handling', () => {
    test('should emit document created event', async () => {
      const eventHandler = jest.fn();
      documentManager.on('documentCreated', eventHandler);
      
      const doc = await documentManager.createDocument('Test Doc');
      
      expect(eventHandler).toHaveBeenCalledWith(doc);
    });

    test('should emit document updated event', async () => {
      const doc = await documentManager.createDocument('Test Doc');
      const eventHandler = jest.fn();
      documentManager.on('documentUpdated', eventHandler);
      
      const updated = await documentManager.updateDocument(doc.id, { title: 'Updated' });
      
      expect(eventHandler).toHaveBeenCalledWith(updated);
    });

    test('should emit document deleted event', async () => {
      const doc = await documentManager.createDocument('Test Doc');
      const eventHandler = jest.fn();
      documentManager.on('documentDeleted', eventHandler);
      
      await documentManager.deleteDocument(doc.id);
      
      expect(eventHandler).toHaveBeenCalledWith(doc.id);
    });
  });

  describe('error handling', () => {
    test('should handle WASM initialization failure', () => {
      global.writemagic_wasm = null;
      
      expect(() => new DocumentManager()).toThrow('WASM module not available');
    });

    test('should handle invalid document data', async () => {
      await expect(
        documentManager.createDocument(null)
      ).rejects.toThrow('Invalid document title');
    });

    test('should handle WASM method errors', async () => {
      global.writemagic_wasm.Document.new.mockImplementation(() => {
        throw new Error('WASM error');
      });
      
      await expect(
        documentManager.createDocument('Test')
      ).rejects.toThrow('WASM error');
    });
  });
});