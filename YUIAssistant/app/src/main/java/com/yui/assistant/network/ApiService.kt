package com.yui.assistant.network

import com.yui.assistant.model.ServerResponse
import okhttp3.MultipartBody
import retrofit2.Response
import retrofit2.http.Multipart
import retrofit2.http.POST
import retrofit2.http.Part

interface ApiService {
    @Multipart
    @POST("api/audio")
    suspend fun uploadAudio(
        @Part audio: MultipartBody.Part,
    ): Response<ServerResponse>

    @retrofit2.http.GET("health")
    suspend fun health(): Response<Map<String, String>>
}
