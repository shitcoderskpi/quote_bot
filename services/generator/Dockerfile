FROM rust:1-slim-bookworm AS builder
WORKDIR /usr/src/generator_rs

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    g++ \
    cmake \
    make \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/generator*

# Build real source
COPY src ./src
RUN cargo build --release

# -------------------------
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libvulkan1 \
    mesa-vulkan-drivers \
    libfontconfig1 \
    libfreetype6 \
    fonts-jetbrains-mono \
    fonts-noto \
    fonts-noto-color-emoji \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/generator_rs/target/release/generator_rs /usr/local/bin/generator_rs
COPY tg_template.tem /app/templates/tg_template.tem



CMD ["generator_rs"]