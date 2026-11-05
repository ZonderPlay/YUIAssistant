//! Абстракция движка распознавания речи.
//! Позволяет собирать сервер в двух режимах:
//!   * `vosk-stt` (по умолчанию) — реальный VOSK;
//!   * без него (`--no-default-features`) — mock-движок для тестов пайплайна
//!     без нативной libvosk и модели.

use std::error::Error;

pub type SttError = Box<dyn Error + Send + Sync>;

pub trait SttEngine: Send + Sync {
    fn transcribe_wav(&self, path: &str) -> Result<String, SttError>;
}

#[cfg(feature = "vosk-stt")]
pub mod vosk_engine {
    use super::{SttEngine, SttError};
    use crate::services::stt;
    use std::sync::Arc;
    use vosk::Model;

    pub struct VoskEngine {
        model: Arc<Model>,
    }

    impl VoskEngine {
        pub fn load(model_dir: &str) -> Result<Self, SttError> {
            // vosk 0.3.x: Model::new возвращает Option (None — модель не найдена/повреждена),
            // поэтому `?` напрямую неприменим — конвертируем в осмысленную ошибку.
            let model = Model::new(model_dir).ok_or_else(|| {
                format!(
                    "Не удалось загрузить VOSK-модель из '{model_dir}'. \
                     Проверьте, что папка существует и содержит файлы модели \
                     (например, vosk-model-small-ru)."
                )
            })?;
            Ok(Self { model: Arc::new(model) })
        }
    }

    impl SttEngine for VoskEngine {
        fn transcribe_wav(&self, path: &str) -> Result<String, SttError> {
            stt::transcribe_audio(&self.model, path)
        }
    }
}

#[cfg(not(feature = "vosk-stt"))]
pub mod mock_engine {
    use super::{SttEngine, SttError};

    /// Mock-движок: «распознаёт» имя файла. Позволяет гонять весь пайплайн
    /// (HTTP → STT → LLM → JSON) без установки Vosk.
    pub struct MockEngine;

    impl MockEngine {
        pub fn new() -> Self {
            Self
        }
    }

    impl SttEngine for MockEngine {
        fn transcribe_wav(&self, path: &str) -> Result<String, SttError> {
            println!("⚠️ [MOCK STT] используется вместо VOSK: {path}");
            // Возвращаем детерминированный текст, чтобы LLM получал запрос
            Ok("Расскажи анекдот".to_string())
        }
    }
}

/// Создаёт движок по features/переменной окружения.
pub fn create_engine(_model_dir: &str) -> Result<Box<dyn SttEngine>, SttError> {
    #[cfg(feature = "vosk-stt")]
    {
        Ok(Box::new(vosk_engine::VoskEngine::load(_model_dir)?))
    }
    #[cfg(not(feature = "vosk-stt"))]
    {
        Ok(Box::new(mock_engine::MockEngine::new()))
    }
}
