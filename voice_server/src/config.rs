//! Конфигурация сервера из переменных окружения (с дефолтами).

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// Адрес, на котором слушает HTTP-сервер
    pub bind_addr: String,
    /// Папка с моделью VOSK
    pub vosk_model_dir: String,
    /// Базовый URL Ollama (без слэша в конце)
    pub ollama_url: String,
    /// Название модели LLM в Ollama
    pub ollama_model: String,
    /// Системная инструкция для LLM
    pub system_prompt: String,
    /// Путь к бинарнику ollama (для автозапуска), если не запущен внешний
    pub ollama_bin: String,
    /// Запускать ли ollama автоматически
    pub autostart_ollama: bool,
}

fn var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: var_or("BIND_ADDR", "0.0.0.0:8080"),
            vosk_model_dir: var_or("VOSK_MODEL_DIR", "./models"),
            ollama_url: var_or("OLLAMA_URL", "http://localhost:11434"),
            ollama_model: var_or("OLLAMA_MODEL", "deepseek-r1:1.5b"),
            system_prompt: var_or(
                "SYSTEM_PROMPT",
                "Ты — голосовой помощник Юи. Отвечай кратко и на русском языке.",
            ),
            ollama_bin: var_or("OLLAMA_BIN", "ollama"),
            autostart_ollama: env::var("AUTOSTART_OLLAMA")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
        }
    }
}
