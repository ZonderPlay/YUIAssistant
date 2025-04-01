package com.yui.assistant.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.yui.assistant.voice.VoiceRecorder
import com.yui.assistant.network.RetrofitClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.asRequestBody

@Composable
fun VoiceButton() {
    val context = LocalContext.current
    val recorder = remember { VoiceRecorder(context) }
    var isRecording by remember { mutableStateOf(false) }

    Box(
        modifier = Modifier
            .size(120.dp)
            .clip(CircleShape)
            .background(if (isRecording) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
            .clickable {
                if (isRecording) {
                    recorder.stopRecording()
                    isRecording = false
                } else {
                    recorder.startRecording { file ->
                        println("🎤 Аудио записано: ${file.absolutePath}")

                        CoroutineScope(Dispatchers.IO).launch {
                            try {
                                val requestFile = file.asRequestBody("audio/wav".toMediaTypeOrNull())
                                val body = MultipartBody.Part.createFormData("audio", file.name, requestFile)

                                val response = RetrofitClient.api.uploadAudio(body)
                                if (response.isSuccessful) {
                                    println("✅ Успешно отправлено. Ответ: ${response.body()?.string()}")
                                } else {
                                    println("❌ Ошибка: ${response.code()} ${response.errorBody()?.string()}")
                                }
                            } catch (e: Exception) {
                                println("⚠️ Ошибка отправки: ${e.message}")
                            }
                        }
                    }
                    isRecording = true
                }
            },
        contentAlignment = Alignment.Center
    ) {
        Text("🎙️", fontSize = MaterialTheme.typography.headlineLarge.fontSize)
    }
}
