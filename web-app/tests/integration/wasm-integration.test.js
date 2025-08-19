/**
 * Integration tests for WASM module integration
 * Tests the complete integration between JavaScript modules and WASM core
 */

import { DocumentManager } from '@/document-manager.js';
import { ProjectWorkspace } from '@/project-workspace.js';
import { WritingSession } from '@/writing-session.js';

describe('WASM Integration Tests', () => {
  let documentManager;
  let projectWorkspace;
  let writingSession;

  beforeEach(async () => {
    // Create more realistic WASM mocks for integration testing
    const mockDocument = {
      get_content: jest.fn().mockReturnValue('Test content'),
      set_content: jest.fn(),
      get_metadata: jest.fn().mockReturnValue(JSON.stringify({
        wordCount: 2,
        characterCount: 12,
        lastModified: new Date().toISOString()
      })),
      free: jest.fn()
    };

    const mockProject = {
      add_document: jest.fn().mockReturnValue(true),
      get_documents: jest.fn().mockReturnValue(['doc-1', 'doc-2']),
      get_metadata: jest.fn().mockReturnValue(JSON.stringify({
        documentCount: 2,
        totalWords: 100,
        createdAt: new Date().toISOString()
      })),
      free: jest.fn()
    };

    const mockWritingSession = {
      start: jest.fn().mockReturnValue(true),
      pause: jest.fn().mockReturnValue(true),
      get_analytics: jest.fn().mockReturnValue(JSON.stringify({
        duration: 300000,
        wordCount: 50,
        characterCount: 250,
        wordsPerMinute: 10
      })),
      free: jest.fn()
    };

    global.writemagic_wasm = {
      Document: { new: jest.fn(() => mockDocument) },
      Project: { new: jest.fn(() => mockProject) },
      WritingSession: { new: jest.fn(() => mockWritingSession) }
    };

    documentManager = new DocumentManager();
    projectWorkspace = new ProjectWorkspace();
    writingSession = new WritingSession();
  });

  describe('Document-Project Integration', () => {
    test('should create document and add to project seamlessly', async () => {
      // Create a new project
      const project = await projectWorkspace.createProject('Integration Test Project');
      expect(project).toBeDefined();
      expect(project.id).toBeDefined();

      // Create a new document
      const document = await documentManager.createDocument('Integration Test Document', 'Test content for integration');
      expect(document).toBeDefined();
      expect(document.id).toBeDefined();

      // Add document to project
      const result = await projectWorkspace.addDocumentToProject(project.id, document);
      expect(result).toBe(true);

      // Verify integration
      const projectDocuments = projectWorkspace.getProjectDocuments(project.id);
      expect(projectDocuments).toContain(document);

      // Verify WASM calls
      expect(global.writemagic_wasm.Project.new).toHaveBeenCalled();
      expect(global.writemagic_wasm.Document.new).toHaveBeenCalled();
      expect(project.wasmInstance.add_document).toHaveBeenCalledWith(document);
    });

    test('should sync document updates across project workspace', async () => {
      const project = await projectWorkspace.createProject('Sync Test Project');
      const document = await documentManager.createDocument('Sync Test Document', 'Original content');
      
      await projectWorkspace.addDocumentToProject(project.id, document);

      // Update document content
      const updatedDocument = await documentManager.updateDocument(document.id, {
        content: 'Updated content for sync test'
      });

      // Verify document is updated in project
      const projectDocument = projectWorkspace.getProjectDocuments(project.id)
        .find(doc => doc.id === document.id);
      
      expect(projectDocument.content).toBe('Updated content for sync test');
      expect(projectDocument.updatedAt).toBe(updatedDocument.updatedAt);
    });

    test('should handle document deletion from projects', async () => {
      const project = await projectWorkspace.createProject('Deletion Test Project');
      const document = await documentManager.createDocument('Document to Delete');
      
      await projectWorkspace.addDocumentToProject(project.id, document);
      
      // Delete document from document manager
      await documentManager.deleteDocument(document.id);
      
      // Remove document from project
      const result = await projectWorkspace.removeDocumentFromProject(project.id, document.id);
      expect(result).toBe(true);

      // Verify document is removed from project
      const projectDocuments = projectWorkspace.getProjectDocuments(project.id);
      expect(projectDocuments.find(doc => doc.id === document.id)).toBeUndefined();
    });
  });

  describe('Writing Session Integration', () => {
    test('should integrate writing session with document editing', async () => {
      const document = await documentManager.createDocument('Session Test Document');
      
      // Start writing session
      const sessionStarted = await writingSession.start();
      expect(sessionStarted).toBe(true);

      // Simulate writing content
      const content = 'This is content written during the session';
      await documentManager.updateDocument(document.id, { content });
      writingSession.addContent(content);

      // Verify integration
      expect(writingSession.getCurrentContent()).toContain(content);
      expect(document.content).toBe(content);

      // Get session analytics
      const analytics = await writingSession.getAnalytics();
      expect(analytics.wordCount).toBeGreaterThan(0);

      // Verify WASM interactions
      expect(global.writemagic_wasm.WritingSession.new).toHaveBeenCalled();
      expect(writingSession.wasmInstance.start).toHaveBeenCalled();
    });

    test('should track writing sessions across project documents', async () => {
      const project = await projectWorkspace.createProject('Session Tracking Project');
      const doc1 = await documentManager.createDocument('Document 1');
      const doc2 = await documentManager.createDocument('Document 2');
      
      await projectWorkspace.addDocumentToProject(project.id, doc1);
      await projectWorkspace.addDocumentToProject(project.id, doc2);

      await writingSession.start();

      // Write to first document
      await documentManager.updateDocument(doc1.id, { content: 'Content for document one' });
      writingSession.addContent('Content for document one');

      // Switch to second document
      await documentManager.updateDocument(doc2.id, { content: 'Content for document two' });
      writingSession.addContent(' Content for document two');

      const analytics = await writingSession.getAnalytics();
      expect(analytics.wordCount).toBe(8); // Total words across both documents

      await writingSession.stop();
    });

    test('should handle session persistence with document state', async () => {
      const document = await documentManager.createDocument('Persistence Test');
      await writingSession.start();
      
      writingSession.addContent('Content before save');
      await documentManager.updateDocument(document.id, { content: 'Content before save' });

      // Save session state
      const sessionState = await writingSession.saveState();
      expect(sessionState.currentContent).toBe('Content before save');

      // Create new instances to simulate reload
      const newWritingSession = new WritingSession();
      const newDocumentManager = new DocumentManager();

      // Restore session state
      await newWritingSession.restoreState(sessionState);
      expect(newWritingSession.getCurrentContent()).toBe('Content before save');
    });
  });

  describe('Multi-Pane Document Editing Integration', () => {
    test('should handle multiple documents in different panes', async () => {
      const project = await projectWorkspace.createProject('Multi-Pane Project');
      
      // Create multiple documents
      const doc1 = await documentManager.createDocument('Pane 1 Document', 'Content 1');
      const doc2 = await documentManager.createDocument('Pane 2 Document', 'Content 2');
      const doc3 = await documentManager.createDocument('Pane 3 Document', 'Content 3');

      await projectWorkspace.addDocumentToProject(project.id, doc1);
      await projectWorkspace.addDocumentToProject(project.id, doc2);
      await projectWorkspace.addDocumentToProject(project.id, doc3);

      // Set up triple pane layout
      projectWorkspace.setLayout('triple');
      expect(projectWorkspace.getPanes().length).toBe(3);

      // Load documents into panes
      projectWorkspace.setPaneContent(0, doc1);
      projectWorkspace.setPaneContent(1, doc2);
      projectWorkspace.setPaneContent(2, doc3);

      // Verify pane contents
      expect(projectWorkspace.getPaneContent(0)).toEqual(doc1);
      expect(projectWorkspace.getPaneContent(1)).toEqual(doc2);
      expect(projectWorkspace.getPaneContent(2)).toEqual(doc3);

      // Test concurrent editing
      await documentManager.updateDocument(doc1.id, { content: 'Updated content 1' });
      await documentManager.updateDocument(doc2.id, { content: 'Updated content 2' });

      // Verify updates reflect in panes
      expect(projectWorkspace.getPaneContent(0).content).toBe('Updated content 1');
      expect(projectWorkspace.getPaneContent(1).content).toBe('Updated content 2');
    });

    test('should sync writing session across multiple panes', async () => {
      projectWorkspace.setLayout('split');
      await writingSession.start();

      const doc1 = await documentManager.createDocument('Split Pane 1');
      const doc2 = await documentManager.createDocument('Split Pane 2');

      projectWorkspace.setPaneContent(0, doc1);
      projectWorkspace.setPaneContent(1, doc2);

      // Write to first pane
      projectWorkspace.setActivePane(0);
      await documentManager.updateDocument(doc1.id, { content: 'Writing in pane 1' });
      writingSession.addContent('Writing in pane 1');

      // Switch to second pane
      projectWorkspace.setActivePane(1);
      await documentManager.updateDocument(doc2.id, { content: ' Writing in pane 2' });
      writingSession.addContent(' Writing in pane 2');

      const analytics = await writingSession.getAnalytics();
      expect(analytics.wordCount).toBe(6); // Combined words from both panes

      // Verify active pane tracking
      expect(projectWorkspace.getActivePane()).toBe(1);
      expect(projectWorkspace.getActivePaneContent()).toEqual(doc2);
    });
  });

  describe('Error Handling and Recovery', () => {
    test('should handle WASM module initialization failure gracefully', () => {
      global.writemagic_wasm = null;

      expect(() => new DocumentManager()).toThrow('WASM module not available');
      expect(() => new ProjectWorkspace()).toThrow();
      expect(() => new WritingSession()).toThrow();
    });

    test('should recover from WASM method failures', async () => {
      const document = await documentManager.createDocument('Error Test Document');
      
      // Mock WASM method to throw error
      document.wasmInstance.set_content.mockImplementation(() => {
        throw new Error('WASM method failed');
      });

      // Should handle error gracefully
      await expect(
        documentManager.updateDocument(document.id, { content: 'new content' })
      ).rejects.toThrow('WASM method failed');

      // Document should still exist but not be updated
      const retrievedDoc = documentManager.getDocument(document.id);
      expect(retrievedDoc).toBeDefined();
      expect(retrievedDoc.content).not.toBe('new content');
    });

    test('should handle memory management correctly', async () => {
      const documents = [];
      const projects = [];

      // Create multiple documents and projects
      for (let i = 0; i < 10; i++) {
        const doc = await documentManager.createDocument(`Document ${i}`);
        const project = await projectWorkspace.createProject(`Project ${i}`);
        documents.push(doc);
        projects.push(project);
      }

      // Delete all documents and projects
      for (const doc of documents) {
        await documentManager.deleteDocument(doc.id);
        expect(doc.wasmInstance.free).toHaveBeenCalled();
      }

      for (const project of projects) {
        await projectWorkspace.deleteProject(project.id);
        expect(project.wasmInstance.free).toHaveBeenCalled();
      }

      // Verify cleanup
      expect(documentManager.getDocuments().length).toBe(0);
      expect(projectWorkspace.getProjects().length).toBe(0);
    });
  });

  describe('Performance Integration', () => {
    test('should handle large document operations efficiently', async () => {
      const startTime = performance.now();
      
      // Create a large document
      const largeContent = 'Lorem ipsum '.repeat(10000); // ~110,000 characters
      const document = await documentManager.createDocument('Large Document', largeContent);
      
      expect(document.content.length).toBeGreaterThan(100000);
      
      const creationTime = performance.now() - startTime;
      expect(creationTime).toBeLessThan(1000); // Should complete within 1 second

      // Test large document update
      const updateStartTime = performance.now();
      const updatedContent = largeContent + ' Additional content';
      await documentManager.updateDocument(document.id, { content: updatedContent });
      
      const updateTime = performance.now() - updateStartTime;
      expect(updateTime).toBeLessThan(500); // Updates should be fast
    });

    test('should handle concurrent operations without conflicts', async () => {
      const promises = [];
      const docCount = 20;

      // Create multiple documents concurrently
      for (let i = 0; i < docCount; i++) {
        promises.push(
          documentManager.createDocument(`Concurrent Doc ${i}`, `Content ${i}`)
        );
      }

      const documents = await Promise.all(promises);
      
      expect(documents.length).toBe(docCount);
      expect(documentManager.getDocuments().length).toBe(docCount);

      // Verify all documents have unique IDs
      const ids = documents.map(doc => doc.id);
      const uniqueIds = [...new Set(ids)];
      expect(uniqueIds.length).toBe(docCount);

      // Test concurrent updates
      const updatePromises = documents.map((doc, i) => 
        documentManager.updateDocument(doc.id, { content: `Updated content ${i}` })
      );

      const updatedDocs = await Promise.all(updatePromises);
      
      updatedDocs.forEach((doc, i) => {
        expect(doc.content).toBe(`Updated content ${i}`);
      });
    });
  });

  describe('State Synchronization', () => {
    test('should maintain consistent state across all components', async () => {
      // Create integrated workspace
      const project = await projectWorkspace.createProject('State Sync Project');
      const document = await documentManager.createDocument('State Sync Document');
      
      await projectWorkspace.addDocumentToProject(project.id, document);
      await writingSession.start();

      // Update document through document manager
      const updatedDoc = await documentManager.updateDocument(document.id, {
        title: 'Updated Title',
        content: 'Updated content'
      });

      // Verify state consistency
      const projectDoc = projectWorkspace.getProjectDocuments(project.id)
        .find(doc => doc.id === document.id);
      
      expect(projectDoc.title).toBe('Updated Title');
      expect(projectDoc.content).toBe('Updated content');
      expect(projectDoc.updatedAt).toBe(updatedDoc.updatedAt);

      // Add content to writing session
      writingSession.addContent('Updated content');
      const analytics = await writingSession.getAnalytics();
      
      expect(analytics.wordCount).toBe(2); // "Updated content"
    });

    test('should handle workspace state persistence and restoration', async () => {
      // Set up complex workspace state
      const project1 = await projectWorkspace.createProject('Project 1');
      const project2 = await projectWorkspace.createProject('Project 2');
      
      const doc1 = await documentManager.createDocument('Doc 1', 'Content 1');
      const doc2 = await documentManager.createDocument('Doc 2', 'Content 2');
      const doc3 = await documentManager.createDocument('Doc 3', 'Content 3');

      await projectWorkspace.addDocumentToProject(project1.id, doc1);
      await projectWorkspace.addDocumentToProject(project1.id, doc2);
      await projectWorkspace.addDocumentToProject(project2.id, doc3);

      projectWorkspace.setLayout('triple');
      projectWorkspace.setPaneContent(0, doc1);
      projectWorkspace.setPaneContent(1, doc2);
      projectWorkspace.setPaneContent(2, doc3);

      // Save complete workspace state
      const workspaceState = await projectWorkspace.saveState();
      
      expect(workspaceState.projects.length).toBe(2);
      expect(workspaceState.currentLayout).toBe('triple');
      expect(workspaceState.panes.length).toBe(3);

      // Create new workspace instance and restore
      const newWorkspace = new ProjectWorkspace();
      const restored = await newWorkspace.restoreState(workspaceState);
      
      expect(restored).toBe(true);
      expect(newWorkspace.getProjects().length).toBe(2);
      expect(newWorkspace.currentLayout).toBe('triple');
      expect(newWorkspace.getPanes().length).toBe(3);
    });
  });
});