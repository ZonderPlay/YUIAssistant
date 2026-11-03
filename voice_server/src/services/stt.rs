#![cfg(feature = "vosk-stt")]

use hound::WavReader;
use serde::Deserialize;
use std::error::Error;
use vosk::{Model, Recognizer};

#[derive(Deserialize)]
struct VoskFinalResult {
    text: String,
}

/// Читает WAV (16-bit PCM) и возвращает моно-сэмпллы i16.
fn read_wav_samples(path: &str) -> Result<Vec<i16>, Box<dyn Error + Send + Sync>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    if spec.channels == 0 {
        return Err("WAV: 0 каналов".into());
    }

    let mut samples: Vec<i16> = Vec::new();
    match spec.sample_format {
        hound::SampleFormat::Int => {
            for s in reader.samples::<i16>() {
                samples.push(s?);
            }
        }
        hound::SampleFormat::Float => {
            for s in reader.samples::<f32>() {
                let v = s?.clamp(-1.0, 1.0);
                samples.push((v * i16::MAX as f32) as i16);
            }
        }
    }

    // Понимиксование в моно, если каналов больше одного
    if spec.channels > 1 {
        let ch = spec.channels as usize;
        samples = samples
            .chunks(ch)
            .map(|c| {
                let sum: i32 = c.iter().map(|&s| s as i32).sum();
                (sum / ch as i32) as i16
            })
            .collect();
    }

    Ok(samples)
}

pub fn transcribe_audio(
    model: &Model,
    path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut recognizer = Recognizer::new(model, 16000.0)
        .ok_or("Ошибка создания распознавателя VOSK")?;

    let samples = read_wav_samples(path)?;
    if samples.is_empty() {
        return Err("WAV-файл не содержит сэмплов".into());
    }

    // Подаём частями, чтобы не держать всё в памяти одним срезом
    const CHUNK: usize = 4096;
    for chunk in samples.chunks(CHUNK) {
        if !recognizer.accept_waveform(chunk) {
            break;
        }
    }

    let result = recognizer.final_result();
    let json_str = serde_json::to_string(&result)?;
    let parsed: VoskFinalResult = serde_json::from_str(&json_str)?;
    Ok(parsed.text.trim().to_string())
}
