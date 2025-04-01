use axum::{extract::Multipart, response::Json, http::StatusCode};
use serde::Serialize;
use std::{fs, fs::File, io::Write, path::Path, sync::Arc};
use uuid::Uuid;
use vosk::Model;
use crate::services::deepseek;

#[derive(Serialize)]
pub struct ApiResponse {
    r#type: String,
    text: String,
}

pub async fn handle_audio(
    mut multipart: Multipart,
    model: Arc<Model>,
) -> Result<Json<ApiResponse>, StatusCode> {
    // Создаём папку storage, если её нет
    let storage_path = Path::new("./storage");
    if !storage_path.exists() {
        if let Err(e) = fs::create_dir_all(storage_path) {
            eprintln!("❌ Ошибка создания папки storage: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        println!("📂 Папка storage создана!");
    }

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field.file_name().unwrap_or("input.wav").to_string();
        let data = field.bytes().await.unwrap();

        let id = Uuid::new_v4().to_string();
        let file_path = format!("./storage/{}_{}", id, file_name);

        if let Ok(mut file) = File::create(&file_path) {
            if file.write_all(&data).is_err() {
                eprintln!("❌ Ошибка записи файла: {}", file_path);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            println!("💾 Файл сохранён: {}", file_path);
        } else {
            eprintln!("❌ Не удалось создать файл: {}", file_path);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        match crate::services::stt::transcribe_audio(&model, &file_path) {
            Ok(text) => {
                println!("🧠 Распознано: {}", text);
                // Отправляем в DeepSeek
                match deepseek::send_to_deepseek(&text).await {
                    Ok(ai_response) => {
                        println!("🤖 DeepSeek ответил: {}", ai_response);
                        return Ok(Json(ApiResponse {
                            r#type: "text".into(),
                            text: ai_response,
                        }));
                    }
                    Err(e) => {
                        eprintln!("⚠️ Ошибка запроса к DeepSeek: {}", e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Ошибка распознавания: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}
