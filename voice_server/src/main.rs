mod routes;
mod services;
use axum::{routing::post, Router, extract::Multipart};
use routes::audio::handle_audio;
use std::{net::SocketAddr, sync::Arc, process::Command, thread, time::Duration};
use tokio;
use vosk::Model;
use reqwest::Client;
use serde_json::json;

/// Проверяет, запущен ли Ollama
async fn is_ollama_running() -> bool {
    let client = Client::new();
    let url = "http://localhost:11434/api/generate";

    let body = json!({
        "model": "deepseek-r1:1.5b",
        "prompt": "Привет!",
        "stream": false
    });

    client.post(url).json(&body).send().await.is_ok()
}

/// Запускает Ollama, если он не работает
fn start_ollama() {
    if !std::path::Path::new("AI/").exists() {
        panic!("❌ Ошибка: Папка AI/ollama не найдена!");
    }

    println!("🚀 Запуск Ollama...");
    let mut cmd = Command::new("AI/ollama")
        .arg("serve")
        .spawn()
        .expect("❌ Ошибка запуска Ollama!");

    thread::sleep(Duration::from_secs(2)); // Ждём пару секунд для запуска
    println!("✅ Ollama запущен!");

    // Опционально: убить процесс при завершении
    std::thread::spawn(move || {
        cmd.wait().expect("❌ Ollama неожиданно завершился!");
    });
}

#[tokio::main]
async fn main() {
    println!("⏳ Проверка Ollama...");

    if !is_ollama_running().await {
        start_ollama();
        tokio::time::sleep(Duration::from_secs(5)).await; // Подождём запуск
    } else {
        println!("✅ Ollama уже работает!");
    }

    println!("⏳ Загрузка модели VOSK...");
    let model = Arc::new(Model::new("models").expect("❌ Ошибка инициализации модели VOSK"));
    println!("✅ Модель VOSK загружена успешно!");

    // Используем async move в качестве обработчика маршрута
    let app = Router::new().route("/api/audio", post({
        let model = model.clone();
        move |multipart: Multipart| {
            let model = model.clone();
            async move {
                handle_audio(multipart, model).await
            }
        }
    }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Сервер запущен: http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
