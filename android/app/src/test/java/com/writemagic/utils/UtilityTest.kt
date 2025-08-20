package com.writemagic.utils

import org.junit.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Utility functions for testing and helper methods.
 * Contains common test utilities and validation functions.
 */
class UtilityTest {

    @Test
    fun textStatistics_CalculateWordCount() {
        // Test basic word counting
        assertEquals(0, "".wordCount())
        assertEquals(1, "hello".wordCount())
        assertEquals(2, "hello world".wordCount())
        assertEquals(5, "The quick brown fox jumps".wordCount())
        
        // Test with multiple spaces
        assertEquals(3, "hello    world   test".wordCount())
        
        // Test with newlines and tabs
        assertEquals(4, "hello\nworld\ttest\nfoo".wordCount())
        
        // Test with punctuation
        assertEquals(6, "Hello, world! How are you?".wordCount())
        
        // Test with numbers
        assertEquals(4, "I have 25 cats total".wordCount())
    }

    @Test
    fun textStatistics_CalculateCharacterCount() {
        // Test character counting including spaces
        assertEquals(0, "".characterCount())
        assertEquals(5, "hello".characterCount())
        assertEquals(11, "hello world".characterCount())
        assertEquals(26, "The quick brown fox jumps!".characterCount())
        
        // Test with special characters
        assertEquals(13, "hello 🌍 world".characterCount())
        assertEquals(21, "Testing: àáâã ñáéíóú".characterCount())
    }

    @Test
    fun textValidation_ValidateDocumentTitle() {
        // Valid titles
        assertTrue("Valid Title".isValidDocumentTitle())
        assertTrue("Document 1".isValidDocumentTitle())
        assertTrue("My Novel - Chapter 1".isValidDocumentTitle())
        
        // Invalid titles
        assertFalse("".isValidDocumentTitle())
        assertFalse("   ".isValidDocumentTitle())
        assertFalse("a".repeat(256).isValidDocumentTitle()) // Too long
        
        // Edge cases
        assertTrue("A".isValidDocumentTitle()) // Minimum valid
        assertTrue("a".repeat(255).isValidDocumentTitle()) // Maximum valid
    }

    @Test
    fun textValidation_ValidateContentType() {
        // Valid content types
        assertTrue("markdown".isValidContentType())
        assertTrue("text".isValidContentType())
        assertTrue("html".isValidContentType())
        assertTrue("json".isValidContentType())
        
        // Invalid content types
        assertFalse("".isValidContentType())
        assertFalse("unknown".isValidContentType())
        assertFalse("MARKDOWN".isValidContentType()) // Case sensitive
        assertFalse("text/plain".isValidContentType()) // MIME type not supported
    }

    @Test
    fun dateTimeFormatting_RelativeTimestamps() {
        // Test relative time formatting
        // Note: These would need actual implementation of time utilities
        
        val now = System.currentTimeMillis()
        val oneHourAgo = now - (60 * 60 * 1000)
        val oneDayAgo = now - (24 * 60 * 60 * 1000)
        val oneWeekAgo = now - (7 * 24 * 60 * 60 * 1000)
        
        // These tests assume implementation of formatRelativeTime function
        assertEquals("1 hour ago", oneHourAgo.formatRelativeTime())
        assertEquals("1 day ago", oneDayAgo.formatRelativeTime())
        assertEquals("1 week ago", oneWeekAgo.formatRelativeTime())
        assertEquals("Just now", now.formatRelativeTime())
    }

    @Test
    fun textProcessing_SanitizeInput() {
        // Test input sanitization
        assertEquals("hello world", "hello world".sanitizeInput())
        assertEquals("test content", "test content\r\n".sanitizeInput())
        assertEquals("normal text", "normal text\u0000".sanitizeInput()) // Remove null bytes
        assertEquals("clean", "clean\u200B".sanitizeInput()) // Remove zero-width space
        
        // Preserve intentional formatting
        assertEquals("line1\nline2", "line1\nline2".sanitizeInput())
        assertEquals("tab\ttab", "tab\ttab".sanitizeInput())
    }

    @Test
    fun filePathValidation_ValidatePaths() {
        // Test file path validation
        assertTrue("/valid/path/to/file.txt".isValidFilePath())
        assertTrue("relative/path/file.md".isValidFilePath())
        assertTrue("file.txt".isValidFilePath())
        
        // Invalid paths
        assertFalse("".isValidFilePath())
        assertFalse("../../../etc/passwd".isValidFilePath()) // Path traversal
        assertFalse("file\u0000name.txt".isValidFilePath()) // Null byte
        assertFalse("con.txt".isValidFilePath()) // Windows reserved name
        assertFalse("file<>.txt".isValidFilePath()) // Invalid characters
    }

    @Test
    fun jsonEscaping_EscapeSpecialCharacters() {
        // Test JSON string escaping
        assertEquals("hello", "hello".escapeJsonString())
        assertEquals("hello\\nworld", "hello\nworld".escapeJsonString())
        assertEquals("quote\\\"test", "quote\"test".escapeJsonString())
        assertEquals("backslash\\\\test", "backslash\\test".escapeJsonString())
        assertEquals("tab\\ttest", "tab\ttest".escapeJsonString())
        
        // Test unicode escaping
        assertEquals("emoji\\ud83d\\ude80", "emoji🚀".escapeJsonString())
    }

    @Test
    fun textStatistics_EstimateReadingTime() {
        // Test reading time estimation (assuming 200 WPM average)
        assertEquals("< 1 min", "Hello world".estimateReadingTime())
        assertEquals("1 min", "word ".repeat(200).trim().estimateReadingTime())
        assertEquals("2 min", "word ".repeat(400).trim().estimateReadingTime())
        assertEquals("5 min", "word ".repeat(1000).trim().estimateReadingTime())
    }

    @Test
    fun memoryManagement_ObjectSizeEstimation() {
        // Test memory usage estimation for objects
        val smallText = "hello"
        val largeText = "text ".repeat(10000)
        
        assertTrue(smallText.estimateMemoryUsage() < largeText.estimateMemoryUsage())
        assertTrue(smallText.estimateMemoryUsage() > 0)
        assertTrue(largeText.estimateMemoryUsage() > 1000) // At least 1KB
    }
}

/**
 * Extension functions for testing utilities.
 * These would be implemented in the actual utils package.
 */
fun String.wordCount(): Int {
    if (this.isBlank()) return 0
    return this.split("\\s+".toRegex()).filter { it.isNotBlank() }.size
}

fun String.characterCount(): Int {
    return this.length
}

fun String.isValidDocumentTitle(): Boolean {
    return this.isNotBlank() && this.trim().length in 1..255
}

fun String.isValidContentType(): Boolean {
    val validTypes = setOf("markdown", "text", "html", "json")
    return this in validTypes
}

fun Long.formatRelativeTime(): String {
    val now = System.currentTimeMillis()
    val diff = now - this
    
    return when {
        diff < 60_000 -> "Just now"
        diff < 3_600_000 -> "${diff / 60_000} ${if (diff / 60_000 == 1L) "minute" else "minutes"} ago"
        diff < 86_400_000 -> "${diff / 3_600_000} ${if (diff / 3_600_000 == 1L) "hour" else "hours"} ago"
        diff < 604_800_000 -> "${diff / 86_400_000} ${if (diff / 86_400_000 == 1L) "day" else "days"} ago"
        else -> "${diff / 604_800_000} ${if (diff / 604_800_000 == 1L) "week" else "weeks"} ago"
    }
}

fun String.sanitizeInput(): String {
    return this.replace("\r\n", "\n")
        .replace("\u0000", "") // Remove null bytes
        .replace("\u200B", "") // Remove zero-width space
        .replace("\uFEFF", "") // Remove BOM
}

fun String.isValidFilePath(): Boolean {
    if (this.isBlank()) return false
    if (this.contains("\u0000")) return false
    if (this.contains("..")) return false
    
    val invalidChars = setOf('<', '>', ':', '"', '|', '?', '*')
    if (this.any { it in invalidChars }) return false
    
    val windowsReserved = setOf("CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                               "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                               "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9")
    val filename = this.substringAfterLast('/').substringBeforeLast('.')
    if (filename.uppercase() in windowsReserved) return false
    
    return true
}

fun String.escapeJsonString(): String {
    return this.replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .replace("\b", "\\b")
        .replace("\u000C", "\\f")
        .replace("[^\\x20-\\x7E]".toRegex()) { match ->
            "\\u${match.value.codePointAt(0).toString(16).padStart(4, '0')}"
        }
}

fun String.estimateReadingTime(): String {
    val wordCount = this.wordCount()
    val wordsPerMinute = 200
    val minutes = (wordCount / wordsPerMinute).coerceAtLeast(0)
    
    return when {
        minutes == 0 -> "< 1 min"
        minutes == 1 -> "1 min"
        else -> "$minutes min"
    }
}

fun String.estimateMemoryUsage(): Long {
    // Rough estimation: 2 bytes per character for UTF-16 + object overhead
    return (this.length * 2) + 64 // 64 bytes estimated object overhead
}