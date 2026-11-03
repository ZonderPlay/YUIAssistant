//! Простейший разбор распознанного текста на интенты MVP.
//! Пока ассистент понимает несколько базовых команд без обращения к LLM.

#[derive(Debug, PartialEq, Eq)]
pub enum Intent {
    Greeting,
    TimeQuestion,
    Help,
    Stop,
    Unknown,
}

pub fn parse_intent(text: &str) -> Intent {
    let t = text.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| t.contains(w));

    if has(&["привет", "здравствуй", "добрый день", "доброе утро", "добрый вечер", "хай"]) {
        Intent::Greeting
    } else if has(&["который час", "сколько времени", "какое время"]) {
        Intent::TimeQuestion
    } else if has(&["помощь", "помоги", "что ты умеешь", "команды"]) {
        Intent::Help
    } else if has(&["стоп", "замолчи", "выключись", "пока", "до свидания"]) {
        Intent::Stop
    } else {
        Intent::Unknown
    }
}

/// Ответ на распознанный интент (для локальных команд без LLM).
pub fn reply(intent: &Intent) -> Option<&'static str> {
    match intent {
        Intent::Greeting => Some("Привет! Чем могу помочь?"),
        Intent::Help => Some("Скажите вопрос — я запишу его, распознаю и отвечу через модель."),
        Intent::Stop => Some("Хорошо, до связи!"),
        _ => None, // TimeQuestion/Unknown — отправляем в LLM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_detected() {
        assert_eq!(parse_intent("Привет, как дела?"), Intent::Greeting);
    }

    #[test]
    fn unknown_falls_through() {
        assert_eq!(parse_intent("расскажи про космос"), Intent::Unknown);
    }
}
