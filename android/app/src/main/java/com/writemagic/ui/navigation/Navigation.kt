package com.writemagic.ui.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
import com.writemagic.ui.screens.*

@Composable
fun WriteMagicNavigation(
    navController: NavHostController,
    startDestination: String = "writing"
) {
    NavHost(
        navController = navController,
        startDestination = startDestination
    ) {
        // Main screens accessible from bottom navigation
        composable("writing") {
            WritingScreen()
        }
        
        composable("projects") {
            ProjectsScreen()
        }
        
        composable("ai") {
            AIScreen()
        }
        
        composable("timeline") {
            TimelineScreen()
        }
        
        composable("settings") {
            SettingsScreen()
        }
        
        // Document management screens
        composable("documents") {
            DocumentListScreen(
                onNavigateToDocument = { documentId ->
                    navController.navigate("document/$documentId")
                },
                onCreateDocument = {
                    navController.navigate("document/new")
                }
            )
        }
        
        composable(
            "document/{documentId}",
            arguments = listOf(navArgument("documentId") { type = NavType.StringType })
        ) { backStackEntry ->
            val documentId = backStackEntry.arguments?.getString("documentId")
            if (documentId != null) {
                DocumentEditorScreen(
                    documentId = documentId,
                    onNavigateBack = { navController.popBackStack() }
                )
            }
        }
        
        // Project detail screens
        composable(
            "project/{projectId}",
            arguments = listOf(navArgument("projectId") { type = NavType.StringType })
        ) { backStackEntry ->
            val projectId = backStackEntry.arguments?.getString("projectId")
            if (projectId != null) {
                ProjectDetailScreen(
                    projectId = projectId,
                    onNavigateBack = { navController.popBackStack() },
                    onNavigateToDocument = { documentId ->
                        navController.navigate("document/$documentId")
                    }
                )
            }
        }
        
        // Settings screens
        composable("settings/ai") {
            AISettingsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }
        
        composable("settings/export") {
            ExportDataScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }
        
        composable("settings/import") {
            ImportDataScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }
    }
}

// Navigation helper functions
object NavigationHelper {
    fun navigateToDocument(navController: NavHostController, documentId: String) {
        navController.navigate("document/$documentId")
    }
    
    fun navigateToProject(navController: NavHostController, projectId: String) {
        navController.navigate("project/$projectId")
    }
    
    fun navigateToCreateDocument(navController: NavHostController) {
        navController.navigate("document/new")
    }
    
    fun navigateToSettings(navController: NavHostController, setting: String? = null) {
        val destination = if (setting != null) "settings/$setting" else "settings"
        navController.navigate(destination)
    }
}