/**
 * Project Workspace - Multi-pane editing interface coordination
 * 
 * This module manages the multi-pane writing environment that allows writers
 * to organize their work across multiple documents, compare versions, and
 * maintain context while working on complex writing projects.
 */

import { EventEmitter } from './utils/event-emitter.js';
import { ContentUtilities } from './content-utilities.js';

/**
 * Pane types for different editing workflows
 */
export const PaneType = {
    EDITOR: 'editor',           // Main editing pane
    REFERENCE: 'reference',     // Reference document pane
    OUTLINE: 'outline',         // Document outline/structure
    NOTES: 'notes',            // Research notes and ideas
    PREVIEW: 'preview',        // Live preview of formatted content
    TIMELINE: 'timeline',      // Version history timeline
    AI_ASSISTANT: 'ai_assistant' // AI suggestion pane
};

/**
 * Layout configurations for different writing workflows
 */
export const LayoutPresets = {
    FOCUS: {
        name: 'Focus Mode',
        panes: [{ type: PaneType.EDITOR, size: 100 }]
    },
    SPLIT: {
        name: 'Split View',
        panes: [
            { type: PaneType.EDITOR, size: 50 },
            { type: PaneType.REFERENCE, size: 50 }
        ]
    },
    RESEARCH: {
        name: 'Research Mode',
        panes: [
            { type: PaneType.EDITOR, size: 40 },
            { type: PaneType.NOTES, size: 30 },
            { type: PaneType.REFERENCE, size: 30 }
        ]
    },
    REVIEW: {
        name: 'Review Mode',
        panes: [
            { type: PaneType.EDITOR, size: 50 },
            { type: PaneType.PREVIEW, size: 30 },
            { type: PaneType.TIMELINE, size: 20 }
        ]
    },
    AI_ENHANCED: {
        name: 'AI-Enhanced Writing',
        panes: [
            { type: PaneType.EDITOR, size: 50 },
            { type: PaneType.AI_ASSISTANT, size: 25 },
            { type: PaneType.OUTLINE, size: 25 }
        ]
    }
};

/**
 * Workspace events for UI coordination
 */
export const WorkspaceEvents = {
    LAYOUT_CHANGED: 'layout_changed',
    PANE_ADDED: 'pane_added',
    PANE_REMOVED: 'pane_removed',
    PANE_RESIZED: 'pane_resized',
    PANE_FOCUSED: 'pane_focused',
    DOCUMENT_CHANGED: 'document_changed',
    PROJECT_LOADED: 'project_loaded',
    WORKSPACE_SAVED: 'workspace_saved',
    SYNC_STATUS_CHANGED: 'sync_status_changed',
    ERROR: 'error'
};

/**
 * Default workspace configuration
 */
export const DEFAULT_WORKSPACE_CONFIG = {
    maxPanes: 5,                    // Maximum number of panes
    minPaneSize: 15,               // Minimum pane size percentage
    autoSaveLayout: true,          // Auto-save layout preferences
    enablePaneSync: true,          // Sync related panes (outline, etc.)
    rememberDocumentPositions: true, // Remember cursor positions
    enableKeyboardNavigation: true,  // Enable keyboard pane navigation
    defaultLayout: LayoutPresets.FOCUS,
    enableCollaboration: false      // Enable real-time collaboration
};

/**
 * ProjectWorkspace - Manages multi-pane writing environment
 * 
 * Features:
 * - Flexible multi-pane layouts with drag-and-drop resizing
 * - Project-aware document management
 * - Cross-pane content synchronization
 * - Layout persistence and restoration
 * - Keyboard-driven navigation
 * - Context-aware pane suggestions
 * - Real-time collaboration support (when enabled)
 */
export class ProjectWorkspace extends EventEmitter {
    constructor(wasmEngine, documentManager, config = {}) {
        super();
        
        this.engine = wasmEngine;
        this.documentManager = documentManager;
        this.config = { ...DEFAULT_WORKSPACE_CONFIG, ...config };
        this.contentUtils = new ContentUtilities();
        
        // Workspace state
        this.currentProject = null;
        this.panes = new Map();           // Active panes by ID
        this.paneOrder = [];              // Ordered list of pane IDs
        this.focusedPaneId = null;        // Currently focused pane
        this.layout = null;               // Current layout configuration
        
        // Session management
        this.sessionId = this.generateSessionId();
        this.isLoading = false;
        this.isDirty = false;
        
        // Pane synchronization
        this.syncEnabled = this.config.enablePaneSync;
        this.syncConnections = new Map(); // Pane sync relationships
        
        // Keyboard navigation
        if (this.config.enableKeyboardNavigation) {
            this.setupKeyboardNavigation();
        }
        
        // Auto-save timer
        this.autoSaveTimer = null;
        
        // Initialize with default layout
        this.setLayout(this.config.defaultLayout);
    }

    /**
     * Load project and set up workspace
     */
    async loadProject(projectId, options = {}) {
        try {
            this.isLoading = true;
            const { restoreLayout = true } = options;

            // Load project from WASM engine
            const wasmProject = await this.engine.get_project(projectId);
            this.currentProject = {
                id: wasmProject.id,
                name: wasmProject.name,
                description: wasmProject.description,
                documentIds: wasmProject.document_ids,
                createdAt: wasmProject.created_at,
                updatedAt: wasmProject.updated_at
            };

            // Load project documents
            const documents = await this.documentManager.listDocuments({ projectId });
            
            // Restore saved layout if available and requested
            if (restoreLayout) {
                await this.restoreProjectLayout(projectId);
            }

            // Open primary document in main editor pane if available
            if (documents.length > 0) {
                const primaryDoc = documents.find(d => d.metadata?.isPrimary) || documents[0];
                await this.openDocumentInPane(this.getMainEditorPaneId(), primaryDoc.id);
            }

            this.isLoading = false;
            this.emit(WorkspaceEvents.PROJECT_LOADED, { 
                project: this.currentProject, 
                documents 
            });

            return this.currentProject;

        } catch (error) {
            this.isLoading = false;
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'load_project' });
            throw error;
        }
    }

    /**
     * Set workspace layout
     */
    setLayout(layoutConfig, options = {}) {
        try {
            const { animate = true, preserveDocuments = true } = options;
            
            // Validate layout configuration
            if (!this.isValidLayout(layoutConfig)) {
                throw new Error('Invalid layout configuration');
            }

            // Store current documents if preserving
            const currentDocuments = preserveDocuments ? this.getCurrentDocuments() : {};

            // Clear existing panes
            this.clearPanes();

            // Create new panes based on layout
            this.layout = { ...layoutConfig };
            let paneIndex = 0;

            for (const paneConfig of layoutConfig.panes) {
                const paneId = `pane_${paneIndex++}`;
                const pane = this.createPane(paneId, paneConfig);
                this.addPane(pane);

                // Restore document if available
                if (preserveDocuments && currentDocuments[pane.type]) {
                    this.openDocumentInPane(paneId, currentDocuments[pane.type], { silent: true });
                }
            }

            // Focus the main editor pane
            const mainPaneId = this.getMainEditorPaneId();
            if (mainPaneId) {
                this.focusPane(mainPaneId);
            }

            // Auto-save layout if enabled
            if (this.config.autoSaveLayout) {
                this.saveLayoutPreference();
            }

            this.emit(WorkspaceEvents.LAYOUT_CHANGED, { 
                layout: this.layout,
                animate 
            });

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'set_layout' });
            throw error;
        }
    }

    /**
     * Add new pane to workspace
     */
    addPane(paneConfig, options = {}) {
        try {
            const { position = 'end', size = 30 } = options;

            // Check pane limit
            if (this.panes.size >= this.config.maxPanes) {
                throw new Error(`Maximum number of panes (${this.config.maxPanes}) reached`);
            }

            // Create pane
            const paneId = paneConfig.id || `pane_${Date.now()}`;
            const pane = this.createPane(paneId, { ...paneConfig, size });
            
            // Add to workspace
            this.panes.set(paneId, pane);
            
            // Update pane order
            if (position === 'start') {
                this.paneOrder.unshift(paneId);
            } else if (position === 'end') {
                this.paneOrder.push(paneId);
            } else if (typeof position === 'number') {
                this.paneOrder.splice(position, 0, paneId);
            }

            // Recalculate sizes to accommodate new pane
            this.recalculatePaneSizes();

            this.isDirty = true;
            this.emit(WorkspaceEvents.PANE_ADDED, { pane, position });

            return pane;

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'add_pane' });
            throw error;
        }
    }

    /**
     * Remove pane from workspace
     */
    removePane(paneId) {
        try {
            if (!this.panes.has(paneId)) {
                throw new Error('Pane not found');
            }

            // Can't remove the last pane
            if (this.panes.size === 1) {
                throw new Error('Cannot remove the last pane');
            }

            const pane = this.panes.get(paneId);
            
            // Close document if open
            if (pane.documentId) {
                this.closeDocumentInPane(paneId);
            }

            // Remove from workspace
            this.panes.delete(paneId);
            this.paneOrder = this.paneOrder.filter(id => id !== paneId);

            // Remove sync connections
            this.syncConnections.delete(paneId);
            for (const [key, connections] of this.syncConnections.entries()) {
                this.syncConnections.set(key, connections.filter(id => id !== paneId));
            }

            // Recalculate sizes
            this.recalculatePaneSizes();

            // Focus another pane if this was focused
            if (this.focusedPaneId === paneId) {
                const nextPane = this.paneOrder[0];
                if (nextPane) {
                    this.focusPane(nextPane);
                }
            }

            this.isDirty = true;
            this.emit(WorkspaceEvents.PANE_REMOVED, { paneId, pane });

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'remove_pane' });
            throw error;
        }
    }

    /**
     * Open document in specific pane
     */
    async openDocumentInPane(paneId, documentId, options = {}) {
        try {
            const { silent = false, preservePosition = true } = options;

            const pane = this.panes.get(paneId);
            if (!pane) {
                throw new Error('Pane not found');
            }

            // Load document if not already cached
            const document = await this.documentManager.loadDocument(documentId);

            // Store previous document position if preserving
            if (preservePosition && pane.documentId) {
                pane.savedPositions = pane.savedPositions || {};
                pane.savedPositions[pane.documentId] = {
                    scrollTop: pane.scrollTop || 0,
                    cursorPosition: pane.cursorPosition || 0
                };
            }

            // Update pane with new document
            pane.documentId = documentId;
            pane.document = document;
            pane.lastModified = document.updatedAt;

            // Restore position if available
            if (preservePosition && pane.savedPositions && pane.savedPositions[documentId]) {
                const saved = pane.savedPositions[documentId];
                pane.scrollTop = saved.scrollTop;
                pane.cursorPosition = saved.cursorPosition;
            }

            // Update synchronized panes
            if (this.syncEnabled) {
                this.updateSyncedPanes(paneId, document);
            }

            this.panes.set(paneId, pane);

            if (!silent) {
                this.emit(WorkspaceEvents.DOCUMENT_CHANGED, { 
                    paneId, 
                    document, 
                    previousDocumentId: pane.previousDocumentId 
                });
            }

            return document;

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'open_document' });
            throw error;
        }
    }

    /**
     * Close document in pane (keep pane open)
     */
    closeDocumentInPane(paneId) {
        const pane = this.panes.get(paneId);
        if (!pane || !pane.documentId) return;

        // Save current position
        pane.savedPositions = pane.savedPositions || {};
        pane.savedPositions[pane.documentId] = {
            scrollTop: pane.scrollTop || 0,
            cursorPosition: pane.cursorPosition || 0
        };

        // Clear document
        pane.previousDocumentId = pane.documentId;
        pane.documentId = null;
        pane.document = null;

        this.panes.set(paneId, pane);
        this.emit(WorkspaceEvents.DOCUMENT_CHANGED, { paneId, document: null });
    }

    /**
     * Focus specific pane
     */
    focusPane(paneId) {
        if (!this.panes.has(paneId)) {
            throw new Error('Pane not found');
        }

        const previouslyFocused = this.focusedPaneId;
        this.focusedPaneId = paneId;

        this.emit(WorkspaceEvents.PANE_FOCUSED, { 
            paneId, 
            previouslyFocused 
        });
    }

    /**
     * Resize pane
     */
    resizePane(paneId, newSize) {
        try {
            const pane = this.panes.get(paneId);
            if (!pane) {
                throw new Error('Pane not found');
            }

            // Validate size constraints
            if (newSize < this.config.minPaneSize) {
                newSize = this.config.minPaneSize;
            }

            const oldSize = pane.size;
            pane.size = newSize;
            this.panes.set(paneId, pane);

            // Redistribute remaining space among other panes
            this.redistributePaneSizes(paneId, oldSize, newSize);

            this.isDirty = true;
            this.emit(WorkspaceEvents.PANE_RESIZED, { 
                paneId, 
                oldSize, 
                newSize 
            });

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'resize_pane' });
            throw error;
        }
    }

    /**
     * Get pane information
     */
    getPane(paneId) {
        return this.panes.get(paneId);
    }

    /**
     * Get all panes
     */
    getAllPanes() {
        return Array.from(this.panes.values());
    }

    /**
     * Get current layout information
     */
    getCurrentLayout() {
        return {
            ...this.layout,
            panes: this.paneOrder.map(id => {
                const pane = this.panes.get(id);
                return {
                    id,
                    type: pane.type,
                    size: pane.size,
                    documentId: pane.documentId
                };
            })
        };
    }

    /**
     * Save workspace state
     */
    async saveWorkspace() {
        try {
            if (!this.currentProject) {
                throw new Error('No project loaded');
            }

            const workspaceState = {
                projectId: this.currentProject.id,
                layout: this.getCurrentLayout(),
                focusedPaneId: this.focusedPaneId,
                sessionId: this.sessionId,
                timestamp: Date.now()
            };

            // Save to local storage
            localStorage.setItem(
                `writemagic_workspace_${this.currentProject.id}`, 
                JSON.stringify(workspaceState)
            );

            // Save layout preference
            if (this.config.autoSaveLayout) {
                this.saveLayoutPreference();
            }

            this.isDirty = false;
            this.emit(WorkspaceEvents.WORKSPACE_SAVED, { workspaceState });

        } catch (error) {
            this.emit(WorkspaceEvents.ERROR, { error, operation: 'save_workspace' });
            throw error;
        }
    }

    /**
     * Create a new pane
     */
    createPane(paneId, config) {
        return {
            id: paneId,
            type: config.type,
            size: config.size || 25,
            documentId: null,
            document: null,
            isActive: false,
            scrollTop: 0,
            cursorPosition: 0,
            savedPositions: {},
            metadata: config.metadata || {},
            createdAt: Date.now()
        };
    }

    /**
     * Get main editor pane ID
     */
    getMainEditorPaneId() {
        for (const [paneId, pane] of this.panes.entries()) {
            if (pane.type === PaneType.EDITOR) {
                return paneId;
            }
        }
        return this.paneOrder[0] || null;
    }

    /**
     * Clear all panes
     */
    clearPanes() {
        this.panes.clear();
        this.paneOrder = [];
        this.focusedPaneId = null;
    }

    /**
     * Validate layout configuration
     */
    isValidLayout(layout) {
        if (!layout || !layout.panes || !Array.isArray(layout.panes)) {
            return false;
        }

        if (layout.panes.length === 0 || layout.panes.length > this.config.maxPanes) {
            return false;
        }

        // Check that total size doesn't exceed 100%
        const totalSize = layout.panes.reduce((sum, pane) => sum + (pane.size || 0), 0);
        return totalSize <= 100;
    }

    /**
     * Recalculate pane sizes to ensure they sum to 100%
     */
    recalculatePaneSizes() {
        if (this.panes.size === 0) return;

        const panes = Array.from(this.panes.values());
        const totalSize = panes.reduce((sum, pane) => sum + pane.size, 0);

        if (totalSize !== 100) {
            const adjustment = 100 / totalSize;
            for (const pane of panes) {
                pane.size = Math.max(this.config.minPaneSize, pane.size * adjustment);
                this.panes.set(pane.id, pane);
            }
        }
    }

    /**
     * Redistribute pane sizes when one pane is resized
     */
    redistributePaneSizes(resizedPaneId, oldSize, newSize) {
        const sizeDiff = newSize - oldSize;
        const otherPanes = this.paneOrder.filter(id => id !== resizedPaneId);
        
        if (otherPanes.length === 0) return;

        const redistributeAmount = sizeDiff / otherPanes.length;
        
        for (const paneId of otherPanes) {
            const pane = this.panes.get(paneId);
            pane.size = Math.max(this.config.minPaneSize, pane.size - redistributeAmount);
            this.panes.set(paneId, pane);
        }
    }

    /**
     * Get currently open documents by pane type
     */
    getCurrentDocuments() {
        const documents = {};
        for (const pane of this.panes.values()) {
            if (pane.documentId) {
                documents[pane.type] = pane.documentId;
            }
        }
        return documents;
    }

    /**
     * Update synchronized panes when document changes
     */
    updateSyncedPanes(changedPaneId, document) {
        const connections = this.syncConnections.get(changedPaneId) || [];
        
        for (const paneId of connections) {
            const pane = this.panes.get(paneId);
            if (!pane) continue;

            // Update pane based on its type and the document change
            switch (pane.type) {
                case PaneType.OUTLINE:
                    this.updateOutlinePane(paneId, document);
                    break;
                case PaneType.PREVIEW:
                    this.updatePreviewPane(paneId, document);
                    break;
                // Add more sync types as needed
            }
        }
    }

    /**
     * Update outline pane with document structure
     */
    updateOutlinePane(paneId, document) {
        const pane = this.panes.get(paneId);
        if (!pane) return;

        // Extract outline from document content
        const outline = this.contentUtils.extractOutline(document.content);
        
        pane.metadata = {
            ...pane.metadata,
            outline,
            sourceDocumentId: document.id,
            lastUpdated: Date.now()
        };

        this.panes.set(paneId, pane);
    }

    /**
     * Update preview pane with rendered content
     */
    updatePreviewPane(paneId, document) {
        const pane = this.panes.get(paneId);
        if (!pane) return;

        // Render content based on content type
        const rendered = this.contentUtils.renderPreview(document.content, document.contentType);
        
        pane.metadata = {
            ...pane.metadata,
            renderedContent: rendered,
            sourceDocumentId: document.id,
            lastUpdated: Date.now()
        };

        this.panes.set(paneId, pane);
    }

    /**
     * Restore project layout from saved state
     */
    async restoreProjectLayout(projectId) {
        try {
            const saved = localStorage.getItem(`writemagic_workspace_${projectId}`);
            if (!saved) return;

            const workspaceState = JSON.parse(saved);
            
            // Check if saved state is recent (within last 7 days)
            const age = Date.now() - workspaceState.timestamp;
            if (age > 7 * 24 * 60 * 60 * 1000) return;

            // Restore layout
            if (workspaceState.layout) {
                this.setLayout(workspaceState.layout, { animate: false });
            }

            // Restore focused pane
            if (workspaceState.focusedPaneId && this.panes.has(workspaceState.focusedPaneId)) {
                this.focusPane(workspaceState.focusedPaneId);
            }

        } catch (error) {
            console.warn('Failed to restore workspace layout:', error);
        }
    }

    /**
     * Save layout preference for future sessions
     */
    saveLayoutPreference() {
        try {
            const layoutPrefs = {
                layout: this.getCurrentLayout(),
                timestamp: Date.now()
            };

            localStorage.setItem('writemagic_layout_preference', JSON.stringify(layoutPrefs));
        } catch (error) {
            console.warn('Failed to save layout preference:', error);
        }
    }

    /**
     * Setup keyboard navigation
     */
    setupKeyboardNavigation() {
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + Number keys to switch panes
            if ((e.ctrlKey || e.metaKey) && e.key >= '1' && e.key <= '9') {
                const paneIndex = parseInt(e.key) - 1;
                const paneId = this.paneOrder[paneIndex];
                if (paneId) {
                    e.preventDefault();
                    this.focusPane(paneId);
                }
            }
            
            // Ctrl/Cmd + Shift + Arrow keys to resize panes
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
                if (this.focusedPaneId) {
                    e.preventDefault();
                    const currentPane = this.panes.get(this.focusedPaneId);
                    const adjustment = e.key === 'ArrowRight' ? 5 : -5;
                    this.resizePane(this.focusedPaneId, currentPane.size + adjustment);
                }
            }
        });
    }

    /**
     * Generate unique session ID
     */
    generateSessionId() {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    /**
     * Cleanup resources
     */
    destroy() {
        if (this.autoSaveTimer) {
            clearTimeout(this.autoSaveTimer);
        }

        this.clearPanes();
        this.removeAllListeners();
    }
}

export default ProjectWorkspace;