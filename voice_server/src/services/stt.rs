#![cfg(feature = "vosk-stt")]

use hound::WavReader;
use std::error::Error;
use vosk::{Model, Recognizer};

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

    // Подаём частями. accept_waveform в vosk 0.3.x возвращает
    // Result<DecodingState, AcceptWaveformError>, а не bool —
    // ошибку транслируем в SttError, состояние декодирования здесь не важно.
    const CHUNK: usize = 4096;
    for chunk in samples.chunks(CHUNK) {
        recognizer
            .accept_waveform(chunk)
            .map_err(|e| format!("VOSK accept_waveform: {e}"))?;
    }

    let result = recognizer.final_result();
    // final_result() -> CompleteResult (текст заимствован из внутреннего JSON),
    // поэтому копируем его сразу (.to_string()), пока заимствование живо.
    let text = match &result {
        vosk::CompleteResult::Single(s) => s.text.to_string(),
        vosk::CompleteResult::Multiple(m) => m
            .alternatives
            .first()
            .map(|a| a.text.to_string())
            .unwrap_or_default(),
    };
    Ok(text.trim().to_string())
}
