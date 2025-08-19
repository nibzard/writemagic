/**
 * Writing Session - Session management, draft handling, recovery
 * 
 * This module manages writing sessions, providing state persistence,
 * draft recovery, session analytics, and seamless user experience
 * across browser refreshes and device switches.
 */

import { EventEmitter } from './utils/event-emitter.js';
import { debounce } from './utils/debounce.js';

/**
 * Session states for user feedback
 */
export const SessionState = {
    ACTIVE: 'active',
    PAUSED: 'paused',
    IDLE: 'idle',
    ENDED: 'ended',
    RECOVERING: 'recovering'
};

/**
 * Session events for UI coordination
 */
export const SessionEvents = {
    SESSION_STARTED: 'session_started',
    SESSION_PAUSED: 'session_paused',
    SESSION_RESUMED: 'session_resumed',
    SESSION_ENDED: 'session_ended',
    STATE_SAVED: 'state_saved',
    STATE_RESTORED: 'state_restored',
    DRAFT_RECOVERED: 'draft_recovered',
    IDLE_DETECTED: 'idle_detected',
    FOCUS_SESSION_STARTED: 'focus_session_started',
    FOCUS_SESSION_ENDED: 'focus_session_ended',
    GOAL_ACHIEVED: 'goal_achieved',
    ERROR: 'error'
};

/**
 * Default session configuration
 */
export const DEFAULT_SESSION_CONFIG = {
    autoSaveInterval: 30000,        // 30 seconds
    idleTimeout: 300000,           // 5 minutes
    sessionTimeout: 3600000,       // 1 hour
    maxDraftHistory: 100,          // Maximum drafts to keep
    enableFocusMode: true,         // Enable focus sessions
    enableGoalTracking: true,      // Track session goals
    enableAnalytics: true,         // Track writing analytics
    compressionEnabled: true,      // Compress large drafts
    syncAcrossDevices: false       // Cross-device synchronization
};

/**
 * Focus session types
 */
export const FocusSessionType = {
    POMODORO: {
        name: 'Pomodoro',
        duration: 25 * 60 * 1000,  // 25 minutes
        breakDuration: 5 * 60 * 1000 // 5 minutes
    },
    SPRINT: {
        name: 'Sprint',
        duration: 15 * 60 * 1000,  // 15 minutes
        breakDuration: 3 * 60 * 1000 // 3 minutes
    },
    DEEP_WORK: {
        name: 'Deep Work',
        duration: 90 * 60 * 1000,  // 90 minutes
        breakDuration: 20 * 60 * 1000 // 20 minutes
    },
    CUSTOM: {
        name: 'Custom',
        duration: null,
        breakDuration: null
    }
};

/**
 * WritingSession - Comprehensive session and state management
 * 
 * Features:
 * - Automatic session state persistence
 * - Draft recovery and history management
 * - Focus mode with customizable time blocks
 * - Writing goal tracking within sessions
 * - Idle detection and pause handling
 * - Cross-tab synchronization
 * - Session analytics and insights
 * - Seamless recovery from crashes or refreshes
 */
export class WritingSession extends EventEmitter {
    constructor(documentManager, projectWorkspace, config = {}) {
        super();
        
        this.documentManager = documentManager;
        this.projectWorkspace = projectWorkspace;
        this.config = { ...DEFAULT_SESSION_CONFIG, ...config };
        
        // Session state
        this.sessionId = this.generateSessionId();
        this.state = SessionState.IDLE;
        this.startTime = null;
        this.lastActivity = Date.now();
        this.totalActiveTime = 0;
        this.pausedTime = 0;
        
        // Current session data
        this.currentSession = null;
        this.sessionGoals = new Map();
        this.sessionProgress = {
            wordsWritten: 0,
            charactersTyped: 0,
            documentsCreated: 0,
            documentsEdited: new Set(),
            timeSpent: 0
        };
        
        // Focus mode
        this.focusSession = null;
        this.focusTimer = null;
        this.isInFocusMode = false;
        
        // Draft management
        this.draftHistory = new Map(); // documentId -> drafts array
        this.draftTimer = null;
        
        // Activity monitoring
        this.idleTimer = null;
        this.activityListeners = new Set();
        
        // Auto-save timer
        this.autoSaveTimer = null;
        
        // Initialize session management
        this.setupActivityMonitoring();
        this.setupAutoSave();
        this.setupUnloadHandlers();
        this.setupCrossTabSync();
        
        // Attempt to recover previous session
        this.attemptSessionRecovery();
    }

    /**
     * Start a new writing session
     */
    startSession(options = {}) {
        try {
            const {
                projectId = null,
                goals = {},
                focusMode = null,
                description = null
            } = options;

            // End any existing session
            if (this.currentSession) {
                this.endSession();
            }

            // Create new session
            this.currentSession = {
                id: this.sessionId,
                projectId,
                startTime: Date.now(),
                endTime: null,
                description,
                goals: new Map(Object.entries(goals)),
                progress: { ...this.sessionProgress },
                state: SessionState.ACTIVE,
                metadata: {
                    userAgent: navigator.userAgent,
                    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
                    version: '1.0.0'
                }
            };

            // Reset session progress
            this.resetSessionProgress();

            // Set session state
            this.state = SessionState.ACTIVE;
            this.startTime = Date.now();
            this.lastActivity = this.startTime;

            // Start focus mode if requested
            if (focusMode && this.config.enableFocusMode) {
                this.startFocusSession(focusMode);
            }

            // Start monitoring and auto-save
            this.startActivityMonitoring();
            this.startAutoSave();

            // Save initial session state
            this.saveSessionState();

            this.emit(SessionEvents.SESSION_STARTED, {
                sessionId: this.sessionId,
                session: this.currentSession
            });

            return this.currentSession;

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'start_session' });
            throw error;
        }
    }

    /**
     * Pause current session
     */
    pauseSession(reason = 'manual') {
        if (!this.currentSession || this.state === SessionState.PAUSED) {
            return;
        }

        try {
            const now = Date.now();
            
            // Update active time
            if (this.state === SessionState.ACTIVE) {
                this.totalActiveTime += now - this.lastActivity;
            }

            this.state = SessionState.PAUSED;
            this.currentSession.state = SessionState.PAUSED;
            this.lastActivity = now;

            // Pause focus session if active
            if (this.isInFocusMode) {
                this.pauseFocusSession();
            }

            // Stop monitoring but keep auto-save for state persistence
            this.stopActivityMonitoring();

            this.saveSessionState();
            this.emit(SessionEvents.SESSION_PAUSED, { 
                sessionId: this.sessionId, 
                reason,
                totalActiveTime: this.totalActiveTime
            });

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'pause_session' });
            throw error;
        }
    }

    /**
     * Resume paused session
     */
    resumeSession() {
        if (!this.currentSession || this.state !== SessionState.PAUSED) {
            return;
        }

        try {
            this.state = SessionState.ACTIVE;
            this.currentSession.state = SessionState.ACTIVE;
            this.lastActivity = Date.now();

            // Resume focus session if it was active
            if (this.focusSession && this.focusSession.isPaused) {
                this.resumeFocusSession();
            }

            // Restart monitoring
            this.startActivityMonitoring();

            this.saveSessionState();
            this.emit(SessionEvents.SESSION_RESUMED, { 
                sessionId: this.sessionId,
                resumeTime: Date.now()
            });

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'resume_session' });
            throw error;
        }
    }

    /**
     * End current session
     */
    endSession(options = {}) {
        if (!this.currentSession) {
            return null;
        }

        try {
            const { saveSession = true, reason = 'manual' } = options;
            const now = Date.now();

            // Update final active time
            if (this.state === SessionState.ACTIVE) {
                this.totalActiveTime += now - this.lastActivity;
            }

            // End focus session if active
            if (this.isInFocusMode) {
                this.endFocusSession();
            }

            // Finalize session data
            this.currentSession.endTime = now;
            this.currentSession.totalActiveTime = this.totalActiveTime;
            this.currentSession.state = SessionState.ENDED;
            this.currentSession.progress = { ...this.sessionProgress };

            // Calculate session statistics
            const sessionStats = this.calculateSessionStats();
            this.currentSession.statistics = sessionStats;

            const completedSession = { ...this.currentSession };

            // Save to history if requested
            if (saveSession) {
                this.saveSessionToHistory(completedSession);
            }

            // Clean up current session
            this.state = SessionState.ENDED;
            this.stopAllTimers();
            this.stopActivityMonitoring();

            // Clear session state
            this.clearSessionState();

            this.emit(SessionEvents.SESSION_ENDED, {
                sessionId: this.sessionId,
                session: completedSession,
                reason,
                statistics: sessionStats
            });

            // Reset for next session
            this.currentSession = null;
            this.resetSessionProgress();
            this.generateNewSessionId();

            return completedSession;

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'end_session' });
            throw error;
        }
    }

    /**
     * Start focus session (Pomodoro, Sprint, etc.)
     */
    startFocusSession(focusType, customDuration = null) {
        try {
            if (this.isInFocusMode) {
                this.endFocusSession();
            }

            const sessionType = typeof focusType === 'string' ? 
                FocusSessionType[focusType.toUpperCase()] : focusType;

            if (!sessionType) {
                throw new Error('Invalid focus session type');
            }

            const duration = customDuration || sessionType.duration;
            
            this.focusSession = {
                type: sessionType.name,
                duration,
                breakDuration: sessionType.breakDuration,
                startTime: Date.now(),
                endTime: Date.now() + duration,
                isPaused: false,
                pausedTime: 0,
                isBreak: false
            };

            this.isInFocusMode = true;

            // Start focus timer
            this.focusTimer = setTimeout(() => {
                this.handleFocusSessionEnd();
            }, duration);

            this.emit(SessionEvents.FOCUS_SESSION_STARTED, {
                focusSession: this.focusSession,
                duration
            });

            // Save state
            this.saveSessionState();

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'start_focus_session' });
            throw error;
        }
    }

    /**
     * End focus session
     */
    endFocusSession() {
        if (!this.isInFocusMode || !this.focusSession) {
            return;
        }

        try {
            if (this.focusTimer) {
                clearTimeout(this.focusTimer);
                this.focusTimer = null;
            }

            const completedFocusSession = { ...this.focusSession };
            completedFocusSession.endTime = Date.now();
            completedFocusSession.completed = true;

            this.isInFocusMode = false;
            this.focusSession = null;

            this.emit(SessionEvents.FOCUS_SESSION_ENDED, {
                focusSession: completedFocusSession,
                completed: true
            });

            this.saveSessionState();

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'end_focus_session' });
            throw error;
        }
    }

    /**
     * Update session progress with document changes
     */
    updateProgress(documentId, changeData) {
        if (!this.currentSession) return;

        try {
            const { wordsAdded = 0, charactersAdded = 0, isNewDocument = false } = changeData;

            // Update progress counters
            this.sessionProgress.wordsWritten += Math.max(0, wordsAdded);
            this.sessionProgress.charactersTyped += Math.max(0, charactersAdded);
            
            if (isNewDocument) {
                this.sessionProgress.documentsCreated++;
            } else {
                this.sessionProgress.documentsEdited.add(documentId);
            }

            // Update session timestamp
            this.lastActivity = Date.now();
            this.currentSession.progress = { ...this.sessionProgress };

            // Check goal achievements
            this.checkGoalAchievements();

            // Reset idle timer
            this.resetIdleTimer();

            // Save state
            this.debouncedStateSave();

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'update_progress' });
        }
    }

    /**
     * Set session goal
     */
    setSessionGoal(type, target, description = null) {
        if (!this.currentSession) {
            throw new Error('No active session');
        }

        const goalId = `${type}_${Date.now()}`;
        const goal = {
            id: goalId,
            type, // 'words', 'time', 'documents', 'focus_sessions'
            target,
            description,
            achieved: false,
            progress: 0,
            createdAt: Date.now()
        };

        this.sessionGoals.set(goalId, goal);
        this.currentSession.goals.set(goalId, goal);

        this.saveSessionState();
        return goalId;
    }

    /**
     * Save draft for recovery
     */
    saveDraft(documentId, content, metadata = {}) {
        try {
            const draft = {
                id: `draft_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
                documentId,
                content,
                timestamp: Date.now(),
                sessionId: this.sessionId,
                wordCount: this.countWords(content),
                characterCount: content.length,
                metadata: {
                    ...metadata,
                    userAgent: navigator.userAgent,
                    url: window.location.href
                }
            };

            // Add to draft history
            if (!this.draftHistory.has(documentId)) {
                this.draftHistory.set(documentId, []);
            }

            const drafts = this.draftHistory.get(documentId);
            drafts.unshift(draft);

            // Limit draft history size
            if (drafts.length > this.config.maxDraftHistory) {
                drafts.splice(this.config.maxDraftHistory);
            }

            // Save to localStorage
            this.saveDraftToStorage(documentId, drafts);

            return draft;

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'save_draft' });
            throw error;
        }
    }

    /**
     * Recover drafts for document
     */
    recoverDrafts(documentId) {
        try {
            // Load from storage
            const storedDrafts = this.loadDraftsFromStorage(documentId);
            if (storedDrafts.length > 0) {
                this.draftHistory.set(documentId, storedDrafts);
                return storedDrafts;
            }

            // Return cached drafts
            return this.draftHistory.get(documentId) || [];

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'recover_drafts' });
            return [];
        }
    }

    /**
     * Get session statistics
     */
    getSessionStats() {
        if (!this.currentSession) return null;

        const now = Date.now();
        const activeTime = this.state === SessionState.ACTIVE ? 
            this.totalActiveTime + (now - this.lastActivity) : 
            this.totalActiveTime;

        return {
            sessionId: this.sessionId,
            duration: now - this.currentSession.startTime,
            activeTime,
            pausedTime: this.pausedTime,
            state: this.state,
            progress: { ...this.sessionProgress },
            goals: Array.from(this.sessionGoals.values()),
            focusSession: this.focusSession,
            isInFocusMode: this.isInFocusMode
        };
    }

    /**
     * Get session history
     */
    getSessionHistory(options = {}) {
        const { limit = 50, projectId = null, dateRange = null } = options;
        
        try {
            const history = this.loadSessionHistory();
            let filtered = history;

            // Filter by project
            if (projectId) {
                filtered = filtered.filter(session => session.projectId === projectId);
            }

            // Filter by date range
            if (dateRange) {
                const { start, end } = dateRange;
                filtered = filtered.filter(session => 
                    session.startTime >= start && session.startTime <= end
                );
            }

            // Sort by start time (most recent first)
            filtered.sort((a, b) => b.startTime - a.startTime);

            // Limit results
            return filtered.slice(0, limit);

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'get_session_history' });
            return [];
        }
    }

    // Private methods

    generateSessionId() {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    generateNewSessionId() {
        this.sessionId = this.generateSessionId();
    }

    resetSessionProgress() {
        this.sessionProgress = {
            wordsWritten: 0,
            charactersTyped: 0,
            documentsCreated: 0,
            documentsEdited: new Set(),
            timeSpent: 0
        };
        this.sessionGoals.clear();
        this.totalActiveTime = 0;
        this.pausedTime = 0;
    }

    setupActivityMonitoring() {
        const activityEvents = ['mousedown', 'mousemove', 'keypress', 'scroll', 'touchstart'];
        
        const activityHandler = debounce(() => {
            this.handleActivity();
        }, 1000);

        activityEvents.forEach(event => {
            document.addEventListener(event, activityHandler, true);
            this.activityListeners.add({ event, handler: activityHandler });
        });
    }

    startActivityMonitoring() {
        this.resetIdleTimer();
    }

    stopActivityMonitoring() {
        if (this.idleTimer) {
            clearTimeout(this.idleTimer);
            this.idleTimer = null;
        }
    }

    handleActivity() {
        if (this.state === SessionState.IDLE) {
            this.resumeSession();
        }

        this.lastActivity = Date.now();
        this.resetIdleTimer();
    }

    resetIdleTimer() {
        if (this.idleTimer) {
            clearTimeout(this.idleTimer);
        }

        this.idleTimer = setTimeout(() => {
            this.handleIdleTimeout();
        }, this.config.idleTimeout);
    }

    handleIdleTimeout() {
        if (this.state === SessionState.ACTIVE) {
            this.state = SessionState.IDLE;
            this.emit(SessionEvents.IDLE_DETECTED, {
                sessionId: this.sessionId,
                idleTime: this.config.idleTimeout
            });
        }
    }

    setupAutoSave() {
        this.debouncedStateSave = debounce(() => {
            this.saveSessionState();
        }, 5000);
    }

    startAutoSave() {
        this.autoSaveTimer = setInterval(() => {
            this.saveSessionState();
        }, this.config.autoSaveInterval);
    }

    stopAllTimers() {
        if (this.autoSaveTimer) {
            clearInterval(this.autoSaveTimer);
            this.autoSaveTimer = null;
        }

        if (this.focusTimer) {
            clearTimeout(this.focusTimer);
            this.focusTimer = null;
        }

        if (this.idleTimer) {
            clearTimeout(this.idleTimer);
            this.idleTimer = null;
        }
    }

    setupUnloadHandlers() {
        window.addEventListener('beforeunload', () => {
            if (this.currentSession) {
                this.saveSessionState(true); // Force immediate save
            }
        });

        // Handle visibility changes
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                this.pauseSession('visibility_hidden');
            } else if (this.state === SessionState.PAUSED) {
                this.resumeSession();
            }
        });
    }

    setupCrossTabSync() {
        if (!this.config.syncAcrossDevices) return;

        // Listen for storage changes to sync across tabs
        window.addEventListener('storage', (e) => {
            if (e.key === 'writemagic_current_session') {
                // Handle cross-tab session synchronization
                this.handleCrossTabSync(e.newValue);
            }
        });
    }

    saveSessionState(immediate = false) {
        if (!this.currentSession) return;

        try {
            const stateData = {
                session: this.currentSession,
                sessionId: this.sessionId,
                state: this.state,
                totalActiveTime: this.totalActiveTime,
                lastActivity: this.lastActivity,
                focusSession: this.focusSession,
                isInFocusMode: this.isInFocusMode,
                timestamp: Date.now()
            };

            localStorage.setItem('writemagic_current_session', JSON.stringify(stateData));
            
            if (!immediate) {
                this.emit(SessionEvents.STATE_SAVED, { sessionId: this.sessionId });
            }

        } catch (error) {
            console.warn('Failed to save session state:', error);
        }
    }

    attemptSessionRecovery() {
        try {
            const savedState = localStorage.getItem('writemagic_current_session');
            if (!savedState) return;

            const stateData = JSON.parse(savedState);
            const timeSinceLastActivity = Date.now() - stateData.lastActivity;

            // Only recover if activity was recent (within session timeout)
            if (timeSinceLastActivity < this.config.sessionTimeout) {
                this.restoreSession(stateData);
            } else {
                this.clearSessionState();
            }

        } catch (error) {
            console.warn('Failed to recover session:', error);
            this.clearSessionState();
        }
    }

    restoreSession(stateData) {
        try {
            this.sessionId = stateData.sessionId;
            this.currentSession = stateData.session;
            this.state = stateData.state;
            this.totalActiveTime = stateData.totalActiveTime;
            this.lastActivity = stateData.lastActivity;
            this.focusSession = stateData.focusSession;
            this.isInFocusMode = stateData.isInFocusMode;

            // Restore session goals
            if (this.currentSession.goals) {
                this.sessionGoals = new Map(this.currentSession.goals);
            }

            // Restore progress
            if (this.currentSession.progress) {
                this.sessionProgress = { 
                    ...this.currentSession.progress,
                    documentsEdited: new Set(this.currentSession.progress.documentsEdited)
                };
            }

            // Resume focus session if it was active
            if (this.isInFocusMode && this.focusSession) {
                const remainingTime = this.focusSession.endTime - Date.now();
                if (remainingTime > 0) {
                    this.focusTimer = setTimeout(() => {
                        this.handleFocusSessionEnd();
                    }, remainingTime);
                } else {
                    this.endFocusSession();
                }
            }

            // Resume monitoring if session was active
            if (this.state === SessionState.ACTIVE) {
                this.startActivityMonitoring();
                this.startAutoSave();
            }

            this.emit(SessionEvents.STATE_RESTORED, {
                sessionId: this.sessionId,
                session: this.currentSession
            });

        } catch (error) {
            this.emit(SessionEvents.ERROR, { error, operation: 'restore_session' });
            this.clearSessionState();
        }
    }

    clearSessionState() {
        try {
            localStorage.removeItem('writemagic_current_session');
        } catch (error) {
            console.warn('Failed to clear session state:', error);
        }
    }

    checkGoalAchievements() {
        for (const [goalId, goal] of this.sessionGoals.entries()) {
            if (goal.achieved) continue;

            let currentProgress = 0;

            switch (goal.type) {
                case 'words':
                    currentProgress = this.sessionProgress.wordsWritten;
                    break;
                case 'time':
                    currentProgress = this.totalActiveTime;
                    break;
                case 'documents':
                    currentProgress = this.sessionProgress.documentsCreated + this.sessionProgress.documentsEdited.size;
                    break;
                case 'characters':
                    currentProgress = this.sessionProgress.charactersTyped;
                    break;
            }

            goal.progress = currentProgress;

            if (currentProgress >= goal.target && !goal.achieved) {
                goal.achieved = true;
                goal.achievedAt = Date.now();
                
                this.emit(SessionEvents.GOAL_ACHIEVED, {
                    goalId,
                    goal,
                    sessionId: this.sessionId
                });
            }

            this.sessionGoals.set(goalId, goal);
        }
    }

    handleFocusSessionEnd() {
        if (!this.focusSession) return;

        const isCompleted = Date.now() >= this.focusSession.endTime;
        
        if (isCompleted && this.focusSession.breakDuration) {
            // Start break session
            this.startBreakSession();
        } else {
            this.endFocusSession();
        }
    }

    startBreakSession() {
        if (!this.focusSession || !this.focusSession.breakDuration) return;

        this.focusSession.isBreak = true;
        this.focusSession.breakStartTime = Date.now();
        this.focusSession.breakEndTime = Date.now() + this.focusSession.breakDuration;

        this.focusTimer = setTimeout(() => {
            this.endFocusSession();
        }, this.focusSession.breakDuration);

        this.emit('focus_break_started', {
            focusSession: this.focusSession,
            breakDuration: this.focusSession.breakDuration
        });
    }

    pauseFocusSession() {
        if (!this.focusSession || this.focusSession.isPaused) return;

        this.focusSession.isPaused = true;
        this.focusSession.pauseStartTime = Date.now();

        if (this.focusTimer) {
            clearTimeout(this.focusTimer);
            this.focusTimer = null;
        }
    }

    resumeFocusSession() {
        if (!this.focusSession || !this.focusSession.isPaused) return;

        const pauseDuration = Date.now() - this.focusSession.pauseStartTime;
        this.focusSession.pausedTime += pauseDuration;
        this.focusSession.endTime += pauseDuration;
        this.focusSession.isPaused = false;
        
        delete this.focusSession.pauseStartTime;

        const remainingTime = this.focusSession.endTime - Date.now();
        if (remainingTime > 0) {
            this.focusTimer = setTimeout(() => {
                this.handleFocusSessionEnd();
            }, remainingTime);
        } else {
            this.endFocusSession();
        }
    }

    calculateSessionStats() {
        if (!this.currentSession) return {};

        const duration = this.currentSession.endTime - this.currentSession.startTime;
        const activeTime = this.totalActiveTime;
        const efficiency = duration > 0 ? (activeTime / duration) * 100 : 0;

        return {
            totalDuration: duration,
            activeTime,
            efficiency: efficiency.toFixed(1),
            wordsPerMinute: activeTime > 0 ? (this.sessionProgress.wordsWritten / (activeTime / 60000)).toFixed(1) : 0,
            goalsAchieved: Array.from(this.sessionGoals.values()).filter(g => g.achieved).length,
            totalGoals: this.sessionGoals.size,
            focusSessionsCompleted: this.focusSession ? 1 : 0
        };
    }

    saveSessionToHistory(session) {
        try {
            const history = this.loadSessionHistory();
            history.unshift(session);

            // Keep only last 500 sessions
            if (history.length > 500) {
                history.splice(500);
            }

            localStorage.setItem('writemagic_session_history', JSON.stringify(history));
        } catch (error) {
            console.warn('Failed to save session to history:', error);
        }
    }

    loadSessionHistory() {
        try {
            const stored = localStorage.getItem('writemagic_session_history');
            return stored ? JSON.parse(stored) : [];
        } catch (error) {
            console.warn('Failed to load session history:', error);
            return [];
        }
    }

    saveDraftToStorage(documentId, drafts) {
        try {
            const key = `writemagic_drafts_${documentId}`;
            
            if (this.config.compressionEnabled && JSON.stringify(drafts).length > 50000) {
                // Compress large draft data (simple compression)
                const compressed = this.compressDrafts(drafts);
                localStorage.setItem(key, compressed);
            } else {
                localStorage.setItem(key, JSON.stringify(drafts));
            }
        } catch (error) {
            console.warn('Failed to save drafts:', error);
        }
    }

    loadDraftsFromStorage(documentId) {
        try {
            const key = `writemagic_drafts_${documentId}`;
            const stored = localStorage.getItem(key);
            
            if (!stored) return [];

            // Try to parse as JSON first
            try {
                return JSON.parse(stored);
            } catch {
                // If that fails, try decompression
                return this.decompressDrafts(stored);
            }
        } catch (error) {
            console.warn('Failed to load drafts:', error);
            return [];
        }
    }

    compressDrafts(drafts) {
        // Simple compression: keep only essential fields for older drafts
        return JSON.stringify(drafts.map((draft, index) => {
            if (index < 5) {
                // Keep full data for recent drafts
                return draft;
            } else {
                // Compress older drafts
                return {
                    id: draft.id,
                    documentId: draft.documentId,
                    timestamp: draft.timestamp,
                    wordCount: draft.wordCount,
                    characterCount: draft.characterCount,
                    contentPreview: draft.content.substring(0, 200) + '...'
                };
            }
        }));
    }

    decompressDrafts(compressed) {
        try {
            return JSON.parse(compressed);
        } catch (error) {
            console.warn('Failed to decompress drafts:', error);
            return [];
        }
    }

    countWords(text) {
        return text.split(/\s+/).filter(word => word.length > 0).length;
    }

    /**
     * Cleanup resources
     */
    destroy() {
        // End current session
        if (this.currentSession) {
            this.endSession({ saveSession: true, reason: 'destroy' });
        }

        // Clear all timers
        this.stopAllTimers();

        // Remove activity listeners
        for (const { event, handler } of this.activityListeners) {
            document.removeEventListener(event, handler, true);
        }
        this.activityListeners.clear();

        // Clear event listeners
        this.removeAllListeners();
    }
}

export default WritingSession;