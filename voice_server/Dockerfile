# voice_server — MVP-образ.
# Этап 1: сборка Rust (реальный Vosk: скачиваем linux-библиотеку).
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl unzip ca-certificates pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Нативная библиотека Vosk для Linux
RUN curl -sSL https://alphacephei.com/vosk/releases/vosk-0.3.45-linux-x64.tar.bz2 \
      -o vosk.tar.bz2 || curl -sSL https://alphacephei.com/vosk/releases/vosk-0.3.45-linux-x64.zip -o vosk.zip ; \
    if [ -f vosk.tar.bz2 ]; then tar -xf vosk.tar.bz2 -C /tmp/voskx || mkdir -p /tmp/voskx && tar -xf vosk.tar.bz2 -C /tmp/voskx; else unzip vosk.zip -d /tmp/voskx; fi ; \
    mkdir -p /app/vosklib && find /tmp/voskx -name 'libvosk.so*' -exec cp -av {} /app/vosklib/ \; && ls /app/vosklib

COPY Cargo.toml Cargo.lock* build.rs ./
COPY src ./src

ENV VOSK_LIB_DIR=/app/vosklib
RUN cargo build --release

# Русская модель Vosk
RUN mkdir -p /app/models && \
    curl -sSL https://alphacephei.com/vosk/models/vosk-model-small-ru-0.22.zip -o /tmp/m.zip && \
    unzip /tmp/m.zip -d /tmp && mv /tmp/vosk-model-small-ru-0.22/* /app/models/ && rm -rf /tmp/m.zip /tmp/vosk-model*

# Финальный образ: Debian + Ollama поверх
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Ollama (официальный установщик, актуальная версия вместо древней v0.1.25)
RUN curl -fsSL https://ollama.com/install.sh | sh

WORKDIR /app
COPY --from=builder /app/target/release/voice_server /usr/local/bin/voice_server
COPY --from=builder /app/vosklib/libvosk.so* /usr/local/lib/
COPY --from=builder /app/models ./models

RUN ldconfig

ENV BIND_ADDR=0.0.0.0:8080 \
    VOSK_MODEL_DIR=/app/models \
    OLLAMA_URL=http://localhost:11434 \
    AUTOSTART_OLLAMA=true

EXPOSE 8080
ENTRYPOINT ["voice_server"]
