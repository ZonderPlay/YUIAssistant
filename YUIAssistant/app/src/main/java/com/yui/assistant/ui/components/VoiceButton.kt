package com.yui.assistant.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.yui.assistant.ui.AssistantPhase
import com.yui.assistant.voice.VoiceRecorder
import java.io.File

/**
 * Кнопка записи. Состояния: idle (🎙️) → recording (⏹, красный) →
 * sending (спиннер) → speaking (🔊).
 */
@Composable
fun VoiceButton(
    phase: AssistantPhase,
    onToggleRecord: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val busy = phase == AssistantPhase.Sending || phase == AssistantPhase.Speaking

    Box(
        modifier = modifier
            .size(120.dp)
            .clip(CircleShape)
            .background(
                when (phase) {
                    AssistantPhase.Recording -> MaterialTheme.colorScheme.error
                    AssistantPhase.Sending, AssistantPhase.Speaking ->
                        MaterialTheme.colorScheme.primary.copy(alpha = 0.4f)
                    else -> MaterialTheme.colorScheme.primary
                }
            )
            .clickable(enabled = !busy) { onToggleRecord() },
        contentAlignment = Alignment.Center,
    ) {
        when (phase) {
            AssistantPhase.Idle -> Text("🎙️", fontSize = MaterialTheme.typography.headlineLarge.fontSize)
            AssistantPhase.Recording -> Text("⏹️", fontSize = MaterialTheme.typography.headlineLarge.fontSize)
            AssistantPhase.Sending -> CircularProgressIndicator(color = MaterialTheme.colorScheme.onPrimary)
            AssistantPhase.Speaking -> Text("🔊", fontSize = MaterialTheme.typography.headlineLarge.fontSize)
        }
    }
}
