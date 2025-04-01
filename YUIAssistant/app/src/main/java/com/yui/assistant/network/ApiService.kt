package com.yui.assistant.network

import okhttp3.MultipartBody
import okhttp3.ResponseBody
import retrofit2.Response
import retrofit2.http.Multipart
import retrofit2.http.POST
import retrofit2.http.Part

interface ApiService {
    @Multipart
    @POST("api/audio") // настроишь путь под свой сервер
    suspend fun uploadAudio(
        @Part audio: MultipartBody.Part
    ): Response<ResponseBody>
}
