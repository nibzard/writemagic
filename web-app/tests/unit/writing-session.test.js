/**
 * Unit tests for WritingSession module
 * Tests writing session management, analytics, and WASM integration
 */

import { WritingSession } from '@/writing-session.js';

describe('WritingSession', () => {
  let writingSession;

  beforeEach(() => {
    // Mock WASM WritingSession module
    global.writemagic_wasm.WritingSession.new.mockReturnValue({
      start: jest.fn(),
      pause: jest.fn(),
      get_analytics: jest.fn().mockReturnValue('{}'),
      free: jest.fn()
    });

    writingSession = new WritingSession();
  });

  describe('initialization', () => {
    test('should initialize with default state', () => {
      expect(writingSession.isActive).toBe(false);
      expect(writingSession.startTime).toBeNull();
      expect(writingSession.totalDuration).toBe(0);
    });

    test('should initialize WASM instance', () => {
      expect(writingSession.wasmInstance).toBeDefined();
      expect(global.writemagic_wasm.WritingSession.new).toHaveBeenCalled();
    });
  });

  describe('session lifecycle', () => {
    test('should start writing session', async () => {
      const result = await writingSession.start();
      
      expect(result).toBe(true);
      expect(writingSession.isActive).toBe(true);
      expect(writingSession.startTime).toBeDefined();
      expect(writingSession.wasmInstance.start).toHaveBeenCalled();
    });

    test('should not start session if already active', async () => {
      await writingSession.start();
      const result = await writingSession.start();
      
      expect(result).toBe(false);
    });

    test('should pause writing session', async () => {
      await writingSession.start();
      
      // Wait a bit to accumulate some duration
      await new Promise(resolve => setTimeout(resolve, 10));
      
      const result = await writingSession.pause();
      
      expect(result).toBe(true);
      expect(writingSession.isActive).toBe(false);
      expect(writingSession.totalDuration).toBeGreaterThan(0);
      expect(writingSession.wasmInstance.pause).toHaveBeenCalled();
    });

    test('should resume paused session', async () => {
      await writingSession.start();
      await writingSession.pause();
      const result = await writingSession.resume();
      
      expect(result).toBe(true);
      expect(writingSession.isActive).toBe(true);
      expect(writingSession.startTime).toBeDefined();
    });

    test('should stop session completely', async () => {
      await writingSession.start();
      const result = await writingSession.stop();
      
      expect(result).toBe(true);
      expect(writingSession.isActive).toBe(false);
      expect(writingSession.startTime).toBeNull();
    });
  });

  describe('session analytics', () => {
    beforeEach(async () => {
      await writingSession.start();
    });

    test('should track writing duration', async () => {
      await new Promise(resolve => setTimeout(resolve, 50));
      await writingSession.pause();
      
      const duration = writingSession.getDuration();
      expect(duration).toBeGreaterThan(40); // Account for timing variance
    });

    test('should track words written', async () => {
      writingSession.addContent('Hello world this is a test');
      
      const wordCount = writingSession.getWordCount();
      expect(wordCount).toBe(7);
    });

    test('should track characters written', async () => {
      const content = 'Hello world!';
      writingSession.addContent(content);
      
      const charCount = writingSession.getCharacterCount();
      expect(charCount).toBe(content.length);
    });

    test('should calculate typing speed (WPM)', async () => {
      writingSession.addContent('Hello world this is a test sentence with multiple words');
      
      // Simulate 1 minute of writing
      writingSession.totalDuration = 60000;
      
      const wpm = writingSession.getWordsPerMinute();
      expect(wpm).toBe(11); // 11 words in 1 minute
    });

    test('should track typing bursts and pauses', async () => {
      writingSession.addContent('First burst');
      await new Promise(resolve => setTimeout(resolve, 100));
      writingSession.addContent(' second burst');
      
      const bursts = writingSession.getTypingBursts();
      expect(bursts.length).toBeGreaterThanOrEqual(2);
    });

    test('should get comprehensive analytics', async () => {
      writingSession.addContent('Sample content for analytics testing');
      await new Promise(resolve => setTimeout(resolve, 50));
      
      const analytics = await writingSession.getAnalytics();
      
      expect(analytics).toHaveProperty('duration');
      expect(analytics).toHaveProperty('wordCount');
      expect(analytics).toHaveProperty('characterCount');
      expect(analytics).toHaveProperty('wordsPerMinute');
      expect(analytics).toHaveProperty('startTime');
      expect(analytics.wordCount).toBe(5);
    });
  });

  describe('content tracking', () => {
    test('should track content additions', () => {
      const content = 'New content added';
      writingSession.addContent(content);
      
      expect(writingSession.contentHistory).toContain(content);
      expect(writingSession.currentContent).toContain(content);
    });

    test('should track content deletions', () => {
      writingSession.addContent('Original content');
      writingSession.deleteContent(9); // Delete 'content'
      
      expect(writingSession.currentContent).toBe('Original ');
      expect(writingSession.deletions).toBe(1);
    });

    test('should track backspace usage', () => {
      writingSession.addContent('Test content');
      writingSession.handleBackspace(4); // Delete 'tent'
      
      expect(writingSession.backspaceCount).toBe(4);
      expect(writingSession.currentContent).toBe('Test con');
    });

    test('should track copy/paste operations', () => {
      writingSession.handlePaste('Pasted content');
      
      expect(writingSession.pasteCount).toBe(1);
      expect(writingSession.currentContent).toContain('Pasted content');
    });
  });

  describe('session goals and targets', () => {
    test('should set word count goal', () => {
      writingSession.setWordGoal(500);
      
      expect(writingSession.wordGoal).toBe(500);
    });

    test('should check goal progress', () => {
      writingSession.setWordGoal(10);
      writingSession.addContent('This is a test with more than ten words here');
      
      const progress = writingSession.getGoalProgress();
      expect(progress.achieved).toBe(true);
      expect(progress.percentage).toBeGreaterThan(100);
    });

    test('should set time goal', () => {
      writingSession.setTimeGoal(30); // 30 minutes
      
      expect(writingSession.timeGoal).toBe(30 * 60 * 1000); // Convert to milliseconds
    });

    test('should check time goal progress', async () => {
      writingSession.setTimeGoal(1); // 1 minute goal
      await writingSession.start();
      
      // Simulate 30 seconds of writing
      writingSession.totalDuration = 30000;
      
      const progress = writingSession.getTimeProgress();
      expect(progress.percentage).toBe(50);
      expect(progress.achieved).toBe(false);
    });
  });

  describe('break reminders', () => {
    test('should set break reminder interval', () => {
      writingSession.setBreakReminder(25); // 25 minutes
      
      expect(writingSession.breakInterval).toBe(25 * 60 * 1000);
    });

    test('should trigger break reminder', async () => {
      const breakHandler = jest.fn();
      writingSession.on('breakReminder', breakHandler);
      
      writingSession.setBreakReminder(0.1); // 0.1 minutes = 6 seconds
      await writingSession.start();
      
      // Simulate time passing
      writingSession.totalDuration = 7000; // 7 seconds
      writingSession.checkBreakReminder();
      
      expect(breakHandler).toHaveBeenCalled();
    });

    test('should track break history', async () => {
      await writingSession.start();
      await writingSession.takeBreak();
      
      expect(writingSession.breakHistory).toHaveLength(1);
      expect(writingSession.breakHistory[0]).toHaveProperty('startTime');
      expect(writingSession.isOnBreak).toBe(true);
    });

    test('should resume from break', async () => {
      await writingSession.start();
      await writingSession.takeBreak();
      await writingSession.resumeFromBreak();
      
      expect(writingSession.isOnBreak).toBe(false);
      expect(writingSession.breakHistory[0]).toHaveProperty('endTime');
    });
  });

  describe('distraction tracking', () => {
    test('should track window focus events', () => {
      writingSession.trackFocus();
      
      // Simulate window blur (distraction)
      window.dispatchEvent(new Event('blur'));
      
      expect(writingSession.distractions).toBe(1);
      expect(writingSession.focusTime).toBeDefined();
    });

    test('should calculate focus percentage', async () => {
      await writingSession.start();
      writingSession.totalDuration = 60000; // 1 minute
      writingSession.focusTime = 45000; // 45 seconds focused
      
      const focusPercentage = writingSession.getFocusPercentage();
      expect(focusPercentage).toBe(75);
    });

    test('should track tab switches', () => {
      document.dispatchEvent(new Event('visibilitychange'));
      
      expect(writingSession.tabSwitches).toBe(1);
    });
  });

  describe('session persistence', () => {
    test('should save session state', async () => {
      await writingSession.start();
      writingSession.addContent('Test content for saving');
      
      const savedState = await writingSession.saveState();
      
      expect(savedState).toHaveProperty('isActive');
      expect(savedState).toHaveProperty('startTime');
      expect(savedState).toHaveProperty('totalDuration');
      expect(savedState).toHaveProperty('currentContent');
      expect(savedState).toHaveProperty('wordCount');
      expect(savedState.currentContent).toBe('Test content for saving');
    });

    test('should restore session state', async () => {
      const savedState = {
        isActive: true,
        startTime: Date.now() - 30000,
        totalDuration: 30000,
        currentContent: 'Restored content',
        wordCount: 2,
        characterCount: 16
      };
      
      const result = await writingSession.restoreState(savedState);
      
      expect(result).toBe(true);
      expect(writingSession.currentContent).toBe('Restored content');
      expect(writingSession.totalDuration).toBe(30000);
    });

    test('should auto-save session periodically', async () => {
      const saveHandler = jest.fn();
      writingSession.on('sessionSaved', saveHandler);
      
      writingSession.enableAutoSave(100); // Save every 100ms
      await writingSession.start();
      writingSession.addContent('Auto-save test');
      
      await new Promise(resolve => setTimeout(resolve, 150));
      
      expect(saveHandler).toHaveBeenCalled();
    });
  });

  describe('session comparison', () => {
    test('should compare with previous sessions', async () => {
      const previousSession = {
        duration: 1800000, // 30 minutes
        wordCount: 500,
        wordsPerMinute: 16.67,
        focusPercentage: 85
      };
      
      writingSession.totalDuration = 2400000; // 40 minutes
      writingSession.addContent('Current session content with many more words than the previous test session had originally');
      
      const comparison = writingSession.compareWithPrevious(previousSession);
      
      expect(comparison.duration.improved).toBe(true);
      expect(comparison.wordCount.improved).toBe(true);
      expect(comparison.duration.percentage).toBeGreaterThan(0);
    });

    test('should calculate session streaks', () => {
      const sessionHistory = [
        { date: '2023-12-01', completed: true },
        { date: '2023-12-02', completed: true },
        { date: '2023-12-03', completed: false },
        { date: '2023-12-04', completed: true }
      ];
      
      const streak = writingSession.calculateStreak(sessionHistory);
      expect(streak.current).toBe(1);
      expect(streak.longest).toBe(2);
    });
  });

  describe('event handling', () => {
    test('should emit session started event', async () => {
      const startHandler = jest.fn();
      writingSession.on('sessionStarted', startHandler);
      
      await writingSession.start();
      
      expect(startHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          startTime: expect.any(Number),
          sessionId: expect.any(String)
        })
      );
    });

    test('should emit goal achieved event', () => {
      const goalHandler = jest.fn();
      writingSession.on('goalAchieved', goalHandler);
      
      writingSession.setWordGoal(5);
      writingSession.addContent('This has exactly five words');
      
      expect(goalHandler).toHaveBeenCalledWith({
        type: 'word',
        goal: 5,
        achieved: 5
      });
    });

    test('should emit milestone events', () => {
      const milestoneHandler = jest.fn();
      writingSession.on('milestone', milestoneHandler);
      
      // Add content to trigger word count milestones
      let content = '';
      for (let i = 0; i < 100; i++) {
        content += `word${i} `;
      }
      writingSession.addContent(content);
      
      expect(milestoneHandler).toHaveBeenCalled();
    });
  });

  describe('error handling', () => {
    test('should handle WASM session creation failure', () => {
      global.writemagic_wasm.WritingSession.new.mockImplementation(() => {
        throw new Error('WASM WritingSession creation failed');
      });
      
      expect(() => new WritingSession()).toThrow('WASM WritingSession creation failed');
    });

    test('should handle invalid session operations', async () => {
      // Try to pause a session that was never started
      const result = await writingSession.pause();
      expect(result).toBe(false);
    });

    test('should handle corrupted session state gracefully', async () => {
      const corruptedState = { invalid: 'data' };
      const result = await writingSession.restoreState(corruptedState);
      
      expect(result).toBe(false);
      expect(writingSession.isActive).toBe(false);
    });
  });
});