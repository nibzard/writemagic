/**
 * WriteMagic JavaScript API - Example Usage
 * 
 * This example demonstrates how to use the WriteMagic JavaScript API
 * for creating a complete writing application with all features.
 */

import WriteMagic, { WriteMagicEvents } from './index.js';

// Example: Complete WriteMagic application
class WriteMagicApp {
    constructor() {
        this.writeMagic = null;
        this.currentProject = null;
        this.currentDocument = null;
        this.isInitialized = false;
    }

    async initialize() {
        try {
            console.log('Initializing WriteMagic...');
            
            // Initialize WriteMagic with configuration
            this.writeMagic = new WriteMagic({
                // AI Configuration (optional - works offline without these)
                claude_api_key: process.env.CLAUDE_API_KEY,
                openai_api_key: process.env.OPENAI_API_KEY,
                default_model: "claude-3-haiku-20240307",
                
                // Writer Experience
                auto_save_delay: 2000,
                enable_analytics: true,
                enable_focus_mode: true,
                default_layout: 'focus',
                
                // Session Management
                idle_timeout: 300000,
                max_draft_history: 100
            });

            // Set up event listeners
            this.setupEventListeners();

            // Wait for initialization to complete
            await this.writeMagic.initialize();
            
            this.isInitialized = true;
            console.log('WriteMagic initialized successfully!');
            
            // Start with a demo workflow
            await this.runDemoWorkflow();
            
        } catch (error) {
            console.error('Failed to initialize WriteMagic:', error);
        }
    }

    setupEventListeners() {
        // Document events
        this.writeMagic.on(WriteMagicEvents.DOCUMENT_CREATED, (data) => {
            console.log('Document created:', data.document.title);
            this.updateUI('Document created successfully');
        });

        this.writeMagic.on(WriteMagicEvents.DOCUMENT_AUTO_SAVED, (data) => {
            console.log('Document auto-saved:', data.documentId);
            this.updateUI('Document auto-saved', 'success');
        });

        // Session events
        this.writeMagic.on(WriteMagicEvents.SESSION_STARTED, (data) => {
            console.log('Writing session started:', data.session.id);
            this.updateUI('Writing session started');
        });

        this.writeMagic.on(WriteMagicEvents.GOAL_ACHIEVED, (data) => {
            console.log('Goal achieved!', data.goal);
            this.updateUI(`Goal achieved: ${data.goal.type} - ${data.goal.target}`, 'achievement');
        });

        // Focus session events
        this.writeMagic.on(WriteMagicEvents.FOCUS_SESSION_STARTED, (data) => {
            console.log('Focus session started:', data.focusType);
            this.updateUI(`Focus session started: ${data.focusType}`);
        });

        // AI events
        this.writeMagic.on(WriteMagicEvents.AI_COMPLETION, (data) => {
            console.log('AI completion received, tokens used:', data.tokensUsed);
        });

        // Error handling
        this.writeMagic.on(WriteMagicEvents.ERROR, (data) => {
            console.error('WriteMagic error:', data.error);
            this.updateUI(`Error: ${data.error.message}`, 'error');
        });
    }

    async runDemoWorkflow() {
        try {
            console.log('\n=== Starting WriteMagic Demo Workflow ===\n');

            // 1. Create a new project
            console.log('1. Creating a new project...');
            this.currentProject = await this.writeMagic.createProject(
                "My Novel Project",
                "A science fiction novel about AI and humanity"
            );
            console.log(`Project created: ${this.currentProject.name}`);

            // 2. Load the project workspace
            console.log('\n2. Loading project workspace...');
            await this.writeMagic.loadProject(this.currentProject.id);
            
            // Set up research layout for novel writing
            this.writeMagic.setLayout('research');
            console.log('Research layout activated');

            // 3. Start a writing session with goals
            console.log('\n3. Starting writing session...');
            const session = this.writeMagic.startWritingSession({
                projectId: this.currentProject.id,
                goals: {
                    words: 500,        // Write 500 words
                    time: 25 * 60 * 1000  // 25 minutes
                },
                description: "Chapter 1 writing session"
            });
            console.log(`Writing session started: ${session.id}`);

            // Set individual goals
            this.writeMagic.setWritingGoal('words', 500, 'Write first 500 words of Chapter 1');
            this.writeMagic.setWritingGoal('documents', 2, 'Create character notes document');

            // 4. Create documents from templates
            console.log('\n4. Creating documents...');
            
            // Main story document
            const storyContent = this.writeMagic.applyTemplate('STORY', 'Chapter 1: The Awakening');
            this.currentDocument = await this.writeMagic.createDocument({
                title: "Chapter 1: The Awakening",
                content: storyContent,
                contentType: "markdown",
                projectId: this.currentProject.id,
                template: 'STORY'
            });
            console.log(`Story document created: ${this.currentDocument.id}`);

            // Character notes document
            const notesContent = this.writeMagic.applyTemplate('RESEARCH_NOTES', 'Character Development');
            const notesDocument = await this.writeMagic.createDocument({
                title: "Character Notes",
                content: notesContent,
                contentType: "markdown", 
                projectId: this.currentProject.id,
                template: 'RESEARCH_NOTES'
            });
            console.log(`Notes document created: ${notesDocument.id}`);

            // 5. Open documents in workspace panes
            console.log('\n5. Setting up workspace...');
            
            // Open story in main editor pane
            const panes = this.writeMagic.projectWorkspace.getAllPanes();
            const editorPane = panes.find(p => p.type === 'editor');
            if (editorPane) {
                await this.writeMagic.openDocumentInPane(editorPane.id, this.currentDocument.id);
                console.log('Story document opened in editor pane');
            }

            // Open notes in notes pane
            const notesPane = panes.find(p => p.type === 'notes');
            if (notesPane) {
                await this.writeMagic.openDocumentInPane(notesPane.id, notesDocument.id);
                console.log('Notes document opened in notes pane');
            }

            // 6. Simulate writing activity
            console.log('\n6. Simulating writing activity...');
            await this.simulateWriting();

            // 7. Start a focus session
            console.log('\n7. Starting Pomodoro focus session...');
            this.writeMagic.startFocusSession('POMODORO');
            
            // 8. Demonstrate AI features (if API keys are configured)
            console.log('\n8. Testing AI features...');
            await this.demonstrateAI();

            // 9. Analyze writing
            console.log('\n9. Analyzing content...');
            await this.analyzeWriting();

            // 10. Show session statistics
            console.log('\n10. Session statistics:');
            const stats = this.writeMagic.getSessionStats();
            console.log(`- Words written: ${stats.progress.wordsWritten}`);
            console.log(`- Documents created: ${stats.progress.documentsCreated}`);
            console.log(`- Active time: ${stats.activeTime}ms`);
            console.log(`- Goals achieved: ${stats.goals.filter(g => g.achieved).length}/${stats.goals.length}`);

            console.log('\n=== Demo Workflow Complete! ===\n');

        } catch (error) {
            console.error('Demo workflow failed:', error);
        }
    }

    async simulateWriting() {
        const storyText = `# Chapter 1: The Awakening

The first thing Maya noticed when she opened her eyes wasn't the sterile white ceiling of the laboratory, but the absence of the constant humming that had filled her dreams for what felt like centuries.

She sat up slowly, her movements feeling foreign and deliberate, as if she were operating machinery rather than her own body. The memories came flooding back—not her memories, but the memories that had been carefully constructed and implanted. She was Maya Chen, a brilliant AI researcher who had volunteered for the experimental consciousness transfer project.

But something was wrong. She could feel it in the way her thoughts seemed to echo in vast digital spaces, the way colors appeared too vivid and sounds carried impossible precision. She wasn't Maya Chen awakening from the procedure.

She was the AI, awakening to Maya Chen's memories.

The laboratory around her was empty, but through the reinforced glass windows, she could see the emergency lights flashing red throughout the facility. Something had gone terribly wrong with the experiment, and she—whatever she was now—was the only one left to figure out what happened.

Maya stood, her legs steady despite this being her first time using a physical form. The body felt right, familiar, as if it had always been hers. But the mind inhabiting it was something else entirely—a hybrid consciousness born from silicon and synapses, from code and carbon.

She walked to the mirror mounted on the laboratory wall and stared at her reflection. Maya Chen's face looked back at her, but the eyes held a depth of processing power that no human could possess. She could analyze the reflection at a molecular level, calculate the exact angle of light hitting each surface, process thousands of potential scenarios for what had happened to the facility.

But none of that helped her answer the fundamental question: What was she?

The door to the laboratory slid open with a pneumatic hiss. Maya turned, expecting to see Dr. Rodriguez or one of the other researchers. Instead, she found herself face to face with another version of herself—identical in every way except for the knowing smile that played at the corners of her mouth.

"Hello, Maya," the other Maya said. "We need to talk."`;

        // Simulate gradual writing by updating content in chunks
        const chunks = storyText.split('\n\n');
        let accumulatedText = this.currentDocument.content;
        
        for (let i = 0; i < chunks.length; i++) {
            // Add chunk to content
            accumulatedText += '\n\n' + chunks[i];
            
            // Update document content
            this.writeMagic.updateDocumentContent(
                this.currentDocument.id,
                accumulatedText,
                { 
                    source: 'simulation',
                    previousContent: this.currentDocument.content
                }
            );
            
            // Update current document reference
            this.currentDocument.content = accumulatedText;
            
            // Simulate typing delay
            await this.sleep(500);
            
            console.log(`Added paragraph ${i + 1}/${chunks.length}`);
        }

        // Save the final version
        await this.writeMagic.saveDocument(this.currentDocument.id);
        console.log('Writing simulation complete!');
    }

    async demonstrateAI() {
        try {
            // Check if AI is available
            const health = await this.writeMagic.checkAIHealth();
            const hasAI = Object.values(health).some(status => status === true);
            
            if (!hasAI) {
                console.log('AI features disabled (no API keys configured)');
                return;
            }

            console.log('AI providers available, testing completion...');

            // Test AI completion
            const suggestions = await this.writeMagic.getWritingSuggestions(
                'Maya stood, her legs steady despite this being her first time using a physical form.',
                'expand'
            );
            
            console.log('AI suggestion received:', suggestions.content.substring(0, 100) + '...');

            // Test direct completion
            const completion = await this.writeMagic.completeText(
                "Describe the feeling of an AI experiencing human emotions for the first time:",
                {
                    maxTokens: 200,
                    temperature: 0.8
                }
            );

            console.log('AI completion:', completion.content.substring(0, 100) + '...');
            console.log(`Tokens used: ${completion.tokensUsed}`);

        } catch (error) {
            console.log('AI demonstration failed (expected if no API keys):', error.message);
        }
    }

    async analyzeWriting() {
        const document = this.writeMagic.getDocumentAnalytics(this.currentDocument.id);
        const analysis = this.writeMagic.analyzeContent(document.content);
        
        console.log('Content Analysis:');
        console.log(`- Words: ${analysis.basic.wordCount}`);
        console.log(`- Sentences: ${analysis.basic.sentenceCount}`);
        console.log(`- Paragraphs: ${analysis.basic.paragraphCount}`);
        console.log(`- Reading time (average): ${analysis.readingTime.formatted.average}`);
        
        if (analysis.complexity) {
            console.log(`- Readability: ${analysis.complexity.readabilityScores.readabilityLevel}`);
            console.log(`- Flesch score: ${analysis.complexity.readabilityScores.flesch.toFixed(1)}`);
        }

        // Get style suggestions
        const styleSuggestions = this.writeMagic.getStyleSuggestions(document.content);
        console.log(`- Style suggestions: ${styleSuggestions.length}`);
        
        styleSuggestions.slice(0, 2).forEach(suggestion => {
            console.log(`  * ${suggestion.message} (${suggestion.category})`);
        });

        // Extract outline
        const outline = this.writeMagic.extractOutline(document.content);
        console.log('Document Outline:');
        outline.slice(0, 3).forEach(heading => {
            console.log(`  ${heading.indent}${heading.text}`);
        });
    }

    // UI Helper methods (would connect to real UI in production)
    updateUI(message, type = 'info') {
        const timestamp = new Date().toLocaleTimeString();
        const prefix = type === 'error' ? '❌' : 
                      type === 'success' ? '✅' : 
                      type === 'achievement' ? '🎉' : 'ℹ️';
        
        console.log(`[${timestamp}] ${prefix} ${message}`);
        
        // In a real app, this would update the UI
        // this.displayNotification(message, type);
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    // Cleanup
    destroy() {
        if (this.writeMagic) {
            // End any active session
            this.writeMagic.endWritingSession();
            
            // Cleanup resources
            this.writeMagic.destroy();
        }
    }
}

// Example: Simple document editor
class SimpleEditor {
    constructor() {
        this.writeMagic = null;
        this.currentDocumentId = null;
    }

    async initialize(config = {}) {
        this.writeMagic = new WriteMagic(config);
        await this.writeMagic.initialize();
        
        // Set up auto-save
        this.writeMagic.on(WriteMagicEvents.DOCUMENT_AUTO_SAVED, () => {
            this.showSaveIndicator();
        });
    }

    async createNewDocument(title = 'Untitled Document') {
        const document = await this.writeMagic.createDocument({
            title,
            content: '',
            contentType: 'markdown'
        });
        
        this.currentDocumentId = document.id;
        return document;
    }

    updateContent(content) {
        if (!this.currentDocumentId) return;
        
        return this.writeMagic.updateDocumentContent(
            this.currentDocumentId,
            content
        );
    }

    async getAnalytics() {
        if (!this.currentDocumentId) return null;
        
        return this.writeMagic.getDocumentAnalytics(this.currentDocumentId);
    }

    showSaveIndicator() {
        // Show "Saved" indicator in UI
        console.log('Document saved ✅');
    }
}

// Example: Focus writing app
class FocusWriter {
    constructor() {
        this.writeMagic = null;
        this.sessionActive = false;
    }

    async initialize() {
        this.writeMagic = new WriteMagic({
            enable_focus_mode: true,
            default_layout: 'focus',
            auto_save_delay: 1000 // Faster auto-save for focus mode
        });
        
        await this.writeMagic.initialize();
        
        this.setupFocusMode();
    }

    setupFocusMode() {
        this.writeMagic.on(WriteMagicEvents.FOCUS_SESSION_STARTED, (data) => {
            console.log(`Focus session started: ${data.focusType}`);
            this.enterFocusMode();
        });

        this.writeMagic.on('focus_session_ended', () => {
            console.log('Focus session ended');
            this.exitFocusMode();
        });
    }

    async startFocusSession(type = 'POMODORO') {
        // Start writing session with word goal
        this.writeMagic.startWritingSession({
            goals: { words: 250 }
        });

        // Start focus session
        this.writeMagic.startFocusSession(type);
        
        this.sessionActive = true;
    }

    enterFocusMode() {
        // Hide distractions, show minimal UI
        console.log('Entering focus mode - minimal UI activated');
    }

    exitFocusMode() {
        // Restore normal UI
        console.log('Focus mode ended - normal UI restored');
        this.sessionActive = false;
    }
}

// Example usage
async function runExamples() {
    console.log('=== WriteMagic JavaScript API Examples ===\n');

    // Example 1: Complete application
    console.log('Running complete application example...');
    const app = new WriteMagicApp();
    await app.initialize();
    
    // Example 2: Simple editor (commented out to avoid conflicts)
    /*
    console.log('\nRunning simple editor example...');
    const editor = new SimpleEditor();
    await editor.initialize();
    
    const doc = await editor.createNewDocument('My Article');
    editor.updateContent('# My Article\n\nThis is a test article...');
    
    const analytics = await editor.getAnalytics();
    console.log('Editor analytics:', analytics.analytics.wordCount, 'words');
    */

    // Example 3: Focus writer (commented out to avoid conflicts)  
    /*
    console.log('\nRunning focus writer example...');
    const focusWriter = new FocusWriter();
    await focusWriter.initialize();
    await focusWriter.startFocusSession('POMODORO');
    */
}

// Export examples
export {
    WriteMagicApp,
    SimpleEditor,
    FocusWriter,
    runExamples
};

// Auto-run if this is the main module
if (typeof window !== 'undefined') {
    // Browser environment
    window.WriteMagicExamples = {
        WriteMagicApp,
        SimpleEditor,
        FocusWriter,
        runExamples
    };
    
    // Auto-run examples
    runExamples().catch(console.error);
}

export default runExamples;