package com.writemagic.core

import android.util.Log
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString

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
 * Handles all FFI calls to the native library.
 */
object WriteMagicCore {
    private const val TAG = "WriteMagicCore"
    private var isInitialized = false
    
    init {
        try {
            System.loadLibrary("writemagic_android")
            Log.i(TAG, "Native library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load native library: ${e.message}")
        }
    }
    
    /**
     * Initialize the WriteMagic core engine with AI integration
     */
    suspend fun initialize(claudeKey: String = "", openaiKey: String = ""): Boolean = withContext(Dispatchers.IO) {
        if (isInitialized) {
            Log.w(TAG, "Core already initialized")
            return@withContext true
        }
        
        Log.i(TAG, "Initializing WriteMagic core...")
        val result = nativeInitialize(claudeKey, openaiKey)
        isInitialized = result
        
        if (result) {
            Log.i(TAG, "WriteMagic core initialized successfully")
        } else {
            Log.e(TAG, "Failed to initialize WriteMagic core")
        }
        
        result
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
            val jsonResult = nativeCreateDocument(title, content, contentType)
            if (jsonResult != null) {
                Json.decodeFromString<Document>(jsonResult)
            } else {
                Log.e(TAG, "Failed to create document")
                null
            }
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
        
        val result = nativeUpdateDocumentContent(documentId, content)
        if (!result) {
            Log.e(TAG, "Failed to update document $documentId")
        }
        result
    }
    
    /**
     * Get document by ID
     */
    suspend fun getDocument(documentId: String): Document? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        try {
            val jsonResult = nativeGetDocument(documentId)
            if (jsonResult != null) {
                Json.decodeFromString<Document>(jsonResult)
            } else {
                Log.w(TAG, "Document $documentId not found")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting document: ${e.message}")
            null
        }
    }
    
    /**
     * List all documents with pagination
     */
    suspend fun listDocuments(offset: Int = 0, limit: Int = 50): DocumentList? = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext null
        }
        
        try {
            val jsonResult = nativeListDocuments(offset, limit)
            if (jsonResult != null) {
                Json.decodeFromString<DocumentList>(jsonResult)
            } else {
                Log.e(TAG, "Failed to list documents")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error listing documents: ${e.message}")
            null
        }
    }
    
    /**
     * Complete text using AI with provider fallback
     */
    suspend fun completeText(prompt: String, model: String? = null): AIResponse = withContext(Dispatchers.IO) {
        if (!isInitialized) {
            Log.e(TAG, "Core not initialized")
            return@withContext AIResponse(error = "Core not initialized", success = false)
        }
        
        try {
            val jsonResult = nativeCompleteText(prompt, model ?: "")
            if (jsonResult != null) {
                Json.decodeFromString<AIResponse>(jsonResult)
            } else {
                Log.e(TAG, "AI completion failed")
                AIResponse(error = "AI completion failed", success = false)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error completing text: ${e.message}")
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
            val jsonResult = nativeCreateProject(name, description)
            if (jsonResult != null) {
                Log.i(TAG, "Successfully created project: $name")
                jsonResult
            } else {
                Log.e(TAG, "Failed to create project")
                null
            }
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
        
        try {
            val jsonResult = nativeGetProject(projectId)
            if (jsonResult != null) {
                jsonResult
            } else {
                Log.w(TAG, "Project $projectId not found")
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting project: ${e.message}")
            null
        }
    }
    
    // Native FFI method declarations
    private external fun nativeInitialize(claudeKey: String, openaiKey: String): Boolean
    private external fun nativeCreateDocument(title: String, content: String, contentType: String): String?
    private external fun nativeUpdateDocumentContent(documentId: String, content: String): Boolean
    private external fun nativeGetDocument(documentId: String): String?
    private external fun nativeListDocuments(offset: Int, limit: Int): String?
    private external fun nativeCompleteText(prompt: String, model: String): String?
    private external fun nativeCreateProject(name: String, description: String): String?
    private external fun nativeGetProject(projectId: String): String?
}