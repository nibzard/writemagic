package com.writemagic.ui

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.text.input.TextFieldValue
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.writemagic.ui.screens.WritingScreen
import com.writemagic.ui.theme.WriteMagicTheme
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Unit tests for WritingScreen UI components and interactions.
 * Tests Compose UI behavior, state management, and user interactions.
 */
@RunWith(AndroidJUnit4::class)
class WritingScreenTest {

    @get:Rule
    val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        // Setup test environment before each test
    }

    @Test
    fun writingScreen_DisplaysCorrectInitialState() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Verify initial UI elements are present
        composeTestRule.onNodeWithText("New Document").assertExists()
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
        composeTestRule.onNodeWithContentDescription("Create new document").assertExists()
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").assertExists()
    }

    @Test
    fun writingScreen_ToolbarInteractions() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Test pane mode toggle
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").assertExists()
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").performClick()
        
        // Test distraction-free mode toggle
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").assertExists()
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").performClick()

        // Test AI assistant toggle
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
    }

    @Test
    fun writingScreen_TextInputFunctionality() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Find the main text editor and input text
        val testText = "This is a test document content."
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
        composeTestRule.onNodeWithContentDescription("Main text editor").performTextInput(testText)

        // Verify text appears (note: this test is limited without proper state hoisting)
        // In a real implementation, you'd want to extract state management for easier testing
    }

    @Test
    fun writingScreen_DocumentTitleEditing() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Test document title editing
        composeTestRule.onNodeWithContentDescription("Document title, editable").assertExists()
        
        // Note: For more comprehensive testing, the WritingScreen component
        // would benefit from extracting state to a ViewModel that can be mocked
    }

    @Test
    fun writingScreen_StatisticsDisplay() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Verify statistics are displayed initially
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        composeTestRule.onNodeWithText("0").assertExists() // Initial word/char count
    }

    @Test
    fun writingScreen_AIAssistantOverlay() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Open AI Assistant
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        
        // Verify AI Assistant overlay appears
        composeTestRule.onNodeWithText("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithText("Ask AI to help with your writing...").assertExists()
        composeTestRule.onNodeWithText("Quick Actions").assertExists()
        
        // Verify quick action buttons exist
        composeTestRule.onNodeWithText("Continue Writing").assertExists()
        composeTestRule.onNodeWithText("Improve Clarity").assertExists()
        composeTestRule.onNodeWithText("Add Examples").assertExists()
        composeTestRule.onNodeWithText("Summarize").assertExists()
        composeTestRule.onNodeWithText("Fix Grammar").assertExists()
        composeTestRule.onNodeWithText("Generate Outline").assertExists()

        // Test closing AI Assistant
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
    }

    @Test
    fun writingScreen_PaneModeLayout() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Toggle to pane mode
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").performClick()
        
        // Verify pane layout elements appear
        composeTestRule.onNodeWithText("Main Draft").assertExists()
        composeTestRule.onNodeWithText("Alternative").assertExists()
    }

    @Test
    fun writingScreen_DistractionFreeModeLayout() {
        composeTestRule.setContent {
            WriteMagicTheme {
                WritingScreen()
            }
        }

        // Toggle distraction-free mode
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").performClick()
        
        // Verify UI changes appropriately
        // Note: Testing visual changes would require screenshot testing or
        // exposing internal state for verification
    }
}

/**
 * Test helper functions for UI testing
 */
class WritingScreenTestHelpers {

    companion object {
        fun createTestDocument(title: String = "Test Document", content: String = "Test content") {
            // Helper function for creating test documents in UI tests
        }
        
        fun simulateTextInput(text: String): TextFieldValue {
            return TextFieldValue(text)
        }
        
        fun createLongTestContent(): String {
            return "This is a very long test content. ".repeat(100) +
                   "It contains multiple sentences and paragraphs. " +
                   "This helps test text handling, word counting, and UI performance."
        }
    }
}