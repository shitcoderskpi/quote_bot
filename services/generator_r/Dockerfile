FROM debian:bookworm-slim

# Устанавливаем зависимости для rsvg-convert и сборки Rust
RUN apt-get update && apt-get install -y --no-install-recommends \
    librsvg2-bin \
    libglib2.0-0 \
    fontconfig \
    ca-certificates \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Рабочая директория
WORKDIR /app

# Копируем исходники и Cargo.toml
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Копируем локальный PNG эмодзи
COPY 1f424.png ./twemoji_png/1f424.png

# Устанавливаем Rust и собираем проект
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$HOME/.cargo/bin:$PATH" && \
    cargo build --release

# Команда запуска
CMD ["./target/release/svg-to-png-converter"]
