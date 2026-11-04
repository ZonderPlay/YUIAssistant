package com.yui.assistant.network

import com.yui.assistant.BuildConfig
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory

object RetrofitClient {
    /**
     * Адрес сервера берётся из BuildConfig (gradle.properties: assistant.serverUrl),
     * а не захардкожен. Для эмулятора хост-машина доступна как 10.0.2.2.
     */
    private val BASE_URL: String =
        if (BuildConfig.DEBUG && BuildConfig.SERVER_URL.isBlank()) "http://10.0.2.2:8080/"
        else normalize(BuildConfig.SERVER_URL)

    private fun normalize(url: String): String =
        if (url.endsWith("/")) url else "$url/"

    private val logger = HttpLoggingInterceptor().apply {
        level = HttpLoggingInterceptor.Level.BODY
    }

    private val client = OkHttpClient.Builder()
        .addInterceptor(logger)
        .connectTimeout(15, java.util.concurrent.TimeUnit.SECONDS)
        .readTimeout(120, java.util.concurrent.TimeUnit.SECONDS) // LLM может думать долго
        .build()

    val api: ApiService by lazy {
        Retrofit.Builder()
            .baseUrl(BASE_URL)
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(ApiService::class.java)
    }
}
