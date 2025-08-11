FROM alpine:latest AS builder

WORKDIR /app

RUN apk add --no-cache \
    linux-headers \
    cmake \
    make \
    g++ \
    spdlog-dev \
    hiredis-dev \
    zlib-dev \
    curl-dev \
    imagemagick-dev \
    pugixml-dev \
    cairo-dev \
    pango-dev \
    simdjson-dev \
    simdjson-static \
    zstd-dev \
    zstd-static \
    zstd-libs

COPY . .

# Patch cppcoro to use ftruncate instead of ftruncate64
RUN sed -i 's/ftruncate64/ftruncate/g' /app/extern/cppcoro/lib/writable_file.cpp

RUN mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_PREFIX_PATH=/usr/lib .. &&\
    cmake --build . -j$(nproc) && \
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
    pugixml \
    font-noto \
    simdjson \
    zstd

COPY --from=builder /app/build/generator .

COPY templates/ ./templates/

EXPOSE 8080

ENTRYPOINT ["./generator"]
