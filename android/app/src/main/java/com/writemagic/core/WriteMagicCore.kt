package com.writemagic.core

import android.util.Log
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import java.util.UUID
import java.util.Date
import java.text.SimpleDateFormat
import java.util.Locale

@Serializable
data class Document(
    val id: String,
    val title: String,
    val content: String,
    val contentType: String,
    val wordCount: Int,
    val characterCount: Int,
    val createdAt: String,
    val updatedAt: String,
    val version: Int,
    val isDeleted: Boolean = false
)

@Serializable
data class DocumentList(
    val documents: List<Document>,
    val count: Int
)

@Serializable
data class AIResponse(
    val completion: String? = null,
    val error: String? = null,
    val success: Boolean
)

/**
 * Main interface to the WriteMagic Rust core engine.
 * This version uses a working in-memory implementation while the native library is being built.
 */
object WriteMagicCore {
    private const val TAG = "WriteMagicCore"
    private var isInitialized = false
    private val documents = mutableMapOf<String, Document>()
    private val projects = mutableMapOf<String, String>()
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss'Z'", Locale.US)
    
    // Simulated AI responses for development
    private val aiResponses = listOf(
        "Here's a helpful suggestion based on your input. This response is generated locally while the full AI integration is being built.",
        "I can help you improve this text. Consider adding more detail to make your point clearer.",
        "Your writing looks good! You might want to vary your sentence structure for better flow.",
        "This is an interesting topic. Have you considered exploring it from a different angle?",
        "The core ideas are solid. Adding specific examples would strengthen your argument."
    )
    
    init {
        // Initialize with some sample content
        createSampleDocument("welcome", "Welcome to WriteMagic", """
            # Welcome to WriteMagic
            
            WriteMagic is an AI-powered writing assistant that helps you create, organize, and improve your content.
            
            ## Features
            - **AI-Powered Writing**: Get intelligent suggestions and completions
            - **Document Management**: Organize your writing projects
            - **Modern Interface**: Clean, professional Material 3 design
            - **Cross-Platform**: Works on Android, iOS, and Web
            
            ## Getting Started
            1. Create a new document from the Documents tab
            2. Start writing and use AI assistance when needed
            3. Organize your work into projects
            4. Export your finished content
            
            Happy writing!
        """.trimIndent())
        
        createSampleDocument("guide", "Quick Start Guide", """
            # Quick Start Guide
            
            ## Creating Documents
            Tap the "+" button to create a new document. Give it a meaningful title and start writing.
            
            ## Using AI Features
            - Select text and tap "AI Assist" for suggestions
            - Use the AI tab for longer-form content generation
            - Ask questions about your writing for feedback
            
            ## Organization
            - Group related documents into projects
            - Use the timeline to track your writing progress
            - Export documents in various formats when complete
            
            ## Tips
            - Save frequently (auto-save is enabled)
            - Use markdown for rich formatting
            - Experiment with different AI prompts for best results
        """.trimIndent())
    }
    
    private fun createSampleDocument(id: String, title: String, content: String) {
        val now = dateFormat.format(Date())
        val doc = Document(
            id = id,
            title = title,
            content = content,
            contentType = "markdown",
            wordCount = content.split("\\s+".toRegex()).size,
            characterCount = content.length,
            createdAt = now,
            updatedAt = now,
            version = 1
        )
        documents[id] = doc
    }
    
    /**
     * Initialize the WriteMagic core engine
     */
    suspend fun initialize(claudeKey: String = "", openaiKey: String = ""): Boolean = withContext(Dispatchers.IO) {
        if (isInitialized) {
            Log.w(TAG, "Core already initialized")
            return@withContext true
        }
        
        Log.i(TAG, "Initializing WriteMagic core (in-memory mode)...")
        isInitialized = true
        Log.i(TAG, "WriteMagic core initialized successfully")
        true
    }
    
    /**
     * Create a new document
     */
    suspend fun createDocument(
        title: String, 
        content: String = "", 
        contentType: String = "markdown"
    ): Document? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        try {
            val id = UUID.randomUUID().toString()
            val now = dateFormat.format(Date())
            val doc = Document(
                id = id,
                title = title,
                content = content,
                contentType = contentType,
                wordCount = if (content.isBlank()) 0 else content.split("\\s+".toRegex()).size,
                characterCount = content.length,
                createdAt = now,
                updatedAt = now,
                version = 1
            )
            documents[id] = doc
            Log.i(TAG, "Created document: $title")
            doc
        } catch (e: Exception) {
            Log.e(TAG, "Error creating document: ${e.message}")
            null
        }
    }
    
    /**
     * Update document content
     */
    suspend fun updateDocumentContent(documentId: String, content: String): Boolean = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext false
        }
        
        val doc = documents[documentId]
        if (doc != null) {
            val now = dateFormat.format(Date())
            val updatedDoc = doc.copy(
                content = content,
                wordCount = if (content.isBlank()) 0 else content.split("\\s+".toRegex()).size,
                characterCount = content.length,
                updatedAt = now,
                version = doc.version + 1
            )
            documents[documentId] = updatedDoc
            Log.i(TAG, "Updated document: $documentId")
            true
        } else {
            Log.w(TAG, "Document not found: $documentId")
            false
        }
    }
    
    /**
     * Get document by ID
     */
    suspend fun getDocument(documentId: String): Document? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        documents[documentId]
    }
    
    /**
     * List all documents with pagination
     */
    suspend fun listDocuments(offset: Int = 0, limit: Int = 50): DocumentList? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        val allDocs = documents.values.toList().sortedByDescending { it.updatedAt }
        val paginatedDocs = allDocs.drop(offset).take(limit)
        DocumentList(
            documents = paginatedDocs,
            count = allDocs.size
        )
    }
    
    /**
     * Complete text using AI simulation
     */
    suspend fun completeText(prompt: String, model: String? = null): AIResponse = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext AIResponse(error = "Core not initialized", success = false)
        }
        
        try {
            // Simulate AI processing time
            delay(1500)
            
            val response = when {
                prompt.contains("help", ignoreCase = true) -> "I'd be happy to help! Could you provide more details about what you're working on?"
                prompt.contains("improve", ignoreCase = true) -> "To improve this text, consider adding more specific examples and varying your sentence structure."
                prompt.contains("write", ignoreCase = true) -> "Here's a suggestion for your writing: Start with a clear topic sentence, then provide supporting details."
                else -> aiResponses.random()
            }
            
            AIResponse(completion = response, success = true)
        } catch (e: Exception) {
            Log.e(TAG, "Error in AI completion: ${e.message}")
            AIResponse(error = e.message ?: "Unknown error", success = false)
        }
    }
    
    /**
     * Create a new project
     */
    suspend fun createProject(name: String, description: String = ""): String? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        try {
            val id = UUID.randomUUID().toString()
            val projectData = Json.encodeToString(mapOf(
                "id" to id,
                "name" to name,
                "description" to description,
                "createdAt" to dateFormat.format(Date()),
                "documentIds" to emptyList<String>()
            ))
            projects[id] = projectData
            Log.i(TAG, "Created project: $name")
            projectData
        } catch (e: Exception) {
            Log.e(TAG, "Error creating project: ${e.message}")
            null
        }
    }
    
    /**
     * Get project by ID
     */
    suspend fun getProject(projectId: String): String? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        projects[projectId]
    }
}