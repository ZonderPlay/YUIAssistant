package com.yui.assistant.model

import com.google.gson.annotations.SerializedName

/**
 * Ответ сервера voice_server (см. src/model/response.rs).
 * type: "text" | "error"
 */
data class ServerResponse(
    @SerializedName("type") val type: String,
    @SerializedName("text") val text: String,
    @SerializedName("transcript") val transcript: String? = null,
) {
    val isError: Boolean get() = type.equals("error", ignoreCase = true)
}
