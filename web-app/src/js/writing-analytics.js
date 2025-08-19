/**
 * Writing Analytics - Content analysis and metrics for writers
 * 
 * This module provides comprehensive analysis of writing content including
 * word counts, reading time estimates, complexity analysis, and writing
 * pattern insights to help writers improve their craft.
 */

/**
 * Reading speed constants (words per minute)
 */
const READING_SPEEDS = {
    SLOW: 200,      // Slow reader
    AVERAGE: 250,   // Average reader
    FAST: 300,      // Fast reader
    SKIMMING: 400   // Skimming speed
};

/**
 * Text complexity indicators
 */
const COMPLEXITY_THRESHOLDS = {
    SENTENCE_LENGTH: {
        SIMPLE: 15,
        MODERATE: 25,
        COMPLEX: 35
    },
    SYLLABLES_PER_WORD: {
        SIMPLE: 1.5,
        MODERATE: 2.0,
        COMPLEX: 2.5
    },
    PARAGRAPH_LENGTH: {
        SHORT: 3,
        MEDIUM: 6,
        LONG: 10
    }
};

/**
 * Content type patterns for analysis
 */
const CONTENT_PATTERNS = {
    HEADINGS: /^#{1,6}\s+(.+)$/gm,
    LISTS: /^[\s]*[-*+]\s+(.+)$/gm,
    NUMBERED_LISTS: /^\s*\d+\.\s+(.+)$/gm,
    LINKS: /\[([^\]]+)\]\([^)]+\)/g,
    EMPHASIS: /(\*\*[^*]+\*\*|__[^_]+__|_[^_]+_|\*[^*]+\*)/g,
    CODE_BLOCKS: /```[\s\S]*?```/g,
    INLINE_CODE: /`[^`]+`/g,
    BLOCKQUOTES: /^>\s+(.+)$/gm
};

/**
 * WritingAnalytics - Comprehensive content analysis for writers
 * 
 * Features:
 * - Detailed word and character counting
 * - Reading time estimation across different speeds
 * - Text complexity analysis (Flesch-Kincaid, etc.)
 * - Writing pattern analysis
 * - Productivity tracking and session metrics
 * - Goal tracking and progress monitoring
 */
export class WritingAnalytics {
    constructor(config = {}) {
        this.config = {
            enableComplexityAnalysis: true,
            enablePatternAnalysis: true,
            enableProductivityTracking: true,
            trackingSampleSize: 1000,     // Number of recent sessions to track
            ...config
        };

        // Session tracking
        this.sessionStart = Date.now();
        this.sessionData = {
            startTime: this.sessionStart,
            keystrokes: 0,
            wordsAdded: 0,
            wordsDeleted: 0,
            pauseDuration: 0,
            lastActivity: this.sessionStart
        };

        // Writing patterns cache
        this.patternCache = new Map();
        
        // Productivity history
        this.productivityHistory = this.loadProductivityHistory();
    }

    /**
     * Analyze document content comprehensively
     */
    analyzeDocument(content, options = {}) {
        const {
            includeComplexity = this.config.enableComplexityAnalysis,
            includePatterns = this.config.enablePatternAnalysis,
            previousContent = null
        } = options;

        const analysis = {
            timestamp: Date.now(),
            
            // Basic metrics
            basic: this.calculateBasicMetrics(content),
            
            // Reading time estimates
            readingTime: this.calculateReadingTime(content),
            
            // Content structure
            structure: this.analyzeStructure(content),
            
            // Session metrics
            session: this.getSessionMetrics()
        };

        // Optional complexity analysis
        if (includeComplexity) {
            analysis.complexity = this.analyzeComplexity(content);
        }

        // Optional pattern analysis
        if (includePatterns) {
            analysis.patterns = this.analyzePatterns(content);
        }

        // Change analysis if previous content provided
        if (previousContent) {
            analysis.changes = this.analyzeChanges(previousContent, content);
        }

        return analysis;
    }

    /**
     * Calculate basic metrics (words, characters, etc.)
     */
    calculateBasicMetrics(content) {
        // Remove code blocks and inline code for accurate word counting
        const cleanContent = content
            .replace(CONTENT_PATTERNS.CODE_BLOCKS, '')
            .replace(CONTENT_PATTERNS.INLINE_CODE, '');

        // Word count (excluding markdown syntax)
        const words = this.extractWords(cleanContent);
        const wordCount = words.length;

        // Character counts
        const characterCount = content.length;
        const characterCountNoSpaces = content.replace(/\s/g, '').length;

        // Sentence count
        const sentences = this.extractSentences(cleanContent);
        const sentenceCount = sentences.length;

        // Paragraph count
        const paragraphs = this.extractParagraphs(content);
        const paragraphCount = paragraphs.length;

        return {
            wordCount,
            characterCount,
            characterCountNoSpaces,
            sentenceCount,
            paragraphCount,
            averageWordsPerSentence: sentenceCount > 0 ? (wordCount / sentenceCount).toFixed(1) : 0,
            averageWordsPerParagraph: paragraphCount > 0 ? (wordCount / paragraphCount).toFixed(1) : 0,
            averageSentencesPerParagraph: paragraphCount > 0 ? (sentenceCount / paragraphCount).toFixed(1) : 0
        };
    }

    /**
     * Calculate reading time estimates
     */
    calculateReadingTime(content) {
        const words = this.extractWords(content);
        const wordCount = words.length;

        return {
            wordCount,
            slow: Math.ceil(wordCount / READING_SPEEDS.SLOW),      // minutes
            average: Math.ceil(wordCount / READING_SPEEDS.AVERAGE),
            fast: Math.ceil(wordCount / READING_SPEEDS.FAST),
            skimming: Math.ceil(wordCount / READING_SPEEDS.SKIMMING),
            
            // Human-readable format
            formatted: {
                slow: this.formatReadingTime(wordCount / READING_SPEEDS.SLOW),
                average: this.formatReadingTime(wordCount / READING_SPEEDS.AVERAGE),
                fast: this.formatReadingTime(wordCount / READING_SPEEDS.FAST),
                skimming: this.formatReadingTime(wordCount / READING_SPEEDS.SKIMMING)
            }
        };
    }

    /**
     * Analyze document structure
     */
    analyzeStructure(content) {
        const headings = this.extractHeadings(content);
        const lists = this.extractLists(content);
        const links = this.extractLinks(content);
        const emphasis = this.extractEmphasis(content);
        const codeBlocks = this.extractCodeBlocks(content);
        const blockquotes = this.extractBlockquotes(content);

        return {
            headings: {
                count: headings.length,
                levels: this.analyzeHeadingLevels(headings),
                outline: this.buildOutline(headings)
            },
            lists: {
                unordered: lists.unordered.length,
                ordered: lists.ordered.length,
                total: lists.unordered.length + lists.ordered.length
            },
            links: {
                count: links.length,
                unique: [...new Set(links.map(link => link.url))].length
            },
            emphasis: {
                bold: emphasis.bold.length,
                italic: emphasis.italic.length,
                total: emphasis.total.length
            },
            code: {
                blocks: codeBlocks.length,
                inline: this.extractInlineCode(content).length
            },
            blockquotes: blockquotes.length
        };
    }

    /**
     * Analyze text complexity using multiple readability metrics
     */
    analyzeComplexity(content) {
        const words = this.extractWords(content);
        const sentences = this.extractSentences(content);
        const syllables = this.countTotalSyllables(words);

        // Flesch Reading Ease Score
        const avgSentenceLength = words.length / sentences.length || 0;
        const avgSyllablesPerWord = syllables / words.length || 0;
        
        const fleschScore = 206.835 - (1.015 * avgSentenceLength) - (84.6 * avgSyllablesPerWord);
        
        // Flesch-Kincaid Grade Level
        const gradeLevel = (0.39 * avgSentenceLength) + (11.8 * avgSyllablesPerWord) - 15.59;

        // Custom complexity indicators
        const sentenceLengthComplexity = this.analyzeSentenceLengthComplexity(sentences);
        const vocabularyComplexity = this.analyzeVocabularyComplexity(words);
        const structuralComplexity = this.analyzeStructuralComplexity(content);

        return {
            readabilityScores: {
                flesch: Math.max(0, Math.min(100, fleschScore)),
                fleschKincaid: Math.max(0, gradeLevel),
                readabilityLevel: this.getReadabilityLevel(fleschScore)
            },
            averages: {
                sentenceLength: avgSentenceLength.toFixed(1),
                syllablesPerWord: avgSyllablesPerWord.toFixed(1),
                wordsPerParagraph: this.calculateAverageWordsPerParagraph(content)
            },
            complexity: {
                sentence: sentenceLengthComplexity,
                vocabulary: vocabularyComplexity,
                structural: structuralComplexity,
                overall: this.calculateOverallComplexity(fleschScore, sentenceLengthComplexity, vocabularyComplexity)
            }
        };
    }

    /**
     * Analyze writing patterns and style
     */
    analyzePatterns(content) {
        const words = this.extractWords(content);
        const sentences = this.extractSentences(content);

        return {
            vocabulary: {
                uniqueWords: [...new Set(words.map(w => w.toLowerCase()))].length,
                lexicalDiversity: this.calculateLexicalDiversity(words),
                wordFrequency: this.analyzeWordFrequency(words),
                mostCommonWords: this.getMostCommonWords(words, 10)
            },
            
            style: {
                sentenceTypes: this.analyzeSentenceTypes(sentences),
                punctuationUsage: this.analyzePunctuation(content),
                passiveVoice: this.detectPassiveVoice(sentences),
                modalVerbs: this.detectModalVerbs(content),
                adverbUsage: this.detectAdverbs(words)
            },
            
            flow: {
                transitionWords: this.detectTransitionWords(content),
                sentenceVariety: this.analyzeSentenceVariety(sentences),
                paragraphFlow: this.analyzeParagraphFlow(content)
            }
        };
    }

    /**
     * Analyze changes between two versions of content
     */
    analyzeChanges(oldContent, newContent) {
        const oldWords = this.extractWords(oldContent);
        const newWords = this.extractWords(newContent);
        
        const wordDiff = newWords.length - oldWords.length;
        const charDiff = newContent.length - oldContent.length;

        return {
            wordChange: {
                added: Math.max(0, wordDiff),
                removed: Math.max(0, -wordDiff),
                net: wordDiff
            },
            characterChange: {
                added: Math.max(0, charDiff),
                removed: Math.max(0, -charDiff),
                net: charDiff
            },
            percentageChange: {
                words: oldWords.length > 0 ? ((wordDiff / oldWords.length) * 100).toFixed(1) : 0,
                characters: oldContent.length > 0 ? ((charDiff / oldContent.length) * 100).toFixed(1) : 0
            }
        };
    }

    /**
     * Track writing session metrics
     */
    updateSessionMetrics(newContent, previousContent = '') {
        const now = Date.now();
        const timeSinceLastActivity = now - this.sessionData.lastActivity;
        
        // Track pause time (gaps > 30 seconds)
        if (timeSinceLastActivity > 30000) {
            this.sessionData.pauseDuration += timeSinceLastActivity;
        }

        // Update activity timestamp
        this.sessionData.lastActivity = now;

        // Calculate word changes
        const oldWordCount = this.extractWords(previousContent).length;
        const newWordCount = this.extractWords(newContent).length;
        const wordDiff = newWordCount - oldWordCount;

        if (wordDiff > 0) {
            this.sessionData.wordsAdded += wordDiff;
        } else if (wordDiff < 0) {
            this.sessionData.wordsDeleted += Math.abs(wordDiff);
        }

        // Estimate keystroke count (rough approximation)
        const contentDiff = Math.abs(newContent.length - previousContent.length);
        this.sessionData.keystrokes += contentDiff;

        return this.getSessionMetrics();
    }

    /**
     * Get current session metrics
     */
    getSessionMetrics() {
        const now = Date.now();
        const sessionDuration = now - this.sessionData.startTime;
        const activeDuration = sessionDuration - this.sessionData.pauseDuration;

        return {
            duration: {
                total: sessionDuration,
                active: activeDuration,
                paused: this.sessionData.pauseDuration,
                formatted: {
                    total: this.formatDuration(sessionDuration),
                    active: this.formatDuration(activeDuration),
                    paused: this.formatDuration(this.sessionData.pauseDuration)
                }
            },
            productivity: {
                wordsAdded: this.sessionData.wordsAdded,
                wordsDeleted: this.sessionData.wordsDeleted,
                netWords: this.sessionData.wordsAdded - this.sessionData.wordsDeleted,
                keystrokes: this.sessionData.keystrokes,
                wordsPerMinute: activeDuration > 0 ? ((this.sessionData.wordsAdded / activeDuration) * 60000).toFixed(1) : 0
            }
        };
    }

    /**
     * Set writing goals and track progress
     */
    setWritingGoal(type, target, timeframe = 'daily') {
        const goals = this.loadWritingGoals();
        const goalId = `${type}_${timeframe}_${Date.now()}`;

        goals[goalId] = {
            id: goalId,
            type, // 'words', 'characters', 'time', 'documents'
            target,
            timeframe, // 'daily', 'weekly', 'monthly'
            created: Date.now(),
            progress: 0,
            achieved: false
        };

        this.saveWritingGoals(goals);
        return goalId;
    }

    /**
     * Update goal progress
     */
    updateGoalProgress(goalId, progress) {
        const goals = this.loadWritingGoals();
        if (goals[goalId]) {
            goals[goalId].progress = progress;
            goals[goalId].achieved = progress >= goals[goalId].target;
            goals[goalId].lastUpdated = Date.now();
            this.saveWritingGoals(goals);
        }
    }

    /**
     * Get current goal status
     */
    getGoalStatus() {
        const goals = this.loadWritingGoals();
        const now = Date.now();
        const activeGoals = {};

        for (const [id, goal] of Object.entries(goals)) {
            // Check if goal is still active based on timeframe
            const isActive = this.isGoalActive(goal, now);
            if (isActive) {
                activeGoals[id] = {
                    ...goal,
                    progressPercentage: ((goal.progress / goal.target) * 100).toFixed(1)
                };
            }
        }

        return activeGoals;
    }

    // Utility methods for text analysis

    extractWords(content) {
        return content
            .replace(/[^\w\s'-]/g, ' ')
            .split(/\s+/)
            .filter(word => word.length > 0);
    }

    extractSentences(content) {
        return content
            .split(/[.!?]+/)
            .map(s => s.trim())
            .filter(s => s.length > 0);
    }

    extractParagraphs(content) {
        return content
            .split(/\n\s*\n/)
            .map(p => p.trim())
            .filter(p => p.length > 0);
    }

    extractHeadings(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.HEADINGS)];
        return matches.map(match => ({
            level: match[0].split(' ')[0].length,
            text: match[1].trim(),
            line: match.index
        }));
    }

    extractLists(content) {
        const unordered = [...content.matchAll(CONTENT_PATTERNS.LISTS)];
        const ordered = [...content.matchAll(CONTENT_PATTERNS.NUMBERED_LISTS)];
        
        return {
            unordered: unordered.map(match => match[1].trim()),
            ordered: ordered.map(match => match[1].trim())
        };
    }

    extractLinks(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.LINKS)];
        return matches.map(match => ({
            text: match[1],
            url: match[2]
        }));
    }

    extractEmphasis(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.EMPHASIS)];
        const bold = matches.filter(m => m[1].startsWith('**') || m[1].startsWith('__'));
        const italic = matches.filter(m => m[1].startsWith('*') || m[1].startsWith('_'));
        
        return {
            bold,
            italic,
            total: matches
        };
    }

    extractCodeBlocks(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.CODE_BLOCKS)];
        return matches.map(match => match[0]);
    }

    extractInlineCode(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.INLINE_CODE)];
        return matches.map(match => match[0]);
    }

    extractBlockquotes(content) {
        const matches = [...content.matchAll(CONTENT_PATTERNS.BLOCKQUOTES)];
        return matches.map(match => match[1].trim());
    }

    countSyllables(word) {
        word = word.toLowerCase();
        if (word.length <= 3) return 1;
        
        const vowels = word.match(/[aeiouy]+/g);
        let syllableCount = vowels ? vowels.length : 1;
        
        // Subtract silent 'e'
        if (word.endsWith('e')) syllableCount--;
        
        return Math.max(1, syllableCount);
    }

    countTotalSyllables(words) {
        return words.reduce((total, word) => total + this.countSyllables(word), 0);
    }

    formatReadingTime(minutes) {
        if (minutes < 1) return 'Less than 1 minute';
        if (minutes < 60) return `${Math.ceil(minutes)} minute${Math.ceil(minutes) > 1 ? 's' : ''}`;
        
        const hours = Math.floor(minutes / 60);
        const remainingMinutes = Math.ceil(minutes % 60);
        
        if (remainingMinutes === 0) {
            return `${hours} hour${hours > 1 ? 's' : ''}`;
        }
        
        return `${hours} hour${hours > 1 ? 's' : ''} ${remainingMinutes} minute${remainingMinutes > 1 ? 's' : ''}`;
    }

    formatDuration(milliseconds) {
        const seconds = Math.floor(milliseconds / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);

        if (hours > 0) {
            return `${hours}h ${minutes % 60}m`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    }

    analyzeHeadingLevels(headings) {
        const levels = {};
        headings.forEach(heading => {
            levels[heading.level] = (levels[heading.level] || 0) + 1;
        });
        return levels;
    }

    buildOutline(headings) {
        return headings.map(heading => ({
            level: heading.level,
            text: heading.text,
            indent: '  '.repeat(heading.level - 1)
        }));
    }

    getReadabilityLevel(fleschScore) {
        if (fleschScore >= 90) return 'Very Easy';
        if (fleschScore >= 80) return 'Easy';
        if (fleschScore >= 70) return 'Fairly Easy';
        if (fleschScore >= 60) return 'Standard';
        if (fleschScore >= 50) return 'Fairly Difficult';
        if (fleschScore >= 30) return 'Difficult';
        return 'Very Difficult';
    }

    calculateLexicalDiversity(words) {
        const uniqueWords = new Set(words.map(w => w.toLowerCase()));
        return (uniqueWords.size / words.length).toFixed(3);
    }

    analyzeWordFrequency(words) {
        const frequency = {};
        words.forEach(word => {
            const lowerWord = word.toLowerCase();
            frequency[lowerWord] = (frequency[lowerWord] || 0) + 1;
        });
        return frequency;
    }

    getMostCommonWords(words, count = 10) {
        const frequency = this.analyzeWordFrequency(words);
        
        // Filter out common stop words
        const stopWords = new Set(['the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by']);
        
        return Object.entries(frequency)
            .filter(([word]) => !stopWords.has(word) && word.length > 2)
            .sort(([,a], [,b]) => b - a)
            .slice(0, count)
            .map(([word, freq]) => ({ word, frequency: freq }));
    }

    // Storage methods for persistence

    loadWritingGoals() {
        try {
            const stored = localStorage.getItem('writemagic_writing_goals');
            return stored ? JSON.parse(stored) : {};
        } catch (error) {
            console.warn('Failed to load writing goals:', error);
            return {};
        }
    }

    saveWritingGoals(goals) {
        try {
            localStorage.setItem('writemagic_writing_goals', JSON.stringify(goals));
        } catch (error) {
            console.warn('Failed to save writing goals:', error);
        }
    }

    loadProductivityHistory() {
        try {
            const stored = localStorage.getItem('writemagic_productivity_history');
            return stored ? JSON.parse(stored) : [];
        } catch (error) {
            console.warn('Failed to load productivity history:', error);
            return [];
        }
    }

    saveProductivityHistory() {
        try {
            localStorage.setItem('writemagic_productivity_history', JSON.stringify(this.productivityHistory));
        } catch (error) {
            console.warn('Failed to save productivity history:', error);
        }
    }

    isGoalActive(goal, currentTime) {
        const timeSinceCreated = currentTime - goal.created;
        const dayInMs = 24 * 60 * 60 * 1000;
        
        switch (goal.timeframe) {
            case 'daily':
                return timeSinceCreated < dayInMs;
            case 'weekly':
                return timeSinceCreated < (7 * dayInMs);
            case 'monthly':
                return timeSinceCreated < (30 * dayInMs);
            default:
                return true;
        }
    }
}

export default WritingAnalytics;