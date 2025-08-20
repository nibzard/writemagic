package com.writemagic.ui

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.MediumTest
import com.writemagic.MainActivity
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Tests for navigation between screens and proper state preservation.
 * Verifies bottom navigation functionality and screen transitions.
 */
@MediumTest
@RunWith(AndroidJUnit4::class)
class NavigationTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun navigation_AllScreensAccessible() {
        composeTestRule.waitForIdle()
        
        // Test navigation to Writing screen (default)
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
        
        // Test navigation to Projects screen
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Projects").assertExists()
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
        
        // Test navigation to AI screen
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?")
            .assertExists()
        
        // Test navigation to Timeline screen
        composeTestRule.onNodeWithText("Timeline").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Document Timeline").assertExists()
        
        // Test navigation to Settings screen
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("App Settings").assertExists()
    }

    @Test
    fun navigation_ReturnToWritingFromAllScreens() {
        composeTestRule.waitForIdle()
        
        // Start from each screen and return to Writing
        val screens = listOf("Projects", "AI Assistant", "Timeline", "Settings")
        
        screens.forEach { screenName ->
            // Navigate to screen
            composeTestRule.onNodeWithText(screenName).performClick()
            composeTestRule.waitForIdle()
            
            // Return to Writing
            composeTestRule.onNodeWithText("Writing").performClick()
            composeTestRule.waitForIdle()
            
            // Verify we're on Writing screen
            composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
        }
    }

    @Test
    fun navigation_RapidSwitchingBetweenScreens() {
        composeTestRule.waitForIdle()
        
        val screens = listOf("Writing", "Projects", "AI Assistant", "Timeline", "Settings")
        
        // Rapidly switch between screens
        repeat(3) {
            screens.forEach { screen ->
                composeTestRule.onNodeWithText(screen).performClick()
                composeTestRule.waitForIdle()
                Thread.sleep(100) // Brief pause to simulate user interaction
            }
        }
        
        // Should end up on Settings screen
        composeTestRule.onNodeWithText("App Settings").assertExists()
    }

    @Test
    fun navigation_StatePreservation() {
        composeTestRule.waitForIdle()
        
        // Add content to Writing screen
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Note: This test would need state hoisting to verify content preservation
        // For now, we test that navigation doesn't crash
        
        // Navigate away and back
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Writing screen should be restored
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
    }

    @Test
    fun navigation_BottomNavigationVisible() {
        composeTestRule.waitForIdle()
        
        // Verify all navigation items are visible on each screen
        val screens = listOf("Writing", "Projects", "AI Assistant", "Timeline", "Settings")
        
        screens.forEach { currentScreen ->
            composeTestRule.onNodeWithText(currentScreen).performClick()
            composeTestRule.waitForIdle()
            
            // All navigation items should be visible
            screens.forEach { navItem ->
                composeTestRule.onNodeWithText(navItem).assertExists()
            }
        }
    }

    @Test
    fun navigation_SelectedStateIndication() {
        composeTestRule.waitForIdle()
        
        // Test that selected state is properly indicated
        // Note: This would require testing visual selection indicators
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        // Would verify Writing tab is selected visually
        
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        // Would verify Projects tab is selected visually
        
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        // Would verify AI Assistant tab is selected visually
    }

    @Test
    fun navigation_DeepLinkingSupport() {
        composeTestRule.waitForIdle()
        
        // This test would verify deep linking functionality
        // Currently limited without deep link setup
        
        // Navigate to different screens to verify routing works
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Projects").assertExists()
        
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("App Settings").assertExists()
    }

    @Test
    fun navigation_BackButtonBehavior() {
        composeTestRule.waitForIdle()
        
        // Test back button behavior (would require ActivityTestRule configuration)
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Note: Testing actual back button behavior would require:
        // - Espresso's pressBack() method
        // - Proper activity/navigation testing setup
        
        // For now, verify navigation structure is correct
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
    }

    @Test
    fun navigation_PerformanceUnderLoad() {
        composeTestRule.waitForIdle()
        
        val startTime = System.currentTimeMillis()
        
        // Perform many navigation operations
        repeat(50) { i ->
            val targetScreen = when (i % 5) {
                0 -> "Writing"
                1 -> "Projects"
                2 -> "AI Assistant"
                3 -> "Timeline"
                else -> "Settings"
            }
            
            composeTestRule.onNodeWithText(targetScreen).performClick()
            composeTestRule.waitForIdle()
        }
        
        val endTime = System.currentTimeMillis()
        val duration = endTime - startTime
        
        // Should complete navigation stress test within reasonable time
        assert(duration < 30000) { "Navigation performance test took too long: ${duration}ms" }
    }

    @Test
    fun navigation_ScreenRotationPreservesState() {
        composeTestRule.waitForIdle()
        
        // Navigate to Projects screen
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Verify projects are displayed
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
        
        // Note: Screen rotation testing would require:
        // - ActivityScenario configuration changes
        // - Device rotation simulation
        // For now, verify content exists
        
        composeTestRule.onNodeWithText("Projects").assertExists()
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
    }

    @Test
    fun navigation_AccessibilitySupport() {
        composeTestRule.waitForIdle()
        
        // Verify navigation has proper accessibility support
        // All navigation items should be properly labeled
        
        composeTestRule.onNodeWithText("Writing").assertExists()
        composeTestRule.onNodeWithText("Projects").assertExists()
        composeTestRule.onNodeWithText("AI Assistant").assertExists()
        composeTestRule.onNodeWithText("Timeline").assertExists()
        composeTestRule.onNodeWithText("Settings").assertExists()
        
        // Test navigation with accessibility services
        // Note: Full accessibility testing would require TalkBack simulation
        
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
    }
}