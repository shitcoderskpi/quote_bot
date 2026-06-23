FROM alpine:latest AS builder

WORKDIR /app
#USER app

RUN apk add --no-cache \
    linux-headers \
    cmake \
    make \
    g++ \
    spdlog-dev \
    hiredis-dev \
    libuv-dev \
    zlib-dev \
    curl-dev \
    imagemagick-dev \
    cairo-dev \
    pango-dev \
    zstd-dev \
    zstd-static \
    zstd-libs

COPY . .

RUN mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=/usr/lib .. &&\
    cmake --build . -j$(nproc) --target generator && \
    strip --strip-unneeded generator

FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache \
    libstdc++ \
    spdlog \
    hiredis \
    librsvg \
    cairo \
    pango \
    imagemagick \
    imagemagick-c++ \
    font-jetbrains-mono-nerd \
    font-noto-emoji \
    font-noto \
    zstd \
    libuv

COPY --from=builder /app/build/generator .

COPY templates/ ./templates/

EXPOSE 8080

ENTRYPOINT ["./generator"]
