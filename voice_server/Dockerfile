# Используем официальный образ Rust
FROM rust:latest AS builder

# Устанавливаем переменную для build.rs
ENV DOCKER_BUILD=true

# Устанавливаем зависимости
RUN apt-get update && \
    apt-get install -y curl unzip && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Создаём структуру папок
RUN mkdir -p ./AI && mkdir -p ./models

# Установка Ollama
RUN curl -sSL https://github.com/jmorganca/ollama/releases/download/v0.1.25/ollama-linux-amd64 -o ./AI/ollama && \
    chmod +x ./AI/ollama

# Установка модели Vosk
RUN curl -sSL https://alphacephei.com/vosk/models/vosk-model-ru-0.42.zip -o /tmp/vosk.zip && \
    unzip /tmp/vosk.zip -d /tmp && \
    mv /tmp/vosk-model-ru-0.42/* ./models/ && \
    rm -rf /tmp/vosk*

# Копируем только необходимые файлы
COPY . .

# Сборка проекта
RUN cargo build --release

# Финальный образ
FROM debian:buster-slim

WORKDIR /app

# Копируем необходимые файлы
COPY --from=builder /app/AI ./AI
COPY --from=builder /app/models ./models
COPY --from=builder /app/target/release/yuiapp /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/yuiapp"]