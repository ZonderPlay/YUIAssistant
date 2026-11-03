//! Универсальный клиент LLM поверх Ollama API (`/api/generate`).
//! Заменяет собой жёстко привязанный к DeepSeek модуль: модель, URL и
//! системный промпт берутся из Config.

use crate::config::Config;
use reqwest::Client;
use serde_json::json;

pub struct LlmClient {
    client: Client,
    url: String,
    model: String,
    system_prompt: String,
}

impl LlmClient {
    pub fn new(cfg: &Config) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("не удалось создать HTTP-клиент"),
            url: format!("{}/api/generate", cfg.ollama_url.trim_end_matches('/')),
            model: cfg.ollama_model.clone(),
            system_prompt: cfg.system_prompt.clone(),
        }
    }

    /// Дешёвая проверка доступности Ollama (GET /api/tags вместо генерации).
    pub async fn is_available(&self) -> bool {
        let tags_url = self.url.replace("/api/generate", "/api/tags");
        self.client
            .get(&tags_url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Отправляет пользовательский промпт, возвращает текст ответа модели.
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        let body = json!({
            "model": self.model,
            "system": self.system_prompt,
            "prompt": prompt,
            "stream": false,
        });

        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Не удалось подключиться к Ollama: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Ошибка чтения ответа Ollama: {e}"))?;

        if !status.is_success() {
            return Err(format!("Ollama вернул ошибку {status}: {text}"));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Ошибка парсинга JSON: {e}. Ответ: {text}"))?;

        let answer = parsed
            .get("response")
            .and_then(|r| r.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or("Ответ модели пустой")?;

        Ok(clean_llm_answer(answer))
    }
}

/// У моделей family deepseek-r1 ответ содержит размышления в тегах
///  ...  — срезаем их, в TTS/UI должен уходить только чистый ответ.
fn clean_llm_answer(raw: &str) -> String {
    let open_tag: String = "<".to_string() + "think>";
    let close_tag: String = "<".to_string() + "/think>";
    let mut out = raw.to_string();

    // Удаляем полные блоки рассуждений (несколько подряд)
    while let (Some(start), Some(end_idx)) =
        (out.find(&open_tag), out.find(&close_tag))
    {
        if end_idx >= start {
            out.replace_range(start..end_idx + close_tag.len(), "");
        } else {
            break;
        }
    }

    // Если закрывающий тег потерян — оставляем только текст после последнего «открытия»
    if let Some(idx) = out.find(&open_tag) {
        out.replace_range(..idx + open_tag.len(), "");
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::clean_llm_answer;

    #[test]
    fn strips_think_block() {
        let open = "<".to_string() + "think>";
        let close = "<".to_string() + "/think>";
        let input = format!("{open} долгие размышления {close}Привет! Я помощник.");
        assert_eq!(clean_llm_answer(&input), "Привет! Я помощник.");
    }

    #[test]
    fn keeps_plain_answer() {
        assert_eq!(clean_llm_answer("Обычный ответ"), "Обычный ответ");
    }
}
