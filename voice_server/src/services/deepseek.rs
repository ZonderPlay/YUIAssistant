use reqwest::Client;
use serde_json::json;

pub async fn send_to_deepseek(prompt: &str) -> Result<String, String> {
    let client = Client::new();
    let url = "http://localhost:11434/api/generate";

    // Добавляем системную инструкцию
    let full_prompt = format!(
        "Ты — помощник. Отвечай кратко и на русском языке.\n\n{}",
        prompt
    );

    let body = json!({
        "model": "deepseek-r1:1.5b",
        "prompt": full_prompt,
        "stream": false
    });

    match client.post(url).json(&body).send().await {
        Ok(response) => {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Ошибка при получении ответа".to_string());
            
            if status.is_success() {
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(parsed_json) => {
                        // Используем правильное поле "response"
                        let result = parsed_json.get("response")
                            .and_then(|r| r.as_str())
                            .unwrap_or("⚠️ Ответ пустой!");

                        Ok(result.to_string())
                    }
                    Err(err) => {
                        Err(format!("Ошибка парсинга JSON: {}. Ответ: {}", err, text))
                    }
                }
            } else {
                Err(format!("DeepSeek вернул ошибку {}: {}", status, text))
            }
        }
        Err(err) => Err(format!("Не удалось подключиться к DeepSeek: {}", err)),
    }
}
