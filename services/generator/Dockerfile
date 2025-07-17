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
    imagemagick-dev

COPY . .

# Patch cppcoro to use ftruncate instead of ftruncate64
RUN sed -i 's/ftruncate64/ftruncate/g' /app/extern/cppcoro/lib/writable_file.cpp

RUN mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    cmake --build . -- -j$(nproc) && \
    strip --strip-unneeded generator

FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache \
    libstdc++ \
    spdlog \
    hiredis \
    librsvg \
    imagemagick \
    imagemagick-c++
COPY --from=builder /app/build/generator .

EXPOSE 8080

ENTRYPOINT ["./generator"]
