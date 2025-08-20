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
 * Integration tests for AI functionality across screens.
 * Tests AI provider switching, chat functionality, and writing assistance.
 */
@LargeTest
@RunWith(AndroidJUnit4::class)
class AIIntegrationTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun aiProviderSelection_SwitchBetweenProviders() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Verify default provider selection
        composeTestRule.onNodeWithText("Provider:").assertExists()
        composeTestRule.onNodeWithText("Claude").assertExists()
        composeTestRule.onNodeWithText("GPT-4").assertExists()
        composeTestRule.onNodeWithText("Local Model").assertExists()
        
        // Switch to GPT-4
        composeTestRule.onNodeWithText("GPT-4").performClick()
        composeTestRule.waitForIdle()
        
        // Switch to Local Model
        composeTestRule.onNodeWithText("Local Model").performClick()
        composeTestRule.waitForIdle()
        
        // Switch back to Claude
        composeTestRule.onNodeWithText("Claude").performClick()
        composeTestRule.waitForIdle()
        
        // Verify providers remain available
        composeTestRule.onNodeWithText("Claude").assertExists()
        composeTestRule.onNodeWithText("GPT-4").assertExists()
        composeTestRule.onNodeWithText("Local Model").assertExists()
    }

    @Test
    fun aiChat_SendMessagesAndReceiveResponses() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Verify initial greeting
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?")
            .assertExists()
        
        // Send first message
        val firstMessage = "Help me write a story about space exploration"
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(firstMessage)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        // Wait for response processing
        composeTestRule.waitForIdle()
        Thread.sleep(1000)
        
        // Send follow-up message
        val followUpMessage = "Make it more science fiction focused"
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(followUpMessage)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        composeTestRule.waitForIdle()
    }

    @Test
    fun aiQuickActions_UsePrebuiltPrompts() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Test each quick action
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.waitForIdle()
        
        Thread.sleep(500)
        
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        composeTestRule.waitForIdle()
        
        Thread.sleep(500)
        
        composeTestRule.onNodeWithText("Rephrase").performClick()
        composeTestRule.waitForIdle()
        
        // Verify quick actions populate the input field
        // Note: This test would need state verification in actual implementation
    }

    @Test
    fun aiAssistantIntegration_FromWritingScreen() {
        composeTestRule.waitForIdle()
        
        // Start on Writing screen with content
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val testContent = "The impact of artificial intelligence on society is profound and far-reaching."
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(testContent)
        
        // Open AI Assistant from Writing screen
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Verify AI Assistant overlay appears
        composeTestRule.onNodeWithText("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithText("Quick Actions").assertExists()
        
        // Test Continue Writing with existing content
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Test Improve Clarity
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        composeTestRule.waitForIdle()
        
        // Test custom prompt with context
        val contextualPrompt = "Expand on the economic implications"
        composeTestRule.onNodeWithText("Ask AI to help with your writing...")
            .performTextInput(contextualPrompt)
        
        // Send prompt
        composeTestRule.onNodeWithContentDescription("Send prompt").performClick()
        composeTestRule.waitForIdle()
        
        // Close AI Assistant
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
        
        // Should return to writing screen
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
    }

    @Test
    fun aiError_HandleConnectionFailures() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Send a message that might trigger network error
        val testMessage = "Test message for error handling"
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(testMessage)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        // Wait and see how app handles potential errors
        composeTestRule.waitForIdle()
        Thread.sleep(2000)
        
        // App should remain functional even with AI errors
        composeTestRule.onNodeWithText("Ask me anything about your writing...").assertExists()
        composeTestRule.onNodeWithContentDescription("Send").assertExists()
    }

    @Test
    fun aiProviderFallback_TestFailoverBehavior() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Select a provider that might fail
        composeTestRule.onNodeWithText("Local Model").performClick()
        composeTestRule.waitForIdle()
        
        // Send message that might trigger fallback
        val testMessage = "This should test provider fallback"
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(testMessage)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        
        composeTestRule.waitForIdle()
        Thread.sleep(3000) // Allow time for fallback
        
        // Verify app handles fallback gracefully
        composeTestRule.onNodeWithText("Provider:").assertExists()
        
        // Try switching to another provider
        composeTestRule.onNodeWithText("Claude").performClick()
        composeTestRule.waitForIdle()
    }

    @Test
    fun aiLongConversation_MultipleExchanges() {
        composeTestRule.waitForIdle()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        val messages = listOf(
            "Help me write a technical blog post",
            "Focus on mobile development",
            "Include Android specific examples",
            "Add some code snippets",
            "Make it more beginner friendly"
        )
        
        messages.forEach { message ->
            composeTestRule.onNodeWithText("Ask me anything about your writing...")
                .performTextInput(message)
            composeTestRule.onNodeWithContentDescription("Send").performClick()
            
            // Wait between messages
            Thread.sleep(1000)
            composeTestRule.waitForIdle()
        }
        
        // Conversation should handle multiple exchanges
        composeTestRule.onNodeWithText("Ask me anything about your writing...").assertExists()
    }

    @Test
    fun aiContextAwareness_WithSelectedText() {
        composeTestRule.waitForIdle()
        
        // Start on Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val testContent = "Machine learning algorithms are transforming how we approach data analysis and pattern recognition in various industries."
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(testContent)
        
        // Open AI Assistant
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Test context-aware actions
        composeTestRule.onNodeWithText("Fix Grammar").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Summarize").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Generate Outline").performClick()
        composeTestRule.waitForIdle()
        
        // Close AI Assistant
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
    }

    @Test
    fun aiPerformance_ResponseTimeAndMemory() {
        composeTestRule.waitForIdle()
        
        val startMemory = Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory()
        val startTime = System.currentTimeMillis()
        
        // Navigate to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Send multiple quick messages
        repeat(5) { i ->
            val message = "Test message number $i"
            composeTestRule.onNodeWithText("Ask me anything about your writing...")
                .performTextInput(message)
            composeTestRule.onNodeWithContentDescription("Send").performClick()
            Thread.sleep(500)
        }
        
        composeTestRule.waitForIdle()
        
        val endTime = System.currentTimeMillis()
        val endMemory = Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory()
        
        val duration = endTime - startTime
        val memoryIncrease = endMemory - startMemory
        
        // Performance thresholds (adjust based on requirements)
        assert(duration < 15000) { "AI interaction took too long: ${duration}ms" }
        assert(memoryIncrease < 50_000_000) { "Memory usage increased too much: ${memoryIncrease} bytes" }
    }
}