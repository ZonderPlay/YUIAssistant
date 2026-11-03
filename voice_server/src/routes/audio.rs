//! POST /api/audio — основной эндпоинт MVP:
//! WAV от клиента → VOSK STT → LLM (Ollama) → типизированный JSON-ответ.

use crate::model::response::ApiResponse;
use crate::services::llm::LlmClient;
use crate::utils::file;
use axum::{extract::Multipart, response::Json, http::StatusCode};
use std::{path::Path, sync::Arc};
use crate::stt_engine::SttEngine;

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024; // ~5 минут 16 кГц mono 16-bit

pub async fn handle_audio(
    mut multipart: Multipart,
    stt: Arc<dyn SttEngine>,
    llm: Arc<LlmClient>,
) -> Result<(StatusCode, Json<ApiResponse>), StatusCode> {
    let storage = Path::new("./storage");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field.file_name().unwrap_or("input.wav").to_string();
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("❌ Не удалось прочитать поле multipart: {e}");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        if data.len() > MAX_UPLOAD_BYTES {
            eprintln!("❌ Слишком большой файл: {} байт", data.len());
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        if data.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let file_path = match file::save_to_storage(storage, &file_name, &data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("❌ Ошибка сохранения файла: {e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        println!("💾 Файл сохранён: {file_path}");

        // STT — блокирующая нативная операция, выносим в blocking-пул
        let stt_clone = stt.clone();
        let path_for_stt = file_path.clone();
        let transcript = tokio::task::spawn_blocking(move || {
            stt_clone.transcribe_wav(&path_for_stt)
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let transcript = match transcript {
            Ok(t) => t,
            Err(e) => {
                eprintln!("⚠️ Ошибка распознавания ({file_path}): {e}");
                return Ok((
                    StatusCode::OK,
                    Json(ApiResponse::error("Не удалось распознать речь. Попробуйте ещё раз.")),
                ));
            }
        };

        if transcript.trim().is_empty() {
            return Ok((
                StatusCode::OK,
                Json(ApiResponse::error(
                    "Я вас не услышала — тишина в записи.",
                )),
            ));
        }

        println!("🧠 Распознано: {transcript}");

        // Быстрые локальные команды (приветствие/помощь/стоп) не гоняем в LLM —
        // отвечаем сразу, это экономит секунды на слабом железе.
        let intent = crate::services::parser::parse_intent(&transcript);
        if let Some(local) = crate::services::parser::reply(&intent) {
            println!("⚡ Локальный интент: {intent:?}");
            return Ok((StatusCode::OK, Json(ApiResponse::text(local, transcript))));
        }

        match llm.generate(&transcript).await {
            Ok(answer) => {
                println!("🤖 Ответ: {answer}");
                return Ok((StatusCode::OK, Json(ApiResponse::text(answer, transcript))));
            }
            Err(e) => {
                eprintln!("⚠️ Ошибка LLM: {e}");
                return Ok((
                    StatusCode::OK,
                    Json(ApiResponse::error(format!(
                        "Модель недоступна: {e}"
                    ))),
                ));
            }
        }
    }

    // Ни одного файла в запросе
    Err(StatusCode::BAD_REQUEST)
}
