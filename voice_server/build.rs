use std::{env, path::Path};
use fs_extra::dir::{copy, CopyOptions};

fn main() {
    // Линковка Vosk
    println!("cargo:rustc-link-search=native=vosk");
    println!("cargo:rustc-link-lib=static=libvosk");

    // Копирование папки AI только при локальной сборке
    if env::var("DOCKER_BUILD").is_err() {
        copy_ai_folder().unwrap_or_else(|e| {
            println!("cargo:warning=Не удалось скопировать папку AI: {}", e);
        });
    }
}

fn copy_ai_folder() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let profile = env::var("PROFILE")?;
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    
    let source = Path::new(&manifest_dir).join("AI");
    let dest = Path::new(&target_dir).join(&profile).join("AI");

    if !source.exists() {
        return Err(format!("Папка AI не найдена по пути: {:?}", source).into());
    }

    let options = CopyOptions {
        copy_inside: true,
        overwrite: true,
        ..Default::default()
    };

    // Удаляем старую папку, если существует
    if dest.exists() {
        fs_extra::remove_items(&[&dest])?;
    }

    // Копируем папку
    copy(&source, &dest, &options)?;
    println!("cargo:warning=Папка AI скопирована в: {:?}", dest);
    
    Ok(())
}