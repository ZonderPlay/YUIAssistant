//! voice_server — MVP голосового ассистента.
//! Цепочка: WAV (multipart POST) → VOSK STT → Ollama LLM → JSON-ответ.

mod config;
mod stt_engine;
mod model;
mod routes;
mod services;
mod utils;

use crate::config::Config;
use crate::services::llm::LlmClient;
use axum::{
    extract::Multipart,
    http::{header, Method},
    routing::{get, post},
    Json, Router,
};
use std::{process::Command, sync::Arc, time::Duration};
use tokio::net::TcpListener;

/// Запускает локальный ollama, если внешняя проверка не прошла.
fn start_ollama(cfg: &Config) {
    println!("🚀 Запуск Ollama ({})...", cfg.ollama_bin);
    match Command::new(&cfg.ollama_bin).arg("serve").spawn() {
        Ok(mut child) => {
            // Убиваем дочерний процесс вместе с сервером
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => eprintln!("⚠️ Не удалось запустить Ollama автоматически: {e}"),
    }
}

#[tokio::main]
async fn main() {
    let cfg = Config::default();

    let llm = Arc::new(LlmClient::new(&cfg));

    println!("⏳ Проверка Ollama ({})...", cfg.ollama_url);
    if llm.is_available().await {
        println!("✅ Ollama уже работает!");
    } else {
        if cfg.autostart_ollama {
            start_ollama(&cfg);
            // ждём готовности до 30 секунд
            for _ in 0..30 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if llm.is_available().await {
                    break;
                }
            }
        }
        if llm.is_available().await {
            println!("✅ Ollama запущен!");
        } else {
            eprintln!(
                "⚠️ Ollama недоступен по {}. Сервер стартует без LLM — \
                 ответы будут ошибочными, пока не поднимете `ollama serve`.",
                cfg.ollama_url
            );
        }
    }

    println!("⏳ Инициализация STT-движка (модель: {})...", cfg.vosk_model_dir);
    let stt: Arc<dyn stt_engine::SttEngine> =
        Arc::from(stt_engine::create_engine(&cfg.vosk_model_dir).expect("❌ Ошибка инициализации STT"));
    println!("✅ STT-движок готов!");

    // CORS — чтобы тестовать с веб-клиента/эмулятора
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![header::CONTENT_TYPE]);

    let app = Router::new()
        .route(
            "/api/audio",
            post({
                let stt = stt.clone();
                let llm = llm.clone();
                move |mp: Multipart| {
                    let stt = stt.clone();
                    let llm = llm.clone();
                    async move { routes::audio::handle_audio(mp, stt, llm).await }
                }
            }),
        )
        .route("/health", get(health))
        .layer(cors);

    let addr = cfg.bind_addr.clone();
    println!("🚀 Сервер запущен: http://{addr}");

    // Фоновая чистка storage от старых записей (раз в 10 минут хранится сутки)
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(600)).await;
            utils::file::cleanup_old_files(std::path::Path::new("./storage"), 86_400);
        }
    });

    let listener = TcpListener::bind(&addr).await.expect("не удалось занять порт");
    axum::serve(listener, app).await.expect("ошибка сервера");
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
