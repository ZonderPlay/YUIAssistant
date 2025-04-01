use std::{env, path::Path, fs};
use fs_extra::dir::{copy, CopyOptions};

fn main() {
    println!("cargo:rustc-link-search=native=vosk");
    println!("cargo:rustc-link-lib=static=libvosk");

    // Получаем директорию с исходным кодом (корень проекта)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Определяем профиль сборки ("debug" или "release")
    let profile = env::var("PROFILE").unwrap();

    // Определяем target-директорию. Если переменная CARGO_TARGET_DIR не задана, используется "target"
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());

    // Определяем путь, куда будет скопирован наш бинарник (например, target/release)
    let binary_dir = Path::new(&target_dir).join(&profile);

    // Исходная директория с ресурсами (папка AI)
    let source = Path::new(&manifest_dir).join("AI");

    // Путь назначения – копия папки AI в директорию с бинарником
    let dest = binary_dir.join("AI");

    // Если в target уже существует папка AI, удаляем её
    if dest.exists() {
        fs::remove_dir_all(&dest).expect("Не удалось удалить старую папку AI");
    }

    // Настройки для копирования (копируем содержимое папки)
    let mut options = CopyOptions::new();
    options.copy_inside = true; // копировать содержимое, а не саму папку

    // Выполняем копирование
    match copy(&source, &dest, &options) {
        Ok(_) => println!("cargo:warning=Папка AI успешно скопирована в {:?}", dest),
        Err(e) => panic!("Ошибка копирования папки AI: {:?}", e),
    }
}