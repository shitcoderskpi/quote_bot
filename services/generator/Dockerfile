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
    mesa-vulkan-drivers \
    fontconfig \
    fonts-noto-color-emoji \
    git \
    ca-certificates \
    # TODO: maybe consider to repack these to png, or use other fonts (like Inter ot similar) so Apple won't be a problem. Just in case \
    && git clone https://github.com/sahibjotsaggu/San-Francisco-Pro-Fonts.git /tmp/sf-pro \
    && git clone https://github.com/supercomputra/SF-Mono-Font.git /tmp/sf-mono \
    && apt-get purge -y fonts-dejavu-core fonts-dejavu \
    && rm -rf /usr/share/fonts/truetype/dejavu \
    && mkdir -p /usr/local/share/fonts/sf-pro /usr/local/share/fonts/sf-mono \
    && cp /tmp/sf-pro/*.otf /usr/local/share/fonts/sf-pro/ \
    && cp /tmp/sf-mono/*.otf /usr/local/share/fonts/sf-mono/ \
    && rm -rf /tmp/sf-pro /tmp/sf-mono \
    && apt-get purge -y git \
    && rm -rf /var/lib/apt/lists/*

COPY 50-emoji.conf /etc/fonts/conf.d/50-emoji.conf
RUN fc-cache -f

COPY --from=builder /usr/src/generator_rs/target/release/generator_rs /usr/local/bin/generator_rs
COPY templates /app/templates


CMD ["generator_rs"]