package com.writemagic.ui

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.writemagic.ui.screens.ProjectsScreen
import com.writemagic.ui.theme.WriteMagicTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Unit tests for ProjectsScreen UI components and project management functionality.
 */
@RunWith(AndroidJUnit4::class)
class ProjectsScreenTest {

    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun projectsScreen_DisplaysInitialState() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Verify screen title and create button
        composeTestRule.onNodeWithText("Projects").assertExists()
        composeTestRule.onNodeWithContentDescription("Create Project").assertExists()
        
        // Verify initial mock projects are displayed
        composeTestRule.onNodeWithText("Novel Draft").assertExists()
        composeTestRule.onNodeWithText("My first novel project").assertExists()
        composeTestRule.onNodeWithText("Blog Posts").assertExists()
        composeTestRule.onNodeWithText("Collection of blog posts").assertExists()
        composeTestRule.onNodeWithText("Technical Docs").assertExists()
        composeTestRule.onNodeWithText("API documentation").assertExists()
    }

    @Test
    fun projectsScreen_ProjectCardsDisplay() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Verify project card details are shown
        composeTestRule.onNodeWithText("12 documents").assertExists() // Novel Draft
        composeTestRule.onNodeWithText("8 documents").assertExists()  // Blog Posts  
        composeTestRule.onNodeWithText("5 documents").assertExists()  // Technical Docs
        
        composeTestRule.onNodeWithText("2 hours ago").assertExists()
        composeTestRule.onNodeWithText("1 day ago").assertExists()
        composeTestRule.onNodeWithText("3 days ago").assertExists()
        
        // Verify options menu exists for each project
        composeTestRule.onNodeWithContentDescription("Options").assertExists()
    }

    @Test
    fun projectsScreen_CreateProjectDialog() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Open create project dialog
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        
        // Verify dialog appears
        composeTestRule.onNodeWithText("Create New Project").assertExists()
        composeTestRule.onNodeWithText("Project Name").assertExists()
        composeTestRule.onNodeWithText("Description").assertExists()
        composeTestRule.onNodeWithText("Create").assertExists()
        composeTestRule.onNodeWithText("Cancel").assertExists()
    }

    @Test
    fun projectsScreen_CreateProjectForm() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Open dialog
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        
        val testProjectName = "Test Project"
        val testDescription = "This is a test project description"
        
        // Fill in form fields
        composeTestRule.onNodeWithText("Project Name").performTextInput(testProjectName)
        composeTestRule.onNodeWithText("Description").performTextInput(testDescription)
        
        // Create button should be enabled with valid input
        composeTestRule.onNodeWithText("Create").assertExists()
        composeTestRule.onNodeWithText("Create").performClick()
        
        // Dialog should close and new project should appear
        // Note: Full verification would require state hoisting for testing
    }

    @Test
    fun projectsScreen_CreateProjectValidation() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Open dialog
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        
        // Create button should be disabled with empty name
        // Note: This test assumes proper form validation implementation
        composeTestRule.onNodeWithText("Create").assertExists()
        
        // Add only description without name
        composeTestRule.onNodeWithText("Description").performTextInput("Description only")
        
        // Button should still be disabled (implementation dependent)
    }

    @Test
    fun projectsScreen_CancelCreateDialog() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Open dialog
        composeTestRule.onNodeWithContentDescription("Create Project").performClick()
        
        // Cancel should close dialog
        composeTestRule.onNodeWithText("Cancel").performClick()
        
        // Dialog should be dismissed
        composeTestRule.onNodeWithText("Create New Project").assertDoesNotExist()
    }

    @Test
    fun projectsScreen_ProjectCardInteraction() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Test clicking on a project card
        composeTestRule.onNodeWithText("Novel Draft").performClick()
        
        // Note: In a full implementation, this would navigate to project details
        // Testing would require navigation testing setup
    }

    @Test
    fun projectsScreen_ProjectOptionsMenu() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Click options menu
        composeTestRule.onNodeWithContentDescription("Options").performClick()
        
        // Note: Options menu functionality would need to be implemented
        // and tested with proper dropdown/menu testing
    }

    @Test
    fun projectsScreen_EmptyStateHandling() {
        // This test would verify behavior when no projects exist
        // Would need a variant of ProjectsScreen with empty state
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }
        
        // With current mock data, projects always exist
        // In a real implementation, you'd test empty state UI
        composeTestRule.onNodeWithText("Projects").assertExists()
    }

    @Test
    fun projectsScreen_ProjectStatisticsDisplay() {
        composeTestRule.setContent {
            WriteMagicTheme {
                ProjectsScreen()
            }
        }

        // Verify document count statistics are properly formatted
        composeTestRule.onNodeWithText("12 documents").assertExists()
        composeTestRule.onNodeWithText("8 documents").assertExists()
        composeTestRule.onNodeWithText("5 documents").assertExists()
        
        // Verify timestamp formatting
        composeTestRule.onNodeWithText("2 hours ago").assertExists()
        composeTestRule.onNodeWithText("1 day ago").assertExists()
        composeTestRule.onNodeWithText("3 days ago").assertExists()
    }
}