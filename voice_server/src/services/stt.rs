use vosk::{Model, Recognizer};
use std::{fs::File, io::Read, error::Error};
use serde::Deserialize;

// Если у вас уже есть такой код, измените тип ошибки:
#[derive(Deserialize)]
struct VoskFinalResult {
    text: String,
}

pub fn transcribe_audio(model: &Model, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut recognizer = Recognizer::new(model, 16000.0)
        .ok_or("Ошибка создания распознавателя")?;

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() % 2 != 0 {
        return Err("Invalid WAV file: length not divisible by 2 (expected 16-bit PCM)".into());
    }

    let samples: &[i16] = unsafe {
        std::slice::from_raw_parts(buffer.as_ptr() as *const i16, buffer.len() / 2)
    };
    let _ = recognizer.accept_waveform(samples);
    let result = recognizer.final_result();
    let json_str = serde_json::to_string(&result)?;
    let parsed: VoskFinalResult = serde_json::from_str(&json_str)?;
    Ok(parsed.text)
}
