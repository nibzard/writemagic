# WriteMagic JavaScript API

A comprehensive JavaScript layer that provides writer-friendly interfaces for document management, built on top of the WriteMagic WASM core engine.

## Features

✨ **Writer-Focused Experience**
- Seamless auto-save with user feedback
- Intuitive project and document organization
- Rich content editing with undo/redo
- Smart suggestions and AI integration hooks
- Accessibility and keyboard navigation support

🎯 **Document Management**
- High-level document operations with auto-save and versioning
- Content validation and analysis
- Draft recovery and session management
- Cross-platform persistence (IndexedDB)

🏗️ **Multi-Pane Workspace**
- Flexible workspace layouts for different writing workflows
- Multiple document editing with drag-and-drop coordination
- Context-aware pane synchronization
- Keyboard-driven navigation

📊 **Writing Analytics**
- Comprehensive content analysis (word count, reading time, complexity)
- Real-time writing metrics and goal tracking
- Session productivity insights
- Writing pattern analysis

🤖 **AI Integration**
- Provider-agnostic AI assistance (Claude, GPT-4, local models)
- Context-aware suggestions and completions
- Non-intrusive writing enhancement
- Intelligent fallback handling

⏱️ **Session Management**
- Focus mode with Pomodoro/Sprint timers
- Writing goal tracking and achievements
- Session analytics and history
- Draft recovery and state persistence

## Quick Start

```javascript
import WriteMagic from './index.js';

// Initialize WriteMagic
const writeMagic = new WriteMagic({
    // AI Configuration (optional)
    claude_api_key: 'your-api-key',
    openai_api_key: 'your-api-key',
    
    // Writer Experience
    auto_save_delay: 2000,
    enable_analytics: true,
    enable_focus_mode: true,
    default_layout: 'focus'
});

// Wait for initialization
await writeMagic.initialize();

// Create a new document
const document = await writeMagic.createDocument({
    title: "My Article",
    content: "# Getting Started\n\nThis is my first article...",
    contentType: "markdown"
});

// Start a writing session
const session = writeMagic.startWritingSession({
    goals: { words: 500 },
    description: "Article writing session"
});

// Update content (triggers auto-save)
writeMagic.updateDocumentContent(document.id, "# Updated Content\n\nNew content here...");

// Get analytics
const analytics = writeMagic.getDocumentAnalytics(document.id);
console.log(`Words: ${analytics.analysis.basic.wordCount}`);
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                  WriteMagic API                         │
│  High-level writer-focused interface                   │
├─────────────────────────────────────────────────────────┤
│  DocumentManager    │  ProjectWorkspace  │ WritingSession│
│  - Auto-save        │  - Multi-pane UI   │ - Focus mode  │
│  - Draft recovery   │  - Layout mgmt     │ - Analytics   │
│  - Versioning       │  - Collaboration   │ - Goals       │
├─────────────────────────────────────────────────────────┤
│  WritingAnalytics   │  ContentUtilities  │ Event System │
│  - Content analysis │  - Text processing │ - Real-time   │
│  - Reading time     │  - Templates       │ - Coordination│
│  - Complexity       │  - Formatting      │ - UI updates  │
├─────────────────────────────────────────────────────────┤
│                    WASM Bindings                        │
│          Rust Core Engine via WebAssembly              │
└─────────────────────────────────────────────────────────┘
```

## Core Modules

### DocumentManager

High-level document operations with auto-save, versioning, and writer-focused features.

```javascript
// Create document with template
const doc = await writeMagic.createDocument({
    title: "Blog Post",
    template: 'BLOG_POST',
    projectId: project.id
});

// Update with auto-save
writeMagic.updateDocumentContent(doc.id, newContent);

// Manual save
await writeMagic.saveDocument(doc.id);

// Get analytics
const analytics = writeMagic.getDocumentAnalytics(doc.id);
```

### ProjectWorkspace

Multi-pane editing interface with flexible layouts and workspace coordination.

```javascript
// Load project and workspace
await writeMagic.loadProject(projectId);

// Set layout for different workflows
writeMagic.setLayout('research'); // 'focus', 'split', 'research', 'review', 'ai_enhanced'

// Open document in specific pane
await writeMagic.openDocumentInPane(paneId, documentId);

// Add new pane
const pane = writeMagic.addPane({
    type: 'reference',
    size: 30
});
```

### WritingSession

Session management with focus modes, goals, and productivity tracking.

```javascript
// Start session with goals
const session = writeMagic.startWritingSession({
    goals: {
        words: 500,
        time: 25 * 60 * 1000 // 25 minutes
    }
});

// Start focus session
writeMagic.startFocusSession('POMODORO'); // 'SPRINT', 'DEEP_WORK', 'CUSTOM'

// Set individual goals
writeMagic.setWritingGoal('words', 1000, 'Complete chapter');

// Get statistics
const stats = writeMagic.getSessionStats();
```

### WritingAnalytics

Comprehensive content analysis and writing metrics.

```javascript
// Analyze content
const analysis = writeMagic.analyzeContent(content);
console.log(`Reading time: ${analysis.readingTime.formatted.average}`);
console.log(`Complexity: ${analysis.complexity.readabilityScores.readabilityLevel}`);

// Get style suggestions
const suggestions = writeMagic.getStyleSuggestions(content);

// Extract outline
const outline = writeMagic.extractOutline(content);
```

### ContentUtilities

Text manipulation, formatting, and validation utilities.

```javascript
// Apply template
const content = writeMagic.applyTemplate('STORY', 'Chapter 1');

// Format content
const html = writeMagic.formatContent(markdown, 'html');

// Count words and estimate reading time
const wordCount = writeMagic.countWords(text);
const readingTime = writeMagic.estimateReadingTime(content);

// Search documents
const results = writeMagic.searchDocuments(query, {
    includeTitle: true,
    includeContent: true
});
```

### AI Integration

Provider-agnostic AI assistance with intelligent fallbacks.

```javascript
// Direct completion
const completion = await writeMagic.completeText(prompt, {
    model: 'claude-3-haiku-20240307',
    maxTokens: 500,
    temperature: 0.7
});

// Writing suggestions
const suggestions = await writeMagic.getWritingSuggestions(content, 'improve');

// Check AI health
const health = await writeMagic.checkAIHealth();
```

## Layout Presets

### Focus Mode
Single editor pane for distraction-free writing.

### Split View
Editor + reference document side-by-side.

### Research Mode
Editor + notes + reference for academic writing.

### Review Mode
Editor + preview + timeline for editing workflow.

### AI-Enhanced
Editor + AI assistant + outline for AI-powered writing.

## Event System

```javascript
import { WriteMagicEvents } from './index.js';

// Document events
writeMagic.on(WriteMagicEvents.DOCUMENT_CREATED, (data) => {
    console.log('Document created:', data.document.title);
});

writeMagic.on(WriteMagicEvents.DOCUMENT_AUTO_SAVED, (data) => {
    updateUI('Document saved');
});

// Session events
writeMagic.on(WriteMagicEvents.SESSION_STARTED, (data) => {
    startSessionUI(data.session);
});

writeMagic.on(WriteMagicEvents.GOAL_ACHIEVED, (data) => {
    showAchievement(data.goal);
});

// Focus mode events
writeMagic.on(WriteMagicEvents.FOCUS_SESSION_STARTED, (data) => {
    enterFocusMode(data.focusType);
});

// AI events
writeMagic.on(WriteMagicEvents.AI_COMPLETION, (data) => {
    console.log('AI response:', data.response);
});
```

## Content Templates

Built-in templates for common writing formats:

- **BLOG_POST**: Structured blog post with introduction, main content, conclusion
- **ARTICLE**: Academic/journalistic article with abstract and references
- **STORY**: Short story template with scene setting
- **MEETING_NOTES**: Meeting notes with agenda and action items
- **RESEARCH_NOTES**: Research documentation with sources and findings

```javascript
// Apply template
const content = writeMagic.applyTemplate('BLOG_POST');

// Create document with template
const doc = await writeMagic.createDocument({
    title: "My Blog Post",
    template: 'BLOG_POST'
});
```

## Writing Goals

Track progress with flexible goal types:

```javascript
// Word count goals
writeMagic.setWritingGoal('words', 1000, 'Complete first draft');

// Time-based goals
writeMagic.setWritingGoal('time', 60 * 60 * 1000, 'Write for 1 hour');

// Document goals
writeMagic.setWritingGoal('documents', 3, 'Create chapter outlines');

// Character goals
writeMagic.setWritingGoal('characters', 5000, 'Write 5000 characters');
```

## Session Types

### Regular Sessions
Standard writing sessions with goal tracking and analytics.

### Focus Sessions
Timed sessions with different work/break patterns:

- **Pomodoro**: 25min work + 5min break
- **Sprint**: 15min work + 3min break  
- **Deep Work**: 90min work + 20min break
- **Custom**: User-defined durations

## Persistence and Recovery

- **Auto-save**: Configurable auto-save with user feedback
- **Draft Recovery**: Automatic draft saving and recovery on browser refresh
- **Session Persistence**: Session state survives page reloads
- **Cross-tab Sync**: Coordinate state across multiple browser tabs
- **Offline Support**: Continue writing without internet connection

## Accessibility

- **Keyboard Navigation**: Full keyboard support for all features
- **Screen Reader**: Compatible with screen reader technologies
- **High Contrast**: Support for high contrast themes
- **Flexible Typography**: Configurable font sizes and spacing
- **Focus Management**: Proper focus handling for complex UI

## Performance

- **Debounced Operations**: Auto-save and analysis use debouncing
- **Lazy Loading**: Components load on-demand
- **Memory Management**: Automatic cleanup and garbage collection
- **Compression**: Large drafts are compressed for storage
- **Caching**: Smart caching for frequently accessed data

## Error Handling

```javascript
// Comprehensive error handling
writeMagic.on(WriteMagicEvents.ERROR, (data) => {
    console.error('Operation failed:', data.error);
    
    switch (data.operation) {
        case 'save':
            showRetryDialog();
            break;
        case 'ai_completion':
            fallbackToLocalSuggestions();
            break;
        default:
            showGenericError(data.error.message);
    }
});

// Graceful degradation
try {
    const aiResponse = await writeMagic.completeText(prompt);
} catch (error) {
    // Continue without AI features
    console.log('AI unavailable, continuing without assistance');
}
```

## Configuration Options

```javascript
const config = {
    // AI Providers
    claude_api_key: 'your-key',
    openai_api_key: 'your-key', 
    default_model: 'claude-3-haiku-20240307',
    
    // Performance
    auto_save_delay: 2000,        // Auto-save delay in milliseconds
    idle_timeout: 300000,         // Idle timeout (5 minutes)
    session_timeout: 3600000,     // Session timeout (1 hour)
    max_draft_history: 100,       // Maximum drafts to keep
    
    // Features
    enable_analytics: true,       // Enable writing analytics
    enable_focus_mode: true,      // Enable focus sessions
    enable_collaboration: false,  // Enable real-time collaboration
    enable_accessibility: true,   // Enable accessibility features
    
    // UI
    default_layout: 'focus',      // Default workspace layout
    enable_keyboard_navigation: true,
    
    // Storage
    database_type: 'indexeddb',   // Storage backend
    compression_enabled: true,    // Compress large data
    sync_across_devices: false    // Cross-device synchronization
};
```

## Browser Compatibility

- **Modern Browsers**: Chrome 80+, Firefox 75+, Safari 14+, Edge 80+
- **WebAssembly**: Required for core engine
- **IndexedDB**: Required for local storage
- **Web Workers**: Used for background processing
- **LocalStorage**: Used for preferences and session state

## Development

```bash
# Install dependencies
npm install

# Build WASM module
cd ../../core/wasm && ./build.sh

# Run development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

## Examples

See `example-usage.js` for comprehensive examples including:

- Complete writing application
- Simple document editor
- Focus writing app with Pomodoro timer
- AI-enhanced writing workflow
- Collaborative editing setup

## API Reference

Full API documentation is available in the source code with comprehensive JSDoc comments. Each module exports TypeScript definitions for enhanced developer experience.

## License

WriteMagic JavaScript API is part of the WriteMagic project and follows the same license terms.