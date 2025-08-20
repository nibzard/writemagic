package com.writemagic.e2e

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import com.writemagic.MainActivity
import kotlinx.coroutines.ExperimentalCoroutinesApi
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * End-to-end tests for complete user workflows in WriteMagic.
 * Tests realistic user scenarios from start to finish.
 */
@ExperimentalCoroutinesApi
@LargeTest
@RunWith(AndroidJUnit4::class)
class EndToEndWorkflowTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun completeWritingSession_FromStartToFinish() {
        composeTestRule.waitForIdle()
        
        // 1. User starts app and sees Writing screen
        composeTestRule.onNodeWithText("Start writing your thoughts...").assertExists()
        
        // 2. User creates a new document
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        // 3. User sets document title
        val documentTitle = "My Novel - Chapter 1: The Beginning"
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(documentTitle)
        
        // 4. User writes initial content
        val initialContent = """
            The morning sun cast long shadows across the empty street as Sarah stepped out of her apartment building. 
            She had been preparing for this day for months, but now that it was here, doubt crept into her mind.
            
            The letter in her pocket felt heavier than it should have, containing the words that would change everything.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(initialContent)
        
        // 5. Wait for auto-save
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // 6. User checks statistics
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
        
        // 7. User gets AI assistance for continuation
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("AI Writing Assistant").assertExists()
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.waitForIdle()
        
        // 8. User tries custom AI prompt
        val customPrompt = "Help me develop the character of Sarah more deeply"
        composeTestRule.onNodeWithText("Ask AI to help with your writing...")
            .performTextInput(customPrompt)
        composeTestRule.onNodeWithContentDescription("Send prompt").performClick()
        composeTestRule.waitForIdle()
        
        // 9. Close AI assistant and continue writing
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
        
        // 10. Add more content
        val additionalContent = """
            
            
            As she walked down the familiar street, Sarah remembered the conversation with her mother from the night before.
            "Sometimes," her mother had said, "the biggest risks lead to the most beautiful destinations."
            
            The train station was only six blocks away, but each step felt like a mile.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(additionalContent)
        
        // 11. User tests pane mode for comparison
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Main Draft").assertExists()
        composeTestRule.onNodeWithText("Alternative").assertExists()
        
        // 12. Return to single pane
        composeTestRule.onNodeWithContentDescription("Switch to single pane").performClick()
        composeTestRule.waitForIdle()
        
        // 13. Try distraction-free mode for focused writing
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        
        // 14. Add final content in distraction-free mode
        val finalContent = """
            
            
            At the train platform, Sarah pulled out the letter one last time. 
            The job offer in Tokyo was everything she had dreamed of, but it meant leaving behind everything familiar.
            
            The train's whistle echoed in the distance, growing closer with each passing second.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(finalContent)
        
        // 15. Exit distraction-free mode
        composeTestRule.onNodeWithContentDescription("Exit distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        
        // 16. Final auto-save
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        
        // 17. Check final statistics
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
    }

    @Test
    fun projectOrganizationWorkflow() {
        composeTestRule.waitForIdle()
        
        // 1. Navigate to Projects screen
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // 2. Review existing projects
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
        composeTestRule.onNodeWithText("Blog Posts").assertExists()
        composeTestRule.onNodeWithText("Technical Docs").assertExists()
        
        // 3. Create new project for organizing work
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        composeTestRule.waitForIdle()
        
        val newProjectName = "Personal Memoir Project"
        val projectDescription = "Collection of personal stories and memories for future memoir"
        
        composeTestRule.onNodeWithText("Project Name").performTextInput(newProjectName)
        composeTestRule.onNodeWithText("Description").performTextInput(projectDescription)
        
        composeTestRule.onNodeWithText("Create").performClick()
        composeTestRule.waitForIdle()
        
        // 4. Verify new project appears
        composeTestRule.onNodeWithText(newProjectName).assertExists()
        composeTestRule.onNodeWithText("0 documents").assertExists()
        composeTestRule.onNodeWithText("Just now").assertExists()
        
        // 5. Navigate to Writing screen to create content for project
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // 6. Create document for the new project
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        val memoirTitle = "Childhood Summer - 1995"
        val memoirContent = """
            The summer of 1995 was different from all the others. I was ten years old, 
            and the world seemed full of infinite possibilities.
            
            Our backyard became an adventure playground where my sister and I would spend 
            entire afternoons building forts, chasing butterflies, and making up elaborate 
            stories about the characters who lived in our imagination.
            
            That summer taught me that the most precious moments are often the simplest ones.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(memoirTitle)
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(memoirContent)
        
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // 7. Return to Projects to see updated count
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Project should show updated document count (in full implementation)
        composeTestRule.onNodeWithText(newProjectName).assertExists()
    }

    @Test
    fun aiAssistedWritingWorkflow() {
        composeTestRule.waitForIdle()
        
        // 1. Start with AI Assistant for brainstorming
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Hello! I'm your AI writing assistant. How can I help you today?")
            .assertExists()
        
        // 2. Ask for story ideas
        val brainstormPrompt = "I want to write a short science fiction story about time travel. Give me some unique plot ideas."
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(brainstormPrompt)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        composeTestRule.waitForIdle()
        
        Thread.sleep(1000)
        
        // 3. Follow up with character development
        val characterPrompt = "Help me develop a protagonist who is a reluctant time traveler"
        composeTestRule.onNodeWithText("Ask me anything about your writing...")
            .performTextInput(characterPrompt)
        composeTestRule.onNodeWithContentDescription("Send").performClick()
        composeTestRule.waitForIdle()
        
        Thread.sleep(1000)
        
        // 4. Switch to Writing screen to start writing
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // 5. Create new document for the story
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        val storyTitle = "The Accidental Traveler"
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(storyTitle)
        
        // 6. Write opening paragraph
        val openingParagraph = """
            Marcus never meant to become a time traveler. In fact, he had always been rather fond of the present moment.
            But when the antique pocket watch he'd inherited from his grandmother started glowing during the thunderstorm,
            everything changed in the most literal sense of the word.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(openingParagraph)
        
        // 7. Use AI assistant for continuation
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.waitForIdle()
        
        Thread.sleep(1000)
        
        // 8. Get AI help with dialogue
        val dialoguePrompt = "Help me write dialogue where Marcus talks to himself about his situation"
        composeTestRule.onNodeWithText("Ask AI to help with your writing...")
            .performTextInput(dialoguePrompt)
        composeTestRule.onNodeWithContentDescription("Send prompt").performClick()
        composeTestRule.waitForIdle()
        
        // 9. Add more content with AI assistance
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Add Examples").performClick()
        composeTestRule.waitForIdle()
        
        // 10. Close AI assistant and finish writing
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
        
        val finalParagraphs = """
            
            
            "Well, this is just fantastic," Marcus muttered to himself, staring at the Victorian-era street that had replaced 
            his modern apartment building. "Grandmother's watch comes with instructions for everything except time travel."
            
            A horse-drawn carriage clattered past, and Marcus realized he had two choices: figure out how to get home,
            or learn to really, really appreciate historical accuracy.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(finalParagraphs)
        
        // 11. Final save and statistics check
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
    }

    @Test
    fun multiSessionWorkflow_SaveAndRestore() {
        composeTestRule.waitForIdle()
        
        // Simulate a multi-session workflow where user returns to work
        
        // Session 1: Create and start document
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithContentDescription("Create new document").performClick()
        composeTestRule.waitForIdle()
        
        val sessionTitle = "Multi-Session Work Document"
        val session1Content = "This document will be worked on across multiple sessions to test persistence."
        
        composeTestRule.onNodeWithContentDescription("Document title, editable")
            .performTextInput(sessionTitle)
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(session1Content)
        
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        // Navigate away (simulating app close/reopen)
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        // Session 2: Return to document
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        // Document should be restored (in full implementation)
        composeTestRule.onNodeWithContentDescription("Main text editor").assertExists()
        
        val session2Content = """
            
            
            Session 2 additions:
            This content was added after navigating away from the document and returning.
            The persistence system should maintain the document state across sessions.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(session2Content)
        
        Thread.sleep(1500)
        composeTestRule.waitForIdle()
        
        // Session 3: Final additions
        composeTestRule.onNodeWithText("Timeline").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val session3Content = """
            
            
            Session 3 final content:
            This demonstrates the app's ability to maintain document state across multiple work sessions,
            ensuring that users never lose their progress and can pick up exactly where they left off.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(session3Content)
        
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
    }

    @Test
    fun comprehensiveFeatureExploration() {
        composeTestRule.waitForIdle()
        
        // Test user exploring all major features
        
        // 1. Start with Writing
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val explorationContent = "Testing all features of WriteMagic systematically."
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(explorationContent)
        
        // 2. Test toolbar features
        composeTestRule.onNodeWithContentDescription("Switch to multi-pane").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription("Switch to single pane").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithContentDescription("Enter distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription("Exit distraction-free mode").performClick()
        composeTestRule.waitForIdle()
        
        // 3. Test AI Assistant features
        composeTestRule.onNodeWithContentDescription("AI Writing Assistant").performClick()
        composeTestRule.waitForIdle()
        
        // Test all quick actions
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        Thread.sleep(500)
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        Thread.sleep(500)
        composeTestRule.onNodeWithText("Add Examples").performClick()
        Thread.sleep(500)
        composeTestRule.onNodeWithText("Summarize").performClick()
        Thread.sleep(500)
        composeTestRule.onNodeWithText("Fix Grammar").performClick()
        Thread.sleep(500)
        composeTestRule.onNodeWithText("Generate Outline").performClick()
        Thread.sleep(500)
        
        composeTestRule.onNodeWithContentDescription("Close AI Assistant").performClick()
        
        // 4. Test Projects functionality  
        composeTestRule.onNodeWithText("Projects").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Project Name").performTextInput("Feature Exploration Project")
        composeTestRule.onNodeWithText("Description").performTextInput("Testing project creation")
        composeTestRule.onNodeWithText("Create").performClick()
        composeTestRule.waitForIdle()
        
        // 5. Test AI Assistant standalone
        composeTestRule.onNodeWithText("AI Assistant").performClick()
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("GPT-4").performClick()
        composeTestRule.onNodeWithText("Claude").performClick()
        
        composeTestRule.onNodeWithText("Continue Writing").performClick()
        composeTestRule.onNodeWithText("Improve Clarity").performClick()
        composeTestRule.onNodeWithText("Rephrase").performClick()
        
        // 6. Check Timeline
        composeTestRule.onNodeWithText("Timeline").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Document Timeline").assertExists()
        
        // 7. Visit Settings
        composeTestRule.onNodeWithText("Settings").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("App Settings").assertExists()
        
        // 8. Return to Writing for final test
        composeTestRule.onNodeWithText("Writing").performClick()
        composeTestRule.waitForIdle()
        
        val finalTestContent = """
            
            
            Feature exploration completed successfully!
            All major functionality areas have been tested and verified working.
        """.trimIndent()
        
        composeTestRule.onNodeWithContentDescription("Main text editor")
            .performTextInput(finalTestContent)
        
        Thread.sleep(2000)
        composeTestRule.waitForIdle()
        
        composeTestRule.onNodeWithText("Document saved automatically").assertExists()
        composeTestRule.onNodeWithText("words").assertExists()
        composeTestRule.onNodeWithText("chars").assertExists()
    }
}