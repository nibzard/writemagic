/**
 * Document Manager - High-level document operations with auto-save, versioning
 * 
 * This module provides a writer-friendly interface for document management,
 * wrapping the WASM core with intuitive features like auto-save, draft handling,
 * and seamless document operations optimized for writing workflows.
 */

import { EventEmitter } from './utils/event-emitter.js';
import { debounce } from './utils/debounce.js';
import { ContentUtilities } from './content-utilities.js';

/**
 * Document save states for user feedback
 */
export const SaveState = {
    SAVED: 'saved',
    SAVING: 'saving', 
    DRAFT: 'draft',
    ERROR: 'error',
    OFFLINE: 'offline'
};

/**
 * Document events for UI updates
 */
export const DocumentEvents = {
    CONTENT_CHANGED: 'content_changed',
    SAVE_STATE_CHANGED: 'save_state_changed',
    AUTO_SAVED: 'auto_saved',
    DOCUMENT_LOADED: 'document_loaded',
    DOCUMENT_CREATED: 'document_created',
    DOCUMENT_UPDATED: 'document_updated',
    ERROR: 'error'
};

/**
 * Configuration options for DocumentManager
 */
export const DEFAULT_CONFIG = {
    autoSaveDelay: 2000,        // 2 seconds auto-save delay
    maxDraftHistory: 50,        // Maximum number of drafts to keep
    contentValidation: true,    // Enable content validation
    conflictResolution: 'last_writer_wins', // Conflict resolution strategy
    enableVersioning: true,     // Enable document versioning
    compressionThreshold: 1000  // Compress content above this character count
};

/**
 * DocumentManager - Provides writer-focused document management
 * 
 * Features:
 * - Auto-save with configurable delay
 * - Draft management and recovery
 * - Content validation and analysis
 * - Seamless offline/online transitions
 * - Version history tracking
 * - Conflict resolution for collaborative editing
 */
export class DocumentManager extends EventEmitter {
    constructor(wasmEngine, config = {}) {
        super();
        
        this.engine = wasmEngine;
        this.config = { ...DEFAULT_CONFIG, ...config };
        this.contentUtils = new ContentUtilities();
        
        // Internal state
        this.documents = new Map(); // Active documents cache
        this.drafts = new Map();    // Document drafts for recovery
        this.saveStates = new Map(); // Save state per document
        this.timers = new Map();    // Auto-save timers
        
        // User session tracking
        this.userId = config.userId || 'anonymous';
        this.isOnline = navigator.onLine;
        
        // Auto-save function with debouncing
        this.debouncedAutoSave = debounce(
            (documentId) => this.performAutoSave(documentId),
            this.config.autoSaveDelay
        );
        
        // Set up offline/online handling
        this.setupOfflineHandling();
        
        // Initialize content validation
        if (this.config.contentValidation) {
            this.setupContentValidation();
        }
    }

    /**
     * Create a new document with writer-friendly defaults
     */
    async createDocument(options = {}) {
        try {
            const {
                title = "Untitled Document",
                content = "",
                contentType = "markdown",
                projectId = null,
                template = null
            } = options;

            // Apply template if provided
            let initialContent = content;
            if (template) {
                initialContent = this.contentUtils.applyTemplate(template, content);
            }

            // Create document through WASM engine
            const wasmDoc = await this.engine.create_document(
                title,
                initialContent,
                contentType,
                this.userId
            );

            // Enhance with writer-specific metadata
            const document = this.enhanceDocument(wasmDoc, {
                projectId,
                template,
                createdInSession: true
            });

            // Cache locally
            this.documents.set(document.id, document);
            this.setSaveState(document.id, SaveState.SAVED);

            // Add to project if specified
            if (projectId) {
                await this.addDocumentToProject(projectId, document.id);
            }

            this.emit(DocumentEvents.DOCUMENT_CREATED, { document });
            return document;

        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'create' });
            throw new Error(`Failed to create document: ${error.message}`);
        }
    }

    /**
     * Load document with enhanced metadata and draft recovery
     */
    async loadDocument(documentId, options = {}) {
        try {
            const { forceFresh = false } = options;

            // Check cache first unless forced refresh
            if (!forceFresh && this.documents.has(documentId)) {
                const cachedDoc = this.documents.get(documentId);
                this.emit(DocumentEvents.DOCUMENT_LOADED, { document: cachedDoc });
                return cachedDoc;
            }

            // Load from WASM engine
            const wasmDoc = await this.engine.get_document(documentId);
            const document = this.enhanceDocument(wasmDoc);

            // Check for unsaved drafts
            await this.checkAndRecoverDraft(document);

            // Cache locally
            this.documents.set(document.id, document);
            this.setSaveState(document.id, SaveState.SAVED);

            this.emit(DocumentEvents.DOCUMENT_LOADED, { document });
            return document;

        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'load' });
            throw new Error(`Failed to load document: ${error.message}`);
        }
    }

    /**
     * Update document content with auto-save and validation
     */
    updateContent(documentId, content, options = {}) {
        try {
            const {
                validateContent = this.config.contentValidation,
                triggerAutoSave = true,
                source = 'user'
            } = options;

            // Get cached document
            const document = this.documents.get(documentId);
            if (!document) {
                throw new Error('Document not loaded');
            }

            // Content validation
            if (validateContent) {
                const validation = this.contentUtils.validateContent(content);
                if (!validation.isValid) {
                    this.emit(DocumentEvents.ERROR, {
                        error: new Error(`Content validation failed: ${validation.errors.join(', ')}`),
                        operation: 'validate'
                    });
                    return false;
                }
            }

            // Update local document
            const previousContent = document.content;
            document.content = content;
            document.updatedAt = new Date().toISOString();
            document.wordCount = this.contentUtils.countWords(content);
            document.characterCount = content.length;

            // Analyze content changes
            const changeAnalysis = this.contentUtils.analyzeChanges(previousContent, content);
            
            // Update cache
            this.documents.set(documentId, document);

            // Save draft for recovery
            this.saveDraft(documentId, content);

            // Set to draft state initially
            this.setSaveState(documentId, SaveState.DRAFT);

            // Emit content changed event
            this.emit(DocumentEvents.CONTENT_CHANGED, {
                document,
                content,
                changeAnalysis,
                source
            });

            // Trigger auto-save if enabled
            if (triggerAutoSave && this.isOnline) {
                this.debouncedAutoSave(documentId);
            }

            return true;

        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'update' });
            return false;
        }
    }

    /**
     * Manually save document (immediate save)
     */
    async saveDocument(documentId, options = {}) {
        try {
            const { 
                skipValidation = false,
                createVersion = this.config.enableVersioning 
            } = options;

            const document = this.documents.get(documentId);
            if (!document) {
                throw new Error('Document not loaded');
            }

            this.setSaveState(documentId, SaveState.SAVING);

            // Content validation before save
            if (!skipValidation && this.config.contentValidation) {
                const validation = this.contentUtils.validateContent(document.content);
                if (!validation.isValid) {
                    this.setSaveState(documentId, SaveState.ERROR);
                    throw new Error(`Cannot save: ${validation.errors.join(', ')}`);
                }
            }

            // Save through WASM engine
            const updatedWasmDoc = await this.engine.update_document(
                documentId,
                document.content,
                this.userId
            );

            // Update local cache with server response
            const updatedDoc = this.enhanceDocument(updatedWasmDoc, document.metadata);
            this.documents.set(documentId, updatedDoc);

            // Clear draft after successful save
            this.clearDraft(documentId);

            // Create version if enabled
            if (createVersion) {
                await this.createDocumentVersion(documentId);
            }

            this.setSaveState(documentId, SaveState.SAVED);
            this.emit(DocumentEvents.DOCUMENT_UPDATED, { document: updatedDoc });

            return updatedDoc;

        } catch (error) {
            this.setSaveState(documentId, SaveState.ERROR);
            this.emit(DocumentEvents.ERROR, { error, operation: 'save' });
            throw error;
        }
    }

    /**
     * Auto-save implementation with error handling
     */
    async performAutoSave(documentId) {
        try {
            if (!this.isOnline) {
                this.setSaveState(documentId, SaveState.OFFLINE);
                return;
            }

            const document = this.documents.get(documentId);
            if (!document) return;

            // Only auto-save if document has been modified
            const saveState = this.saveStates.get(documentId);
            if (saveState !== SaveState.DRAFT) return;

            await this.saveDocument(documentId, { skipValidation: false });
            this.emit(DocumentEvents.AUTO_SAVED, { documentId, document });

        } catch (error) {
            // Auto-save failures are non-blocking
            console.warn(`Auto-save failed for document ${documentId}:`, error);
            this.setSaveState(documentId, SaveState.ERROR);
        }
    }

    /**
     * Get document with writer analytics
     */
    getDocumentWithAnalytics(documentId) {
        const document = this.documents.get(documentId);
        if (!document) return null;

        return {
            ...document,
            analytics: {
                wordCount: document.wordCount,
                characterCount: document.characterCount,
                readingTime: this.contentUtils.estimateReadingTime(document.content),
                complexity: this.contentUtils.analyzeComplexity(document.content),
                lastModified: document.updatedAt,
                saveState: this.saveStates.get(documentId) || SaveState.SAVED
            }
        };
    }

    /**
     * List documents with optional filtering and sorting
     */
    async listDocuments(options = {}) {
        try {
            const {
                projectId = null,
                sortBy = 'updatedAt',
                sortOrder = 'desc',
                includeDeleted = false,
                searchQuery = null
            } = options;

            // Get documents from project or all documents
            let documents = [];
            
            if (projectId) {
                const wasmDocs = await this.engine.list_project_documents(projectId);
                documents = wasmDocs.map(doc => this.enhanceDocument(doc));
            } else {
                // For now, return cached documents (future: implement list_all_documents in WASM)
                documents = Array.from(this.documents.values());
            }

            // Filter out deleted documents if not requested
            if (!includeDeleted) {
                documents = documents.filter(doc => !doc.isDeleted);
            }

            // Search filter
            if (searchQuery) {
                documents = this.contentUtils.searchDocuments(documents, searchQuery);
            }

            // Sort documents
            documents.sort((a, b) => {
                const aVal = a[sortBy];
                const bVal = b[sortBy];
                
                if (sortOrder === 'desc') {
                    return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
                } else {
                    return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
                }
            });

            return documents;

        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'list' });
            throw error;
        }
    }

    /**
     * Duplicate document with new title
     */
    async duplicateDocument(documentId, newTitle = null) {
        try {
            const sourceDoc = await this.loadDocument(documentId);
            const title = newTitle || `Copy of ${sourceDoc.title}`;

            return await this.createDocument({
                title,
                content: sourceDoc.content,
                contentType: sourceDoc.contentType,
                projectId: sourceDoc.metadata?.projectId
            });

        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'duplicate' });
            throw error;
        }
    }

    /**
     * Add document to project
     */
    async addDocumentToProject(projectId, documentId) {
        try {
            await this.engine.add_document_to_project(projectId, documentId, this.userId);
            
            // Update local cache
            const document = this.documents.get(documentId);
            if (document) {
                document.metadata = { ...document.metadata, projectId };
                this.documents.set(documentId, document);
            }

            return true;
        } catch (error) {
            this.emit(DocumentEvents.ERROR, { error, operation: 'add_to_project' });
            throw error;
        }
    }

    /**
     * Enhanced document with writer-specific metadata
     */
    enhanceDocument(wasmDoc, additionalMetadata = {}) {
        return {
            id: wasmDoc.id,
            title: wasmDoc.title,
            content: wasmDoc.content,
            contentType: wasmDoc.content_type,
            wordCount: wasmDoc.word_count,
            characterCount: wasmDoc.character_count,
            createdAt: wasmDoc.created_at,
            updatedAt: wasmDoc.updated_at,
            createdBy: wasmDoc.created_by,
            isDeleted: wasmDoc.is_deleted,
            metadata: {
                ...additionalMetadata,
                projectId: wasmDoc.project_id
            }
        };
    }

    /**
     * Set save state with UI updates
     */
    setSaveState(documentId, state) {
        this.saveStates.set(documentId, state);
        this.emit(DocumentEvents.SAVE_STATE_CHANGED, { documentId, state });
    }

    /**
     * Save draft for recovery
     */
    saveDraft(documentId, content) {
        const drafts = this.drafts.get(documentId) || [];
        const draft = {
            content,
            timestamp: Date.now(),
            userId: this.userId
        };
        
        drafts.unshift(draft);
        
        // Limit draft history
        if (drafts.length > this.config.maxDraftHistory) {
            drafts.splice(this.config.maxDraftHistory);
        }
        
        this.drafts.set(documentId, drafts);
        
        // Save to localStorage for persistence
        try {
            localStorage.setItem(`writemagic_draft_${documentId}`, JSON.stringify(drafts));
        } catch (error) {
            console.warn('Failed to save draft to localStorage:', error);
        }
    }

    /**
     * Clear draft after successful save
     */
    clearDraft(documentId) {
        this.drafts.delete(documentId);
        try {
            localStorage.removeItem(`writemagic_draft_${documentId}`);
        } catch (error) {
            console.warn('Failed to clear draft from localStorage:', error);
        }
    }

    /**
     * Check and recover draft on document load
     */
    async checkAndRecoverDraft(document) {
        try {
            const stored = localStorage.getItem(`writemagic_draft_${document.id}`);
            if (!stored) return;

            const drafts = JSON.parse(stored);
            if (!drafts.length) return;

            const latestDraft = drafts[0];
            const draftAge = Date.now() - latestDraft.timestamp;
            
            // Only offer recovery for recent drafts (within 24 hours)
            if (draftAge < 24 * 60 * 60 * 1000) {
                this.emit('draft_recovery_available', {
                    document,
                    draft: latestDraft,
                    age: draftAge
                });
            }

        } catch (error) {
            console.warn('Failed to check draft recovery:', error);
        }
    }

    /**
     * Create document version for history
     */
    async createDocumentVersion(documentId) {
        // This would integrate with the version control domain
        // For now, this is a placeholder for future implementation
        console.log(`Creating version for document ${documentId}`);
    }

    /**
     * Set up offline/online handling
     */
    setupOfflineHandling() {
        window.addEventListener('online', () => {
            this.isOnline = true;
            this.handleOnlineResume();
        });

        window.addEventListener('offline', () => {
            this.isOnline = false;
            this.handleOfflineMode();
        });
    }

    /**
     * Handle coming back online
     */
    async handleOnlineResume() {
        console.log('Connection restored, syncing pending changes...');
        
        // Attempt to save any documents in draft state
        for (const [documentId, state] of this.saveStates.entries()) {
            if (state === SaveState.DRAFT || state === SaveState.OFFLINE) {
                try {
                    await this.performAutoSave(documentId);
                } catch (error) {
                    console.warn(`Failed to sync document ${documentId}:`, error);
                }
            }
        }
    }

    /**
     * Handle offline mode
     */
    handleOfflineMode() {
        console.log('Connection lost, enabling offline mode...');
        
        // Update all draft states to offline
        for (const [documentId, state] of this.saveStates.entries()) {
            if (state === SaveState.DRAFT) {
                this.setSaveState(documentId, SaveState.OFFLINE);
            }
        }
    }

    /**
     * Set up content validation rules
     */
    setupContentValidation() {
        // This can be extended with custom validation rules
        // For now, basic validation is handled in ContentUtilities
    }

    /**
     * Cleanup resources
     */
    destroy() {
        // Clear all auto-save timers
        for (const timer of this.timers.values()) {
            clearTimeout(timer);
        }
        this.timers.clear();

        // Clear caches
        this.documents.clear();
        this.drafts.clear();
        this.saveStates.clear();

        // Remove event listeners
        this.removeAllListeners();
    }
}

export default DocumentManager;