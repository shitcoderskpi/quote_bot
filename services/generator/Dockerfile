FROM alpine:latest AS builder

WORKDIR /app

RUN apk add --no-cache \
    cmake \
    make \
    g++ \
    spdlog-dev \
    hiredis-dev

COPY . .

RUN mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    cmake --build . -- -j$(nproc) && \
    strip --strip-unneeded generator

FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache \
    libstdc++ \
    spdlog \
    hiredis

COPY --from=builder /app/build/generator .

ENTRYPOINT ["./generator"]
