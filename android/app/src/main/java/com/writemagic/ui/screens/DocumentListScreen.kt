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
import kotlinx.coroutines.launch
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import com.writemagic.core.WriteMagicCore
import com.writemagic.core.Document
import com.writemagic.ui.components.DocumentCard
import com.writemagic.ui.components.EmptyDocumentsCard

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DocumentListScreen(
    onNavigateToDocument: (String) -> Unit,
    onCreateDocument: () -> Unit
) {
    var documents by remember { mutableStateOf<List<Document>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var searchQuery by remember { mutableStateOf("") }
    var showSearch by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    
    // Load documents when screen is displayed
    LaunchedEffect(Unit) {
        loadDocuments { docs, error ->
            documents = docs
            errorMessage = error
            isLoading = false
        }
    }
    
    // Filter documents based on search
    val filteredDocuments = remember(documents, searchQuery) {
        if (searchQuery.isBlank()) {
            documents
        } else {
            documents.filter { doc ->
                doc.title.contains(searchQuery, ignoreCase = true) ||
                doc.content.contains(searchQuery, ignoreCase = true)
            }
        }
    }
    
    Column(
        modifier = Modifier.fillMaxSize()
    ) {
        // Top bar with search
        TopAppBar(
            title = { 
                if (showSearch) {
                    OutlinedTextField(
                        value = searchQuery,
                        onValueChange = { searchQuery = it },
                        placeholder = { Text("Search documents...") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true
                    )
                } else {
                    Text("Documents")
                }
            },
            actions = {
                if (showSearch) {
                    IconButton(
                        onClick = { 
                            showSearch = false
                            searchQuery = ""
                        }
                    ) {
                        Icon(Icons.Default.Close, contentDescription = "Close search")
                    }
                } else {
                    IconButton(onClick = { showSearch = true }) {
                        Icon(Icons.Default.Search, contentDescription = "Search")
                    }
                }
                
                IconButton(onClick = onCreateDocument) {
                    Icon(Icons.Default.Add, contentDescription = "Create document")
                }
            }
        )
        
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
                }
            }
        }
        
        // Content
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 16.dp)
        ) {
            when {
                isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                
                filteredDocuments.isEmpty() && searchQuery.isNotBlank() -> {
                    // No search results
                    Column(
                        modifier = Modifier.align(Alignment.Center),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            Icons.Default.SearchOff,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = "No documents found",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            text = "Try a different search term",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                        )
                    }
                }
                
                documents.isEmpty() -> {
                    // No documents at all
                    EmptyDocumentsCard(
                        onCreateDocument = onCreateDocument,
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                
                else -> {
                    // Document list
                    LazyColumn(
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                        contentPadding = PaddingValues(vertical = 16.dp)
                    ) {
                        items(filteredDocuments) { document ->
                            DocumentCard(
                                document = document,
                                onClick = { onNavigateToDocument(document.id) },
                                onEdit = { onNavigateToDocument(document.id) },
                                onDelete = { 
                                    scope.launch {
                                        // TODO: Implement delete functionality
                                        // For now, just refresh the list
                                        loadDocuments { docs, error ->
                                            documents = docs
                                            errorMessage = error
                                        }
                                    }
                                }
                            )
                        }
                    }
                }
            }
        }
    }
}

private suspend fun loadDocuments(onResult: (List<Document>, String?) -> Unit) {
    try {
        val result = WriteMagicCore.listDocuments(0, 50)
        if (result != null) {
            onResult(result.documents, null)
        } else {
            onResult(emptyList(), "Failed to load documents")
        }
    } catch (e: Exception) {
        onResult(emptyList(), "Error loading documents: ${e.message}")
    }
}