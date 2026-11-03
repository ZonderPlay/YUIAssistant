package com.yui.assistant

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material3.Scaffold
import androidx.core.content.ContextCompat
import com.yui.assistant.ui.MainScreen
import com.yui.assistant.ui.theme.YUIAssistantTheme

class MainActivity : ComponentActivity() {

    private val requestMicPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { /* UI сам отразит статус */ }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        if (ContextCompat.checkSelfPermission(this, Manifest.permission.RECORD_AUDIO)
            != PackageManager.PERMISSION_GRANTED
        ) {
            requestMicPermission.launch(Manifest.permission.RECORD_AUDIO)
        }

        setContent {
            YUIAssistantTheme {
                Scaffold { contentPadding ->
                    MainScreen(paddingValues = contentPadding)
                }
            }
        }
    }
}
