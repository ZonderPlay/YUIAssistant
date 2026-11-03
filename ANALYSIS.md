# Анализ репозитория YUIAssistant

## Что это за проект

Полноценный пайплайн голосового ассистента из двух частей:

```
[Android-приложение YUIAssistant]  --HTTP multipart (WAV)-->  [Rust-сервер voice_server]
        🎙️ запись звука                                              🧠 VOSK STT
        🔊 воспроизведение ответа                                    🤖 Ollama LLM (deepseek-r1:1.5b)
                                                                     (TTS — заглушка, не реализован)
```

Общий поток данных (`handle_audio` в `voice_server/src/routes/audio.rs`):

1. Приложение пишет микрофон в WAV (16 кГц, mono, 16-bit PCM) и отправляет POST `/api/audio` (multipart).
2. Сервер сохраняет файл в `./storage/<uuid>_input.wav`.
3. `services::stt::transcribe_audio()` прогоняет аудио через VOSK → текст.
4. `services::deepseek::send_to_deepseek()` отправляет текст в локальный Ollama (`http://localhost:11434/api/generate`, модель `deepseek-r1:1.5b`, системный промпт «Ты — помощник. Отвечай кратко и на русском языке»).
5. Ответ возвращается как JSON `{ "type": "text", "text": "..." }`.

---

## Часть 1: `voice_server` (Rust, axum)

### Структура
| Файл | Назначение | Состояние |
|---|---|---|
| `src/main.rs` | Точка входа: проверка/запуск Ollama, загрузка модели VOSK (`./models`), роутер axum, порт 8080 | Рабочий |
| `src/routes/audio.rs` | Обработчик `POST /api/audio`: приём multipart, сохранение, STT → LLM, JSON-ответ | Рабочий |
| `src/services/stt.rs` | Распознавание речи через `vosk` (Recognizer, 16000 Гц, парсинг final result JSON) | Рабочий |
| `src/services/deepseek.rs` | HTTP-клиент к Ollama API (reqwest, `stream: false`, поле `response`) | Рабочий |
| `src/services/llm.rs` | — | **Пустой файл** |
| `src/services/tts.rs` | — | **Пустой файл** (озвучка не реализована) |
| `src/services/parser.rs` | — | **Пустой файл** |
| `src/model/response.rs` | — | **Пустой + не подключён** в main.rs (модули `model`/`utils` объявлены? — нет) |
| `src/utils/file.rs` | — | **Пустой файл** |
| `build.rs` | Линковка `libvosk` (Windows .dll/.lib) + копирование папки `AI/ollama` в target | Windows-ориентированный |
| `Dockerfile` | Сборка: скачивает ollama v0.1.25 и vosk-model-ru-0.42 | Содержит ошибки (см. ниже) |

### Ключевые зависимости (`Cargo.toml`)
`axum 0.6 (multipart)`, `tokio 1.32`, `vosk 0.3.1`, `hound 3.5.1` (для WAV, но фактически не используется — стт читает сырые байты), `reqwest`, `serde/serde_json`, `uuid`, `once_cell`, `tracing`.

### Особенности реализации STT
В `stt.rs` весь файл читается как байты и трактуются как i16-сэмпллы **вместе с 44-байтным WAV-заголовком** (из-за `unsafe from_raw_parts`). Заголовок «съедается» распознавателем как шум; корректнее было бы использовать `hound` для декодирования WAV (он уже в зависимостях).

### Проблемы и баги сервера
1. **Бинарник называется `voice_server`, а Dockerfile копирует `target/release/yuiapp`** — сборка Docker упадёт.
2. `main.rs` не объявляет модули `model` и `utils` (`mod model; mod utils;` отсутствуют) — файлы-заглушки просто не компилируются как часть crate.
3. `is_ollama_running()` реально отправляет генерацию промпта «Привет!» вместо дешёвой проверки `GET /api/tags` — лишняя нагрузка/задержка при старте.
4. Ошибка загрузки модели VOSK ведёт к `expect` (panic) без внятной диагностики пути `models`.
5. Нет CORS-слоя (tower-http) — веб-клиент к этому API не подключишь.
6. Нет TTS: клиент получает только текст, озвучку приходится делать на телефоне (Android TextToSpeech) или досчитать на сервере.
7. Хранение всех записей в `./storage` без ротации/очистки.
8. build.rs линкует Windows-библиотеки Vosk (`libvosk.lib/dll`), что ломает переносимость сборки (в Docker линковка идёт иначе, и там build.rs частично обходится через `DOCKER_BUILD=true`).

---

## Часть 2: `YUIAssistant` (Android, Kotlin + Jetpack Compose)

### Структура
| Файл | Назначение | Состояние |
|---|---|---|
| `MainActivity.kt` | Запрашивает `RECORD_AUDIO`, показывает `MainScreen` | Рабочий |
| `ui/MainScreen.kt` | Центральный экран с кнопкой | Рабочий |
| `ui/components/VoiceButton.kt` | Toggle-кнопка 🎙️: старт/стоп записи, отправка multipart на сервер через Retrofit | Рабочий, но ответ только печатается в консоль (`println`), UI не обновляется |
| `voice/VoiceRecorder.kt` | `AudioRecord` 16 кГц mono PCM16, запись в temp-файл, дописывание WAV-заголовка после остановки | Рабочий (заголовок дописывается постфактум — см. нюанс ниже) |
| `network/RetrofitClient.kt` | BASE_URL `http://193.41.78.224:8080/`, OkHttp + logging-interceptor | Рабочий, но URL захардкожен |
| `network/ApiService.kt` | `@Multipart @POST("api/audio") uploadAudio(part): Response<ResponseBody>` | Рабочий, тип ответа не типизирован (стоило бы `ServerResponse`) |
| `model/ServerResponse.kt` | Модель ответа сервера | **Пустой файл** |
| `voice/WakeWordDetector.kt` | Детектор слова активации («YUI») | **Пустой файл** (только package) |
| `training/VoiceTrainer.kt` | Обучение голоса / тренировка wake word | **Пустой файл** |
| `utils/PermissionUtils.kt` | Проверка/запрос разрешения на микрофон | Рабочий |

### Manifest / конфигурация
- Разрешения: `RECORD_AUDIO`, `INTERNET`, `FOREGROUND_SERVICE`, READ/WRITE_EXTERNAL_STORAGE (последние два устарели для targetSdk 35 и не нужны).
- `network_security_config.xml` разрешает cleartext HTTP только для `193.41.78.224` (ваш сервер). При смене адреса нужно править и этот файл.
- minSdk 26, compile/target SDK 35, Compose BOM 2024.04, Kotlin 2.0.
- В `build.gradle.kts` дублируются зависимости (retrofit/converter-gson/core-ktx объявлены дважды: через catalog и строками) — стоит причесовать.

### Нюанс записи звука
`VoiceRecorder` пишет «голый» PCM в файл, а WAV-заголовок дописывает в конце (`writeWavHeader`, читая весь файл целиком в память). Для коротких записей это нормально, но:
- нет ограничения длительности;
- при падении процесса между stop и записью заголовка файл останется битым;
- надёжнее писать заголовок сразу с placeholder и патчить размеры на `stopRecording()`.

---

## Итог: что готово, а что нет

**Уже работает end-to-end:** запись → отправка → VOSK → Ollama → текстовый JSON-ответ.

**Не реализовано (пустышки):**
1. Wake-word детекция (`WakeWordDetector.kt`) — сейчас ассистент включается только кнопкой.
2. TTS / озвучка ответа (`services/tts.rs`) — сервер возвращает текст, клиент его не произносит.
3. Парсер команд/интентов (`parser.rs`), модель ответа (`model/response.rs`), работа с файлами (`utils/file.rs`).
4. Типизированная обработка ответа в Android (`ServerResponse.kt`) и вывод в UI.
5. VoiceTrainer (персонализация/обучение).

**Что починить перед переиспользованием:**
- Имя бинарника в Dockerfile (`yuiapp` → `voice_server`), либо `[[bin]] name = "yuiapp"` в Cargo.toml.
- Добавить `mod model; mod utils;` в main.rs (или удалить пустые файлы).
- Заменить «проверку Ollama» на `GET /api/tags`; вынести URL/модель в env-переменные.
- Использовать `hound` для чтения WAV вместо unsafe-каста байтов.
- Вынести BASE_URL Android в `local.properties`/BuildConfig/DataStore вместо хардкода IP.
- Убрать лишние legacy-разрешения хранилища и дубли зависимостей в gradle.

---

## Рекомендации по вашему будущему приложению

### Архитектура (проверенный паттерн из этого проекта)
Клиент (микрофон + playback) ↔ лёгкий API-сервер ↔ три сервиса: STT, LLM, TTS. Такое разделение позволяет менять движки независимо.

### Варианты выбора компонентов
| Звено | Локально/бесплатно | Облачно/качественно |
|---|---|---|
| STT | Vosk (как здесь), whisper.cpp, faster-whisper | OpenAI Whisper API, Yandex SpeechKit |
| LLM | Ollama (deepseek-r1:1.5b и т.п.) | OpenAI/DeepSeek/Anthropic API |
| TTS | Piper, sherpa-onnx | ElevenLabs, OpenAI TTS, гугловский TTS |
| Wake word | Porcupine (Picovoice), openWakeWord | — |
| Клиент | Android AudioRecord+Compose (здесь), Python sounddevice | — |

### Минимальный MVP-цикл, который стоит собрать первым
1. Кнопка «записать» → WAV 16k mono.
2. POST на сервер → STT → текст запроса.
3. Текст → LLM → ответ.
4. Ответ → TTS → воспроизведение.
5. Только после этого — wake word, история диалогов, интенты/функции-действия.

Этот репозиторий — хорошая отправная точка: серверная цепочка STT→LLM уже рабочая, не хватает озвучки, wake-word и нормальной обработки ответа в UI клиента.
