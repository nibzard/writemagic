package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import com.writemagic.core.WriteMagicCore
import com.writemagic.core.Document
import com.writemagic.ui.components.WritingToolbar
import com.writemagic.ui.components.FormatAction

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DocumentEditorScreen(
    documentId: String,
    onNavigateBack: () -> Unit
) {
    var document by remember { mutableStateOf<Document?>(null) }
    var documentContent by remember { mutableStateOf(TextFieldValue("")) }
    var documentTitle by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(true) }
    var isSaving by remember { mutableStateOf(false) }
    var lastSaved by remember { mutableStateOf<String?>(null) }
    var isDistractionFree by remember { mutableStateOf(false) }
    var showAIAssist by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current
    val scrollState = rememberScrollState()
    
    // Load document if not new
    LaunchedEffect(documentId) {
        if (documentId != "new") {
            try {
                val result = WriteMagicCore.getDocument(documentId)
                if (result != null) {
                    val doc = Json.decodeFromString<Document>(result)
                    document = doc
                    documentTitle = doc.title
                    documentContent = TextFieldValue(doc.content)
                } else {
                    errorMessage = "Document not found"
                }
            } catch (e: Exception) {
                errorMessage = "Error loading document: ${e.message}"
            } finally {
                isLoading = false
            }
        } else {
            isLoading = false
            LaunchedEffect(Unit) {
                focusRequester.requestFocus()
            }
        }
    }
    
    // Auto-save functionality
    LaunchedEffect(documentContent.text, documentTitle) {
        if (!isLoading && documentContent.text.isNotEmpty()) {
            delay(2000) // Auto-save after 2 seconds of inactivity
            
            scope.launch {
                try {
                    isSaving = true
                    
                    if (documentId == "new") {
                        // Create new document
                        val result = WriteMagicCore.createDocument(
                            title = documentTitle.ifBlank { "Untitled Document" },
                            content = documentContent.text,
                            contentType = "markdown"
                        )
                        if (result != null) {
                            val newDoc = Json.decodeFromString<Document>(result)
                            document = newDoc
                            lastSaved = "Just saved"
                        }
                    } else {
                        // Update existing document
                        val success = WriteMagicCore.updateDocumentContent(documentId, documentContent.text)
                        if (success) {
                            lastSaved = "Auto-saved"
                        }
                    }
                } catch (e: Exception) {
                    errorMessage = "Save failed: ${e.message}"
                } finally {
                    isSaving = false
                    delay(3000)
                    lastSaved = null
                }
            }
        }
    }
    
    // Manual save function
    fun saveDocument() {
        scope.launch {
            try {
                isSaving = true
                
                if (documentId == "new") {
                    val result = WriteMagicCore.createDocument(
                        title = documentTitle.ifBlank { "Untitled Document" },
                        content = documentContent.text,
                        contentType = "markdown"
                    )
                    if (result != null) {
                        val newDoc = Json.decodeFromString<Document>(result)
                        document = newDoc
                        lastSaved = "Saved"
                    }
                } else {
                    val success = WriteMagicCore.updateDocumentContent(documentId, documentContent.text)
                    if (success) {
                        lastSaved = "Saved"
                    }
                }
            } catch (e: Exception) {
                errorMessage = "Save failed: ${e.message}"
            } finally {
                isSaving = false
                delay(3000)
                lastSaved = null
            }
        }
    }
    
    // Format text function
    fun formatText(action: FormatAction) {
        val selection = documentContent.selection
        val text = documentContent.text
        
        when (action) {
            FormatAction.Bold -> {
                val newText = if (selection.collapsed) {
                    text.substring(0, selection.start) + "**bold text**" + text.substring(selection.end)
                } else {
                    text.substring(0, selection.start) + "**${text.substring(selection.start, selection.end)}**" + text.substring(selection.end)
                }
                documentContent = documentContent.copy(
                    text = newText,
                    selection = androidx.compose.ui.text.TextRange(selection.start + 2, selection.start + 2 + "bold text".length)
                )
            }
            FormatAction.Italic -> {
                val newText = if (selection.collapsed) {
                    text.substring(0, selection.start) + "*italic text*" + text.substring(selection.end)
                } else {
                    text.substring(0, selection.start) + "*${text.substring(selection.start, selection.end)}*" + text.substring(selection.end)
                }
                documentContent = documentContent.copy(text = newText)
            }
            FormatAction.BulletList -> {
                val newText = text.substring(0, selection.start) + "\n- " + text.substring(selection.end)
                documentContent = documentContent.copy(text = newText)
            }
            FormatAction.NumberedList -> {
                val newText = text.substring(0, selection.start) + "\n1. " + text.substring(selection.end)
                documentContent = documentContent.copy(text = newText)
            }
            else -> { /* Handle other format actions */ }
        }
    }
    
    if (isLoading) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            CircularProgressIndicator()
        }
        return
    }
    
    Scaffold(
        topBar = {
            if (!isDistractionFree) {
                TopAppBar(
                    title = {
                        if (documentId == "new") {
                            OutlinedTextField(
                                value = documentTitle,
                                onValueChange = { documentTitle = it },
                                placeholder = { Text("Document title...") },
                                singleLine = true,
                                modifier = Modifier.fillMaxWidth()
                            )
                        } else {
                            Text(documentTitle)
                        }
                    },
                    navigationIcon = {
                        IconButton(onClick = onNavigateBack) {
                            Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                        }
                    },
                    actions = {
                        // Save status indicator
                        when {
                            isSaving -> {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(20.dp),
                                    strokeWidth = 2.dp
                                )
                            }
                            lastSaved != null -> {
                                Text(
                                    text = lastSaved!!,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                        }
                        
                        IconButton(onClick = { saveDocument() }) {
                            Icon(Icons.Default.Save, contentDescription = "Save")
                        }
                        
                        IconButton(onClick = { showAIAssist = true }) {
                            Icon(Icons.Default.SmartToy, contentDescription = "AI Assistant")
                        }
                        
                        IconButton(onClick = { isDistractionFree = !isDistractionFree }) {
                            Icon(
                                if (isDistractionFree) Icons.Default.FullscreenExit else Icons.Default.Fullscreen,
                                contentDescription = if (isDistractionFree) "Exit distraction-free" else "Distraction-free mode"
                            )
                        }
                    }
                )
            }
        },
        bottomBar = {
            WritingToolbar(
                isDistractionFree = isDistractionFree,
                onDistractionFreeToggle = { isDistractionFree = !isDistractionFree },
                onFormatText = ::formatText,
                onAIAssist = { showAIAssist = true },
                onSave = ::saveDocument
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            // Error message
            errorMessage?.let { error ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            Icons.Default.Error,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onErrorContainer
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = error,
                            color = MaterialTheme.colorScheme.onErrorContainer
                        )
                        Spacer(modifier = Modifier.weight(1f))
                        IconButton(onClick = { errorMessage = null }) {
                            Icon(
                                Icons.Default.Close,
                                contentDescription = "Dismiss",
                                tint = MaterialTheme.colorScheme.onErrorContainer
                            )
                        }
                    }
                }
            }
            
            // Writing area
            BasicTextField(
                value = documentContent,
                onValueChange = { documentContent = it },
                modifier = Modifier
                    .fillMaxSize()
                    .focusRequester(focusRequester)
                    .padding(16.dp)
                    .verticalScroll(scrollState),
                textStyle = TextStyle(
                    fontSize = if (isDistractionFree) 18.sp else 16.sp,
                    lineHeight = if (isDistractionFree) 28.sp else 24.sp,
                    fontFamily = FontFamily.Serif,
                    color = MaterialTheme.colorScheme.onSurface
                ),
                keyboardOptions = KeyboardOptions(
                    imeAction = ImeAction.Default
                ),
                decorationBox = { innerTextField ->
                    Box(
                        modifier = Modifier.fillMaxSize()
                    ) {
                        if (documentContent.text.isEmpty()) {
                            Text(
                                "Start writing your thoughts...",
                                style = TextStyle(
                                    fontSize = if (isDistractionFree) 18.sp else 16.sp,
                                    lineHeight = if (isDistractionFree) 28.sp else 24.sp,
                                    fontFamily = FontFamily.Serif,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            )
                        }
                        innerTextField()
                    }
                }
            )
        }
    }
    
    // AI Assistant overlay
    if (showAIAssist) {
        // You can implement an AI assistant dialog here
        // For now, just dismiss it
        LaunchedEffect(Unit) {
            delay(100)
            showAIAssist = false
        }
    }
}