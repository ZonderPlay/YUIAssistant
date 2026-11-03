//! Линковка нативной библиотеки Vosk.
//! Пути настраиваются переменными окружения, чтобы сборка работала
//! и на Linux/macOS (Docker), и на Windows:
//!   VOSK_LIB_DIR  — папка с libvosk.so/.dll/.dylib (по умолчанию ./vosk)
//!   VOSK_STATIC   — "true", если линкуем .lib вместо динамики

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=VOSK_LIB_DIR");
    println!("cargo:rerun-if-env-changed=VOSK_STATIC");
    println!("cargo:rerun-if-changed=build.rs");

    // Нативная линковка нужна только если включена feature vosk-stt
    if env::var("CARGO_FEATURE_VOSK_STT").is_err() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = env::var("VOSK_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("vosk"));

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let use_static = env::var("VOSK_STATIC").map(|v| v == "true").unwrap_or(false);

    if use_static && target_os == "windows" {
        println!("cargo:rustc-link-lib=static=libvosk");
    } else {
        println!("cargo:rustc-link-lib=dylib=vosk");
    }
}
