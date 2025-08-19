/**
 * Content Utilities - Text manipulation helpers and writing tools
 * 
 * This module provides a comprehensive set of utilities for content manipulation,
 * formatting, validation, and analysis to support writers with their creative process.
 */

/**
 * Text formatting and manipulation utilities
 */
export const TextFormat = {
    MARKDOWN: 'markdown',
    HTML: 'html',
    PLAIN: 'plain'
};

/**
 * Content templates for common writing formats
 */
export const CONTENT_TEMPLATES = {
    BLOG_POST: {
        name: 'Blog Post',
        content: `# [Blog Post Title]

## Introduction
Write a compelling introduction that hooks your readers...

## Main Content
### Section 1
Your main points go here...

### Section 2
Continue developing your ideas...

## Conclusion
Wrap up with key takeaways...

---
*Published on [Date]*`
    },
    
    ARTICLE: {
        name: 'Article',
        content: `# [Article Title]

**Abstract:** Brief summary of the article content...

## Introduction
Introduce the topic and its importance...

## Background
Provide necessary context and background information...

## Analysis
### Key Point 1
Detailed analysis and evidence...

### Key Point 2
Continue your analysis...

## Conclusion
Summarize findings and implications...

## References
1. [Reference 1]
2. [Reference 2]`
    },
    
    STORY: {
        name: 'Short Story',
        content: `# [Story Title]

*Setting: [Where and when does this take place?]*

---

[Opening paragraph - establish scene, character, or conflict]

[Continue your story...]

---

*Word count: [XXX] words*`
    },
    
    MEETING_NOTES: {
        name: 'Meeting Notes',
        content: `# Meeting Notes - [Topic]

**Date:** [Date]
**Attendees:** [Names]
**Duration:** [Time]

## Agenda
1. [Item 1]
2. [Item 2]
3. [Item 3]

## Discussion
### [Topic 1]
- Key points discussed
- Decisions made
- Action items

### [Topic 2]
- [Notes]

## Action Items
- [ ] [Action] - Assigned to [Person] - Due: [Date]
- [ ] [Action] - Assigned to [Person] - Due: [Date]

## Next Steps
[What happens next...]`
    },
    
    RESEARCH_NOTES: {
        name: 'Research Notes',
        content: `# Research Notes: [Topic]

**Research Question:** [Your main question]
**Date:** [Date]

## Key Sources
1. [Source 1] - [Brief description]
2. [Source 2] - [Brief description]

## Main Findings
### [Theme 1]
- [Finding]
- [Evidence/Quote]
- [Significance]

### [Theme 2]
- [Finding]
- [Evidence/Quote]
- [Significance]

## Connections & Insights
[How do these findings connect? What patterns emerge?]

## Questions for Further Research
- [Question 1]
- [Question 2]

## References
[Full citations]`
    }
};

/**
 * Common writing style improvements
 */
const STYLE_IMPROVEMENTS = {
    PASSIVE_VOICE_INDICATORS: [
        /\b(am|is|are|was|were|being|been|be)\s+\w+ed\b/gi,
        /\b(am|is|are|was|were|being|been|be)\s+\w+en\b/gi
    ],
    WEAK_WORDS: [
        'very', 'really', 'quite', 'rather', 'somewhat', 'pretty', 'fairly',
        'just', 'only', 'actually', 'basically', 'literally', 'definitely'
    ],
    WORDY_PHRASES: {
        'a lot of': 'many',
        'a number of': 'several',
        'at this point in time': 'now',
        'due to the fact that': 'because',
        'in order to': 'to',
        'it is important to note that': '',
        'the fact of the matter is': '',
        'what I mean to say is': ''
    }
};

/**
 * ContentUtilities - Comprehensive text processing and writing assistance
 * 
 * Features:
 * - Content validation and error detection
 * - Text formatting and conversion
 * - Template application and management
 * - Content analysis and suggestions
 * - Search and replace operations
 * - Outline extraction and generation
 * - Writing style analysis and improvement suggestions
 */
export class ContentUtilities {
    constructor(config = {}) {
        this.config = {
            enableSpellCheck: true,
            enableGrammarCheck: true,
            enableStyleSuggestions: true,
            maxSuggestions: 10,
            ...config
        };

        // Initialize spell check dictionary (placeholder - would use real dictionary)
        this.spellCheckDict = new Set();
        this.customDictionary = new Set();
        
        // Style patterns cache
        this.stylePatterns = this.compileStylePatterns();
    }

    /**
     * Validate content for common issues
     */
    validateContent(content, options = {}) {
        const {
            checkSpelling = this.config.enableSpellCheck,
            checkGrammar = this.config.enableGrammarCheck,
            checkStyle = this.config.enableStyleSuggestions
        } = options;

        const validation = {
            isValid: true,
            errors: [],
            warnings: [],
            suggestions: []
        };

        // Basic content checks
        if (!content || content.trim().length === 0) {
            validation.isValid = false;
            validation.errors.push('Content cannot be empty');
            return validation;
        }

        // Check for extremely long paragraphs
        const paragraphs = this.extractParagraphs(content);
        const longParagraphs = paragraphs.filter(p => this.countWords(p) > 200);
        if (longParagraphs.length > 0) {
            validation.warnings.push(`${longParagraphs.length} paragraph(s) are very long (>200 words)`);
        }

        // Check for very short sentences that might indicate fragments
        const sentences = this.extractSentences(content);
        const shortSentences = sentences.filter(s => this.countWords(s) < 3);
        if (shortSentences.length > 0) {
            validation.warnings.push(`${shortSentences.length} very short sentence(s) detected`);
        }

        // Style checks
        if (checkStyle) {
            const styleIssues = this.analyzeStyle(content);
            validation.suggestions.push(...styleIssues);
        }

        // Set overall validity
        validation.isValid = validation.errors.length === 0;

        return validation;
    }

    /**
     * Apply content template
     */
    applyTemplate(templateKey, customContent = '') {
        const template = CONTENT_TEMPLATES[templateKey];
        if (!template) {
            throw new Error(`Template '${templateKey}' not found`);
        }

        let content = template.content;

        // Replace placeholders with custom content if provided
        if (customContent) {
            // Simple placeholder replacement (can be enhanced)
            content = content.replace(/\[.*?\]/g, customContent);
        }

        return content;
    }

    /**
     * Extract document outline from content
     */
    extractOutline(content) {
        const headings = [];
        const lines = content.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();
            
            // Markdown headings
            const mdHeadingMatch = line.match(/^(#{1,6})\s+(.+)$/);
            if (mdHeadingMatch) {
                headings.push({
                    level: mdHeadingMatch[1].length,
                    text: mdHeadingMatch[2].trim(),
                    line: i + 1,
                    type: 'markdown'
                });
                continue;
            }

            // Alternative heading detection (underlined headings)
            if (i < lines.length - 1) {
                const nextLine = lines[i + 1].trim();
                if (nextLine.match(/^=+$/) && line.length > 0) {
                    headings.push({
                        level: 1,
                        text: line,
                        line: i + 1,
                        type: 'underlined'
                    });
                } else if (nextLine.match(/^-+$/) && line.length > 0) {
                    headings.push({
                        level: 2,
                        text: line,
                        line: i + 1,
                        type: 'underlined'
                    });
                }
            }
        }

        // Build hierarchical outline
        return this.buildHierarchicalOutline(headings);
    }

    /**
     * Generate outline from content structure
     */
    generateOutline(content, options = {}) {
        const { maxDepth = 3, includeSentences = false } = options;
        
        const outline = this.extractOutline(content);
        
        // Add sentence-level outline if requested
        if (includeSentences) {
            const paragraphs = this.extractParagraphs(content);
            paragraphs.forEach((paragraph, index) => {
                const sentences = this.extractSentences(paragraph);
                if (sentences.length > 1) {
                    outline.push({
                        level: maxDepth + 1,
                        text: `Paragraph ${index + 1} (${sentences.length} sentences)`,
                        line: null,
                        type: 'generated',
                        sentences: sentences.map(s => s.substring(0, 50) + (s.length > 50 ? '...' : ''))
                    });
                }
            });
        }

        return outline.filter(item => item.level <= maxDepth);
    }

    /**
     * Count words in text
     */
    countWords(text) {
        if (!text) return 0;
        
        return text
            .replace(/[^\w\s'-]/g, ' ')
            .split(/\s+/)
            .filter(word => word.length > 0)
            .length;
    }

    /**
     * Count sentences in text
     */
    countSentences(text) {
        if (!text) return 0;
        
        return text
            .split(/[.!?]+/)
            .filter(sentence => sentence.trim().length > 0)
            .length;
    }

    /**
     * Estimate reading time
     */
    estimateReadingTime(content, wordsPerMinute = 250) {
        const wordCount = this.countWords(content);
        const minutes = wordCount / wordsPerMinute;
        
        if (minutes < 1) {
            return 'Less than 1 minute';
        } else if (minutes < 60) {
            return `${Math.ceil(minutes)} minute${Math.ceil(minutes) > 1 ? 's' : ''}`;
        } else {
            const hours = Math.floor(minutes / 60);
            const remainingMinutes = Math.ceil(minutes % 60);
            return `${hours}h ${remainingMinutes}m`;
        }
    }

    /**
     * Analyze content complexity
     */
    analyzeComplexity(content) {
        const words = this.extractWords(content);
        const sentences = this.extractSentences(content);
        const paragraphs = this.extractParagraphs(content);

        const avgWordsPerSentence = sentences.length > 0 ? words.length / sentences.length : 0;
        const avgSentencesPerParagraph = paragraphs.length > 0 ? sentences.length / paragraphs.length : 0;
        const avgSyllablesPerWord = this.calculateAverageSyllables(words);

        let complexity = 'Simple';
        let score = 0;

        // Score based on various factors
        if (avgWordsPerSentence > 20) score += 2;
        else if (avgWordsPerSentence > 15) score += 1;

        if (avgSyllablesPerWord > 1.5) score += 2;
        else if (avgSyllablesPerWord > 1.3) score += 1;

        if (avgSentencesPerParagraph > 8) score += 1;

        // Determine complexity level
        if (score >= 4) complexity = 'Complex';
        else if (score >= 2) complexity = 'Moderate';

        return {
            level: complexity,
            score,
            details: {
                avgWordsPerSentence: avgWordsPerSentence.toFixed(1),
                avgSentencesPerParagraph: avgSentencesPerParagraph.toFixed(1),
                avgSyllablesPerWord: avgSyllablesPerWord.toFixed(2)
            }
        };
    }

    /**
     * Search documents by content
     */
    searchDocuments(documents, query, options = {}) {
        const {
            caseSensitive = false,
            wholeWords = false,
            includeTitle = true,
            includeContent = true,
            maxResults = 50
        } = options;

        const searchQuery = caseSensitive ? query : query.toLowerCase();
        const results = [];

        for (const document of documents) {
            const matches = [];
            let score = 0;

            // Search in title
            if (includeTitle) {
                const title = caseSensitive ? document.title : document.title.toLowerCase();
                if (this.searchInText(title, searchQuery, wholeWords)) {
                    score += 10; // Higher weight for title matches
                    matches.push({ type: 'title', text: document.title });
                }
            }

            // Search in content
            if (includeContent) {
                const content = caseSensitive ? document.content : document.content.toLowerCase();
                const contentMatches = this.findContentMatches(content, searchQuery, wholeWords);
                matches.push(...contentMatches);
                score += contentMatches.length;
            }

            if (matches.length > 0) {
                results.push({
                    document,
                    matches,
                    score,
                    relevance: this.calculateRelevance(query, document, matches)
                });
            }
        }

        // Sort by relevance and return top results
        return results
            .sort((a, b) => b.relevance - a.relevance)
            .slice(0, maxResults);
    }

    /**
     * Analyze changes between content versions
     */
    analyzeChanges(oldContent, newContent, options = {}) {
        const { contextLines = 3, ignoreWhitespace = false } = options;

        // Prepare content for comparison
        let oldLines = oldContent.split('\n');
        let newLines = newContent.split('\n');

        if (ignoreWhitespace) {
            oldLines = oldLines.map(line => line.trim());
            newLines = newLines.map(line => line.trim());
        }

        const changes = {
            added: [],
            removed: [],
            modified: [],
            summary: {
                linesAdded: 0,
                linesRemoved: 0,
                linesModified: 0,
                wordsChanged: 0
            }
        };

        // Simple line-by-line comparison (could be enhanced with proper diff algorithm)
        const maxLength = Math.max(oldLines.length, newLines.length);
        
        for (let i = 0; i < maxLength; i++) {
            const oldLine = oldLines[i] || '';
            const newLine = newLines[i] || '';

            if (i >= oldLines.length) {
                // Line added
                changes.added.push({
                    line: i + 1,
                    content: newLine,
                    context: this.getContext(newLines, i, contextLines)
                });
                changes.summary.linesAdded++;
            } else if (i >= newLines.length) {
                // Line removed
                changes.removed.push({
                    line: i + 1,
                    content: oldLine,
                    context: this.getContext(oldLines, i, contextLines)
                });
                changes.summary.linesRemoved++;
            } else if (oldLine !== newLine) {
                // Line modified
                changes.modified.push({
                    line: i + 1,
                    oldContent: oldLine,
                    newContent: newLine,
                    wordChanges: this.analyzeWordChanges(oldLine, newLine)
                });
                changes.summary.linesModified++;
            }
        }

        // Calculate total word changes
        changes.summary.wordsChanged = changes.modified.reduce(
            (total, change) => total + change.wordChanges.added + change.wordChanges.removed,
            0
        );

        return changes;
    }

    /**
     * Format content for different output types
     */
    formatContent(content, outputFormat, options = {}) {
        const { preserveFormatting = true, addMetadata = false } = options;

        switch (outputFormat) {
            case TextFormat.HTML:
                return this.convertMarkdownToHtml(content, preserveFormatting);
            
            case TextFormat.PLAIN:
                return this.convertToPlainText(content);
            
            case TextFormat.MARKDOWN:
                return preserveFormatting ? content : this.normalizeMarkdown(content);
            
            default:
                return content;
        }
    }

    /**
     * Render content preview
     */
    renderPreview(content, contentType = 'markdown') {
        switch (contentType.toLowerCase()) {
            case 'markdown':
                return this.convertMarkdownToHtml(content);
            case 'html':
                return content; // Return as-is, but should be sanitized in production
            case 'plain':
                return this.convertPlainTextToHtml(content);
            default:
                return this.escapeHtml(content);
        }
    }

    /**
     * Extract and analyze writing style
     */
    analyzeStyle(content) {
        const suggestions = [];
        const words = this.extractWords(content);
        const sentences = this.extractSentences(content);

        // Check for passive voice
        const passiveVoice = this.detectPassiveVoice(content);
        if (passiveVoice.length > 0) {
            suggestions.push({
                type: 'style',
                category: 'passive_voice',
                message: `Found ${passiveVoice.length} instance(s) of passive voice`,
                instances: passiveVoice.slice(0, 3), // Show first 3 examples
                severity: 'warning'
            });
        }

        // Check for weak words
        const weakWords = this.detectWeakWords(words);
        if (weakWords.length > 0) {
            suggestions.push({
                type: 'style',
                category: 'weak_words',
                message: `Found ${weakWords.length} weak word(s)`,
                instances: weakWords.slice(0, 5),
                severity: 'suggestion'
            });
        }

        // Check for wordy phrases
        const wordyPhrases = this.detectWordyPhrases(content);
        if (wordyPhrases.length > 0) {
            suggestions.push({
                type: 'style',
                category: 'wordy_phrases',
                message: `Found ${wordyPhrases.length} wordy phrase(s)`,
                instances: wordyPhrases.slice(0, 3),
                severity: 'suggestion'
            });
        }

        // Check sentence length variety
        const sentenceLengths = sentences.map(s => this.countWords(s));
        const avgLength = sentenceLengths.reduce((a, b) => a + b, 0) / sentenceLengths.length;
        const lengthVariance = this.calculateVariance(sentenceLengths);
        
        if (lengthVariance < 10) {
            suggestions.push({
                type: 'style',
                category: 'sentence_variety',
                message: 'Consider varying your sentence lengths for better flow',
                severity: 'suggestion'
            });
        }

        return suggestions;
    }

    // Helper methods

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

    buildHierarchicalOutline(headings) {
        // Convert flat heading list to hierarchical structure
        const outline = [];
        const stack = [];

        headings.forEach(heading => {
            // Pop stack until we find a parent level
            while (stack.length > 0 && stack[stack.length - 1].level >= heading.level) {
                stack.pop();
            }

            const item = {
                ...heading,
                children: [],
                parent: stack.length > 0 ? stack[stack.length - 1] : null
            };

            if (stack.length > 0) {
                stack[stack.length - 1].children.push(item);
            } else {
                outline.push(item);
            }

            stack.push(item);
        });

        return outline;
    }

    calculateAverageSyllables(words) {
        if (words.length === 0) return 0;
        
        const totalSyllables = words.reduce((total, word) => {
            return total + this.countSyllables(word);
        }, 0);
        
        return totalSyllables / words.length;
    }

    countSyllables(word) {
        word = word.toLowerCase();
        if (word.length <= 3) return 1;
        
        const vowels = word.match(/[aeiouy]+/g);
        let syllableCount = vowels ? vowels.length : 1;
        
        if (word.endsWith('e')) syllableCount--;
        
        return Math.max(1, syllableCount);
    }

    searchInText(text, query, wholeWords = false) {
        if (wholeWords) {
            const wordRegex = new RegExp(`\\b${this.escapeRegex(query)}\\b`, 'gi');
            return wordRegex.test(text);
        } else {
            return text.includes(query);
        }
    }

    findContentMatches(content, query, wholeWords = false) {
        const matches = [];
        const lines = content.split('\n');

        lines.forEach((line, index) => {
            if (this.searchInText(line, query, wholeWords)) {
                matches.push({
                    type: 'content',
                    line: index + 1,
                    text: line.substring(0, 100) + (line.length > 100 ? '...' : ''),
                    context: this.getContext(lines, index, 1)
                });
            }
        });

        return matches;
    }

    calculateRelevance(query, document, matches) {
        let relevance = 0;
        
        // Title matches get higher score
        const titleMatches = matches.filter(m => m.type === 'title').length;
        relevance += titleMatches * 10;
        
        // Content matches
        const contentMatches = matches.filter(m => m.type === 'content').length;
        relevance += contentMatches * 2;
        
        // Query length factor (longer queries should have higher precision)
        relevance *= Math.log(query.length + 1);
        
        // Document length factor (prefer matches in shorter documents)
        const docWordCount = this.countWords(document.content);
        if (docWordCount > 0) {
            relevance *= Math.max(0.1, 1000 / docWordCount);
        }
        
        return relevance;
    }

    getContext(lines, index, contextLines) {
        const start = Math.max(0, index - contextLines);
        const end = Math.min(lines.length, index + contextLines + 1);
        return lines.slice(start, end);
    }

    analyzeWordChanges(oldLine, newLine) {
        const oldWords = this.extractWords(oldLine);
        const newWords = this.extractWords(newLine);
        
        return {
            added: Math.max(0, newWords.length - oldWords.length),
            removed: Math.max(0, oldWords.length - newWords.length),
            oldWordCount: oldWords.length,
            newWordCount: newWords.length
        };
    }

    detectPassiveVoice(content) {
        const matches = [];
        STYLE_IMPROVEMENTS.PASSIVE_VOICE_INDICATORS.forEach(pattern => {
            const found = [...content.matchAll(pattern)];
            matches.push(...found.map(match => ({
                text: match[0],
                position: match.index
            })));
        });
        return matches;
    }

    detectWeakWords(words) {
        const weakWords = [];
        words.forEach((word, index) => {
            if (STYLE_IMPROVEMENTS.WEAK_WORDS.includes(word.toLowerCase())) {
                weakWords.push({ word, position: index });
            }
        });
        return weakWords;
    }

    detectWordyPhrases(content) {
        const phrases = [];
        Object.entries(STYLE_IMPROVEMENTS.WORDY_PHRASES).forEach(([wordy, concise]) => {
            const regex = new RegExp(this.escapeRegex(wordy), 'gi');
            const matches = [...content.matchAll(regex)];
            matches.forEach(match => {
                phrases.push({
                    original: match[0],
                    suggestion: concise,
                    position: match.index
                });
            });
        });
        return phrases;
    }

    calculateVariance(numbers) {
        const mean = numbers.reduce((a, b) => a + b, 0) / numbers.length;
        const squaredDiffs = numbers.map(n => Math.pow(n - mean, 2));
        return squaredDiffs.reduce((a, b) => a + b, 0) / numbers.length;
    }

    convertMarkdownToHtml(markdown, preserveFormatting = true) {
        // Simple markdown to HTML conversion (would use proper library in production)
        let html = markdown;
        
        // Headers
        html = html.replace(/^### (.*)/gm, '<h3>$1</h3>');
        html = html.replace(/^## (.*)/gm, '<h2>$1</h2>');
        html = html.replace(/^# (.*)/gm, '<h1>$1</h1>');
        
        // Bold and italic
        html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
        html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
        
        // Links
        html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');
        
        // Line breaks
        html = html.replace(/\n\n/g, '</p><p>');
        html = `<p>${html}</p>`;
        
        return html;
    }

    convertToPlainText(content) {
        return content
            .replace(/[#*_`\[\]()]/g, '')
            .replace(/\n\s*\n/g, '\n\n')
            .trim();
    }

    convertPlainTextToHtml(text) {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/\n\n/g, '</p><p>')
            .replace(/\n/g, '<br>');
    }

    normalizeMarkdown(markdown) {
        // Normalize markdown formatting
        return markdown
            .replace(/^#+ /gm, match => match.replace(/ +/g, ' '))
            .replace(/\n{3,}/g, '\n\n')
            .trim();
    }

    escapeHtml(text) {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    escapeRegex(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    compileStylePatterns() {
        // Pre-compile regex patterns for better performance
        return {
            passiveVoice: STYLE_IMPROVEMENTS.PASSIVE_VOICE_INDICATORS,
            weakWords: new RegExp(`\\b(${STYLE_IMPROVEMENTS.WEAK_WORDS.join('|')})\\b`, 'gi')
        };
    }
}

export default ContentUtilities;