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

data class ImportSource(
    val id: String,
    val title: String,
    val description: String,
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val supportedFormats: List<String>,
    val features: List<String>
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportDataScreen(
    onNavigateBack: () -> Unit
) {
    var selectedSource by remember { mutableStateOf<String?>(null) }
    var isImporting by remember { mutableStateOf(false) }
    var importProgress by remember { mutableStateOf(0f) }
    var importStatus by remember { mutableStateOf<String?>(null) }
    var showImportDialog by remember { mutableStateOf(false) }
    var showFileSelector by remember { mutableStateOf(false) }
    var selectedFile by remember { mutableStateOf<String?>(null) }
    var preserveStructure by remember { mutableStateOf(true) }
    var importMetadata by remember { mutableStateOf(true) }
    var overwriteExisting by remember { mutableStateOf(false) }
    
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    
    val importSources = remember {
        listOf(
            ImportSource(
                id = "writemagic",
                title = "WriteMagic Export",
                description = "Import data from a previous WriteMagic export",
                icon = Icons.Default.Upload,
                supportedFormats = listOf("JSON", "ZIP"),
                features = listOf("Documents", "Projects", "Settings", "AI History", "Timeline")
            ),
            ImportSource(
                id = "notion",
                title = "Notion",
                description = "Import pages and databases from Notion",
                icon = Icons.Default.Description,
                supportedFormats = listOf("Markdown", "HTML", "CSV"),
                features = listOf("Pages", "Databases", "Attachments")
            ),
            ImportSource(
                id = "obsidian",
                title = "Obsidian",
                description = "Import notes from Obsidian vault",
                icon = Icons.Default.Article,
                supportedFormats = listOf("Markdown", "ZIP"),
                features = listOf("Notes", "Links", "Attachments", "Tags")
            ),
            ImportSource(
                id = "markdown",
                title = "Markdown Files",
                description = "Import individual markdown documents",
                icon = Icons.Default.TextSnippet,
                supportedFormats = listOf("MD", "TXT", "ZIP"),
                features = listOf("Text Content", "Basic Formatting")
            ),
            ImportSource(
                id = "word",
                title = "Microsoft Word",
                description = "Import Word documents",
                icon = Icons.Default.Description,
                supportedFormats = listOf("DOCX", "DOC"),
                features = listOf("Text Content", "Formatting", "Images")
            ),
            ImportSource(
                id = "google_docs",
                title = "Google Docs",
                description = "Import from Google Docs export",
                icon = Icons.Default.CloudDownload,
                supportedFormats = listOf("DOCX", "ODT", "PDF"),
                features = listOf("Text Content", "Comments", "Formatting")
            ),
            ImportSource(
                id = "evernote",
                title = "Evernote",
                description = "Import notes from Evernote export",
                icon = Icons.Default.Note,
                supportedFormats = listOf("ENEX", "HTML"),
                features = listOf("Notes", "Notebooks", "Tags", "Attachments")
            ),
            ImportSource(
                id = "plaintext",
                title = "Plain Text",
                description = "Import plain text files",
                icon = Icons.Default.TextFields,
                supportedFormats = listOf("TXT", "RTF"),
                features = listOf("Raw Text Content")
            )
        )
    }
    
    fun startImport() {
        scope.launch {
            isImporting = true
            importProgress = 0f
            importStatus = "Preparing import..."
            
            try {
                delay(500)
                importStatus = "Reading file..."
                importProgress = 0.2f
                delay(1000)
                
                importStatus = "Parsing content..."
                importProgress = 0.4f
                delay(1200)
                
                importStatus = "Converting documents..."
                importProgress = 0.6f
                delay(1000)
                
                importStatus = "Creating projects..."
                importProgress = 0.8f
                delay(800)
                
                importStatus = "Finalizing import..."
                importProgress = 0.95f
                delay(500)
                
                importProgress = 1.0f
                importStatus = "Import completed successfully!"
                
                delay(2000)
                importStatus = null
                showImportDialog = false
                selectedFile = null
            } catch (e: Exception) {
                importStatus = "Import failed: ${e.message}"
                delay(3000)
                importStatus = null
            } finally {
                isImporting = false
                importProgress = 0f
            }
        }
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Import Data") },
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
                        text = "Import Your Data",
                        style = MaterialTheme.typography.headlineSmall
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Choose a source to import your existing documents and notes into WriteMagic.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            
            // Important note
            item {
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.tertiaryContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.Top
                    ) {
                        Icon(
                            Icons.Default.Info,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onTertiaryContainer
                        )
                        Spacer(modifier = Modifier.width(12.dp))
                        Column {
                            Text(
                                text = "Before You Start",
                                style = MaterialTheme.typography.titleSmall,
                                color = MaterialTheme.colorScheme.onTertiaryContainer
                            )
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = "Make sure you have exported your data from the source application. Import will create new documents and won't modify your original files.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onTertiaryContainer
                            )
                        }
                    }
                }
            }
            
            // Import sources
            item {
                Text(
                    text = "Select Import Source",
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.padding(vertical = 8.dp)
                )
            }
            
            items(importSources) { source ->
                ImportSourceCard(
                    source = source,
                    isSelected = selectedSource == source.id,
                    onSelectionChanged = { 
                        selectedSource = if (selectedSource == source.id) null else source.id
                    }
                )
            }
            
            // File selection and options
            selectedSource?.let { sourceId ->
                val source = importSources.find { it.id == sourceId }
                
                item {
                    Text(
                        text = "Import Options",
                        style = MaterialTheme.typography.titleLarge,
                        modifier = Modifier.padding(vertical = 8.dp)
                    )
                }
                
                // File selection
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Text(
                                text = "Select File",
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.padding(bottom = 8.dp)
                            )
                            
                            Button(
                                onClick = { 
                                    showFileSelector = true
                                    selectedFile = "sample_export.json" // Simulate file selection
                                },
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Icon(Icons.Default.AttachFile, contentDescription = null)
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    if (selectedFile != null) "Change File ($selectedFile)" else "Choose File"
                                )
                            }
                            
                            if (source != null) {
                                Spacer(modifier = Modifier.height(8.dp))
                                Text(
                                    text = "Supported formats: ${source.supportedFormats.joinToString(", ")}",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                }
                
                // Import settings
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Text(
                                text = "Import Settings",
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.padding(bottom = 16.dp)
                            )
                            
                            // Preserve structure
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = "Preserve Structure",
                                        style = MaterialTheme.typography.bodyLarge
                                    )
                                    Text(
                                        text = "Maintain folder/project organization",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                                Switch(
                                    checked = preserveStructure,
                                    onCheckedChange = { preserveStructure = it }
                                )
                            }
                            
                            Spacer(modifier = Modifier.height(16.dp))
                            
                            // Import metadata
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = "Import Metadata",
                                        style = MaterialTheme.typography.bodyLarge
                                    )
                                    Text(
                                        text = "Include creation dates, tags, etc.",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                                Switch(
                                    checked = importMetadata,
                                    onCheckedChange = { importMetadata = it }
                                )
                            }
                            
                            Spacer(modifier = Modifier.height(16.dp))
                            
                            // Overwrite existing
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = "Overwrite Existing",
                                        style = MaterialTheme.typography.bodyLarge
                                    )
                                    Text(
                                        text = "Replace documents with same name",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                                Switch(
                                    checked = overwriteExisting,
                                    onCheckedChange = { overwriteExisting = it }
                                )
                            }
                        }
                    }
                }
                
                // Features preview
                if (source != null) {
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
                                    text = "What Will Be Imported",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                                Spacer(modifier = Modifier.height(8.dp))
                                source.features.forEach { feature ->
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Icon(
                                            Icons.Default.CheckCircle,
                                            contentDescription = null,
                                            modifier = Modifier.size(16.dp),
                                            tint = MaterialTheme.colorScheme.onPrimaryContainer
                                        )
                                        Spacer(modifier = Modifier.width(8.dp))
                                        Text(
                                            text = feature,
                                            style = MaterialTheme.typography.bodyMedium,
                                            color = MaterialTheme.colorScheme.onPrimaryContainer
                                        )
                                    }
                                    Spacer(modifier = Modifier.height(4.dp))
                                }
                            }
                        }
                    }
                }
                
                // Import button
                item {
                    Button(
                        onClick = { showImportDialog = true },
                        modifier = Modifier.fillMaxWidth(),
                        enabled = selectedFile != null && !isImporting
                    ) {
                        Icon(
                            Icons.Default.Upload,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Start Import")
                    }
                }
            }
        }
    }
    
    // Import confirmation dialog
    if (showImportDialog) {
        AlertDialog(
            onDismissRequest = { if (!isImporting) showImportDialog = false },
            title = { Text("Import Data") },
            text = {
                Column {
                    if (isImporting) {
                        Text("Importing your data...")
                        Spacer(modifier = Modifier.height(16.dp))
                        LinearProgressIndicator(
                            progress = importProgress,
                            modifier = Modifier.fillMaxWidth()
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        importStatus?.let { status ->
                            Text(
                                text = status,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        Text("Are you sure you want to import from ${selectedFile}? This will add new documents to your WriteMagic workspace.")
                    }
                }
            },
            confirmButton = {
                if (!isImporting) {
                    TextButton(
                        onClick = { startImport() }
                    ) {
                        Text("Import")
                    }
                }
            },
            dismissButton = {
                if (!isImporting) {
                    TextButton(
                        onClick = { showImportDialog = false }
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
fun ImportSourceCard(
    source: ImportSource,
    isSelected: Boolean,
    onSelectionChanged: () -> Unit
) {
    Card(
        onClick = onSelectionChanged,
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
            Icon(
                imageVector = source.icon,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = if (isSelected) {
                    MaterialTheme.colorScheme.onPrimaryContainer
                } else {
                    MaterialTheme.colorScheme.primary
                }
            )
            
            Spacer(modifier = Modifier.width(16.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = source.title,
                    style = MaterialTheme.typography.titleMedium,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer
                    } else {
                        MaterialTheme.colorScheme.onSurface
                    }
                )
                
                Text(
                    text = source.description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.8f)
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    }
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                Text(
                    text = "Formats: ${source.supportedFormats.joinToString(", ")}",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f)
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                    }
                )
            }
            
            if (isSelected) {
                Icon(
                    Icons.Default.CheckCircle,
                    contentDescription = "Selected",
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.onPrimaryContainer
                )
            }
        }
    }
}