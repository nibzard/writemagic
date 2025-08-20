package com.writemagic.ui

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.writemagic.ui.screens.AIScreen
import com.writemagic.ui.theme.WriteMagicTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Unit tests for AIScreen UI components and chat functionality.
 */
@RunWith(AndroidJUnit4::class)
class AIScreenTest {

    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun aiScreen_DisplaysInitialState() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        // Verify initial greeting message
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?").assertExists()
        
        // Verify provider selection
        composeTestRule.onNodeWithText("Provider:").assertExists()
        composeTestRule.onNodeWithText("Claude").assertExists()
        composeTestRule.onNodeWithText("GPT-4").assertExists()
        composeTestRule.onNodeWithText("Local Model").assertExists()
        
        // Verify input area
        composeTestRule.onNodeWithText("Ask me anything about your writing...").assertExists()
        composeTestRule.onNodeWithContentDescription("Send").assertExists()
    }

    @Test
    fun aiScreen_ProviderSelection() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        // Test provider switching
        composeTestRule.onNodeWithText("GPT-4").performClick()
        composeTestRule.onNodeWithText("Local Model").performClick()
        composeTestRule.onNodeWithText("Claude").performClick()
        
        // Verify providers remain available after selection
        composeTestRule.onNodeWithText("Claude").assertExists()
        composeTestRule.onNodeWithText("GPT-4").assertExists()
        composeTestRule.onNodeWithText("Local Model").assertExists()
    }

    @Test
    fun aiScreen_MessageInput() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        val testMessage = "Help me improve this paragraph"
        
        // Input text in the message field
        composeTestRule.onNodeWithText("Ask me anything about your writing...").performTextInput(testMessage)
        
        // Send button should become enabled
        composeTestRule.onNodeWithContentDescription("Send").assertExists()
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        // Verify message appears in chat (would need state hoisting for full verification)
    }

    @Test
    fun aiScreen_QuickActions() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        // Verify quick action buttons exist
        composeTestRule.onNodeWithText("Continue Writing").assertExists()
        composeTestRule.onNodeWithText("Improve Clarity").assertExists()
        composeTestRule.onNodeWithText("Rephrase").assertExists()
        
        // Test clicking quick actions
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        composeTestRule.onNodeWithText("Rephrase").performClick()
    }

    @Test
    fun aiScreen_MessageBubbleDisplay() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        // Verify initial AI message is displayed correctly
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?").assertExists()
        
        // AI icon should be present for AI messages
        composeTestRule.onNodeWithContentDescription("AI").assertExists()
    }

    @Test
    fun aiScreen_EmptyInputValidation() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        // Send button should be disabled with empty input
        // Note: This test assumes proper enabled/disabled state management
        composeTestRule.onNodeWithContentDescription("Send").assertExists()
        
        // With empty input, send should not work (implementation dependent)
        // Would need to verify button state or add test IDs
    }

    @Test
    fun aiScreen_ProcessingIndicator() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        val testMessage = "Test message"
        
        // Input and send message to trigger processing
        composeTestRule.onNodeWithText("Ask me anything about your writing...").performTextInput(testMessage)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        // Processing indicator should appear
        // Note: This would need proper async state management testing
    }

    @Test
    fun aiScreen_LongMessageHandling() {
        composeTestRule.setContent {
            WriteMagicTheme {
                AIScreen()
            }
        }

        val longMessage = "This is a very long message that should test the text field's ability to handle multiple lines and extensive content. ".repeat(10)
        
        // Test inputting long message
        composeTestRule.onNodeWithText("Ask me anything about your writing...").performTextInput(longMessage)
        
        // Verify input field can handle long text
        // Note: Actual verification would depend on TextFieldValue state exposure
    }
}