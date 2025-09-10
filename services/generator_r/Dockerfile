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

# Копируем локальный PNG эмодзи (всю папку)
COPY 72x72 ./twemoji

# Устанавливаем Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Добавляем Rust в PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Сборка проекта
RUN cargo build --release

# Команда запуска
CMD ["/app/target/release/svg-to-png-converter"]
