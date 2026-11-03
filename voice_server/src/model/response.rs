//! Типизированный ответ API. Единая структура для сервера и клиента.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Text,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub r#type: ResponseType,
    pub text: String,
    /// Распознанный запрос (удобно для отладки MVP)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transcript: Option<String>,
}

impl ApiResponse {
    pub fn text(body: impl Into<String>, transcript: impl Into<String>) -> Self {
        Self {
            r#type: ResponseType::Text,
            text: body.into(),
            transcript: Some(transcript.into()),
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            r#type: ResponseType::Error,
            text: msg.into(),
            transcript: None,
        }
    }
}
