package com.writemagic.ui

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import com.writemagic.MainActivity
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Large-scale UI tests for complete writing workflows.
 * Tests end-to-end user interactions and complex UI scenarios.
 */
@LargeTest
@RunWith(AndroidJUnit4::class)
class WritingWorkflowTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun completeWritingWorkflow_CreateEditAndSave() {
        // Wait for app to initialize
        composeTestRule.waitForIdle()
        
        // Navigate to Writing screen (should be default)
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Create new document
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        // Edit document title
        val testTitle = "My Test Document"
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(testTitle)
        
        // Add content to document
        val testContent = """
            This is a test document created during automated testing.
            
            It contains multiple paragraphs and various formatting elements.
            
            - List item 1
            - List item 2
            - List item 3
            
            This helps verify that the writing interface works correctly.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(testContent)
        
        // Wait for auto-save to trigger
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // Verify auto-save status appears
        composeTestRule.onNodeWithText("Document saved automatically")
            .assertExists()
        
        // Verify statistics are updated
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        
        // Test pane mode
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Main Draft").assertExists()
        composeTestRule.onNodeWithText("Alternative").assertExists()
        
        // Switch back to single pane
        composeTestRule.onNodeWithContentDescription("Switch to single pane").performClick()
        composeTestRule.waitForIdle()
    }

    @Test
    fun aiAssistantWorkflow_OpenUseAndClose() {
        composeTestRule.waitForIdle()
        
        // Navigate to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        
        // Add some initial content
        val initialContent = "The future of artificial intelligence is"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(initialContent)
        
        // Open AI Assistant
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Verify AI Assistant opened
        composeTestRule.onNodeWithText("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithText("Quick Actions").assertExists()
        
        // Test custom prompt
        val customPrompt = "Help me expand on this topic"
        composeTestRule.onNodeWithText("Ask AI to help with your writing...")
            .performTextInput(customPrompt)
        
        // Test quick action
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Close AI Assistant
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
        
        // Verify assistant is closed
        composeTestRule.onNodeWithText("AI Writing Assistant").assertDoesNotExist()
    }

    @Test
    fun distractionFreeModeWorkflow() {
        composeTestRule.waitForIdle()
        
        // Navigate to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        
        // Enable distraction-free mode
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        
        // Verify UI changes for distraction-free mode
        // Note: Visual changes would need screenshot testing or exposed state
        
        // Should still be able to edit text
        val content = "Writing in distraction-free mode"
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(content)
        
        // Exit distraction-free mode
        composeTestRule.onNodeWithContentDescription("Exit distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        
        // Verify normal mode restored
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
    }

    @Test
    fun navigationBetweenScreens() {
        composeTestRule.waitForIdle()
        
        // Test navigation to each screen
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Projects").assertExists()
        
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?")
            .assertExists()
        
        composeTestRule.onNodeWithText("Timeline").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Document Timeline").assertExists()
        
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("App Settings").assertExists()
        
        // Return to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
    }

    @Test
    fun longContentHandling() {
        composeTestRule.waitForIdle()
        
        // Navigate to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        
        // Create very long content
        val longContent = "This is a very long paragraph that tests the app's ability to handle extensive text content. ".repeat(100) +
                         "It includes multiple sentences and should test scrolling, performance, and text handling capabilities."
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(longContent)
        
        composeTestRule.waitForIdle()
        
        // Verify statistics are calculated correctly for long content
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        
        // Wait for auto-save
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
    }

    @Test
    fun errorHandlingScenarios() {
        composeTestRule.waitForIdle()
        
        // Test behavior when FFI operations might fail
        // This would be more meaningful with actual FFI mocking
        
        composeTestRule.onNodeWithText("Writing").performClick()
        
        // Try to create document (may fail without native library)
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        
        // App should handle FFI errors gracefully
        composeTestRule.waitForIdle()
        
        // Should still allow text input even if save fails
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput("Test content despite potential FFI errors")
    }

    @Test
    fun accessibilityCompliance() {
        composeTestRule.waitForIdle()
        
        // Test that all interactive elements have proper content descriptions
        composeTestRule.onNodeWithContentDescription("Create new document").assertExists()
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").assertExists()
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").assertExists()
        composeTestRule.onNodeWithContentDescription("Document title, editable").assertExists()
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
        
        // Navigation elements should be accessible
        composeTestRule.onNodeWithText("Writing").assertExists()
        composeTestRule.onNodeWithText("Projects").assertExists()
        composeTestRule.onNodeWithText("AI Assistant").assertExists()
        composeTestRule.onNodeWithText("Timeline").assertExists()
        composeTestRule.onNodeWithText("Settings").assertExists()
    }

    @Test
    fun performanceUnderLoad() {
        composeTestRule.waitForIdle()
        
        val startTime = System.currentTimeMillis()
        
        // Perform rapid UI interactions
        repeat(20) {
            composeTestRule.onNodeWithText("Writing").performClick()
            composeTestRule.onNodeWithText("Projects").performClick()
            composeTestRule.onNodeWithText("AI Assistant").performClick()
        }
        
        val endTime = System.currentTimeMillis()
        val duration = endTime - startTime
        
        // Should complete within reasonable time (10 seconds max)
        assert(duration < 10000) { "Navigation performance test took too long: ${duration}ms" }
        
        composeTestRule.waitForIdle()
    }
}