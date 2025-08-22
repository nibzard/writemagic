package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen() {
    var darkModeEnabled by remember { mutableStateOf(false) }
    var autoSaveEnabled by remember { mutableStateOf(true) }
    var aiProvider by remember { mutableStateOf("Claude") }
    var syncEnabled by remember { mutableStateOf(false) }
    
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        item {
            Text(
                "Settings",
                style = MaterialTheme.typography.headlineMedium,
                modifier = Modifier.padding(bottom = 16.dp)
            )
        }
        
        // Appearance Section
        item {
            SettingsSection(title = "Appearance") {
                SettingsItem(
                    title = "Dark Mode",
                    description = "Use dark theme throughout the app",
                    icon = Icons.Default.DarkMode,
                    endContent = {
                        Switch(
                            checked = darkModeEnabled,
                            onCheckedChange = { darkModeEnabled = it }
                        )
                    }
                )
            }
        }
        
        // Writing Section
        item {
            SettingsSection(title = "Writing") {
                SettingsItem(
                    title = "Auto Save",
                    description = "Automatically save documents while writing",
                    icon = Icons.Default.Save,
                    endContent = {
                        Switch(
                            checked = autoSaveEnabled,
                            onCheckedChange = { autoSaveEnabled = it }
                        )
                    }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "Word Count Goal",
                    description = "Set daily writing goals",
                    icon = Icons.Default.TrendingUp,
                    onClick = { /* Open word count settings */ }
                )
            }
        }
        
        // AI Section
        item {
            SettingsSection(title = "AI Assistant") {
                SettingsItem(
                    title = "AI Provider",
                    description = "Current: $aiProvider",
                    icon = Icons.Default.SmartToy,
                    onClick = { /* Open provider selection */ }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "API Configuration",
                    description = "Configure API keys and settings",
                    icon = Icons.Default.Key,
                    onClick = { /* Open API settings */ }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "Usage & Billing",
                    description = "View API usage and costs",
                    icon = Icons.Default.Receipt,
                    onClick = { /* Open usage stats */ }
                )
            }
        }
        
        // Sync Section
        item {
            SettingsSection(title = "Sync & Backup") {
                SettingsItem(
                    title = "Cloud Sync",
                    description = "Sync documents across devices",
                    icon = Icons.Default.Cloud,
                    endContent = {
                        Switch(
                            checked = syncEnabled,
                            onCheckedChange = { syncEnabled = it }
                        )
                    }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "Export Data",
                    description = "Export all documents and projects",
                    icon = Icons.Default.Download,
                    onClick = { /* Export data */ }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "Import Data",
                    description = "Import documents from other apps",
                    icon = Icons.Default.Upload,
                    onClick = { /* Import data */ }
                )
            }
        }
        
        // Support Section
        item {
            SettingsSection(title = "Support") {
                SettingsItem(
                    title = "Help & Tutorials",
                    description = "Learn how to use WriteMagic",
                    icon = Icons.Default.Help,
                    onClick = { /* Open help */ }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "Send Feedback",
                    description = "Report bugs or suggest features",
                    icon = Icons.Default.Feedback,
                    onClick = { /* Open feedback */ }
                )
                
                HorizontalDivider()
                
                SettingsItem(
                    title = "About",
                    description = "Version info and credits",
                    icon = Icons.Default.Info,
                    onClick = { /* Open about */ }
                )
            }
        }
    }
}

@Composable
fun SettingsSection(
    title: String,
    content: @Composable ColumnScope.() -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            content()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsItem(
    title: String,
    description: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    onClick: (() -> Unit)? = null,
    endContent: @Composable (() -> Unit)? = null
) {
    val modifier = if (onClick != null) {
        Modifier.fillMaxWidth()
    } else {
        Modifier.fillMaxWidth()
    }
    
    if (onClick != null) {
        Surface(
            onClick = onClick,
            modifier = modifier
        ) {
            SettingsItemContent(title, description, icon, endContent)
        }
    } else {
        SettingsItemContent(title, description, icon, endContent)
    }
}

@Composable
private fun SettingsItemContent(
    title: String,
    description: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    endContent: @Composable (() -> Unit)?
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Spacer(modifier = Modifier.width(16.dp))
        
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge
            )
            Text(
                text = description,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        
        if (endContent != null) {
            endContent()
        } else {
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}