package com.writemagic.ui.screens

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.selection.LocalTextSelectionColors
import androidx.compose.foundation.text.selection.TextSelectionColors
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.material.icons.filled.ArrowForward
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Spellcheck
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalAccessibilityManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.delay
import com.writemagic.core.WriteMagicCore
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class, ExperimentalComposeUiApi::class)
@Composable
fun WritingScreen() {
    var documentContent by remember { mutableStateOf(TextFieldValue("")) }
    var documentTitle by remember { mutableStateOf("New Document") }
    var currentDocumentId by remember { mutableStateOf<String?>(null) }
    var isPaneMode by remember { mutableStateOf(false) }
    var showAIAssistant by remember { mutableStateOf(false) }
    var isLoading by remember { mutableStateOf(false) }
    var statusMessage by remember { mutableStateOf("") }
    var showStatusMessage by remember { mutableStateOf(false) }
    var isSaving by remember { mutableStateOf(false) }
    var isDistractionFreeMode by remember { mutableStateOf(false) }
    var wordCount by remember { mutableStateOf(0) }
    var characterCount by remember { mutableStateOf(0) }
    
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current
    val accessibilityManager = LocalAccessibilityManager.current
    
    // Calculate statistics
    LaunchedEffect(documentContent.text) {
        wordCount = documentContent.text.split("\\s+".toRegex()).filter { it.isNotBlank() }.size
        characterCount = documentContent.text.length
    }
    
    // Auto-save effect with visual feedback
    LaunchedEffect(documentContent.text, currentDocumentId) {
        if (currentDocumentId != null && documentContent.text.isNotEmpty()) {
            delay(1000) // Auto-save after 1 second of no changes
            isSaving = true
            val success = WriteMagicCore.updateDocumentContent(currentDocumentId!!, documentContent.text)
            if (success) {
                statusMessage = "Document saved automatically"
                showStatusMessage = true
            } else {
                statusMessage = "Auto-save failed - check connection"
                showStatusMessage = true
            }
            isSaving = false
            delay(3000)
            showStatusMessage = false
        }
    }
    
    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(if (isDistractionFreeMode) 8.dp else 16.dp)
        ) {
            // Top toolbar with animation
            AnimatedVisibility(
                visible = !isDistractionFreeMode,
                enter = slideInVertically() + fadeIn(),
                exit = slideOutVertically() + fadeOut()
            ) {
                WritingToolbar(
                    documentTitle = documentTitle,
                    onTitleChange = { documentTitle = it },
                    isPaneMode = isPaneMode,
                    onPaneModeToggle = { isPaneMode = !isPaneMode },
                    showAIAssistant = showAIAssistant,
                    onAIAssistantToggle = { showAIAssistant = !showAIAssistant },
                    isDistractionFreeMode = isDistractionFreeMode,
                    onDistractionFreeToggle = { isDistractionFreeMode = !isDistractionFreeMode },
                    onNewDocument = {
                        LaunchedEffect(Unit) {
                            isLoading = true
                            val newDoc = WriteMagicCore.createDocument("New Document", "", "markdown")
                            if (newDoc != null) {
                                currentDocumentId = newDoc.id
                                documentTitle = newDoc.title
                                documentContent = TextFieldValue(newDoc.content)
                                statusMessage = "New document created"
                                showStatusMessage = true
                            } else {
                                statusMessage = "Failed to create document"
                                showStatusMessage = true
                            }
                            isLoading = false
                        }
                    },
                    modifier = Modifier.padding(bottom = 16.dp)
                )
            }
            // Main writing area with gesture support
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .pointerInput(Unit) {
                        detectHorizontalDragGestures(
                            onDragEnd = {
                                // Gesture to toggle pane mode
                                isPaneMode = !isPaneMode
                            }
                        ) { _, _ -> }
                    }
            ) {
                if (isPaneMode) {
                    // Multi-pane layout with drag support
                    Row(
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        // Main pane
                        EnhancedWritingPane(
                            content = documentContent,
                            onContentChange = { documentContent = it },
                            title = "Main Draft",
                            focusRequester = focusRequester,
                            isDistractionFreeMode = isDistractionFreeMode,
                            modifier = Modifier.weight(1f)
                        )
                        
                        // Secondary pane
                        EnhancedWritingPane(
                            content = TextFieldValue(""),
                            onContentChange = { },
                            title = "Alternative",
                            isDistractionFreeMode = isDistractionFreeMode,
                            modifier = Modifier.weight(1f)
                        )
                    }
                } else {
                    // Single pane layout
                    EnhancedWritingPane(
                        content = documentContent,
                        onContentChange = { documentContent = it },
                        title = if (isDistractionFreeMode) "" else "Document",
                        focusRequester = focusRequester,
                        isDistractionFreeMode = isDistractionFreeMode,
                        modifier = Modifier.fillMaxSize()
                    )
                }
            }
        }
        
        // Status overlay with auto-save indicator
        AnimatedVisibility(
            visible = showStatusMessage || isSaving,
            enter = slideInVertically(initialOffsetY = { -it }) + fadeIn(),
            exit = slideOutVertically(targetOffsetY = { -it }) + fadeOut(),
            modifier = Modifier
                .align(Alignment.TopCenter)
                .padding(top = 8.dp)
        ) {
            StatusIndicator(
                message = statusMessage,
                isSaving = isSaving,
                modifier = Modifier.padding(horizontal = 16.dp)
            )
        }
        
        // Word count and statistics overlay
        AnimatedVisibility(
            visible = !isDistractionFreeMode,
            enter = slideInVertically(initialOffsetY = { it }) + fadeIn(),
            exit = slideOutVertically(targetOffsetY = { it }) + fadeOut(),
            modifier = Modifier
                .align(Alignment.BottomStart)
                .padding(16.dp)
        ) {
            WritingStatistics(
                wordCount = wordCount,
                characterCount = characterCount
            )
        }
        
        // Enhanced AI Assistant overlay
        if (showAIAssistant) {
            EnhancedAIAssistantOverlay(
                onDismiss = { showAIAssistant = false },
                onSuggestion = { suggestion ->
                    val newText = documentContent.text + "\n\n$suggestion"
                    documentContent = TextFieldValue(
                        text = newText,
                        selection = TextRange(newText.length)
                    )
                },
                onReplaceSelection = { replacement ->
                    val start = documentContent.selection.start
                    val end = documentContent.selection.end
                    val newText = documentContent.text.replaceRange(start, end, replacement)
                    documentContent = TextFieldValue(
                        text = newText,
                        selection = TextRange(start + replacement.length)
                    )
                },
                currentContent = documentContent.text,
                currentSelection = documentContent.text.substring(
                    documentContent.selection.start,
                    documentContent.selection.end
                )
            )
        }
    }
}

// Enhanced toolbar component
@Composable
fun WritingToolbar(
    documentTitle: String,
    onTitleChange: (String) -> Unit,
    isPaneMode: Boolean,
    onPaneModeToggle: () -> Unit,
    showAIAssistant: Boolean,
    onAIAssistantToggle: () -> Unit,
    isDistractionFreeMode: Boolean,
    onDistractionFreeToggle: () -> Unit,
    onNewDocument: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Document title with editing capability
                Row(
                    modifier = Modifier.weight(1f),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        Icons.Default.Description,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.padding(end = 8.dp)
                    )
                    BasicTextField(
                        value = documentTitle,
                        onValueChange = onTitleChange,
                        textStyle = MaterialTheme.typography.headlineSmall.copy(
                            color = MaterialTheme.colorScheme.onSurface
                        ),
                        singleLine = true,
                        decorationBox = { innerTextField ->
                            Box {
                                if (documentTitle.isEmpty()) {
                                    Text(
                                        "Document Title",
                                        style = MaterialTheme.typography.headlineSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                                innerTextField()
                            }
                        },
                        modifier = Modifier.semantics {
                            contentDescription = "Document title, editable"
                        }
                    )
                }
                
                // Action buttons
                Row {
                    IconButton(
                        onClick = onNewDocument,
                        modifier = Modifier.semantics {
                            contentDescription = "Create new document"
                        }
                    ) {
                        Icon(Icons.Default.Add, contentDescription = "New document")
                    }
                    
                    IconButton(
                        onClick = onPaneModeToggle,
                        modifier = Modifier.semantics {
                            contentDescription = if (isPaneMode) "Switch to single pane" else "Switch to multi-pane"
                        }
                    ) {
                        Icon(
                            if (isPaneMode) Icons.Default.ViewColumn else Icons.Default.ViewAgenda,
                            contentDescription = if (isPaneMode) "Multi-pane mode" else "Single pane mode"
                        )
                    }
                    
                    IconButton(
                        onClick = onDistractionFreeToggle,
                        modifier = Modifier.semantics {
                            contentDescription = if (isDistractionFreeMode) "Exit distraction-free mode" else "Enter distraction-free mode"
                        }
                    ) {
                        Icon(
                            if (isDistractionFreeMode) Icons.Default.Visibility else Icons.Default.VisibilityOff,
                            contentDescription = "Distraction-free mode",
                            tint = if (isDistractionFreeMode) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    
                    IconButton(
                        onClick = onAIAssistantToggle,
                        modifier = Modifier.semantics {
                            contentDescription = "AI Writing Assistant"
                        }
                    ) {
                        Icon(
                            Icons.Default.SmartToy,
                            contentDescription = "AI Assistant",
                            tint = if (showAIAssistant) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }
    }
}

// Enhanced writing pane with better accessibility and typography
@Composable
fun EnhancedWritingPane(
    content: TextFieldValue,
    onContentChange: (TextFieldValue) -> Unit,
    title: String,
    modifier: Modifier = Modifier,
    focusRequester: FocusRequester? = null,
    isDistractionFreeMode: Boolean = false
) {
    val customTextSelectionColors = TextSelectionColors(
        handleColor = MaterialTheme.colorScheme.primary,
        backgroundColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.4f)
    )
    
    Card(
        modifier = modifier,
        elevation = CardDefaults.cardElevation(defaultElevation = if (isDistractionFreeMode) 0.dp else 4.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (isDistractionFreeMode) Color.Transparent else MaterialTheme.colorScheme.surface
        ),
        shape = if (isDistractionFreeMode) RoundedCornerShape(0.dp) else RoundedCornerShape(12.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(if (isDistractionFreeMode) 0.dp else 16.dp)
        ) {
            if (title.isNotEmpty()) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier
                        .padding(bottom = 12.dp)
                        .semantics { contentDescription = "Writing pane: $title" }
                )
            }
            
            CompositionLocalProvider(LocalTextSelectionColors provides customTextSelectionColors) {
                BasicTextField(
                    value = content,
                    onValueChange = onContentChange,
                    modifier = Modifier
                        .fillMaxSize()
                        .then(focusRequester?.let { Modifier.focusRequester(it) } ?: Modifier)
                        .semantics {
                            contentDescription = "Main text editor"
                            stateDescription = if (content.text.isEmpty()) "Empty document" else "Document with ${content.text.length} characters"
                        },
                    textStyle = TextStyle(
                        fontSize = if (isDistractionFreeMode) 18.sp else 16.sp,
                        lineHeight = if (isDistractionFreeMode) 28.sp else 24.sp,
                        fontFamily = FontFamily.Serif,
                        color = MaterialTheme.colorScheme.onSurface
                    ),
                    keyboardOptions = KeyboardOptions(
                        imeAction = ImeAction.Default
                    ),
                    decorationBox = { innerTextField ->
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .padding(if (isDistractionFreeMode) 16.dp else 12.dp)
                        ) {
                            if (content.text.isEmpty()) {
                                Text(
                                    "Start writing your thoughts...",
                                    style = TextStyle(
                                        fontSize = if (isDistractionFreeMode) 18.sp else 16.sp,
                                        lineHeight = if (isDistractionFreeMode) 28.sp else 24.sp,
                                        fontFamily = FontFamily.Serif,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    ),
                                    modifier = Modifier.semantics {
                                        contentDescription = "Placeholder text: Start writing your thoughts"
                                    }
                                )
                            }
                            innerTextField()
                        }
                    }
                )
            }
        }
    }
}

// Status indicator component
@Composable
fun StatusIndicator(
    message: String,
    isSaving: Boolean,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (isSaving) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.tertiaryContainer
        ),
        shape = RoundedCornerShape(16.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (isSaving) {
                CircularProgressIndicator(
                    modifier = Modifier.size(16.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimaryContainer
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    "Saving...",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onPrimaryContainer
                )
            } else {
                Icon(
                    Icons.Default.CheckCircle,
                    contentDescription = null,
                    modifier = Modifier.size(16.dp),
                    tint = MaterialTheme.colorScheme.onTertiaryContainer
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    message,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onTertiaryContainer
                )
            }
        }
    }
}

// Writing statistics component
@Composable
fun WritingStatistics(
    wordCount: Int,
    characterCount: Int,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.8f)
        ),
        shape = RoundedCornerShape(8.dp)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    "$wordCount",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    "words",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Column(
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    "$characterCount",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    "chars",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

// Enhanced AI Assistant with improved UX
@Composable
fun EnhancedAIAssistantOverlay(
    onDismiss: () -> Unit,
    onSuggestion: (String) -> Unit,
    onReplaceSelection: (String) -> Unit,
    currentContent: String,
    currentSelection: String,
    modifier: Modifier = Modifier
) {
    var isGenerating by remember { mutableStateOf(false) }
    var customPrompt by remember { mutableStateOf("") }
    var lastResponse by remember { mutableStateOf("") }
    
    Card(
        modifier = modifier
            .fillMaxWidth()
            .padding(16.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 12.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surface
        )
    ) {
        Column(
            modifier = Modifier.padding(20.dp)
        ) {
            // Header with close button
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        Icons.Default.SmartToy,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.padding(end = 8.dp)
                    )
                    Text(
                        "AI Writing Assistant",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.SemiBold
                    )
                }
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.semantics {
                        contentDescription = "Close AI Assistant"
                    }
                ) {
                    Icon(Icons.Default.Close, contentDescription = "Close")
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Custom prompt input with enhanced design
            OutlinedTextField(
                value = customPrompt,
                onValueChange = { customPrompt = it },
                label = { Text("Ask AI to help with your writing...") },
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics {
                        contentDescription = "AI prompt input field"
                    },
                enabled = !isGenerating,
                trailingIcon = {
                    if (customPrompt.isNotEmpty()) {
                        IconButton(
                            onClick = {
                                if (!isGenerating) {
                                    isGenerating = true
                                    LaunchedEffect(Unit) {
                                        val response = WriteMagicCore.completeText("$customPrompt\n\nContext: ${currentContent.take(500)}")
                                        if (response.success && response.completion != null) {
                                            lastResponse = response.completion
                                            onSuggestion(response.completion)
                                        } else {
                                            lastResponse = "AI completion failed: ${response.error ?: "Unknown error"}"
                                        }
                                        isGenerating = false
                                        customPrompt = ""
                                    }
                                }
                            },
                            enabled = !isGenerating
                        ) {
                            if (isGenerating) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(20.dp),
                                    strokeWidth = 2.dp
                                )
                            } else {
                                Icon(Icons.Default.Send, contentDescription = "Send prompt")
                            }
                        }
                    }
                },
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                keyboardActions = KeyboardActions(
                    onSend = {
                        if (customPrompt.isNotEmpty() && !isGenerating) {
                            // Trigger AI generation
                        }
                    }
                )
            )
            
            // Quick action buttons grid
            Text(
                "Quick Actions",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            
            LazyColumn(
                modifier = Modifier.heightIn(max = 300.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                // Continue writing suggestion
                item {
                    AIActionCard(
                        title = "Continue Writing",
                        description = "Continue from where you left off",
                        icon = Icons.Default.ArrowForward,
                        enabled = !isGenerating && currentContent.isNotEmpty(),
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val prompt = "Continue writing from this text:\n\n${currentContent.takeLast(300)}"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    onSuggestion(response.completion)
                                } else {
                                    lastResponse = "Failed to generate continuation"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
                
                // Improve clarity
                item {
                    AIActionCard(
                        title = "Improve Clarity",
                        description = "Make your writing clearer and more concise",
                        icon = Icons.Default.Lightbulb,
                        enabled = !isGenerating && currentContent.isNotEmpty(),
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val targetText = if (currentSelection.isNotEmpty()) currentSelection else currentContent.takeLast(300)
                                val prompt = "Rewrite this text with more clarity and better flow:\n\n$targetText"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    if (currentSelection.isNotEmpty()) {
                                        onReplaceSelection(response.completion)
                                    } else {
                                        onSuggestion(response.completion)
                                    }
                                } else {
                                    lastResponse = "Failed to improve clarity"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
                
                // Add examples
                item {
                    AIActionCard(
                        title = "Add Examples",
                        description = "Include supporting examples and details",
                        icon = Icons.Default.AddCircle,
                        enabled = !isGenerating && currentContent.isNotEmpty(),
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val prompt = "Add supporting examples and details to this content:\n\n${currentContent.takeLast(300)}"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    onSuggestion(response.completion)
                                } else {
                                    lastResponse = "Failed to add examples"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
                
                // Summarize
                item {
                    AIActionCard(
                        title = "Summarize",
                        description = "Create a concise summary of your content",
                        icon = Icons.Default.Compress,
                        enabled = !isGenerating && currentContent.length > 100,
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val prompt = "Summarize this content in a few concise sentences:\n\n$currentContent"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    onSuggestion(response.completion)
                                } else {
                                    lastResponse = "Failed to create summary"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
                
                // Fix grammar
                item {
                    AIActionCard(
                        title = "Fix Grammar",
                        description = "Correct grammar and improve style",
                        icon = Icons.Default.Spellcheck,
                        enabled = !isGenerating && currentContent.isNotEmpty(),
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val targetText = if (currentSelection.isNotEmpty()) currentSelection else currentContent
                                val prompt = "Fix grammar, spelling, and improve the style of this text:\n\n$targetText"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    if (currentSelection.isNotEmpty()) {
                                        onReplaceSelection(response.completion)
                                    } else {
                                        onSuggestion("\n\n--- Corrected Version ---\n${response.completion}")
                                    }
                                } else {
                                    lastResponse = "Failed to fix grammar"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
                
                // Generate outline
                item {
                    AIActionCard(
                        title = "Generate Outline",
                        description = "Create a structured outline for your content",
                        icon = Icons.Default.List,
                        enabled = !isGenerating && currentContent.isNotEmpty(),
                        onClick = {
                            isGenerating = true
                            LaunchedEffect(Unit) {
                                val prompt = "Create a structured outline based on this content:\n\n$currentContent"
                                val response = WriteMagicCore.completeText(prompt)
                                if (response.success && response.completion != null) {
                                    onSuggestion("\n\n--- Outline ---\n${response.completion}")
                                } else {
                                    lastResponse = "Failed to generate outline"
                                }
                                isGenerating = false
                            }
                        }
                    )
                }
            }
        }
    }
}

// AI Action Card component
@Composable
fun AIActionCard(
    title: String,
    description: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    enabled: Boolean = true,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .then(
                if (enabled) {
                    Modifier.toggleable(
                        value = false,
                        onValueChange = { onClick() }
                    )
                } else {
                    Modifier
                }
            ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (enabled) MaterialTheme.colorScheme.surfaceVariant else MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
        )
    ) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = if (enabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .size(24.dp)
                    .padding(end = 4.dp)
            )
            Spacer(modifier = Modifier.width(12.dp))
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Medium,
                    color = if (enabled) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodySmall,
                    color = if (enabled) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
                    modifier = Modifier.padding(top = 2.dp)
                )
            }
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = null,
                tint = if (enabled) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}