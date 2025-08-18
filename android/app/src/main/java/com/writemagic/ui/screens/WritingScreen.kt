package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WritingScreen() {
    var documentContent by remember { mutableStateOf("") }
    var isPaneMode by remember { mutableStateOf(false) }
    var showAIAssistant by remember { mutableStateOf(false) }
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        // Top toolbar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                "Document.md",
                style = MaterialTheme.typography.headlineSmall
            )
            
            Row {
                IconButton(onClick = { isPaneMode = !isPaneMode }) {
                    Icon(
                        if (isPaneMode) Icons.Default.ViewColumn else Icons.Default.ViewAgenda,
                        contentDescription = "Toggle pane mode"
                    )
                }
                IconButton(onClick = { showAIAssistant = !showAIAssistant }) {
                    Icon(Icons.Default.SmartToy, contentDescription = "AI Assistant")
                }
            }
        }
        
        if (isPaneMode) {
            // Multi-pane layout
            Row(
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                // Main pane
                Card(
                    modifier = Modifier.weight(1f),
                    elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
                ) {
                    WritingPane(
                        content = documentContent,
                        onContentChange = { documentContent = it },
                        title = "Main Draft"
                    )
                }
                
                // Secondary pane
                Card(
                    modifier = Modifier.weight(1f),
                    elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
                ) {
                    WritingPane(
                        content = "",
                        onContentChange = { },
                        title = "Alternative"
                    )
                }
            }
        } else {
            // Single pane layout
            Card(
                modifier = Modifier.fillMaxSize(),
                elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
            ) {
                WritingPane(
                    content = documentContent,
                    onContentChange = { documentContent = it },
                    title = "Document"
                )
            }
        }
        
        // AI Assistant overlay
        if (showAIAssistant) {
            AIAssistantOverlay(
                onDismiss = { showAIAssistant = false },
                onSuggestion = { suggestion ->
                    documentContent += "\n\n$suggestion"
                }
            )
        }
    }
}

@Composable
fun WritingPane(
    content: String,
    onContentChange: (String) -> Unit,
    title: String
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.padding(bottom = 8.dp)
        )
        
        BasicTextField(
            value = content,
            onValueChange = onContentChange,
            modifier = Modifier.fillMaxSize(),
            textStyle = TextStyle(
                fontSize = 16.sp,
                lineHeight = 24.sp
            ),
            decorationBox = { innerTextField ->
                Box(
                    modifier = Modifier.fillMaxSize()
                ) {
                    if (content.isEmpty()) {
                        Text(
                            "Start writing...",
                            style = TextStyle(
                                fontSize = 16.sp,
                                color = Color.Gray
                            )
                        )
                    }
                    innerTextField()
                }
            }
        )
    }
}

@Composable
fun AIAssistantOverlay(
    onDismiss: () -> Unit,
    onSuggestion: (String) -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(16.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "AI Assistant",
                    style = MaterialTheme.typography.titleMedium
                )
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, contentDescription = "Close")
                }
            }
            
            Spacer(modifier = Modifier.height(8.dp))
            
            // AI suggestion buttons
            LazyColumn {
                item {
                    TextButton(
                        onClick = { onSuggestion("Continue writing from here...") }
                    ) {
                        Text("Continue Writing")
                    }
                }
                item {
                    TextButton(
                        onClick = { onSuggestion("Rewrite this section with more clarity...") }
                    ) {
                        Text("Improve Clarity")
                    }
                }
                item {
                    TextButton(
                        onClick = { onSuggestion("Add supporting examples...") }
                    ) {
                        Text("Add Examples")
                    }
                }
            }
        }
    }
}