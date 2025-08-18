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
import com.writemagic.ui.theme.WriteMagicTheme
import com.writemagic.ui.WriteMagicApp

class MainActivity : ComponentActivity() {
    
    companion object {
        init {
            try {
                // Load native Rust library
                System.loadLibrary("writemagic_android")
            } catch (e: UnsatisfiedLinkError) {
                // Handle gracefully for development
                println("Native library not found: ${e.message}")
            }
        }
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        
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
    
    // JNI method declarations for Rust FFI
    external fun initializeCore(): Boolean
    external fun createDocument(title: String): String
    external fun saveDocument(documentId: String, content: String): Boolean
    external fun loadDocument(documentId: String): String
    external fun processAIRequest(prompt: String, context: String): String
}