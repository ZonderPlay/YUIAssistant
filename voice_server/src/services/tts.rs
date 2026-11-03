//! Сервис озвучки ответа (TTS).
//! MVP-подход: сам синтез речи делать не нужно — Android умеет нативный
//! TextToSpeech. Здесь сервер лишь готовит текст для озвучки: убирает
//! размышления модели, markdown и слишком «технические» символы.

/// Готовит ответ LLM к озвучиванию на клиенте.
pub fn prepare_for_tts(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            // markdown-артефакты, которые TTS читает вслух
            '*' | '#' | '`' | '_' => continue,
            '\n' => out.push(' '),
            c if c.is_control() => continue,
            c => out.push(c),
        }
    }
    // схлопываем повторяющиеся пробелы
    let mut result = String::with_capacity(out.len());
    let mut prev_space = false;
    for ch in out.chars() {
        let is_space = ch == ' ';
        if !(is_space && prev_space) {
            result.push(ch);
        }
        prev_space = is_space;
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown() {
        assert_eq!(prepare_for_tts("**Привет!**\n`мир`"), "Привет! мир");
    }
}
