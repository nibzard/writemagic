/**
 * Example usage of WriteMagic WASM engine
 */

import init, { 
    WriteMagicEngine, 
    WasmCompletionRequest,
    init_logging,
    init_panic_hook 
} from './pkg/writemagic_wasm.js';

async function main() {
    // Initialize WASM module
    await init();
    
    // Initialize logging and panic hook for debugging
    init_logging();
    init_panic_hook();
    
    // Create and configure the engine
    const engine = new WriteMagicEngine();
    
    // Configuration with AI providers (optional)
    const config = {
        claude_api_key: process.env.CLAUDE_API_KEY, // Optional
        openai_api_key: process.env.OPENAI_API_KEY, // Optional
        default_model: "claude-3-haiku-20240307",
        log_level: "info",
        enable_content_filtering: true,
        database_type: "indexeddb"
    };
    
    try {
        // Initialize the engine
        await engine.initialize(JSON.stringify(config));
        console.log('Engine initialized:', engine.is_initialized());
        
        // Create a new document
        const document = await engine.create_document(
            "My First Document",
            "This is the initial content of my document.",
            "markdown", // content type
            null // created_by (optional)
        );
        console.log('Created document:', document.id, document.title);
        
        // Update the document
        const updatedDoc = await engine.update_document(
            document.id,
            "This is the updated content with more information.",
            null // updated_by (optional)
        );
        console.log('Updated document word count:', updatedDoc.word_count);
        
        // Create a project
        const project = await engine.create_project(
            "My Writing Project",
            "A sample project for testing the WASM engine",
            null // created_by (optional)
        );
        console.log('Created project:', project.id, project.name);
        
        // Add document to project
        const updatedProject = await engine.add_document_to_project(
            project.id,
            document.id,
            null // updated_by (optional)
        );
        console.log('Project now has documents:', updatedProject.document_ids.length);
        
        // List documents in the project
        const projectDocuments = await engine.list_project_documents(project.id);
        console.log('Documents in project:', projectDocuments.length);
        
        // Test AI completion (if API keys are configured)
        if (config.claude_api_key || config.openai_api_key) {
            try {
                const completionRequest = new WasmCompletionRequest(
                    "Write a creative opening paragraph for a science fiction story.",
                    "claude-3-haiku-20240307"
                );
                completionRequest.set_max_tokens(150);
                completionRequest.set_temperature(0.7);
                
                const aiResponse = await engine.complete_text(completionRequest);
                console.log('AI Response:', aiResponse.content);
                console.log('Tokens used:', aiResponse.tokens_used);
                
                // Check AI provider health
                const health = await engine.get_ai_provider_health();
                console.log('AI Provider Health:', health);
                
            } catch (aiError) {
                console.log('AI completion failed (API keys may not be configured):', aiError.message);
            }
        } else {
            console.log('AI features disabled - no API keys configured');
        }
        
        // Retrieve document by ID
        const retrievedDoc = await engine.get_document(document.id);
        console.log('Retrieved document:', retrievedDoc.title, 'Characters:', retrievedDoc.character_count);
        
        // Retrieve project by ID
        const retrievedProject = await engine.get_project(project.id);
        console.log('Retrieved project:', retrievedProject.name, 'Documents:', retrievedProject.document_ids.length);
        
    } catch (error) {
        console.error('Error:', error.message, 'Code:', error.code);
    }
}

main().catch(console.error);