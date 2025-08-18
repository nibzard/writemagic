package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

data class ChatMessage(
    val content: String,
    val isUser: Boolean,
    val timestamp: String = "Now"
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AIScreen() {
    var messages by remember {
        mutableStateOf(
            listOf(
                ChatMessage("Hello! I'm your AI writing assistant. How can I help you today?", false)
            )
        )
    }
    var inputText by remember { mutableStateOf("") }
    var selectedProvider by remember { mutableStateOf("Claude") }
    var isProcessing by remember { mutableStateOf(false) }
    
    val providers = listOf("Claude", "GPT-4", "Local Model")
    
    Column(
        modifier = Modifier.fillMaxSize()
    ) {
        // Provider selection bar
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    Icons.Default.SmartToy,
                    contentDescription = "AI Provider"
                )
                Text(
                    "Provider:",
                    style = MaterialTheme.typography.labelMedium
                )
                
                providers.forEach { provider ->
                    FilterChip(
                        onClick = { selectedProvider = provider },
                        label = { Text(provider) },
                        selected = selectedProvider == provider
                    )
                }
            }
        }
        
        // Chat messages
        LazyColumn(
            modifier = Modifier
                .weight(1f)
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(messages) { message ->
                MessageBubble(message)
            }
            
            if (isProcessing) {
                item {
                    ProcessingIndicator()
                }
            }
        }
        
        // Input area
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                verticalAlignment = Alignment.Bottom
            ) {
                OutlinedTextField(
                    value = inputText,
                    onValueChange = { inputText = it },
                    modifier = Modifier.weight(1f),
                    placeholder = { Text("Ask me anything about your writing...") },
                    maxLines = 4
                )
                
                Spacer(modifier = Modifier.width(8.dp))
                
                FilledTonalIconButton(
                    onClick = {
                        if (inputText.isNotBlank() && !isProcessing) {
                            messages = messages + ChatMessage(inputText, true)
                            isProcessing = true
                            
                            // Simulate AI response
                            val userMessage = inputText
                            inputText = ""
                            
                            // Mock AI response after delay
                            // In real implementation, this would call the Rust FFI
                            // processAIRequest(userMessage, context)
                        }
                    },
                    enabled = inputText.isNotBlank() && !isProcessing
                ) {
                    Icon(Icons.Default.Send, contentDescription = "Send")
                }
            }
        }
        
        // Quick actions
        LazyRow(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            item {
                AssistantChip(
                    onClick = { inputText = "Continue writing from where I left off" },
                    label = { Text("Continue Writing") }
                )
            }
            item {
                AssistantChip(
                    onClick = { inputText = "Improve the clarity of this paragraph" },
                    label = { Text("Improve Clarity") }
                )
            }
            item {
                AssistantChip(
                    onClick = { inputText = "Suggest alternative phrasings" },
                    label = { Text("Rephrase") }
                )
            }
        }
    }
}

@Composable
fun MessageBubble(message: ChatMessage) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (message.isUser) Arrangement.End else Arrangement.Start
    ) {
        if (!message.isUser) {
            Icon(
                Icons.Default.SmartToy,
                contentDescription = "AI",
                modifier = Modifier
                    .size(32.dp)
                    .padding(end = 8.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        }
        
        Card(
            modifier = Modifier
                .widthIn(max = 280.dp)
                .clip(RoundedCornerShape(16.dp)),
            colors = CardDefaults.cardColors(
                containerColor = if (message.isUser) {
                    MaterialTheme.colorScheme.primary
                } else {
                    MaterialTheme.colorScheme.surfaceVariant
                }
            )
        ) {
            Text(
                text = message.content,
                modifier = Modifier.padding(12.dp),
                color = if (message.isUser) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                }
            )
        }
        
        if (message.isUser) {
            Icon(
                Icons.Default.Person,
                contentDescription = "User",
                modifier = Modifier
                    .size(32.dp)
                    .padding(start = 8.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        }
    }
}

@Composable
fun ProcessingIndicator() {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.Start
    ) {
        Icon(
            Icons.Default.SmartToy,
            contentDescription = "AI",
            modifier = Modifier
                .size(32.dp)
                .padding(end = 8.dp),
            tint = MaterialTheme.colorScheme.primary
        )
        
        Card(
            modifier = Modifier.clip(RoundedCornerShape(16.dp)),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceVariant
            )
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                CircularProgressIndicator(
                    modifier = Modifier.size(16.dp),
                    strokeWidth = 2.dp
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("AI is thinking...")
            }
        }
    }
}