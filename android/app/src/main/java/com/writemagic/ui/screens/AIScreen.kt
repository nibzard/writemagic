package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import com.writemagic.core.WriteMagicCore

@Serializable
data class AIResponse(
    val completion: String? = null,
    val error: String? = null,
    val success: Boolean
)

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
    var errorMessage by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    
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
                            val userMessage = inputText
                            messages = messages + ChatMessage(userMessage, true)
                            inputText = ""
                            isProcessing = true
                            errorMessage = null
                            
                            scope.launch {
                                try {
                                    val model = when (selectedProvider) {
                                        "Claude" -> "claude-3-sonnet"
                                        "GPT-4" -> "gpt-4"
                                        else -> ""
                                    }
                                    
                                    val aiResponse = WriteMagicCore.completeText(userMessage, model)
                                    
                                    if (aiResponse.success && aiResponse.completion != null) {
                                        messages = messages + ChatMessage(aiResponse.completion, false)
                                    } else {
                                        val errorMsg = aiResponse.error ?: "AI request failed"
                                        messages = messages + ChatMessage("Sorry, I encountered an error: $errorMsg", false)
                                        errorMessage = errorMsg
                                    }
                                } catch (e: Exception) {
                                    val errorMsg = "Error: ${e.message}"
                                    messages = messages + ChatMessage("Sorry, I encountered an error. Please try again.", false)
                                    errorMessage = errorMsg
                                } finally {
                                    isProcessing = false
                                }
                            }
                        }
                    },
                    enabled = inputText.isNotBlank() && !isProcessing
                ) {
                    Icon(Icons.Default.Send, contentDescription = "Send")
                }
            }
        }
        
        // Error message display
        errorMessage?.let { error ->
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer
                )
            ) {
                Row(
                    modifier = Modifier.padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        Icons.Default.Warning,
                        contentDescription = null,
                        modifier = Modifier.size(20.dp),
                        tint = MaterialTheme.colorScheme.onErrorContainer
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = error,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                    Spacer(modifier = Modifier.weight(1f))
                    IconButton(
                        onClick = { errorMessage = null }
                    ) {
                        Icon(
                            Icons.Default.Close,
                            contentDescription = "Dismiss",
                            modifier = Modifier.size(16.dp),
                            tint = MaterialTheme.colorScheme.onErrorContainer
                        )
                    }
                }
            }
        }
        
        // Quick actions
        LazyRow(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            item {
                SuggestionChip(
                    onClick = { 
                        if (!isProcessing) {
                            inputText = "Continue writing from where I left off"
                        }
                    },
                    label = { Text("Continue Writing") },
                    enabled = !isProcessing
                )
            }
            item {
                SuggestionChip(
                    onClick = { 
                        if (!isProcessing) {
                            inputText = "Improve the clarity of this paragraph"
                        }
                    },
                    label = { Text("Improve Clarity") },
                    enabled = !isProcessing
                )
            }
            item {
                SuggestionChip(
                    onClick = { 
                        if (!isProcessing) {
                            inputText = "Suggest alternative phrasings"
                        }
                    },
                    label = { Text("Rephrase") },
                    enabled = !isProcessing
                )
            }
            item {
                SuggestionChip(
                    onClick = { 
                        if (!isProcessing) {
                            inputText = "Help me brainstorm ideas for my writing"
                        }
                    },
                    label = { Text("Brainstorm") },
                    enabled = !isProcessing
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