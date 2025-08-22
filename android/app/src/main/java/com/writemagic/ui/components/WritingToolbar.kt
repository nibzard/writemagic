package com.writemagic.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun WritingToolbar(
    isDistractionFree: Boolean,
    onDistractionFreeToggle: () -> Unit,
    onFormatText: (FormatAction) -> Unit,
    onAIAssist: () -> Unit,
    onSave: () -> Unit,
    modifier: Modifier = Modifier
) {
    if (!isDistractionFree) {
        Surface(
            modifier = modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.surface,
            shadowElevation = 4.dp,
            shape = RoundedCornerShape(bottomStart = 12.dp, bottomEnd = 12.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Formatting tools
                Row(
                    horizontalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    FormatButton(
                        icon = Icons.Default.FormatBold,
                        description = "Bold",
                        onClick = { onFormatText(FormatAction.Bold) }
                    )
                    FormatButton(
                        icon = Icons.Default.FormatItalic,
                        description = "Italic",
                        onClick = { onFormatText(FormatAction.Italic) }
                    )
                    FormatButton(
                        icon = Icons.Default.FormatListBulleted,
                        description = "Bullet List",
                        onClick = { onFormatText(FormatAction.BulletList) }
                    )
                    FormatButton(
                        icon = Icons.Default.FormatListNumbered,
                        description = "Numbered List",
                        onClick = { onFormatText(FormatAction.NumberedList) }
                    )
                }
                
                // Action tools
                Row(
                    horizontalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    IconButton(onClick = onAIAssist) {
                        Icon(
                            Icons.Default.SmartToy,
                            contentDescription = "AI Assistant",
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                    
                    IconButton(onClick = onSave) {
                        Icon(
                            Icons.Default.Save,
                            contentDescription = "Save",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    
                    IconButton(onClick = onDistractionFreeToggle) {
                        Icon(
                            Icons.Default.FullscreenExit,
                            contentDescription = "Distraction-free mode",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun FormatButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    description: String,
    onClick: () -> Unit
) {
    IconButton(
        onClick = onClick,
        modifier = Modifier.size(40.dp)
    ) {
        Icon(
            icon,
            contentDescription = description,
            modifier = Modifier.size(20.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

enum class FormatAction {
    Bold,
    Italic,
    BulletList,
    NumberedList,
    Heading1,
    Heading2,
    Quote
}

@Composable
fun FloatingWritingToolbar(
    visible: Boolean,
    onDismiss: () -> Unit,
    onFormatText: (FormatAction) -> Unit,
    modifier: Modifier = Modifier
) {
    if (visible) {
        Card(
            modifier = modifier,
            elevation = CardDefaults.cardElevation(defaultElevation = 8.dp),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceVariant
            )
        ) {
            Row(
                modifier = Modifier.padding(8.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                FormatButton(
                    icon = Icons.Default.FormatBold,
                    description = "Bold",
                    onClick = { 
                        onFormatText(FormatAction.Bold)
                        onDismiss()
                    }
                )
                FormatButton(
                    icon = Icons.Default.FormatItalic,
                    description = "Italic",
                    onClick = { 
                        onFormatText(FormatAction.Italic)
                        onDismiss()
                    }
                )
                FormatButton(
                    icon = Icons.Default.FormatQuote,
                    description = "Quote",
                    onClick = { 
                        onFormatText(FormatAction.Quote)
                        onDismiss()
                    }
                )
                
                VerticalDivider(
                    modifier = Modifier
                        .height(32.dp)
                        .width(1.dp)
                )
                
                IconButton(onClick = onDismiss) {
                    Icon(
                        Icons.Default.Close,
                        contentDescription = "Close",
                        modifier = Modifier.size(16.dp)
                    )
                }
            }
        }
    }
}