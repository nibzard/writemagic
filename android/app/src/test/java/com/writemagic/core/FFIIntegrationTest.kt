package com.writemagic.core

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Tests for FFI integration, data marshaling, and error handling.
 * Focuses on the boundary between Kotlin and Rust code.
 */
@ExperimentalCoroutinesApi
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [33])
class FFIIntegrationTest {

    @Test
    fun `JSON serialization should work for Document`() {
        val document = Document(
            id = "test-123",
            title = "Test Document",
            content = "This is test content with special characters: àáâã",
            contentType = "markdown",
            wordCount = 8,
            characterCount = 48,
            createdAt = "2024-01-01T12:00:00Z",
            updatedAt = "2024-01-01T12:30:00Z",
            version = 1,
            isDeleted = false
        )

        // Test serialization
        val jsonString = Json.encodeToString(Document.serializer(), document)
        assertNotNull(jsonString)
        assertTrue(jsonString.contains("test-123"))
        assertTrue(jsonString.contains("Test Document"))

        // Test deserialization
        val deserializedDocument = Json.decodeFromString<Document>(jsonString)
        assertEquals(document.id, deserializedDocument.id)
        assertEquals(document.title, deserializedDocument.title)
        assertEquals(document.content, deserializedDocument.content)
        assertEquals(document.wordCount, deserializedDocument.wordCount)
    }

    @Test
    fun `JSON serialization should handle special characters in content`() {
        val document = Document(
            id = "special-chars",
            title = "Special Characters Test",
            content = "Content with emojis 🚀🎉 and unicode: ñáéíóú 中文 العربية",
            contentType = "markdown",
            wordCount = 7,
            characterCount = 55,
            createdAt = "2024-01-01T00:00:00Z",
            updatedAt = "2024-01-01T00:00:00Z",
            version = 1
        )

        val jsonString = Json.encodeToString(Document.serializer(), document)
        val deserialized = Json.decodeFromString<Document>(jsonString)
        
        assertEquals(document.content, deserialized.content)
        assertTrue(deserialized.content.contains("🚀"))
        assertTrue(deserialized.content.contains("中文"))
    }

    @Test
    fun `DocumentList JSON serialization should handle empty and populated lists`() {
        // Test empty list
        val emptyList = DocumentList(emptyList(), 0)
        val emptyJson = Json.encodeToString(DocumentList.serializer(), emptyList)
        val deserializedEmpty = Json.decodeFromString<DocumentList>(emptyJson)
        
        assertEquals(0, deserializedEmpty.count)
        assertTrue(deserializedEmpty.documents.isEmpty())

        // Test populated list
        val documents = listOf(
            Document("1", "Doc 1", "Content 1", "markdown", 2, 9, "2024-01-01T00:00:00Z", "2024-01-01T00:00:00Z", 1),
            Document("2", "Doc 2", "Content 2", "text", 2, 9, "2024-01-01T00:00:00Z", "2024-01-01T00:00:00Z", 1)
        )
        val populatedList = DocumentList(documents, 2)
        val populatedJson = Json.encodeToString(DocumentList.serializer(), populatedList)
        val deserializedPopulated = Json.decodeFromString<DocumentList>(populatedJson)
        
        assertEquals(2, deserializedPopulated.count)
        assertEquals(2, deserializedPopulated.documents.size)
        assertEquals("Doc 1", deserializedPopulated.documents[0].title)
    }

    @Test
    fun `AIResponse JSON serialization should handle success and error cases`() {
        // Test success case
        val successResponse = AIResponse(
            completion = "This is a successful AI completion with various punctuation: !@#$%^&*()",
            error = null,
            success = true
        )
        
        val successJson = Json.encodeToString(AIResponse.serializer(), successResponse)
        val deserializedSuccess = Json.decodeFromString<AIResponse>(successJson)
        
        assertTrue(deserializedSuccess.success)
        assertNotNull(deserializedSuccess.completion)
        assertEquals(successResponse.completion, deserializedSuccess.completion)

        // Test error case
        val errorResponse = AIResponse(
            completion = null,
            error = "Network timeout: Failed to connect to AI service after 30 seconds",
            success = false
        )
        
        val errorJson = Json.encodeToString(AIResponse.serializer(), errorResponse)
        val deserializedError = Json.decodeFromString<AIResponse>(errorJson)
        
        assertFalse(deserializedError.success)
        assertNotNull(deserializedError.error)
        assertEquals(errorResponse.error, deserializedError.error)
    }

    @Test
    fun `JSON parsing should handle malformed data gracefully`() {
        // Test malformed Document JSON
        val malformedDocumentJson = """{"id":"test","title":"Test"}""" // Missing required fields
        
        assertFailsWith<Exception> {
            Json.decodeFromString<Document>(malformedDocumentJson)
        }

        // Test invalid JSON structure
        val invalidJson = """{"invalid": json structure}"""
        
        assertFailsWith<Exception> {
            Json.decodeFromString<Document>(invalidJson)
        }
    }

    @Test
    fun `Document model should validate required fields`() {
        // All required fields present
        val validDocument = Document(
            id = "valid-id",
            title = "Valid Title",
            content = "Valid content",
            contentType = "markdown",
            wordCount = 2,
            characterCount = 13,
            createdAt = "2024-01-01T00:00:00Z",
            updatedAt = "2024-01-01T00:00:00Z",
            version = 1
        )
        
        // Verify all required fields are set
        assertTrue(validDocument.id.isNotBlank())
        assertTrue(validDocument.title.isNotBlank())
        assertTrue(validDocument.contentType.isNotBlank())
        assertTrue(validDocument.wordCount >= 0)
        assertTrue(validDocument.characterCount >= 0)
        assertTrue(validDocument.version > 0)
    }

    @Test
    fun `FFI error handling should manage native library loading failures`() = runTest {
        // Test behavior when native library is not available
        try {
            // This should trigger UnsatisfiedLinkError in test environment
            WriteMagicCore.initialize("test-key", "test-key")
        } catch (e: UnsatisfiedLinkError) {
            // Expected in test environment without native library
            assertTrue(e.message?.contains("writemagic_android") == true || e.message?.contains("no writemagic_android") == true)
        }
    }

    @Test
    fun `Data marshaling should preserve precision for numeric fields`() {
        val document = Document(
            id = "precision-test",
            title = "Precision Test",
            content = "Test content for precision validation",
            contentType = "markdown",
            wordCount = 123456,
            characterCount = 987654321,
            createdAt = "2024-12-31T23:59:59.999Z",
            updatedAt = "2024-12-31T23:59:59.999Z",
            version = 999999
        )

        val json = Json.encodeToString(Document.serializer(), document)
        val deserialized = Json.decodeFromString<Document>(json)

        assertEquals(123456, deserialized.wordCount)
        assertEquals(987654321, deserialized.characterCount)
        assertEquals(999999, deserialized.version)
    }

    @Test
    fun `Content validation should handle edge cases`() {
        // Test very long content
        val longContent = "a".repeat(100000)
        val longDocument = Document(
            id = "long-content",
            title = "Long Content Test",
            content = longContent,
            contentType = "markdown",
            wordCount = 1,
            characterCount = 100000,
            createdAt = "2024-01-01T00:00:00Z",
            updatedAt = "2024-01-01T00:00:00Z",
            version = 1
        )

        val json = Json.encodeToString(Document.serializer(), longDocument)
        val deserialized = Json.decodeFromString<Document>(json)
        assertEquals(longContent, deserialized.content)

        // Test empty content
        val emptyDocument = Document(
            id = "empty-content",
            title = "Empty Content Test",
            content = "",
            contentType = "markdown",
            wordCount = 0,
            characterCount = 0,
            createdAt = "2024-01-01T00:00:00Z",
            updatedAt = "2024-01-01T00:00:00Z",
            version = 1
        )

        val emptyJson = Json.encodeToString(Document.serializer(), emptyDocument)
        val deserializedEmpty = Json.decodeFromString<Document>(emptyJson)
        assertEquals("", deserializedEmpty.content)
        assertEquals(0, deserializedEmpty.wordCount)
        assertEquals(0, deserializedEmpty.characterCount)
    }
}