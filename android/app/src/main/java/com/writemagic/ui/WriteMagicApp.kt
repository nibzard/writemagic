package com.writemagic.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.writemagic.ui.screens.*

sealed class Screen(val route: String, val title: String, val icon: ImageVector) {
    object Writing : Screen("writing", "Writing", Icons.Default.Edit)
    object Projects : Screen("projects", "Projects", Icons.Default.Folder)
    object AI : Screen("ai", "AI Assistant", Icons.Default.SmartToy)
    object Timeline : Screen("timeline", "Timeline", Icons.Default.Timeline)
    object Settings : Screen("settings", "Settings", Icons.Default.Settings)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WriteMagicApp() {
    val navController = rememberNavController()
    val screens = listOf(
        Screen.Writing,
        Screen.Projects,
        Screen.AI,
        Screen.Timeline,
        Screen.Settings
    )
    
    Scaffold(
        bottomBar = {
            NavigationBar {
                val navBackStackEntry by navController.currentBackStackEntryAsState()
                val currentDestination = navBackStackEntry?.destination
                
                screens.forEach { screen ->
                    NavigationBarItem(
                        icon = { Icon(screen.icon, contentDescription = screen.title) },
                        label = { Text(screen.title) },
                        selected = currentDestination?.hierarchy?.any { it.route == screen.route } == true,
                        onClick = {
                            navController.navigate(screen.route) {
                                popUpTo(navController.graph.findStartDestination().id) {
                                    saveState = true
                                }
                                launchSingleTop = true
                                restoreState = true
                            }
                        }
                    )
                }
            }
        }
    ) { innerPadding ->
        NavHost(
            navController,
            startDestination = Screen.Writing.route,
            Modifier.padding(innerPadding)
        ) {
            composable(Screen.Writing.route) { WritingScreen() }
            composable(Screen.Projects.route) { ProjectsScreen() }
            composable(Screen.AI.route) { AIScreen() }
            composable(Screen.Timeline.route) { TimelineScreen() }
            composable(Screen.Settings.route) { SettingsScreen() }
        }
    }
}