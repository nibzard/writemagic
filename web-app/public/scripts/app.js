/**
 * WriteMagic Web Application
 * 
 * Main application entry point that initializes and coordinates all components
 */

import WriteMagic, { WriteMagicEvents } from '../src/js/index.js';
import ServiceWorkerManager from '../src/js/utils/service-worker-manager.js';
import NetworkIndicator from '../src/js/utils/network-indicator.js';

/**
 * WriteMagic Web Application Class
 * Manages the complete user interface and user experience
 */
class WriteMagicApp {
    constructor() {
        this.writeMagic = null;
        this.currentDocument = null;
        this.currentProject = null;
        this.activeLayout = 'focus';
        this.isDarkMode = false;
        this.isInitialized = false;
        this.autosaveTimer = null;
        
        // Enhanced offline capabilities
        this.serviceWorkerManager = null;
        this.networkIndicator = null;
        this.isOfflineMode = false;
        
        // UI Elements
        this.elements = {};
        
        // State management
        this.state = {
            sidebarCollapsed: false,
            analyticsCollapsed: true,
            focusSessionActive: false,
            settingsOpen: false,
            currentSessionId: null,
            unsavedChanges: false,
            lastWordCount: 0,
            sessionStartTime: null,
            networkStatus: {
                isOnline: navigator.onLine,
                quality: 'unknown',
                offlineReady: false
            }
        };
        
        // Event handlers
        this.handlers = new Map();
        
        // Initialize when DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => this.init());
        } else {
            this.init();
        }
    }

    /**
     * Initialize the application
     */
    async init() {
        try {
            console.log('WriteMagic Web App initializing...');
            
            // Cache DOM elements
            this.cacheElements();
            
            // Setup event listeners
            this.setupEventListeners();
            
            // Initialize WriteMagic core
            await this.initializeWriteMagic();
            
            // Initialize Service Worker and offline capabilities
            await this.initializeServiceWorker();
            
            // Setup UI
            this.setupUI();
            
            // Load user preferences
            this.loadUserPreferences();
            
            // Initial content load
            await this.loadInitialContent();
            
            // Hide loading screen
            this.hideLoadingScreen();
            
            // Mark as initialized
            this.isInitialized = true;
            
            console.log('WriteMagic Web App initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize WriteMagic App:', error);
            this.showError('Failed to initialize application. Please refresh the page.');
        }
    }

    /**
     * Cache frequently used DOM elements
     */
    cacheElements() {
        const elements = [
            'app', 'loading-screen', 'loading-status',
            'project-sidebar', 'sidebar-toggle',
            'main-editor', 'editor-document-title', 'editor-word-count', 'editor-save-status',
            'ai-messages', 'ai-input', 'ai-send-btn', 'ai-suggestions',
            'word-count', 'session-time', 'current-project', 'current-document',
            'ai-status', 'settings-btn',
            'project-list', 'new-project-btn', 'new-document-btn',
            'layout-toolbar', 'workspace-panes',
            'analytics-panel', 'analytics-toggle', 'analytics-content',
            'settings-modal', 'focus-session-modal',
            'sr-announcements'
        ];
        
        elements.forEach(id => {
            this.elements[this.camelCase(id)] = document.getElementById(id);
        });
        
        // Cache element collections
        this.elements.layoutBtns = document.querySelectorAll('.layout-btn');
        this.elements.panes = document.querySelectorAll('.pane');
        this.elements.settingsTabs = document.querySelectorAll('#settings-modal .tab-btn');
        this.elements.settingsPanels = document.querySelectorAll('#settings-modal .tab-panel');
    }

    /**
     * Convert hyphenated string to camelCase
     */
    camelCase(str) {
        return str.replace(/-([a-z])/g, (g) => g[1].toUpperCase());
    }

    /**
     * Setup all event listeners
     */
    setupEventListeners() {
        // Sidebar toggle
        this.elements.sidebarToggle?.addEventListener('click', () => this.toggleSidebar());
        
        // Document editor
        this.elements.mainEditor?.addEventListener('input', (e) => this.handleEditorInput(e));
        this.elements.mainEditor?.addEventListener('keydown', (e) => this.handleEditorKeydown(e));
        
        // Layout buttons
        this.elements.layoutBtns.forEach(btn => {
            btn.addEventListener('click', () => this.setLayout(btn.dataset.layout));
        });
        
        // Session controls
        document.getElementById('start-focus-session')?.addEventListener('click', () => this.openFocusSessionModal());
        document.getElementById('set-writing-goal')?.addEventListener('click', () => this.openWritingGoalModal());
        
        // AI assistant
        this.elements.aiSendBtn?.addEventListener('click', () => this.sendAIMessage());
        this.elements.aiInput?.addEventListener('keydown', (e) => this.handleAIInputKeydown(e));
        this.elements.aiSuggestions?.addEventListener('click', (e) => this.handleAISuggestionClick(e));
        
        // Analytics panel
        this.elements.analyticsToggle?.addEventListener('click', () => this.toggleAnalytics());
        
        // Settings
        this.elements.settingsBtn?.addEventListener('click', () => this.openSettings());
        
        // Project management
        this.elements.newProjectBtn?.addEventListener('click', () => this.createNewProject());
        this.elements.newDocumentBtn?.addEventListener('click', () => this.createNewDocument());
        
        // Modal handling
        this.setupModalEventListeners();
        
        // Keyboard shortcuts
        this.setupKeyboardShortcuts();
        
        // Window events
        window.addEventListener('resize', () => this.handleWindowResize());
        window.addEventListener('beforeunload', (e) => this.handleBeforeUnload(e));
        window.addEventListener('online', () => this.handleOnlineStatus(true));
        window.addEventListener('offline', () => this.handleOnlineStatus(false));
        
        // Theme detection
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', (e) => this.handleSystemThemeChange(e));
    }

    /**
     * Initialize WriteMagic core engine
     */
    async initializeWriteMagic() {
        this.updateLoadingStatus('Loading WriteMagic engine...');
        
        const config = {
            auto_save_delay: 2000,
            enable_analytics: true,
            enable_focus_mode: true,
            default_layout: 'focus',
            enable_keyboard_navigation: true,
            enable_accessibility: true
        };
        
        this.writeMagic = new WriteMagic(config);
        
        // Setup WriteMagic event listeners
        this.writeMagic.on(WriteMagicEvents.INITIALIZED, () => {
            console.log('WriteMagic core initialized');
            this.updateLoadingStatus('Setting up workspace...');
        });
        
        this.writeMagic.on(WriteMagicEvents.DOCUMENT_AUTO_SAVED, (data) => {
            this.updateSaveStatus('saved');
            this.state.unsavedChanges = false;
        });
        
        this.writeMagic.on(WriteMagicEvents.DOCUMENT_UPDATED, (data) => {
            this.state.unsavedChanges = true;
            this.updateSaveStatus('saving');
        });
        
        this.writeMagic.on(WriteMagicEvents.AI_COMPLETION, (data) => {
            this.addAIMessage(data.response, 'ai');
        });
        
        this.writeMagic.on(WriteMagicEvents.ERROR, (data) => {
            this.showError(data.error.message);
        });
        
        // Wait for full initialization
        while (!this.writeMagic.isInitialized) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
    }

    /**
     * Initialize Service Worker and offline capabilities
     */
    async initializeServiceWorker() {
        this.updateLoadingStatus('Setting up offline capabilities...');
        
        try {
            if (!('serviceWorker' in navigator)) {
                console.warn('[App] Service Workers not supported');
                return;
            }
            
            // Initialize Service Worker manager
            this.serviceWorkerManager = new ServiceWorkerManager({
                enableCacheDebugging: true,
                enableNetworkMonitoring: true,
                enableStorageMonitoring: true
            });
            
            // Setup Service Worker event handlers
            this.setupServiceWorkerEventHandlers();
            
            // Initialize network indicator
            await this.initializeNetworkIndicator();
            
            console.log('[App] Service Worker and offline capabilities initialized');
            
        } catch (error) {
            console.error('[App] Service Worker initialization failed:', error);
            // Continue without Service Worker capabilities
        }
    }

    /**
     * Setup Service Worker event handlers
     */
    setupServiceWorkerEventHandlers() {
        if (!this.serviceWorkerManager) return;
        
        // Handle Service Worker updates
        this.serviceWorkerManager.on('update_available', (data) => {
            const shouldUpdate = confirm(
                'A new version of WriteMagic is available. Update now? (Recommended)'
            );
            if (shouldUpdate) {
                this.serviceWorkerManager.updateServiceWorker();
            } else {
                this.showNotification('Update postponed. Refresh the page to update later.', 'info', 5000);
            }
        });
        
        // Handle offline/online transitions
        this.serviceWorkerManager.on('network_change', (data) => {
            this.handleNetworkStatusChange(data);
        });
        
        // Handle background sync completion
        this.serviceWorkerManager.on('sync_complete', (data) => {
            console.log(`[App] Background sync completed: ${data.successful} successful, ${data.failed} failed`);
            
            if (data.successful > 0) {
                this.showNotification(
                    `Successfully synced ${data.successful} item${data.successful > 1 ? 's' : ''}`, 
                    'success'
                );
            }
            
            if (data.failed > 0) {
                this.showNotification(
                    `Failed to sync ${data.failed} item${data.failed > 1 ? 's' : ''}`, 
                    'warning'
                );
            }
        });
        
        // Handle storage warnings
        this.serviceWorkerManager.on('storage_warning', (data) => {
            const percent = Math.round((data.usage / data.quota) * 100);
            this.showNotification(
                `Storage space is running low (${percent}% used). Consider clearing old data.`, 
                'warning', 
                8000
            );
        });
        
        // Handle Service Worker activation
        this.serviceWorkerManager.on('activated', (data) => {
            console.log(`[App] Service Worker activated: v${data.version}`);
            this.checkOfflineCapabilities();
        });
    }

    /**
     * Initialize network indicator
     */
    async initializeNetworkIndicator() {
        try {
            const headerRight = this.elements.headerRight || document.querySelector('.header-right');
            
            if (headerRight && this.serviceWorkerManager) {
                this.networkIndicator = new NetworkIndicator(headerRight, this.serviceWorkerManager);
                
                // Make available globally for debugging
                if (typeof window !== 'undefined') {
                    window.swManager = this.serviceWorkerManager;
                    window.networkIndicator = this.networkIndicator;
                }
            }
        } catch (error) {
            console.warn('[App] Network indicator initialization failed:', error);
        }
    }

    /**
     * Handle network status changes
     */
    handleNetworkStatusChange(data) {
        const wasOffline = this.isOfflineMode;
        this.isOfflineMode = !data.isOnline;
        
        // Update state
        this.state.networkStatus = {
            isOnline: data.isOnline,
            quality: data.quality,
            offlineReady: this.state.networkStatus.offlineReady
        };
        
        // Update UI
        if (this.isOfflineMode) {
            document.body.classList.add('offline-mode');
            
            if (!wasOffline) {
                this.showNotification(
                    'You\'re now offline. Changes will sync when reconnected.', 
                    'warning', 
                    5000
                );
            }
            
            // Disable AI features
            this.disableAIFeatures();
            
        } else {
            document.body.classList.remove('offline-mode');
            
            if (wasOffline) {
                this.showNotification('Back online! Syncing your changes...', 'success');
                
                // Re-enable AI features
                this.enableAIFeatures();
                
                // Trigger background sync
                if (this.serviceWorkerManager) {
                    this.serviceWorkerManager.forceSync();
                }
            }
        }
        
        // Update AI health indicator
        this.updateAIHealthIndicator();
    }

    /**
     * Check offline capabilities
     */
    async checkOfflineCapabilities() {
        if (!this.serviceWorkerManager) return;
        
        try {
            const isOfflineReady = await this.serviceWorkerManager.isOfflineReady();
            this.state.networkStatus.offlineReady = isOfflineReady;
            
            if (isOfflineReady) {
                console.log('[App] Application is ready for offline use');
                
                // Show offline readiness notification (only once per session)
                if (!sessionStorage.getItem('offline-ready-shown')) {
                    this.showNotification(
                        'WriteMagic is now available offline!', 
                        'success', 
                        4000
                    );
                    sessionStorage.setItem('offline-ready-shown', 'true');
                }
            }
            
        } catch (error) {
            console.warn('[App] Failed to check offline capabilities:', error);
        }
    }

    /**
     * Disable AI features when offline
     */
    disableAIFeatures() {
        // Disable AI send button
        if (this.elements.aiSendBtn) {
            this.elements.aiSendBtn.disabled = true;
            this.elements.aiSendBtn.title = 'AI features are not available offline';
        }
        
        // Disable AI suggestion chips
        const suggestionChips = document.querySelectorAll('.suggestion-chip');
        suggestionChips.forEach(chip => {
            chip.disabled = true;
            chip.style.opacity = '0.5';
        });
        
        // Show offline message in AI panel
        this.addAIMessage(
            'AI features are not available offline. Your messages will be sent when you reconnect.', 
            'system'
        );
    }

    /**
     * Enable AI features when online
     */
    enableAIFeatures() {
        // Enable AI send button
        if (this.elements.aiSendBtn) {
            this.elements.aiSendBtn.disabled = false;
            this.elements.aiSendBtn.title = 'Send message to AI assistant';
        }
        
        // Enable AI suggestion chips
        const suggestionChips = document.querySelectorAll('.suggestion-chip');
        suggestionChips.forEach(chip => {
            chip.disabled = false;
            chip.style.opacity = '1';
        });
    }

    /**
     * Update AI health indicator based on network status
     */
    updateAIHealthIndicator() {
        const aiStatus = this.elements.aiStatus;
        if (!aiStatus) return;
        
        aiStatus.className = 'header-button ai-health-indicator';
        
        if (this.isOfflineMode) {
            aiStatus.classList.add('offline');
            aiStatus.setAttribute('aria-label', 'AI service offline (no connection)');
        } else {
            aiStatus.classList.add('online');
            aiStatus.setAttribute('aria-label', 'AI service online');
            
            // Recheck AI health when back online
            setTimeout(() => this.monitorAIHealth(), 1000);
        }
    }

    /**
     * Setup initial UI state
     */
    setupUI() {
        this.updateLoadingStatus('Setting up interface...');
        
        // Set initial layout
        this.setLayout('focus');
        
        // Initialize analytics panel
        this.updateAnalytics();
        
        // Setup AI health monitoring
        this.monitorAIHealth();
        
        // Initialize session timer
        this.startSessionTimer();
        
        // Apply initial theme
        this.applyTheme();
    }

    /**
     * Load user preferences from localStorage
     */
    loadUserPreferences() {
        try {
            const prefs = localStorage.getItem('writemagic-preferences');
            if (prefs) {
                const preferences = JSON.parse(prefs);
                
                // Apply theme
                if (preferences.theme) {
                    this.setTheme(preferences.theme);
                }
                
                // Apply layout preference
                if (preferences.layout) {
                    this.setLayout(preferences.layout);
                }
                
                // Apply sidebar state
                if (preferences.sidebarCollapsed) {
                    this.setSidebarState(preferences.sidebarCollapsed);
                }
                
                // Apply accessibility preferences
                if (preferences.accessibility) {
                    this.applyAccessibilityPreferences(preferences.accessibility);
                }
            }
        } catch (error) {
            console.warn('Failed to load user preferences:', error);
        }
    }

    /**
     * Save user preferences to localStorage
     */
    saveUserPreferences() {
        try {
            const preferences = {
                theme: this.getCurrentTheme(),
                layout: this.activeLayout,
                sidebarCollapsed: this.state.sidebarCollapsed,
                accessibility: this.getAccessibilityPreferences(),
                version: '1.0.0'
            };
            
            localStorage.setItem('writemagic-preferences', JSON.stringify(preferences));
        } catch (error) {
            console.warn('Failed to save user preferences:', error);
        }
    }

    /**
     * Load initial content
     */
    async loadInitialContent() {
        this.updateLoadingStatus('Loading your content...');
        
        try {
            // Try to load the welcome document or create it
            const documents = await this.writeMagic.listDocuments();
            let welcomeDoc = documents.find(doc => doc.id === 'welcome');
            
            if (!welcomeDoc) {
                // Create welcome document
                welcomeDoc = await this.writeMagic.createDocument({
                    title: 'Welcome to WriteMagic',
                    content: this.elements.mainEditor.value || '',
                    type: 'markdown'
                });
            }
            
            this.currentDocument = welcomeDoc;
            this.updateDocumentUI();
            
        } catch (error) {
            console.warn('Failed to load initial content:', error);
            // Continue with default content
        }
    }

    /**
     * Hide loading screen with animation
     */
    hideLoadingScreen() {
        const loadingScreen = this.elements.loadingScreen;
        const app = this.elements.app;
        
        if (loadingScreen && app) {
            loadingScreen.classList.add('hidden');
            app.style.display = 'flex';
            
            // Remove loading screen after animation
            setTimeout(() => {
                loadingScreen.style.display = 'none';
            }, 500);
        }
    }

    /**
     * Update loading status message
     */
    updateLoadingStatus(message) {
        if (this.elements.loadingStatus) {
            this.elements.loadingStatus.textContent = message;
        }
    }

    /**
     * Handle editor input
     */
    handleEditorInput(event) {
        if (!this.currentDocument || !this.writeMagic.isInitialized) return;
        
        const content = event.target.value;
        
        // Update document through WriteMagic
        this.writeMagic.updateDocumentContent(this.currentDocument.id, content);
        
        // Update word count
        this.updateWordCount(content);
        
        // Update analytics
        this.updateAnalytics();
        
        // Clear any existing autosave timer
        if (this.autosaveTimer) {
            clearTimeout(this.autosaveTimer);
        }
        
        // Set new autosave timer
        this.autosaveTimer = setTimeout(async () => {
            try {
                await this.writeMagic.saveDocument(this.currentDocument.id);
                this.updateSaveStatus('saved');
            } catch (error) {
                console.error('Autosave failed:', error);
                this.updateSaveStatus('error');
            }
        }, 2000);
        
        // Mark as having unsaved changes
        this.state.unsavedChanges = true;
        this.updateSaveStatus('saving');
    }

    /**
     * Handle editor keyboard shortcuts
     */
    handleEditorKeydown(event) {
        // Ctrl/Cmd + S for manual save
        if ((event.ctrlKey || event.metaKey) && event.key === 's') {
            event.preventDefault();
            this.saveDocument();
        }
        
        // Ctrl/Cmd + Space for AI assistance
        if ((event.ctrlKey || event.metaKey) && event.key === ' ') {
            event.preventDefault();
            this.triggerAIAssistance();
        }
        
        // Tab handling for accessibility
        if (event.key === 'Tab' && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
            // Allow normal tab behavior in editor
            return;
        }
        
        // Escape to exit focus mode
        if (event.key === 'Escape' && this.activeLayout === 'focus') {
            this.setLayout('split');
        }
    }

    /**
     * Update word count display
     */
    updateWordCount(content) {
        const wordCount = this.writeMagic.countWords(content);
        
        if (this.elements.wordCount) {
            this.elements.wordCount.textContent = wordCount.toString();
        }
        
        if (this.elements.editorWordCount) {
            this.elements.editorWordCount.textContent = `${wordCount} words`;
        }
        
        // Update session progress
        const wordsAdded = Math.max(0, wordCount - this.state.lastWordCount);
        this.state.lastWordCount = wordCount;
        
        if (wordsAdded > 0 && this.state.focusSessionActive) {
            this.updateSessionProgress(wordsAdded);
        }
    }

    /**
     * Update analytics display
     */
    updateAnalytics() {
        if (!this.currentDocument || !this.writeMagic.isInitialized) return;
        
        try {
            const content = this.elements.mainEditor?.value || '';
            const analytics = this.writeMagic.analyzeContent(content);
            
            // Update analytics display
            const analyticsWords = document.getElementById('analytics-words');
            const analyticsReadingTime = document.getElementById('analytics-reading-time');
            const analyticsComplexity = document.getElementById('analytics-complexity');
            
            if (analyticsWords) {
                analyticsWords.textContent = analytics.wordCount || 0;
            }
            
            if (analyticsReadingTime) {
                const readingTime = this.writeMagic.estimateReadingTime(content);
                analyticsReadingTime.textContent = `${readingTime} min`;
            }
            
            if (analyticsComplexity) {
                analyticsComplexity.textContent = analytics.complexity?.gradeLevel || '-';
            }
            
        } catch (error) {
            console.warn('Failed to update analytics:', error);
        }
    }

    /**
     * Update save status indicator
     */
    updateSaveStatus(status) {
        const saveStatusEl = this.elements.editorSaveStatus;
        if (!saveStatusEl) return;
        
        saveStatusEl.className = 'pane-action-btn';
        
        switch (status) {
            case 'saving':
                saveStatusEl.classList.add('saving');
                saveStatusEl.innerHTML = `
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" class="spinning">
                        <circle cx="6" cy="6" r="5" fill="none" stroke="currentColor" stroke-width="2"/>
                        <path d="M6 1a5 5 0 015 5" stroke="currentColor" stroke-width="2" fill="none"/>
                    </svg>
                `;
                saveStatusEl.setAttribute('aria-label', 'Saving document...');
                break;
                
            case 'saved':
                saveStatusEl.classList.add('saved');
                saveStatusEl.innerHTML = `
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                        <path d="M10 3L4.5 8.5 2 6"/>
                    </svg>
                `;
                saveStatusEl.setAttribute('aria-label', 'Document saved');
                break;
                
            case 'error':
                saveStatusEl.classList.add('error');
                saveStatusEl.innerHTML = `
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                        <circle cx="6" cy="6" r="5"/>
                        <path d="M4 4l4 4M8 4l-4 4" stroke="white" stroke-width="1"/>
                    </svg>
                `;
                saveStatusEl.setAttribute('aria-label', 'Save failed');
                break;
        }
    }

    /**
     * Set workspace layout
     */
    setLayout(layoutName) {
        if (!this.writeMagic?.isInitialized) return;
        
        this.activeLayout = layoutName;
        
        // Update layout buttons
        this.elements.layoutBtns.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.layout === layoutName);
        });
        
        // Update workspace classes
        const workspace = this.elements.workspacePanes;
        if (workspace) {
            workspace.className = `workspace-panes layout-${layoutName}`;
        }
        
        // Update WriteMagic layout
        this.writeMagic.setLayout(layoutName);
        
        // Handle AI pane visibility
        const aiPane = document.getElementById('pane-ai');
        if (aiPane) {
            const showAI = ['split', 'ai_enhanced'].includes(layoutName);
            aiPane.style.display = showAI ? 'flex' : 'none';
        }
        
        // Save preference
        this.saveUserPreferences();
        
        // Announce layout change for screen readers
        this.announceToScreenReader(`Layout changed to ${layoutName} mode`);
    }

    /**
     * Toggle sidebar
     */
    toggleSidebar() {
        this.state.sidebarCollapsed = !this.state.sidebarCollapsed;
        this.setSidebarState(this.state.sidebarCollapsed);
    }

    /**
     * Set sidebar state
     */
    setSidebarState(collapsed) {
        this.state.sidebarCollapsed = collapsed;
        
        const sidebar = this.elements.projectSidebar;
        if (sidebar) {
            sidebar.classList.toggle('collapsed', collapsed);
        }
        
        this.saveUserPreferences();
    }

    /**
     * Toggle analytics panel
     */
    toggleAnalytics() {
        this.state.analyticsCollapsed = !this.state.analyticsCollapsed;
        
        const panel = this.elements.analyticsPanel;
        const toggle = this.elements.analyticsToggle;
        
        if (panel && toggle) {
            if (this.state.analyticsCollapsed) {
                panel.classList.add('collapsed');
                panel.classList.remove('expanded');
            } else {
                panel.classList.remove('collapsed');
                panel.classList.add('expanded');
                this.updateAnalytics();
            }
        }
    }

    /**
     * Open focus session modal
     */
    openFocusSessionModal() {
        const modal = document.getElementById('focus-session-modal');
        if (modal) {
            modal.classList.add('active');
            this.setupFocusSessionModal();
        }
    }

    /**
     * Setup focus session modal interactions
     */
    setupFocusSessionModal() {
        // Focus session type selection
        const typeButtons = document.querySelectorAll('.session-type-btn');
        typeButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                typeButtons.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                
                // Show/hide custom duration input
                const customDuration = document.getElementById('custom-duration');
                if (customDuration) {
                    customDuration.style.display = 
                        btn.dataset.duration === 'custom' ? 'block' : 'none';
                }
            });
        });
        
        // Start focus session
        const startBtn = document.getElementById('focus-start');
        if (startBtn) {
            startBtn.onclick = () => {
                const activeBtn = document.querySelector('.session-type-btn.active');
                if (activeBtn) {
                    let duration = parseInt(activeBtn.dataset.duration);
                    
                    if (activeBtn.dataset.duration === 'custom') {
                        const customMinutes = document.getElementById('custom-minutes');
                        duration = customMinutes ? parseInt(customMinutes.value) * 60 : 1500;
                    }
                    
                    const goal = document.getElementById('session-goal-input')?.value || null;
                    
                    this.startFocusSession(duration, goal);
                    this.closeModal('focus-session-modal');
                }
            };
        }
    }

    /**
     * Start focus session
     */
    startFocusSession(duration, goal = null) {
        this.state.focusSessionActive = true;
        this.state.sessionStartTime = Date.now();
        
        // Apply focus mode
        document.body.classList.add('focus-mode');
        
        // Start WriteMagic session
        const session = this.writeMagic.startWritingSession({
            duration: duration,
            goal: goal,
            type: 'focus'
        });
        
        this.state.currentSessionId = session?.id;
        
        // Update UI
        const sessionBtn = document.getElementById('start-focus-session');
        if (sessionBtn) {
            sessionBtn.textContent = 'End Focus';
            sessionBtn.classList.add('active');
            sessionBtn.onclick = () => this.endFocusSession();
        }
        
        // Set timer for auto-end
        setTimeout(() => {
            if (this.state.focusSessionActive) {
                this.endFocusSession();
                this.showNotification('Focus session completed!', 'success');
            }
        }, duration * 1000);
        
        this.announceToScreenReader(`Focus session started for ${Math.round(duration / 60)} minutes`);
    }

    /**
     * End focus session
     */
    endFocusSession() {
        this.state.focusSessionActive = false;
        this.state.currentSessionId = null;
        
        // Remove focus mode
        document.body.classList.remove('focus-mode');
        
        // End WriteMagic session
        const session = this.writeMagic.endWritingSession();
        
        // Update UI
        const sessionBtn = document.getElementById('start-focus-session');
        if (sessionBtn) {
            sessionBtn.textContent = 'Focus';
            sessionBtn.classList.remove('active');
            sessionBtn.onclick = () => this.openFocusSessionModal();
        }
        
        // Show session summary
        if (session) {
            this.showSessionSummary(session);
        }
        
        this.announceToScreenReader('Focus session ended');
    }

    /**
     * Show session summary
     */
    showSessionSummary(session) {
        const wordsWritten = session.statistics?.wordsAdded || 0;
        const timeSpent = Math.round(session.duration / 60);
        
        this.showNotification(
            `Session complete! You wrote ${wordsWritten} words in ${timeSpent} minutes.`,
            'success',
            5000
        );
    }

    /**
     * Handle AI message sending
     */
    async sendAIMessage() {
        const input = this.elements.aiInput;
        if (!input || !input.value.trim()) return;
        
        const message = input.value.trim();
        input.value = '';
        
        // Add user message to chat
        this.addAIMessage(message, 'user');
        
        // Show typing indicator
        this.showAITyping(true);
        
        try {
            // Get AI response
            const response = await this.writeMagic.completeText(message, {
                maxTokens: 500,
                temperature: 0.7
            });
            
            // Hide typing indicator and show response
            this.showAITyping(false);
            this.addAIMessage(response.content, 'ai');
            
        } catch (error) {
            this.showAITyping(false);
            this.addAIMessage('Sorry, I encountered an error. Please try again.', 'ai');
            console.error('AI completion failed:', error);
        }
    }

    /**
     * Add message to AI chat
     */
    addAIMessage(content, sender) {
        const messagesContainer = this.elements.aiMessages;
        if (!messagesContainer) return;
        
        const messageDiv = document.createElement('div');
        messageDiv.className = `ai-message ${sender}-message`;
        
        const contentDiv = document.createElement('div');
        contentDiv.className = 'message-content';
        
        // Handle different message types
        if (sender === 'system') {
            contentDiv.innerHTML = `<em>${content}</em>`;
            messageDiv.classList.add('system-message');
        } else if (sender === 'ai') {
            contentDiv.innerHTML = this.formatAIMessage(content);
        } else {
            contentDiv.textContent = content;
        }
        
        messageDiv.appendChild(contentDiv);
        messagesContainer.appendChild(messageDiv);
        
        // Scroll to bottom
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
        
        // Announce new AI message to screen readers
        if (sender === 'ai') {
            this.announceToScreenReader(`AI assistant: ${content.substring(0, 100)}${content.length > 100 ? '...' : ''}`);
        } else if (sender === 'system') {
            this.announceToScreenReader(`System: ${content}`);
        }
    }

    /**
     * Format AI message content
     */
    formatAIMessage(content) {
        // Simple formatting - could be enhanced with a markdown parser
        return content
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            .replace(/`(.*?)`/g, '<code>$1</code>')
            .replace(/\n/g, '<br>');
    }

    /**
     * Show/hide AI typing indicator
     */
    showAITyping(show) {
        let typingIndicator = document.getElementById('ai-typing-indicator');
        
        if (show) {
            if (!typingIndicator) {
                typingIndicator = document.createElement('div');
                typingIndicator.id = 'ai-typing-indicator';
                typingIndicator.className = 'ai-message ai-message typing';
                typingIndicator.innerHTML = `
                    <div class="message-content">
                        <div class="typing-dots">
                            <span></span>
                            <span></span>
                            <span></span>
                        </div>
                        AI is thinking...
                    </div>
                `;
                this.elements.aiMessages.appendChild(typingIndicator);
            }
        } else {
            if (typingIndicator) {
                typingIndicator.remove();
            }
        }
        
        // Scroll to bottom
        if (this.elements.aiMessages) {
            this.elements.aiMessages.scrollTop = this.elements.aiMessages.scrollHeight;
        }
    }

    /**
     * Handle AI input keydown
     */
    handleAIInputKeydown(event) {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            this.sendAIMessage();
        }
    }

    /**
     * Handle AI suggestion clicks
     */
    handleAISuggestionClick(event) {
        if (event.target.classList.contains('suggestion-chip')) {
            const suggestion = event.target.dataset.suggestion;
            const selectedText = this.getSelectedText();
            
            if (selectedText) {
                this.processAISuggestion(suggestion, selectedText);
            } else {
                // Show message about selecting text first
                this.showNotification('Please select some text first', 'info');
            }
        }
    }

    /**
     * Get selected text from editor
     */
    getSelectedText() {
        const editor = this.elements.mainEditor;
        if (!editor) return '';
        
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        
        return editor.value.substring(start, end);
    }

    /**
     * Process AI suggestion for selected text
     */
    async processAISuggestion(suggestionType, selectedText) {
        try {
            this.showAITyping(true);
            
            const response = await this.writeMagic.getWritingSuggestions(selectedText, suggestionType);
            
            this.showAITyping(false);
            this.addAIMessage(response.content, 'ai');
            
        } catch (error) {
            this.showAITyping(false);
            this.addAIMessage('Sorry, I couldn\'t process that suggestion. Please try again.', 'ai');
            console.error('AI suggestion failed:', error);
        }
    }

    /**
     * Monitor AI service health
     */
    async monitorAIHealth() {
        const statusIndicator = this.elements.aiStatus;
        if (!statusIndicator) return;
        
        try {
            const health = await this.writeMagic.checkAIHealth();
            
            statusIndicator.className = 'header-button ai-health-indicator';
            
            if (health.status === 'healthy') {
                statusIndicator.classList.add('online');
                statusIndicator.setAttribute('aria-label', 'AI service online');
            } else {
                statusIndicator.classList.add('offline');
                statusIndicator.setAttribute('aria-label', 'AI service offline');
            }
            
        } catch (error) {
            statusIndicator.classList.add('error');
            statusIndicator.setAttribute('aria-label', 'AI service error');
            console.warn('AI health check failed:', error);
        }
        
        // Check again in 30 seconds
        setTimeout(() => this.monitorAIHealth(), 30000);
    }

    /**
     * Start session timer
     */
    startSessionTimer() {
        const updateTimer = () => {
            if (this.state.sessionStartTime) {
                const elapsed = Date.now() - this.state.sessionStartTime;
                const minutes = Math.floor(elapsed / 60000);
                const seconds = Math.floor((elapsed % 60000) / 1000);
                
                const timerEl = this.elements.sessionTime;
                if (timerEl) {
                    timerEl.textContent = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
                }
            }
        };
        
        if (!this.state.sessionStartTime) {
            this.state.sessionStartTime = Date.now();
        }
        
        setInterval(updateTimer, 1000);
        updateTimer(); // Initial call
    }

    /**
     * Setup modal event listeners
     */
    setupModalEventListeners() {
        // Generic modal close handlers
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal-backdrop') || e.target.classList.contains('modal-close')) {
                const modal = e.target.closest('.modal');
                if (modal) {
                    modal.classList.remove('active');
                }
            }
        });
        
        // Escape key to close modals
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                const activeModal = document.querySelector('.modal.active');
                if (activeModal) {
                    activeModal.classList.remove('active');
                }
            }
        });
        
        // Settings modal tabs
        this.elements.settingsTabs.forEach(tab => {
            tab.addEventListener('click', () => this.switchSettingsTab(tab.dataset.tab));
        });
        
        // Settings save/cancel
        document.getElementById('settings-save')?.addEventListener('click', () => this.saveSettings());
        document.getElementById('settings-cancel')?.addEventListener('click', () => this.closeModal('settings-modal'));
    }

    /**
     * Setup keyboard shortcuts
     */
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Only handle shortcuts when not typing in inputs
            if (e.target.matches('input, textarea, select')) {
                return;
            }
            
            const isCtrlOrCmd = e.ctrlKey || e.metaKey;
            
            if (isCtrlOrCmd) {
                switch (e.key) {
                    case 'n':
                        e.preventDefault();
                        this.createNewDocument();
                        break;
                    case 's':
                        e.preventDefault();
                        this.saveDocument();
                        break;
                    case ',':
                        e.preventDefault();
                        this.openSettings();
                        break;
                    case 'b':
                        e.preventDefault();
                        this.toggleSidebar();
                        break;
                    case '1':
                        e.preventDefault();
                        this.setLayout('focus');
                        break;
                    case '2':
                        e.preventDefault();
                        this.setLayout('split');
                        break;
                    case '3':
                        e.preventDefault();
                        this.setLayout('research');
                        break;
                    case '4':
                        e.preventDefault();
                        this.setLayout('ai_enhanced');
                        break;
                }
            }
            
            // Function keys
            switch (e.key) {
                case 'F11':
                    e.preventDefault();
                    this.toggleFullscreen();
                    break;
            }
        });
    }

    /**
     * Open settings modal
     */
    openSettings() {
        const modal = this.elements.settingsModal;
        if (modal) {
            modal.classList.add('active');
            this.loadCurrentSettings();
        }
    }

    /**
     * Close modal by ID
     */
    closeModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.classList.remove('active');
        }
    }

    /**
     * Switch settings tab
     */
    switchSettingsTab(tabName) {
        // Update tab buttons
        this.elements.settingsTabs.forEach(tab => {
            tab.classList.toggle('active', tab.dataset.tab === tabName);
        });
        
        // Update tab panels
        this.elements.settingsPanels.forEach(panel => {
            panel.classList.toggle('active', panel.id === `tab-${tabName}`);
        });
    }

    /**
     * Load current settings into modal
     */
    loadCurrentSettings() {
        // Theme setting
        const themeSelect = document.getElementById('theme-select');
        if (themeSelect) {
            themeSelect.value = this.getCurrentTheme();
        }
        
        // Auto-save delay
        const autoSaveDelay = document.getElementById('auto-save-delay');
        if (autoSaveDelay && this.writeMagic?.config) {
            autoSaveDelay.value = this.writeMagic.config.auto_save_delay;
        }
        
        // Load other settings...
    }

    /**
     * Save settings from modal
     */
    saveSettings() {
        // Get values from form
        const themeSelect = document.getElementById('theme-select');
        const autoSaveDelay = document.getElementById('auto-save-delay');
        
        if (themeSelect) {
            this.setTheme(themeSelect.value);
        }
        
        if (autoSaveDelay) {
            // Update auto-save delay
            // This would require updating WriteMagic config
        }
        
        // Save preferences
        this.saveUserPreferences();
        
        // Close modal
        this.closeModal('settings-modal');
        
        this.showNotification('Settings saved', 'success');
    }

    /**
     * Apply theme
     */
    applyTheme(theme = null) {
        const themeToApply = theme || this.getCurrentTheme();
        
        // Remove existing theme classes
        document.body.className = document.body.className
            .replace(/theme-\w+/g, '');
        
        // Add new theme class
        document.body.classList.add(`theme-${themeToApply}`);
        
        // Save preference
        if (theme) {
            this.saveUserPreferences();
        }
    }

    /**
     * Set theme
     */
    setTheme(theme) {
        this.applyTheme(theme);
    }

    /**
     * Get current theme
     */
    getCurrentTheme() {
        const classList = document.body.classList;
        for (const className of classList) {
            if (className.startsWith('theme-')) {
                return className.replace('theme-', '');
            }
        }
        return 'light'; // default
    }

    /**
     * Handle system theme change
     */
    handleSystemThemeChange(e) {
        if (this.getCurrentTheme() === 'auto') {
            // Auto theme follows system preference
            const theme = e.matches ? 'dark' : 'light';
            document.body.className = document.body.className
                .replace(/theme-(light|dark)/g, '');
            document.body.classList.add(`theme-${theme}`);
        }
    }

    /**
     * Create new document
     */
    async createNewDocument() {
        try {
            const document = await this.writeMagic.createDocument({
                title: 'Untitled Document',
                content: '',
                type: 'markdown'
            });
            
            this.currentDocument = document;
            this.updateDocumentUI();
            
            // Focus editor
            if (this.elements.mainEditor) {
                this.elements.mainEditor.focus();
            }
            
            this.showNotification('New document created', 'success');
            
        } catch (error) {
            console.error('Failed to create document:', error);
            this.showError('Failed to create new document');
        }
    }

    /**
     * Create new project
     */
    async createNewProject() {
        const projectName = prompt('Enter project name:');
        if (!projectName) return;
        
        try {
            const project = await this.writeMagic.createProject(projectName);
            this.currentProject = project;
            
            // Update project list UI
            this.updateProjectList();
            
            this.showNotification(`Project "${projectName}" created`, 'success');
            
        } catch (error) {
            console.error('Failed to create project:', error);
            this.showError('Failed to create new project');
        }
    }

    /**
     * Save current document
     */
    async saveDocument() {
        if (!this.currentDocument) return;
        
        try {
            await this.writeMagic.saveDocument(this.currentDocument.id);
            this.updateSaveStatus('saved');
            this.state.unsavedChanges = false;
            
            this.showNotification('Document saved', 'success');
            
        } catch (error) {
            console.error('Failed to save document:', error);
            this.updateSaveStatus('error');
            this.showError('Failed to save document');
        }
    }

    /**
     * Update document UI elements
     */
    updateDocumentUI() {
        if (!this.currentDocument) return;
        
        // Update document title
        if (this.elements.editorDocumentTitle) {
            this.elements.editorDocumentTitle.textContent = this.currentDocument.title;
        }
        
        if (this.elements.currentDocument) {
            this.elements.currentDocument.textContent = this.currentDocument.title;
        }
        
        // Load document content
        if (this.elements.mainEditor && this.currentDocument.content) {
            this.elements.mainEditor.value = this.currentDocument.content;
            this.updateWordCount(this.currentDocument.content);
        }
        
        // Update page title
        document.title = `${this.currentDocument.title} - WriteMagic`;
    }

    /**
     * Update project list UI
     */
    updateProjectList() {
        // Implementation would update the project sidebar
        // This is a placeholder for the UI update logic
    }

    /**
     * Show notification
     */
    showNotification(message, type = 'info', duration = 3000) {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;
        
        // Add to page
        document.body.appendChild(notification);
        
        // Animate in
        setTimeout(() => notification.classList.add('show'), 100);
        
        // Remove after duration
        setTimeout(() => {
            notification.classList.remove('show');
            setTimeout(() => notification.remove(), 300);
        }, duration);
        
        // Also announce to screen readers
        this.announceToScreenReader(message);
    }

    /**
     * Show error message
     */
    showError(message) {
        this.showNotification(message, 'error', 5000);
    }

    /**
     * Announce to screen readers
     */
    announceToScreenReader(message) {
        const announcer = this.elements.srAnnouncements;
        if (announcer) {
            announcer.textContent = message;
            
            // Clear after announcement
            setTimeout(() => {
                announcer.textContent = '';
            }, 1000);
        }
    }

    /**
     * Handle window resize
     */
    handleWindowResize() {
        // Update layout based on window size
        if (window.innerWidth < 768 && !this.state.sidebarCollapsed) {
            this.setSidebarState(true);
        }
    }

    /**
     * Handle before unload (warn about unsaved changes)
     */
    handleBeforeUnload(event) {
        if (this.state.unsavedChanges) {
            event.preventDefault();
            event.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
            return event.returnValue;
        }
    }

    /**
     * Handle online/offline status
     */
    handleOnlineStatus(isOnline) {
        const statusMessage = isOnline ? 'Back online' : 'You are offline';
        const statusType = isOnline ? 'success' : 'warning';
        
        this.showNotification(statusMessage, statusType);
        
        // Update AI status based on connectivity
        if (!isOnline) {
            const aiStatus = this.elements.aiStatus;
            if (aiStatus) {
                aiStatus.classList.remove('online');
                aiStatus.classList.add('offline');
                aiStatus.setAttribute('aria-label', 'AI service offline (no connection)');
            }
        } else {
            // Recheck AI health when back online
            this.monitorAIHealth();
        }
    }

    /**
     * Toggle fullscreen
     */
    toggleFullscreen() {
        if (document.fullscreenElement) {
            document.exitFullscreen();
        } else {
            document.documentElement.requestFullscreen();
        }
    }

    /**
     * Get accessibility preferences
     */
    getAccessibilityPreferences() {
        return {
            highContrast: document.body.classList.contains('high-contrast'),
            largeText: document.body.classList.contains('large-text'),
            reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
            screenReader: document.body.classList.contains('screen-reader-mode')
        };
    }

    /**
     * Apply accessibility preferences
     */
    applyAccessibilityPreferences(prefs) {
        if (prefs.highContrast) {
            document.body.classList.add('high-contrast');
        }
        
        if (prefs.largeText) {
            document.body.classList.add('large-text');
        }
        
        if (prefs.screenReader) {
            document.body.classList.add('screen-reader-mode');
        }
    }
}

// Initialize the application when DOM is loaded
const app = new WriteMagicApp();

// Export for potential external access
window.WriteMagicApp = app;