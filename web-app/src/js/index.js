/**
 * WriteMagic JavaScript API - Main export with clean writer API
 * 
 * This is the main entry point for the WriteMagic JavaScript layer,
 * providing a clean, writer-focused API that abstracts the complexity
 * of the underlying WASM engine and provides a delightful developer experience.
 */

import init, { 
    WriteMagicEngine, 
    WasmCompletionRequest,
    init_logging,
    init_panic_hook,
    log,
    log_error
} from '../../../core/wasm/pkg/writemagic_wasm.js';

import DocumentManager from './document-manager.js';
import ProjectWorkspace from './project-workspace.js';
import WritingAnalytics from './writing-analytics.js';
import ContentUtilities from './content-utilities.js';
import WritingSession from './writing-session.js';

/**
 * WriteMagic initialization configuration
 */
export const DEFAULT_CONFIG = {
    // Engine configuration
    claude_api_key: null,
    openai_api_key: null,
    default_model: "claude-3-haiku-20240307",
    log_level: "info",
    enable_content_filtering: true,
    database_type: "indexeddb",
    
    // Writer experience settings
    auto_save_delay: 2000,
    enable_analytics: true,
    enable_focus_mode: true,
    enable_collaboration: false,
    
    // Session management
    session_timeout: 3600000,     // 1 hour
    idle_timeout: 300000,         // 5 minutes
    max_draft_history: 100,
    
    // UI preferences
    default_layout: 'focus',      // 'focus', 'split', 'research', 'review', 'ai_enhanced'
    enable_keyboard_navigation: true,
    enable_accessibility: true,
    
    // Performance
    compression_enabled: true,
    sync_across_devices: false
};

/**
 * WriteMagic API Events for application integration
 */
export const WriteMagicEvents = {
    // Initialization
    INITIALIZED: 'writemagic:initialized',
    INIT_ERROR: 'writemagic:init_error',
    
    // Documents
    DOCUMENT_CREATED: 'writemagic:document_created',
    DOCUMENT_LOADED: 'writemagic:document_loaded',
    DOCUMENT_UPDATED: 'writemagic:document_updated',
    DOCUMENT_AUTO_SAVED: 'writemagic:document_auto_saved',
    
    // Projects
    PROJECT_LOADED: 'writemagic:project_loaded',
    PROJECT_CREATED: 'writemagic:project_created',
    
    // Sessions
    SESSION_STARTED: 'writemagic:session_started',
    SESSION_ENDED: 'writemagic:session_ended',
    FOCUS_SESSION_STARTED: 'writemagic:focus_session_started',
    
    // Writing
    CONTENT_ANALYZED: 'writemagic:content_analyzed',
    GOAL_ACHIEVED: 'writemagic:goal_achieved',
    
    // AI
    AI_COMPLETION: 'writemagic:ai_completion',
    AI_ERROR: 'writemagic:ai_error',
    
    // Errors
    ERROR: 'writemagic:error'
};

/**
 * Main WriteMagic class - High-level API for writers
 * 
 * This class provides a clean, writer-focused interface that combines
 * all the underlying modules into a cohesive writing experience.
 */
export class WriteMagic {
    constructor(config = {}) {
        this.config = { ...DEFAULT_CONFIG, ...config };
        this.isInitialized = false;
        
        // Core components (initialized after WASM loads)
        this.wasmEngine = null;
        this.documentManager = null;
        this.projectWorkspace = null;
        this.writingAnalytics = null;
        this.contentUtils = null;
        this.writingSession = null;
        
        // Event handling
        this.eventCallbacks = new Map();
        
        // Initialize immediately
        this.initialize();
    }

    /**
     * Initialize WriteMagic engine and all components
     */
    async initialize() {
        try {
            // Initialize WASM module
            await init();
            
            // Initialize logging and error handling
            init_logging();
            init_panic_hook();
            
            log("WriteMagic JavaScript API initializing...");
            
            // Create and initialize WASM engine
            this.wasmEngine = new WriteMagicEngine();
            await this.wasmEngine.initialize(JSON.stringify(this.config));
            
            // Initialize content utilities (no dependencies)
            this.contentUtils = new ContentUtilities({
                enableSpellCheck: this.config.enable_spell_check,
                enableGrammarCheck: this.config.enable_grammar_check,
                enableStyleSuggestions: true
            });
            
            // Initialize writing analytics
            this.writingAnalytics = new WritingAnalytics({
                enableComplexityAnalysis: this.config.enable_analytics,
                enablePatternAnalysis: this.config.enable_analytics,
                enableProductivityTracking: this.config.enable_analytics
            });
            
            // Initialize document manager
            this.documentManager = new DocumentManager(this.wasmEngine, {
                autoSaveDelay: this.config.auto_save_delay,
                maxDraftHistory: this.config.max_draft_history,
                contentValidation: true,
                enableVersioning: true,
                userId: this.config.user_id || 'anonymous'
            });
            
            // Initialize project workspace
            this.projectWorkspace = new ProjectWorkspace(this.wasmEngine, this.documentManager, {
                enableFocusMode: this.config.enable_focus_mode,
                enableKeyboardNavigation: this.config.enable_keyboard_navigation,
                defaultLayout: this.getLayoutPreset(this.config.default_layout),
                enableCollaboration: this.config.enable_collaboration
            });
            
            // Initialize writing session
            this.writingSession = new WritingSession(this.documentManager, this.projectWorkspace, {
                autoSaveInterval: this.config.auto_save_delay,
                idleTimeout: this.config.idle_timeout,
                sessionTimeout: this.config.session_timeout,
                maxDraftHistory: this.config.max_draft_history,
                enableFocusMode: this.config.enable_focus_mode,
                enableAnalytics: this.config.enable_analytics
            });
            
            // Set up event forwarding
            this.setupEventForwarding();
            
            this.isInitialized = true;
            
            log("WriteMagic JavaScript API initialized successfully");
            this.emit(WriteMagicEvents.INITIALIZED, { config: this.config });
            
            return this;
            
        } catch (error) {
            log_error(`WriteMagic initialization failed: ${error.message}`);
            this.emit(WriteMagicEvents.INIT_ERROR, { error });
            throw new Error(`WriteMagic initialization failed: ${error.message}`);
        }
    }

    // Document Management API

    /**
     * Create a new document with writer-friendly options
     */
    async createDocument(options = {}) {
        this.ensureInitialized();
        return await this.documentManager.createDocument(options);
    }

    /**
     * Load document by ID
     */
    async loadDocument(documentId, options = {}) {
        this.ensureInitialized();
        return await this.documentManager.loadDocument(documentId, options);
    }

    /**
     * Update document content
     */
    updateDocumentContent(documentId, content, options = {}) {
        this.ensureInitialized();
        
        // Update through document manager
        const success = this.documentManager.updateContent(documentId, content, options);
        
        // Update session progress
        if (success) {
            const previousContent = options.previousContent || '';
            const changeData = this.writingAnalytics.analyzeChanges(previousContent, content);
            this.writingSession.updateProgress(documentId, {
                wordsAdded: Math.max(0, changeData.wordChange.net),
                charactersAdded: Math.max(0, changeData.characterChange.net)
            });
        }
        
        return success;
    }

    /**
     * Save document manually
     */
    async saveDocument(documentId, options = {}) {
        this.ensureInitialized();
        return await this.documentManager.saveDocument(documentId, options);
    }

    /**
     * Get document with comprehensive analytics
     */
    getDocumentAnalytics(documentId) {
        this.ensureInitialized();
        
        const document = this.documentManager.getDocumentWithAnalytics(documentId);
        if (!document) return null;
        
        // Enhance with writing analytics
        const contentAnalysis = this.writingAnalytics.analyzeDocument(document.content);
        
        return {
            ...document,
            analysis: contentAnalysis
        };
    }

    /**
     * List documents with filtering and sorting
     */
    async listDocuments(options = {}) {
        this.ensureInitialized();
        return await this.documentManager.listDocuments(options);
    }

    /**
     * Duplicate document
     */
    async duplicateDocument(documentId, newTitle = null) {
        this.ensureInitialized();
        return await this.documentManager.duplicateDocument(documentId, newTitle);
    }

    // Project Management API

    /**
     * Create a new project
     */
    async createProject(name, description = null) {
        this.ensureInitialized();
        
        const project = await this.wasmEngine.create_project(
            name, 
            description, 
            this.config.user_id || null
        );
        
        this.emit(WriteMagicEvents.PROJECT_CREATED, { project });
        return project;
    }

    /**
     * Load project and set up workspace
     */
    async loadProject(projectId, options = {}) {
        this.ensureInitialized();
        
        const project = await this.projectWorkspace.loadProject(projectId, options);
        this.emit(WriteMagicEvents.PROJECT_LOADED, { project });
        
        return project;
    }

    /**
     * Add document to project
     */
    async addDocumentToProject(projectId, documentId) {
        this.ensureInitialized();
        return await this.documentManager.addDocumentToProject(projectId, documentId);
    }

    // Workspace Management API

    /**
     * Set workspace layout
     */
    setLayout(layoutName, options = {}) {
        this.ensureInitialized();
        
        const layout = this.getLayoutPreset(layoutName);
        return this.projectWorkspace.setLayout(layout, options);
    }

    /**
     * Open document in specific pane
     */
    async openDocumentInPane(paneId, documentId, options = {}) {
        this.ensureInitialized();
        return await this.projectWorkspace.openDocumentInPane(paneId, documentId, options);
    }

    /**
     * Add new pane to workspace
     */
    addPane(paneConfig, options = {}) {
        this.ensureInitialized();
        return this.projectWorkspace.addPane(paneConfig, options);
    }

    /**
     * Remove pane from workspace
     */
    removePane(paneId) {
        this.ensureInitialized();
        return this.projectWorkspace.removePane(paneId);
    }

    /**
     * Focus specific pane
     */
    focusPane(paneId) {
        this.ensureInitialized();
        return this.projectWorkspace.focusPane(paneId);
    }

    // Writing Session API

    /**
     * Start a writing session
     */
    startWritingSession(options = {}) {
        this.ensureInitialized();
        
        const session = this.writingSession.startSession(options);
        this.emit(WriteMagicEvents.SESSION_STARTED, { session });
        
        return session;
    }

    /**
     * End current writing session
     */
    endWritingSession(options = {}) {
        this.ensureInitialized();
        
        const session = this.writingSession.endSession(options);
        if (session) {
            this.emit(WriteMagicEvents.SESSION_ENDED, { session });
        }
        
        return session;
    }

    /**
     * Start focus session (Pomodoro, etc.)
     */
    startFocusSession(focusType, customDuration = null) {
        this.ensureInitialized();
        
        this.writingSession.startFocusSession(focusType, customDuration);
        this.emit(WriteMagicEvents.FOCUS_SESSION_STARTED, { focusType, customDuration });
    }

    /**
     * Set writing goal for current session
     */
    setWritingGoal(type, target, description = null) {
        this.ensureInitialized();
        return this.writingSession.setSessionGoal(type, target, description);
    }

    /**
     * Get current session statistics
     */
    getSessionStats() {
        this.ensureInitialized();
        return this.writingSession.getSessionStats();
    }

    /**
     * Get writing session history
     */
    getSessionHistory(options = {}) {
        this.ensureInitialized();
        return this.writingSession.getSessionHistory(options);
    }

    // AI Integration API

    /**
     * Complete text using AI
     */
    async completeText(prompt, options = {}) {
        this.ensureInitialized();
        
        try {
            const {
                model = this.config.default_model,
                maxTokens = 500,
                temperature = 0.7,
                context = null
            } = options;
            
            const request = new WasmCompletionRequest(prompt, model);
            
            if (maxTokens) request.set_max_tokens(maxTokens);
            if (temperature !== undefined) request.set_temperature(temperature);
            if (context) request.set_context(context);
            
            const response = await this.wasmEngine.complete_text(request);
            
            this.emit(WriteMagicEvents.AI_COMPLETION, { 
                prompt, 
                response: response.content,
                model: response.model,
                tokensUsed: response.tokens_used
            });
            
            return {
                content: response.content,
                model: response.model,
                tokensUsed: response.tokens_used,
                finishReason: response.finish_reason
            };
            
        } catch (error) {
            this.emit(WriteMagicEvents.AI_ERROR, { error, prompt });
            throw error;
        }
    }

    /**
     * Get AI writing suggestions for content
     */
    async getWritingSuggestions(content, suggestionType = 'improve') {
        this.ensureInitialized();
        
        const prompts = {
            improve: `Please suggest improvements for this text while maintaining the author's voice:\n\n${content}`,
            expand: `Please expand on this text with additional details and examples:\n\n${content}`,
            summarize: `Please provide a concise summary of this text:\n\n${content}`,
            rewrite: `Please rewrite this text for better clarity and flow:\n\n${content}`,
            grammar: `Please fix any grammar and spelling errors in this text:\n\n${content}`
        };
        
        const prompt = prompts[suggestionType] || prompts.improve;
        
        return await this.completeText(prompt, {
            maxTokens: 1000,
            temperature: 0.3
        });
    }

    /**
     * Check AI provider health
     */
    async checkAIHealth() {
        this.ensureInitialized();
        return await this.wasmEngine.get_ai_provider_health();
    }

    // Content Analysis API

    /**
     * Analyze content comprehensively
     */
    analyzeContent(content, options = {}) {
        this.ensureInitialized();
        return this.writingAnalytics.analyzeDocument(content, options);
    }

    /**
     * Get writing style suggestions
     */
    getStyleSuggestions(content) {
        this.ensureInitialized();
        return this.contentUtils.analyzeStyle(content);
    }

    /**
     * Extract document outline
     */
    extractOutline(content) {
        this.ensureInitialized();
        return this.contentUtils.extractOutline(content);
    }

    /**
     * Search through documents
     */
    searchDocuments(query, options = {}) {
        this.ensureInitialized();
        
        // Get all documents (this would be enhanced with proper search indexing)
        const documents = Array.from(this.documentManager.documents.values());
        return this.contentUtils.searchDocuments(documents, query, options);
    }

    // Utility API

    /**
     * Apply content template
     */
    applyTemplate(templateKey, customContent = '') {
        this.ensureInitialized();
        return this.contentUtils.applyTemplate(templateKey, customContent);
    }

    /**
     * Format content for export
     */
    formatContent(content, outputFormat, options = {}) {
        this.ensureInitialized();
        return this.contentUtils.formatContent(content, outputFormat, options);
    }

    /**
     * Count words in text
     */
    countWords(text) {
        this.ensureInitialized();
        return this.contentUtils.countWords(text);
    }

    /**
     * Estimate reading time
     */
    estimateReadingTime(content, wordsPerMinute = 250) {
        this.ensureInitialized();
        return this.contentUtils.estimateReadingTime(content, wordsPerMinute);
    }

    // Event Management API

    /**
     * Add event listener
     */
    on(event, callback) {
        if (!this.eventCallbacks.has(event)) {
            this.eventCallbacks.set(event, new Set());
        }
        this.eventCallbacks.get(event).add(callback);
        return this;
    }

    /**
     * Remove event listener
     */
    off(event, callback) {
        if (this.eventCallbacks.has(event)) {
            this.eventCallbacks.get(event).delete(callback);
        }
        return this;
    }

    /**
     * Emit event
     */
    emit(event, data) {
        if (this.eventCallbacks.has(event)) {
            for (const callback of this.eventCallbacks.get(event)) {
                try {
                    callback(data);
                } catch (error) {
                    log_error(`Error in event callback for '${event}': ${error.message}`);
                }
            }
        }
    }

    // Private Methods

    ensureInitialized() {
        if (!this.isInitialized) {
            throw new Error('WriteMagic is not initialized. Wait for initialization to complete.');
        }
    }

    getLayoutPreset(layoutName) {
        const presets = {
            focus: {
                name: 'Focus Mode',
                panes: [{ type: 'editor', size: 100 }]
            },
            split: {
                name: 'Split View',
                panes: [
                    { type: 'editor', size: 50 },
                    { type: 'reference', size: 50 }
                ]
            },
            research: {
                name: 'Research Mode',
                panes: [
                    { type: 'editor', size: 40 },
                    { type: 'notes', size: 30 },
                    { type: 'reference', size: 30 }
                ]
            },
            review: {
                name: 'Review Mode',
                panes: [
                    { type: 'editor', size: 50 },
                    { type: 'preview', size: 30 },
                    { type: 'timeline', size: 20 }
                ]
            },
            ai_enhanced: {
                name: 'AI-Enhanced Writing',
                panes: [
                    { type: 'editor', size: 50 },
                    { type: 'ai_assistant', size: 25 },
                    { type: 'outline', size: 25 }
                ]
            }
        };

        return presets[layoutName] || presets.focus;
    }

    setupEventForwarding() {
        // Forward document manager events
        this.documentManager.on('document_created', (data) => {
            this.emit(WriteMagicEvents.DOCUMENT_CREATED, data);
        });
        
        this.documentManager.on('document_loaded', (data) => {
            this.emit(WriteMagicEvents.DOCUMENT_LOADED, data);
        });
        
        this.documentManager.on('document_updated', (data) => {
            this.emit(WriteMagicEvents.DOCUMENT_UPDATED, data);
        });
        
        this.documentManager.on('auto_saved', (data) => {
            this.emit(WriteMagicEvents.DOCUMENT_AUTO_SAVED, data);
        });

        // Forward writing session events
        this.writingSession.on('goal_achieved', (data) => {
            this.emit(WriteMagicEvents.GOAL_ACHIEVED, data);
        });

        // Forward error events
        const forwardErrors = (source) => {
            source.on('error', (data) => {
                this.emit(WriteMagicEvents.ERROR, data);
            });
        };

        forwardErrors(this.documentManager);
        forwardErrors(this.projectWorkspace);
        forwardErrors(this.writingSession);
    }

    /**
     * Clean up resources
     */
    destroy() {
        if (this.writingSession) {
            this.writingSession.destroy();
        }
        
        if (this.projectWorkspace) {
            this.projectWorkspace.destroy();
        }
        
        if (this.documentManager) {
            this.documentManager.destroy();
        }
        
        this.eventCallbacks.clear();
        this.isInitialized = false;
    }
}

// Export individual components for advanced usage
export {
    DocumentManager,
    ProjectWorkspace,
    WritingAnalytics,
    ContentUtilities,
    WritingSession
};

// Export WASM components for direct access
export {
    WriteMagicEngine,
    WasmCompletionRequest,
    init_logging,
    init_panic_hook,
    log,
    log_error
};

// Default export
export default WriteMagic;