package com.yui.assistant.voice

/**
 * Детектор приветственного слова («Юя», «Юи»…).
 * В MVP выключен: требует офлайн-модели KWS/Porcupine и фонового сервиса.
 * Интерфейс оставлен, чтобы подключить его позже без переделки UI:
 *   WakeWordDetector.start { onWake() } — при обнаружении активировать микрофон.
 */
class WakeWordDetector {
    fun start(onWake: () -> Unit) = Unit // TODO (post-MVP): Porcupine/KWS
    fun stop() = Unit
}
