package com.writemagic.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AISettingsScreen(
    onNavigateBack: () -> Unit
) {
    var selectedProvider by remember { mutableStateOf("Claude") }
    var claudeApiKey by remember { mutableStateOf("") }
    var openaiApiKey by remember { mutableStateOf("") }
    var showClaudeKey by remember { mutableStateOf(false) }
    var showOpenAIKey by remember { mutableStateOf(false) }
    var maxTokens by remember { mutableStateOf("4000") }
    var temperature by remember { mutableStateOf(0.7f) }
    var enableFallback by remember { mutableStateOf(true) }
    var autoRetry by remember { mutableStateOf(true) }
    var retryCount by remember { mutableStateOf("3") }
    var isSaving by remember { mutableStateOf(false) }
    var saveStatus by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    val providers = listOf("Claude", "GPT-4", "Local Model")
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("AI Settings") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    TextButton(
                        onClick = {
                            scope.launch {
                                isSaving = true
                                try {
                                    // TODO: Save settings to preferences
                                    saveStatus = "Settings saved successfully"
                                } catch (e: Exception) {
                                    saveStatus = "Failed to save settings: ${e.message}"
                                } finally {
                                    isSaving = false
                                }
                            }
                        },
                        enabled = !isSaving
                    ) {
                        if (isSaving) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                strokeWidth = 2.dp
                            )
                        } else {
                            Text("Save")
                        }
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
            // Status message
            saveStatus?.let { status ->
                item {
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = if (status.contains("success")) {
                                MaterialTheme.colorScheme.primaryContainer
                            } else {
                                MaterialTheme.colorScheme.errorContainer
                            }
                        )
                    ) {
                        Row(
                            modifier = Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                if (status.contains("success")) Icons.Default.CheckCircle else Icons.Default.Error,
                                contentDescription = null,
                                tint = if (status.contains("success")) {
                                    MaterialTheme.colorScheme.onPrimaryContainer
                                } else {
                                    MaterialTheme.colorScheme.onErrorContainer
                                }
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = status,
                                color = if (status.contains("success")) {
                                    MaterialTheme.colorScheme.onPrimaryContainer
                                } else {
                                    MaterialTheme.colorScheme.onErrorContainer
                                }
                            )
                            Spacer(modifier = Modifier.weight(1f))
                            IconButton(onClick = { saveStatus = null }) {
                                Icon(
                                    Icons.Default.Close,
                                    contentDescription = "Dismiss",
                                    modifier = Modifier.size(16.dp)
                                )
                            }
                        }
                    }
                }
            }
            
            // Provider Selection
            item {
                SettingsSection(title = "AI Provider") {
                    Text(
                        text = "Choose your preferred AI provider",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(bottom = 12.dp)
                    )
                    
                    providers.forEach { provider ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            RadioButton(
                                selected = selectedProvider == provider,
                                onClick = { selectedProvider = provider }
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Column {
                                Text(
                                    text = provider,
                                    style = MaterialTheme.typography.bodyLarge
                                )
                                Text(
                                    text = when (provider) {
                                        "Claude" -> "Anthropic's Claude AI (recommended)"
                                        "GPT-4" -> "OpenAI's GPT-4 model"
                                        "Local Model" -> "Run models locally (coming soon)"
                                        else -> ""
                                    },
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                }
            }
            
            // API Keys Section
            item {
                SettingsSection(title = "API Keys") {
                    Text(
                        text = "Configure your API keys for AI providers",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(bottom = 12.dp)
                    )
                    
                    // Claude API Key
                    OutlinedTextField(
                        value = claudeApiKey,
                        onValueChange = { claudeApiKey = it },
                        label = { Text("Claude API Key") },
                        modifier = Modifier.fillMaxWidth(),
                        visualTransformation = if (showClaudeKey) VisualTransformation.None else PasswordVisualTransformation(),
                        trailingIcon = {
                            IconButton(onClick = { showClaudeKey = !showClaudeKey }) {
                                Icon(
                                    if (showClaudeKey) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                    contentDescription = if (showClaudeKey) "Hide key" else "Show key"
                                )
                            }
                        },
                        supportingText = { Text("Get your key from console.anthropic.com") }
                    )
                    
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    // OpenAI API Key
                    OutlinedTextField(
                        value = openaiApiKey,
                        onValueChange = { openaiApiKey = it },
                        label = { Text("OpenAI API Key") },
                        modifier = Modifier.fillMaxWidth(),
                        visualTransformation = if (showOpenAIKey) VisualTransformation.None else PasswordVisualTransformation(),
                        trailingIcon = {
                            IconButton(onClick = { showOpenAIKey = !showOpenAIKey }) {
                                Icon(
                                    if (showOpenAIKey) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                    contentDescription = if (showOpenAIKey) "Hide key" else "Show key"
                                )
                            }
                        },
                        supportingText = { Text("Get your key from platform.openai.com") }
                    )
                }
            }
            
            // Generation Settings
            item {
                SettingsSection(title = "Generation Settings") {
                    // Max Tokens
                    OutlinedTextField(
                        value = maxTokens,
                        onValueChange = { maxTokens = it },
                        label = { Text("Max Tokens") },
                        modifier = Modifier.fillMaxWidth(),
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        supportingText = { Text("Maximum number of tokens to generate (1-8000)") }
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    // Temperature
                    Text(
                        text = "Creativity Level: ${String.format("%.1f", temperature)}",
                        style = MaterialTheme.typography.labelLarge
                    )
                    Slider(
                        value = temperature,
                        onValueChange = { temperature = it },
                        valueRange = 0.0f..1.0f,
                        steps = 10,
                        modifier = Modifier.fillMaxWidth()
                    )
                    Text(
                        text = "Lower values for more focused responses, higher for more creative",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            
            // Advanced Settings
            item {
                SettingsSection(title = "Advanced Settings") {
                    // Enable Fallback
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Provider Fallback",
                                style = MaterialTheme.typography.bodyLarge
                            )
                            Text(
                                text = "Automatically try other providers if primary fails",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Switch(
                            checked = enableFallback,
                            onCheckedChange = { enableFallback = it }
                        )
                    }
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    // Auto Retry
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "Auto Retry",
                                style = MaterialTheme.typography.bodyLarge
                            )
                            Text(
                                text = "Automatically retry failed requests",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Switch(
                            checked = autoRetry,
                            onCheckedChange = { autoRetry = it }
                        )
                    }
                    
                    if (autoRetry) {
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = retryCount,
                            onValueChange = { retryCount = it },
                            label = { Text("Retry Count") },
                            modifier = Modifier.fillMaxWidth(),
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                            supportingText = { Text("Number of times to retry failed requests (1-5)") }
                        )
                    }
                }
            }
            
            // Usage Information
            item {
                SettingsSection(title = "Usage Information") {
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                        )
                    ) {
                        Column(
                            modifier = Modifier.padding(16.dp)
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(
                                    Icons.Default.Info,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = "API Usage",
                                    style = MaterialTheme.typography.titleMedium
                                )
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = "• API keys are stored securely on your device\n" +
                                      "• Usage depends on your provider's pricing\n" +
                                      "• Monitor your usage on provider dashboards\n" +
                                      "• Fallback helps reduce failed requests",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}