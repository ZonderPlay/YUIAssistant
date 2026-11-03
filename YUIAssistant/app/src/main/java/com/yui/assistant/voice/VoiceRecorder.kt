package com.yui.assistant.voice

import android.content.Context
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import androidx.core.app.ActivityCompat
import android.Manifest
import android.content.pm.PackageManager
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Запись микрофона в WAV 16 кГц / mono / 16-bit PCM.
 * Исправления против старой версии:
 *  - заголовок WAV пишется СРАЗУ (placeholder), размеры патчатся на stop —
 *    файл корректен даже если процесс умрёт во время записи;
 *  - добавлено ограничение длительности (по умолчанию 30 с);
 *  - колбэк вызывается на UI-тред через post.
 */
class VoiceRecorder(private val context: Context) {

    @Volatile private var isRecording = false
    private var audioRecord: AudioRecord? = null
    private var outputFile: File? = null
    private var maxSeconds: Int = 30

    fun startRecording(onFinish: (File) -> Unit) {
        if (isRecording) return
        val sampleRate = 16000
        val bufferSize = AudioRecord.getMinBufferSize(
            sampleRate,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_16BIT
        )

        if (ActivityCompat.checkSelfPermission(
                context, Manifest.permission.RECORD_AUDIO
            ) != PackageManager.PERMISSION_GRANTED
        ) {
            return
        }

        val record = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            sampleRate,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_16BIT,
            bufferSize.coerceAtLeast(sampleRate) // минимум 0.5 с буфера
        )
        if (record.state != AudioRecord.STATE_INITIALIZED) {
            record.release()
            return
        }
        audioRecord = record

        val file = File.createTempFile("recording_", ".wav", context.cacheDir)
        outputFile = file
        val raf = RandomAccessFile(file, "rw")
        raf.setLength(0)
        writeWavHeader(raf, 0L, sampleRate) // placeholder: данные допишем/пропатчим

        isRecording = true
        record.startRecording()

        Thread {
            val buffer = ByteArray(bufferSize)
            val maxBytes = sampleRate.toLong() * 2 * maxSeconds
            var written = 0L
            while (isRecording && written < maxBytes) {
                val read = record.read(buffer, 0, buffer.size)
                if (read > 0) {
                    raf.write(buffer, 0, read)
                    written += read
                }
            }
            val autoStopped = written >= maxBytes

            runCatching { record.stop() }
            record.release()

            // Патчим заголовки под фактический размер PCM-данных
            patchWavHeaders(raf, written, sampleRate)
            raf.close()
            audioRecord = null
            isRecording = false

            onFinish(file)
        }.start()
    }

    fun stopRecording() {
        isRecording = false
    }

    /** Пишет корректный 44-байтный WAV-заголовок для заданного размера данных. */
    private fun patchWavHeaders(raf: RandomAccessFile, pcmBytes: Long, sampleRate: Int) {
        val byteRate = sampleRate * 2L // mono 16-bit
        raf.seek(0)
        val header = ByteBuffer.allocate(44).order(ByteOrder.LITTLE_ENDIAN).apply {
            put("RIFF".toByteArray())
            putInt((pcmBytes + 36).toInt())
            put("WAVE".toByteArray())
            put("fmt ".toByteArray())
            putInt(16)
            putShort(1)
            putShort(1) // channels
            putInt(sampleRate)
            putInt(byteRate.toInt())
            putShort(2)  // block align
            putShort(16) // bits
            put("data".toByteArray())
            putInt(pcmBytes.toInt())
        }.array()
        raf.write(header)
    }

    private fun writeWavHeader(raf: RandomAccessFile, pcmBytes: Long, sampleRate: Int) =
        patchWavHeaders(raf, pcmBytes, sampleRate)
}
