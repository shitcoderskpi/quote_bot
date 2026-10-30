# syntax=docker/dockerfile:1.7
FROM debian:bookworm-slim AS fonts

ADD https://github.com/sahibjotsaggu/San-Francisco-Pro-Fonts.git#master /src/sf-pro
ADD https://github.com/supercomputra/SF-Mono-Font.git#master /src/sf-mono
RUN mkdir -p /out/sf-pro /out/sf-mono \
    && cp /src/sf-pro/*.otf /out/sf-pro/ \
    && cp /src/sf-mono/*.otf /out/sf-mono/

FROM rust:1-slim-bookworm AS builder
WORKDIR /usr/src/generator_rs

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libfontconfig1-dev libfreetype6-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release/deps/generator*

COPY src ./src

RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        mesa-vulkan-drivers \
        fontconfig \
        fonts-noto-color-emoji \
    && rm -rf /var/lib/apt/lists/*

COPY --from=fonts /out/ /usr/local/share/fonts/
COPY 50-emoji.conf /etc/fonts/conf.d/50-emoji.conf
RUN fc-cache -f

COPY --from=builder /usr/src/generator_rs/target/release/generator /usr/local/bin/generator
COPY templates /app/templates

RUN useradd --system --no-create-home app
USER app

CMD ["generator"]