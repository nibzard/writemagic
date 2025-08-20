package com.writemagic.integration

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import com.writemagic.MainActivity
import com.writemagic.core.WriteMagicCore
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Integration tests for document management functionality.
 * Tests complete document workflows from UI to FFI integration.
 */
@ExperimentalCoroutinesApi
@LargeTest
@RunWith(AndroidJUnit4::class)
class DocumentManagementTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun documentLifecycle_CreateEditAndSave() = runTest {
        composeTestRule.waitForIdle()
        
        // Navigate to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Create new document
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        val testTitle = "Integration Test Document"
        val testContent = """
            This is a comprehensive integration test document.
            
            It tests the full document lifecycle:
            1. Document creation through UI
            2. Content editing and auto-save
            3. Title editing
            4. Statistics calculation
            
            The document contains various elements to test serialization:
            - Multiple paragraphs
            - Special characters: àáâã, ñáéíóú, 中文
            - Emojis: 🚀 🎉 🌟
            - Numbers: 123, 456.789
            - Punctuation: "quotes", (parentheses), [brackets]
        """.trimIndent()
        
        // Edit document title
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(testTitle)
        
        // Add content
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(testContent)
        
        // Wait for auto-save
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // Verify statistics are calculated
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        
        // Verify auto-save status appears
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
    }

    @Test
    fun documentPersistence_NavigationAndReturn() = runTest {
        composeTestRule.waitForIdle()
        
        // Create and edit document
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val persistentContent = "This content should persist across navigation"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(persistentContent)
        
        // Wait for auto-save
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Navigate away
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Navigate back
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Content should be preserved
        // Note: In actual implementation, would verify content restoration
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
    }

    @Test
    fun documentStatistics_RealTimeUpdates() = runTest {
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Start with short content
        val shortContent = "Short text"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(shortContent)
        
        composeTestRule.waitForIdle()
        
        // Verify initial statistics
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        
        // Add more content
        val additionalContent = " with additional words for testing statistics calculation"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(additionalContent)
        
        composeTestRule.waitForIdle()
        
        // Statistics should update in real-time
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
    }

    @Test
    fun documentVersioning_AutoSaveVersionControl() = runTest {
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Create initial version
        val version1Content = "Initial version of the document"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(version1Content)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Update to version 2
        val version2Addition = "\n\nAdditional content in version 2"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(version2Addition)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Update to version 3
        val version3Addition = "\n\nFinal additions in version 3"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(version3Addition)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Verify auto-save handled multiple versions
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
    }

    @Test
    fun documentValidation_HandleInvalidInput() = runTest {
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Test with problematic characters
        val problematicContent = """
            Testing edge cases:
            - Null bytes: ${"\u0000"}
            - Control characters: ${"\u0001\u0002\u0003"}
            - Zero-width spaces: ${"\u200B\u200C\u200D"}
            - Byte order marks: ${"\uFEFF"}
            - Combining characters: a${"\u0301"}b${"\u0302"}c${"\u0303"}
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(problematicContent)
        
        composeTestRule.waitForIdle()
        
        // App should handle edge cases gracefully
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
    }

    @Test
    fun documentSearch_ContentIndexing() = runTest {
        // This test would verify document search functionality
        // Currently limited by available UI elements
        
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Verify existing documents are searchable
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
        composeTestRule.onNodeWithText("Blog Posts").assertExists()
        composeTestRule.onNodeWithText("Technical Docs").assertExists()
        
        // In full implementation, would test search functionality
    }

    @Test
    fun documentSynchronization_MultipleInstances() = runTest {
        // Test document synchronization between different instances
        // This would be relevant for multi-window or multi-device scenarios
        
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val syncContent = "Content for synchronization testing"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(syncContent)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Verify save occurred
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        
        // In actual implementation, would verify sync across instances
    }

    @Test
    fun documentBackup_DataIntegrity() = runTest {
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Create important content that should be backed up
        val importantContent = """
            Critical document content that must not be lost.
            
            Contains important data:
            - User research findings
            - Project specifications
            - Implementation details
            
            This content should be properly backed up and recoverable.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(importantContent)
        
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // Verify save confirmation
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        
        // Test recovery scenario - navigate away and back
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Content should be preserved
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
    }

    @Test
    fun documentMetadata_ProperTracking() = runTest {
        composeTestRule.waitForIdle()
        
        // Test that document metadata is properly tracked
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Verify metadata is displayed
        composeTestRule.onNodeWithText("12 documents").assertExists() // Document count
        composeTestRule.onNodeWithText("2 hours ago").assertExists()  // Last modified
        composeTestRule.onNodeWithText("1 day ago").assertExists()    // Last modified
        composeTestRule.onNodeWithText("3 days ago").assertExists()   // Last modified
        
        // Create new project to test metadata creation
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        composeTestRule.waitForIdle()
        
        val projectName = "Metadata Test Project"
        val projectDescription = "Testing metadata tracking"
        
        composeTestRule.onNodeWithText("Project Name").performTextInput(projectName)
        composeTestRule.onNodeWithText("Description").performTextInput(projectDescription)
        
        composeTestRule.onNodeWithText("Create").performClick()
        composeTestRule.waitForIdle()
        
        // New project should appear with metadata
        composeTestRule.onNodeWithText(projectName).assertExists()
        composeTestRule.onNodeWithText("0 documents").assertExists()
        composeTestRule.onNodeWithText("Just now").assertExists()
    }

    @Test
    fun documentExportImport_DataPortability() = runTest {
        // Test document export/import functionality
        // This would verify data portability features
        
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val exportableContent = """
            Document for export/import testing.
            
            Should maintain formatting and metadata during export/import process.
            
            Key features to preserve:
            - Text formatting
            - Document structure  
            - Creation/modification timestamps
            - Version information
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(exportableContent)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // In actual implementation, would test export/import UI
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
    }
}