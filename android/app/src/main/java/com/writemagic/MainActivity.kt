package com.writemagic

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch
import com.writemagic.ui.theme.WriteMagicTheme
import com.writemagic.ui.WriteMagicApp
import com.writemagic.core.WriteMagicCore

class MainActivity : ComponentActivity() {
    
    companion object {
        // Note: Library loading is now handled in WriteMagicCore
        // to provide better error handling and fallback behavior
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        
        // Initialize WriteMagic core with persistent SQLite
        lifecycleScope.launch {
            val initialized = WriteMagicCore.initialize()
            if (initialized) {
                println("WriteMagic core initialized successfully")
            } else {
                println("Failed to initialize WriteMagic core")
            }
        }
        
        setContent {
            WriteMagicTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    WriteMagicApp()
                }
            }
        }
    }
}