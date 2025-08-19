/**
 * Unit tests for ProjectWorkspace module
 * Tests project management and multi-pane workspace functionality
 */

import { ProjectWorkspace } from '@/project-workspace.js';

describe('ProjectWorkspace', () => {
  let workspace;

  beforeEach(() => {
    // Mock WASM Project module
    global.writemagic_wasm.Project.new.mockReturnValue({
      add_document: jest.fn(),
      get_documents: jest.fn().mockReturnValue([]),
      get_metadata: jest.fn().mockReturnValue('{}'),
      free: jest.fn()
    });

    workspace = new ProjectWorkspace();
  });

  describe('initialization', () => {
    test('should initialize with empty project list', () => {
      expect(workspace.getProjects()).toEqual([]);
    });

    test('should initialize with default layout', () => {
      expect(workspace.currentLayout).toBe('single');
      expect(workspace.getPanes()).toEqual([]);
    });
  });

  describe('project creation', () => {
    test('should create new project with basic properties', async () => {
      const project = await workspace.createProject('Test Project', 'Test description');
      
      expect(project).toHaveProperty('id');
      expect(project).toHaveProperty('name', 'Test Project');
      expect(project).toHaveProperty('description', 'Test description');
      expect(project).toHaveProperty('documents', []);
      expect(project).toHaveProperty('createdAt');
      expect(global.writemagic_wasm.Project.new).toHaveBeenCalled();
    });

    test('should assign unique IDs to projects', async () => {
      const project1 = await workspace.createProject('Project 1');
      const project2 = await workspace.createProject('Project 2');
      
      expect(project1.id).not.toBe(project2.id);
    });

    test('should add project to workspace list', async () => {
      const project = await workspace.createProject('Test Project');
      const projects = workspace.getProjects();
      
      expect(projects).toContain(project);
      expect(projects.length).toBe(1);
    });
  });

  describe('project management', () => {
    let testProject;

    beforeEach(async () => {
      testProject = await workspace.createProject('Test Project');
    });

    test('should retrieve project by ID', () => {
      const retrieved = workspace.getProject(testProject.id);
      expect(retrieved).toEqual(testProject);
    });

    test('should update project properties', async () => {
      const updates = {
        name: 'Updated Project',
        description: 'Updated description'
      };
      
      const updated = await workspace.updateProject(testProject.id, updates);
      
      expect(updated.name).toBe(updates.name);
      expect(updated.description).toBe(updates.description);
      expect(updated.updatedAt).not.toBe(testProject.updatedAt);
    });

    test('should delete project', async () => {
      const result = await workspace.deleteProject(testProject.id);
      
      expect(result).toBe(true);
      expect(workspace.getProject(testProject.id)).toBeUndefined();
      expect(workspace.getProjects().length).toBe(0);
    });
  });

  describe('document management within projects', () => {
    let testProject;
    const mockDocument = {
      id: 'doc-1',
      title: 'Test Document',
      content: 'Test content'
    };

    beforeEach(async () => {
      testProject = await workspace.createProject('Test Project');
    });

    test('should add document to project', async () => {
      const result = await workspace.addDocumentToProject(testProject.id, mockDocument);
      
      expect(result).toBe(true);
      expect(testProject.documents).toContain(mockDocument);
      expect(testProject.wasmInstance.add_document).toHaveBeenCalledWith(mockDocument);
    });

    test('should remove document from project', async () => {
      await workspace.addDocumentToProject(testProject.id, mockDocument);
      const result = await workspace.removeDocumentFromProject(testProject.id, mockDocument.id);
      
      expect(result).toBe(true);
      expect(testProject.documents).not.toContain(mockDocument);
    });

    test('should get all documents in project', () => {
      testProject.documents = [mockDocument];
      const documents = workspace.getProjectDocuments(testProject.id);
      
      expect(documents).toEqual([mockDocument]);
    });

    test('should handle non-existent project for document operations', async () => {
      const result = await workspace.addDocumentToProject('non-existent', mockDocument);
      expect(result).toBe(false);
    });
  });

  describe('multi-pane layout management', () => {
    test('should switch to split layout', () => {
      workspace.setLayout('split');
      
      expect(workspace.currentLayout).toBe('split');
      expect(workspace.getPanes().length).toBe(2);
    });

    test('should switch to triple layout', () => {
      workspace.setLayout('triple');
      
      expect(workspace.currentLayout).toBe('triple');
      expect(workspace.getPanes().length).toBe(3);
    });

    test('should switch to quad layout', () => {
      workspace.setLayout('quad');
      
      expect(workspace.currentLayout).toBe('quad');
      expect(workspace.getPanes().length).toBe(4);
    });

    test('should handle invalid layout gracefully', () => {
      workspace.setLayout('invalid');
      
      expect(workspace.currentLayout).toBe('single');
      expect(workspace.getPanes().length).toBe(1);
    });
  });

  describe('pane content management', () => {
    const mockDocument = {
      id: 'doc-1',
      title: 'Test Document',
      content: 'Test content'
    };

    beforeEach(() => {
      workspace.setLayout('split');
    });

    test('should set document in pane', () => {
      const result = workspace.setPaneContent(0, mockDocument);
      
      expect(result).toBe(true);
      expect(workspace.getPaneContent(0)).toEqual(mockDocument);
    });

    test('should clear pane content', () => {
      workspace.setPaneContent(0, mockDocument);
      const result = workspace.clearPane(0);
      
      expect(result).toBe(true);
      expect(workspace.getPaneContent(0)).toBeNull();
    });

    test('should handle invalid pane index', () => {
      const result = workspace.setPaneContent(99, mockDocument);
      expect(result).toBe(false);
    });

    test('should get active pane', () => {
      workspace.setPaneContent(1, mockDocument);
      workspace.setActivePane(1);
      
      expect(workspace.getActivePane()).toBe(1);
      expect(workspace.getActivePaneContent()).toEqual(mockDocument);
    });
  });

  describe('workspace state management', () => {
    test('should save workspace state', async () => {
      const project = await workspace.createProject('Test Project');
      workspace.setLayout('split');
      
      const state = await workspace.saveState();
      
      expect(state).toHaveProperty('projects');
      expect(state).toHaveProperty('currentLayout', 'split');
      expect(state).toHaveProperty('panes');
      expect(state).toHaveProperty('savedAt');
    });

    test('should restore workspace state', async () => {
      const savedState = {
        projects: [testUtils.createMockProject()],
        currentLayout: 'triple',
        panes: [null, null, { id: 'doc-1', title: 'Test' }],
        savedAt: new Date().toISOString()
      };
      
      const result = await workspace.restoreState(savedState);
      
      expect(result).toBe(true);
      expect(workspace.currentLayout).toBe('triple');
      expect(workspace.getProjects().length).toBe(1);
      expect(workspace.getPanes().length).toBe(3);
    });

    test('should handle invalid state restoration', async () => {
      const result = await workspace.restoreState(null);
      expect(result).toBe(false);
    });
  });

  describe('search and filtering', () => {
    beforeEach(async () => {
      const project1 = await workspace.createProject('Web Project', 'Frontend development');
      const project2 = await workspace.createProject('Mobile App', 'React Native app');
      const project3 = await workspace.createProject('API Service', 'Backend API development');
    });

    test('should search projects by name', () => {
      const results = workspace.searchProjects('Web');
      
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('Web Project');
    });

    test('should search projects by description', () => {
      const results = workspace.searchProjects('development');
      
      expect(results).toHaveLength(2);
    });

    test('should filter projects by criteria', () => {
      const results = workspace.filterProjects(project => 
        project.name.toLowerCase().includes('app')
      );
      
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('Mobile App');
    });
  });

  describe('event handling', () => {
    test('should emit project created event', async () => {
      const eventHandler = jest.fn();
      workspace.on('projectCreated', eventHandler);
      
      const project = await workspace.createProject('Test Project');
      
      expect(eventHandler).toHaveBeenCalledWith(project);
    });

    test('should emit layout changed event', () => {
      const eventHandler = jest.fn();
      workspace.on('layoutChanged', eventHandler);
      
      workspace.setLayout('split');
      
      expect(eventHandler).toHaveBeenCalledWith('split', 2);
    });

    test('should emit pane content changed event', () => {
      workspace.setLayout('split');
      const eventHandler = jest.fn();
      workspace.on('paneContentChanged', eventHandler);
      
      const mockDocument = testUtils.createMockDocument();
      workspace.setPaneContent(0, mockDocument);
      
      expect(eventHandler).toHaveBeenCalledWith(0, mockDocument);
    });
  });

  describe('collaboration features', () => {
    test('should track project collaborators', async () => {
      const project = await workspace.createProject('Collaborative Project');
      const collaborator = { id: 'user-1', name: 'Alice', role: 'editor' };
      
      const result = await workspace.addCollaborator(project.id, collaborator);
      
      expect(result).toBe(true);
      expect(project.collaborators).toContain(collaborator);
    });

    test('should remove collaborators', async () => {
      const project = await workspace.createProject('Collaborative Project');
      const collaborator = { id: 'user-1', name: 'Alice', role: 'editor' };
      
      await workspace.addCollaborator(project.id, collaborator);
      const result = await workspace.removeCollaborator(project.id, 'user-1');
      
      expect(result).toBe(true);
      expect(project.collaborators).not.toContain(collaborator);
    });

    test('should update collaborator permissions', async () => {
      const project = await workspace.createProject('Collaborative Project');
      const collaborator = { id: 'user-1', name: 'Alice', role: 'editor' };
      
      await workspace.addCollaborator(project.id, collaborator);
      const result = await workspace.updateCollaboratorRole(project.id, 'user-1', 'admin');
      
      expect(result).toBe(true);
      const updatedCollaborator = project.collaborators.find(c => c.id === 'user-1');
      expect(updatedCollaborator.role).toBe('admin');
    });
  });

  describe('performance optimization', () => {
    test('should implement lazy loading for large projects', async () => {
      const project = await workspace.createProject('Large Project');
      
      // Mock large document list
      const mockDocuments = Array.from({ length: 1000 }, (_, i) => ({
        id: `doc-${i}`,
        title: `Document ${i}`,
        content: `Content for document ${i}`
      }));
      
      project.documents = mockDocuments;
      
      const lazyDocuments = workspace.getLazyProjectDocuments(project.id, 0, 10);
      
      expect(lazyDocuments).toHaveLength(10);
      expect(lazyDocuments[0].title).toBe('Document 0');
      expect(lazyDocuments[9].title).toBe('Document 9');
    });

    test('should implement virtual scrolling for document lists', () => {
      const virtualList = workspace.createVirtualList(1000, 50); // 1000 items, 50px height each
      
      expect(virtualList.totalHeight).toBe(50000);
      expect(virtualList.getVisibleRange(0, 500)).toEqual({ start: 0, end: 10 });
      expect(virtualList.getVisibleRange(1000, 500)).toEqual({ start: 20, end: 30 });
    });
  });

  describe('error handling', () => {
    test('should handle WASM project creation failure', async () => {
      global.writemagic_wasm.Project.new.mockImplementation(() => {
        throw new Error('WASM Project creation failed');
      });
      
      await expect(
        workspace.createProject('Test Project')
      ).rejects.toThrow('WASM Project creation failed');
    });

    test('should handle invalid project operations', async () => {
      await expect(
        workspace.updateProject('non-existent', { name: 'Updated' })
      ).rejects.toThrow('Project not found');
    });

    test('should handle pane operation errors gracefully', () => {
      const result = workspace.setPaneContent(-1, testUtils.createMockDocument());
      expect(result).toBe(false);
    });
  });
});