package com.yui.assistant

import android.annotation.SuppressLint
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.Scaffold
import com.yui.assistant.ui.MainScreen
import com.yui.assistant.ui.theme.YUIAssistantTheme
import com.yui.assistant.utils.PermissionUtils

class MainActivity : ComponentActivity() {

    @SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        if (!PermissionUtils.hasRecordPermission(this)) {
            PermissionUtils.requestRecordPermission(this, 100)
        }

        setContent {
            YUIAssistantTheme {
                Scaffold {
                    MainScreen()
                }
            }
        }
    }
}
