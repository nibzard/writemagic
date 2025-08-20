package com.writemagic

import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.test.ext.junit.rules.ActivityScenarioRule

/**
 * Test helper utilities and extensions for WriteMagic Android tests.
 */
object TestHelpers {
    
    /**
     * Common test data for document testing
     */
    object TestData {
        const val SAMPLE_TITLE = "Test Document Title"
        const val SAMPLE_CONTENT = """
            This is sample content for testing purposes.
            It contains multiple lines and various elements.
            
            - List item 1
            - List item 2
            
            This helps test text processing and statistics calculation.
        """.trimIndent()
        
        const val UNICODE_CONTENT = "Content with unicode: 中文 العربية 🚀 ñáéíóú"
        const val LONG_CONTENT = "Long content. ".repeat(1000)
        
        val AI_PROMPTS = listOf(
            "Continue writing from where I left off",
            "Improve the clarity of this text",
            "Help me expand on this topic",
            "Suggest alternative phrasings"
        )
        
        val PROJECT_NAMES = listOf(
            "Test Novel Project",
            "Technical Documentation",
            "Blog Post Collection",
            "Research Notes"
        )
    }
    
    /**
     * Test timing utilities
     */
    object Timing {
        const val SHORT_DELAY = 500L
        const val MEDIUM_DELAY = 1000L
        const val LONG_DELAY = 2000L
        const val AUTO_SAVE_DELAY = 1500L
    }
    
    /**
     * Common test assertions
     */
    object Assertions {
        fun verifyDocumentSaved(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        }
        
        fun verifyStatisticsVisible(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("words").assertExists()
            composeTestRule.onNodeWithText("chars").assertExists()
        }
        
        fun verifyAIAssistantOpen(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("AI Writing Assistant").assertExists()
            composeTestRule.onNodeWithText("Quick Actions").assertExists()
        }
    }
    
    /**
     * Navigation helpers
     */
    object Navigation {
        fun navigateToWriting(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("Writing").performClick()
            composeTestRule.waitForIdle()
        }
        
        fun navigateToProjects(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("Projects").performClick()
            composeTestRule.waitForIdle()
        }
        
        fun navigateToAI(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("AI Assistant").performClick()
            composeTestRule.waitForIdle()
        }
        
        fun navigateToTimeline(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("Timeline").performClick()
            composeTestRule.waitForIdle()
        }
        
        fun navigateToSettings(composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>) {
            composeTestRule.onNodeWithText("Settings").performClick()
            composeTestRule.waitForIdle()
        }
    }
    
    /**
     * Content creation helpers
     */
    object ContentHelpers {
        fun createDocument(
            composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>,
            title: String = TestData.SAMPLE_TITLE,
            content: String = TestData.SAMPLE_CONTENT
        ) {
            Navigation.navigateToWriting(composeTestRule)
            composeTestRule.onNodeWithContentDescription("Create new document").performClick()
            composeTestRule.waitForIdle()
            
            composeTestRule.onNodeWithContentDescription("Document title, editable")
                .performTextInput(title)
            
            composeTestRule.onNodeWithContentDescription("Main text editor")
                .performTextInput(content)
            
            Thread.sleep(Timing.AUTO_SAVE_DELAY)
            composeTestRule.waitForIdle()
        }
        
        fun createProject(
            composeTestRule: AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>,
            name: String,
            description: String = "Test project description"
        ) {
            Navigation.navigateToProjects(composeTestRule)
            composeTestRule.onNodeWithContentDescription("Create Project").performClick()
            composeTestRule.waitForIdle()
            
            composeTestRule.onNodeWithText("Project Name").performTextInput(name)
            composeTestRule.onNodeWithText("Description").performTextInput(description)
            composeTestRule.onNodeWithText("Create").performClick()
            composeTestRule.waitForIdle()
        }
    }
    
    /**
     * Performance testing utilities
     */
    object Performance {
        fun measureExecutionTime(action: () -> Unit): Long {
            val startTime = System.currentTimeMillis()
            action()
            return System.currentTimeMillis() - startTime
        }
        
        fun measureMemoryUsage(): Long {
            val runtime = Runtime.getRuntime()
            return runtime.totalMemory() - runtime.freeMemory()
        }
        
        fun assertPerformance(duration: Long, maxDurationMs: Long, operation: String) {
            assert(duration < maxDurationMs) { 
                "$operation took too long: ${duration}ms (max: ${maxDurationMs}ms)" 
            }
        }
        
        fun assertMemoryUsage(memoryIncrease: Long, maxIncreaseBytes: Long, operation: String) {
            assert(memoryIncrease < maxIncreaseBytes) {
                "$operation used too much memory: ${memoryIncrease} bytes (max: ${maxIncreaseBytes} bytes)"
            }
        }
    }
}

/**
 * Extension functions for test utilities
 */
fun AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>.waitForAutoSave() {
    Thread.sleep(TestHelpers.Timing.AUTO_SAVE_DELAY)
    this.waitForIdle()
}

fun AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>.performTextInputWithDelay(
    text: String,
    delay: Long = TestHelpers.Timing.SHORT_DELAY
) {
    Thread.sleep(delay)
    this.waitForIdle()
}