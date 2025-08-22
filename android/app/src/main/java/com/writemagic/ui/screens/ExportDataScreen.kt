package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

data class ExportOption(
    val id: String,
    val title: String,
    val description: String,
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val formats: List<String>,
    val estimatedSize: String
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ExportDataScreen(
    onNavigateBack: () -> Unit
) {
    var selectedOptions by remember { mutableStateOf(setOf<String>()) }
    var selectedFormat by remember { mutableStateOf("JSON") }
    var includeMetadata by remember { mutableStateOf(true) }
    var isExporting by remember { mutableStateOf(false) }
    var exportProgress by remember { mutableStateOf(0f) }
    var exportStatus by remember { mutableStateOf<String?>(null) }
    var showExportDialog by remember { mutableStateOf(false) }
    
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    
    val exportOptions = remember {
        listOf(
            ExportOption(
                id = "documents",
                title = "All Documents",
                description = "Export all your documents with content and metadata",
                icon = Icons.Default.Description,
                formats = listOf("JSON", "Markdown", "PDF", "DOCX"),
                estimatedSize = "~2.5 MB"
            ),
            ExportOption(
                id = "projects",
                title = "Projects",
                description = "Export project structures and organization",
                icon = Icons.Default.Folder,
                formats = listOf("JSON", "CSV"),
                estimatedSize = "~150 KB"
            ),
            ExportOption(
                id = "ai_history",
                title = "AI Interaction History",
                description = "Export your AI conversation history and prompts",
                icon = Icons.Default.SmartToy,
                formats = listOf("JSON", "TXT"),
                estimatedSize = "~800 KB"
            ),
            ExportOption(
                id = "settings",
                title = "App Settings",
                description = "Export your preferences and configuration",
                icon = Icons.Default.Settings,
                formats = listOf("JSON"),
                estimatedSize = "~5 KB"
            ),
            ExportOption(
                id = "timeline",
                title = "Writing Timeline",
                description = "Export your writing progress and version history",
                icon = Icons.Default.Timeline,
                formats = listOf("JSON", "CSV"),
                estimatedSize = "~300 KB"
            )
        )
    }
    
    val availableFormats = remember(selectedOptions) {
        if (selectedOptions.isEmpty()) {
            emptyList()
        } else {
            exportOptions
                .filter { it.id in selectedOptions }
                .flatMap { it.formats }
                .distinct()
        }
    }
    
    fun startExport() {
        scope.launch {
            isExporting = true
            exportProgress = 0f
            exportStatus = "Preparing export..."
            
            try {
                // Simulate export process
                delay(500)
                exportStatus = "Collecting documents..."
                exportProgress = 0.2f
                delay(1000)
                
                exportStatus = "Processing projects..."
                exportProgress = 0.4f
                delay(800)
                
                exportStatus = "Generating export file..."
                exportProgress = 0.7f
                delay(1000)
                
                exportStatus = "Finalizing..."
                exportProgress = 0.9f
                delay(500)
                
                exportProgress = 1.0f
                exportStatus = "Export completed successfully!"
                
                delay(2000)
                exportStatus = null
                showExportDialog = false
            } catch (e: Exception) {
                exportStatus = "Export failed: ${e.message}"
                delay(3000)
                exportStatus = null
            } finally {
                isExporting = false
                exportProgress = 0f
            }
        }
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Export Data") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                }
            )
        }
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Header
            item {
                Column {
                    Text(
                        text = "Export Your Data",
                        style = MaterialTheme.typography.headlineSmall
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Select the data you want to export. Your data will be packaged in a downloadable file.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            
            // Export options
            item {
                Text(
                    text = "What to Export",
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.padding(vertical = 8.dp)
                )
            }
            
            items(exportOptions) { option ->
                ExportOptionCard(
                    option = option,
                    isSelected = option.id in selectedOptions,
                    onSelectionChanged = { isSelected ->
                        selectedOptions = if (isSelected) {
                            selectedOptions + option.id
                        } else {
                            selectedOptions - option.id
                        }
                    }
                )
            }
            
            // Format selection
            if (selectedOptions.isNotEmpty()) {
                item {
                    Text(
                        text = "Export Format",
                        style = MaterialTheme.typography.titleLarge,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }
                
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            availableFormats.forEach { format ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 4.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    RadioButton(
                                        selected = selectedFormat == format,
                                        onClick = { selectedFormat = format }
                                    )
                                    Spacer(modifier = Modifier.width(8.dp))
                                    Column {
                                        Text(
                                            text = format,
                                            style = MaterialTheme.typography.bodyLarge
                                        )
                                        Text(
                                            text = when (format) {
                                                "JSON" -> "Machine-readable format, preserves all data"
                                                "Markdown" -> "Human-readable text format"
                                                "PDF" -> "Formatted documents for sharing"
                                                "DOCX" -> "Microsoft Word format"
                                                "CSV" -> "Spreadsheet-compatible format"
                                                "TXT" -> "Plain text format"
                                                else -> "Standard format"
                                            },
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Options
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Text(
                                text = "Export Options",
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.padding(bottom = 8.dp)
                            )
                            
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = "Include Metadata",
                                        style = MaterialTheme.typography.bodyLarge
                                    )
                                    Text(
                                        text = "Export creation dates, modification times, etc.",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                                Switch(
                                    checked = includeMetadata,
                                    onCheckedChange = { includeMetadata = it }
                                )
                            }
                        }
                    }
                }
                
                // Export summary
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.primaryContainer
                        )
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Text(
                                text = "Export Summary",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            
                            val totalSize = exportOptions
                                .filter { it.id in selectedOptions }
                                .sumOf { 
                                    (it.estimatedSize.replace("~", "").replace(" MB", "").replace(" KB", "").toFloatOrNull() ?: 0f).toDouble()
                                }
                            
                            Text(
                                text = "• ${selectedOptions.size} data types selected\n" +
                                      "• Format: $selectedFormat\n" +
                                      "• Estimated size: ~${String.format("%.1f", totalSize)} MB\n" +
                                      "• Metadata: ${if (includeMetadata) "Included" else "Excluded"}",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }
                    }
                }
                
                // Export button
                item {
                    Button(
                        onClick = { showExportDialog = true },
                        modifier = Modifier.fillMaxWidth(),
                        enabled = selectedOptions.isNotEmpty() && !isExporting
                    ) {
                        Icon(
                            Icons.Default.Download,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Start Export")
                    }
                }
            }
        }
    }
    
    // Export confirmation dialog
    if (showExportDialog) {
        AlertDialog(
            onDismissRequest = { if (!isExporting) showExportDialog = false },
            title = { Text("Export Data") },
            text = {
                Column {
                    if (isExporting) {
                        Text("Exporting your data...")
                        Spacer(modifier = Modifier.height(16.dp))
                        LinearProgressIndicator(
                            progress = exportProgress,
                            modifier = Modifier.fillMaxWidth()
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        exportStatus?.let { status ->
                            Text(
                                text = status,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        Text("Are you sure you want to export the selected data? This will create a downloadable file with your WriteMagic data.")
                    }
                }
            },
            confirmButton = {
                if (!isExporting) {
                    TextButton(
                        onClick = { startExport() }
                    ) {
                        Text("Export")
                    }
                }
            },
            dismissButton = {
                if (!isExporting) {
                    TextButton(
                        onClick = { showExportDialog = false }
                    ) {
                        Text("Cancel")
                    }
                }
            }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ExportOptionCard(
    option: ExportOption,
    isSelected: Boolean,
    onSelectionChanged: (Boolean) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(
            defaultElevation = if (isSelected) 4.dp else 2.dp
        ),
        colors = CardDefaults.cardColors(
            containerColor = if (isSelected) {
                MaterialTheme.colorScheme.primaryContainer
            } else {
                MaterialTheme.colorScheme.surface
            }
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.Top
        ) {
            Checkbox(
                checked = isSelected,
                onCheckedChange = onSelectionChanged
            )
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Icon(
                imageVector = option.icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = if (isSelected) {
                    MaterialTheme.colorScheme.onPrimaryContainer
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                }
            )
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = option.title,
                    style = MaterialTheme.typography.titleMedium,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer
                    } else {
                        MaterialTheme.colorScheme.onSurface
                    }
                )
                
                Text(
                    text = option.description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    }
                )
                
                Spacer(modifier = Modifier.height(4.dp))
                
                Text(
                    text = "Formats: ${option.formats.joinToString(", ")} • ${option.estimatedSize}",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                    }
                )
            }
        }
    }
}