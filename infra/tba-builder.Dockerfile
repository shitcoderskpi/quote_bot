FROM ubuntu:24.04
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates make cmake g++ gperf zlib1g-dev libssl-dev git \
    && rm -rf /var/lib/apt/lists/*
COPY . /src
WORKDIR /src/build
RUN cmake -DCMAKE_BUILD_TYPE=Release .. \
    && cmake --build . --target telegram-bot-api -j"$(nproc)"
RUN BIN=$(find /src/build -type f -name telegram-bot-api -perm -u+x | head -n1) \
    && test -n "$BIN" \
    && cp "$BIN" /telegram-bot-api \
    && strip /telegram-bot-api
