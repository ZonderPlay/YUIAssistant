//! Утилиты работы с файлами хранилища.

use std::{fs, io, path::Path};
use uuid::Uuid;

/// Сохраняет байты в папку storage с уникальным именем. Возвращает путь.
pub fn save_to_storage(dir: &Path, file_name: &str, data: &[u8]) -> io::Result<String> {
    fs::create_dir_all(dir)?;
    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("input.wav");
    let path = dir.join(format!("{}_{}", Uuid::new_v4(), safe_name));
    fs::write(&path, data)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Удаляет файлы старше `max_age_sec` секунд из указанной папки.
pub fn cleanup_old_files(dir: &Path, max_age_sec: u64) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age.as_secs() > max_age_sec {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}
