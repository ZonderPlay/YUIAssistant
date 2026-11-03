package com.yui.assistant.ui

import android.app.Application
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.yui.assistant.model.ServerResponse
import com.yui.assistant.network.RetrofitClient
import com.yui.assistant.voice.VoiceRecorder
import java.io.File
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.asRequestBody

enum class AssistantPhase { Idle, Recording, Sending, Speaking }

data class ChatMessage(val fromUser: Boolean, val text: String)

data class UiState(
    val phase: AssistantPhase = AssistantPhase.Idle,
    val messages: List<ChatMessage> = emptyList(),
)

/**
 * Логика экрана: запись → отправка на сервер → показ ответа → озвучка (TTS).
 */
class MainViewModel(app: Application) : AndroidViewModel(app), TextToSpeech.OnInitListener {

    private val recorder = VoiceRecorder(app)

    private val tts = TextToSpeech(app, this)
    private var ttsReady = false

    private val _state = MutableStateFlow(UiState())
    val state: StateFlow<UiState> = _state

    override fun onInit(status: Int) {
        if (status == TextToSpeech.SUCCESS) {
            // Русский язык может отсутствовать в движке TTS — не падаем, а
            // просто оставляем дефолтный; speak() всё равно произнесёт текст.
            runCatching { tts.setLanguage(Locale("ru")) }
            tts.setOnUtteranceProgressListener(object : UtteranceProgressListener() {
                override fun onStart(utteranceId: String?) {}
                override fun onDone(utteranceId: String?) = onSpeechFinished()
                @Deprecated("Обязателен для API < 21", level = DeprecationLevel.WARNING)
                override fun onError(utteranceId: String?) = onSpeechFinished()
            })
            ttsReady = true
        }
    }

    /** Клик по микрофонной кнопке: старт/стоп записи. */
    fun onMicClick() {
        when (_state.value.phase) {
            AssistantPhase.Idle -> {
                _state.value = _state.value.copy(phase = AssistantPhase.Recording)
                recorder.startRecording { file ->
                    sendAudio(file) { text -> speak(text) }
                }
            }
            AssistantPhase.Recording -> recorder.stopRecording()
            else -> Unit
        }
    }

    /** Отправляет WAV на сервер и кладёт ответ в чат. */
    fun sendAudio(file: File, onResult: (String?) -> Unit) {
        _state.value = _state.value.copy(phase = AssistantPhase.Sending)
        viewModelScope.launch {
            val result: String? = try {
                val requestFile = file.asRequestBody("audio/wav".toMediaTypeOrNull())
                val body = MultipartBody.Part.createFormData("audio", file.name, requestFile)
                val response = RetrofitClient.api.uploadAudio(body)
                if (response.isSuccessful) {
                    val server: ServerResponse? = response.body()
                    when {
                        server == null -> "Сервер вернул пустой ответ"
                        server.isError -> "⚠️ ${server.text}"
                        else -> {
                            addMessage(ChatMessage(fromUser = true, text = server.transcript ?: ""))
                            server.text
                        }
                    }
                } else {
                    "Ошибка сервера: ${response.code()}"
                }
            } catch (e: Exception) {
                "Не удалось связаться с сервером: ${e.message}"
            } finally {
                file.delete()
            }

            if (result != null) {
                _state.value = _state.value.copy(
                    phase = AssistantPhase.Speaking,
                    messages = _state.value.messages + ChatMessage(false, result),
                )
            } else {
                _state.value = _state.value.copy(phase = AssistantPhase.Idle)
            }
            onResult(result)
        }
    }

    private fun speak(text: String?) {
        if (text == null || !ttsReady) return
        runCatching { tts.speak(text, TextToSpeech.QUEUE_FLUSH, null, "answer") }
            .onFailure { onSpeechFinished() }
    }

    fun onSpeechFinished() {
        if (_state.value.phase == AssistantPhase.Speaking) {
            _state.value = _state.value.copy(phase = AssistantPhase.Idle)
        }
    }

    private fun addMessage(msg: ChatMessage) {
        if (msg.text.isBlank()) return
        _state.value = _state.value.copy(messages = _state.value.messages + msg)
    }

    override fun onCleared() {
        super.onCleared()
        recorder.stopRecording()
        tts.shutdown()
    }
}
