package com.writemagic.ui.screens

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

data class GitCommit(
    val id: String,
    val message: String,
    val author: String,
    val timestamp: String,
    val branch: String,
    val filesChanged: Int
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TimelineScreen() {
    val commits = remember {
        listOf(
            GitCommit("abc123", "Initial draft of chapter 1", "You", "2 hours ago", "main", 1),
            GitCommit("def456", "Add character descriptions", "You", "4 hours ago", "characters", 3),
            GitCommit("ghi789", "Revise opening paragraph", "You", "1 day ago", "main", 1),
            GitCommit("jkl012", "Experiment with different ending", "You", "2 days ago", "alternative-ending", 2),
            GitCommit("mno345", "Add dialogue scenes", "You", "3 days ago", "main", 4)
        )
    }
    
    var selectedBranch by remember { mutableStateOf("all") }
    val branches = commits.map { it.branch }.distinct() + "all"
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        // Header
        Text(
            "Writing Timeline",
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier.padding(bottom = 16.dp)
        )
        
        // Branch filter
        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    "Filter by branch:",
                    style = MaterialTheme.typography.labelMedium,
                    modifier = Modifier.padding(bottom = 8.dp)
                )
                
                LazyRow(
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(branches) { branch ->
                        FilterChip(
                            onClick = { selectedBranch = branch },
                            label = { Text(branch) },
                            selected = selectedBranch == branch,
                            leadingIcon = if (branch != "all") {
                                { Icon(Icons.Default.AccountTree, contentDescription = null, modifier = Modifier.size(16.dp)) }
                            } else null
                        )
                    }
                }
            }
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        // Timeline visualization
        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
        ) {
            TimelineVisualization(
                commits = commits.filter { selectedBranch == "all" || it.branch == selectedBranch },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp)
                    .padding(16.dp)
            )
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        // Commits list
        LazyColumn(
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(commits.filter { selectedBranch == "all" || it.branch == selectedBranch }) { commit ->
                CommitCard(commit)
            }
        }
    }
}

@Composable
fun TimelineVisualization(
    commits: List<GitCommit>,
    modifier: Modifier = Modifier
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val branchColors = mapOf(
        "main" to primaryColor,
        "characters" to Color(0xFF4CAF50), // Green
        "alternative-ending" to Color(0xFFFF9800) // Orange
    )
    
    Canvas(modifier = modifier) {
        val width = size.width
        val height = size.height
        val centerY = height / 2
        
        // Draw main timeline
        drawLine(
            color = primaryColor,
            start = Offset(50f, centerY),
            end = Offset(width - 50f, centerY),
            strokeWidth = 4.dp.toPx()
        )
        
        // Draw commit points
        commits.forEachIndexed { index, commit ->
            val x = 50f + (index * (width - 100f) / maxOf(1, commits.size - 1))
            val color = branchColors[commit.branch] ?: primaryColor
            
            // Draw branch line if not main
            if (commit.branch != "main") {
                val branchY = centerY + (if (commit.branch.hashCode() % 2 == 0) -40f else 40f)
                drawLine(
                    color = color,
                    start = Offset(x, centerY),
                    end = Offset(x, branchY),
                    strokeWidth = 2.dp.toPx()
                )
                
                // Draw commit point on branch
                drawCircle(
                    color = color,
                    radius = 8.dp.toPx(),
                    center = Offset(x, branchY)
                )
            } else {
                // Draw commit point on main line
                drawCircle(
                    color = color,
                    radius = 8.dp.toPx(),
                    center = Offset(x, centerY)
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CommitCard(commit: GitCommit) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = commit.message,
                        style = MaterialTheme.typography.titleMedium
                    )
                    
                    Spacer(modifier = Modifier.height(4.dp))
                    
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                Icons.Default.AccountTree,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp)
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Text(
                                commit.branch,
                                style = MaterialTheme.typography.bodySmall
                            )
                        }
                        
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                Icons.Default.Edit,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp)
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Text(
                                "${commit.filesChanged} files",
                                style = MaterialTheme.typography.bodySmall
                            )
                        }
                    }
                }
                
                Column(horizontalAlignment = Alignment.End) {
                    Text(
                        commit.timestamp,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        commit.id.take(7),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}