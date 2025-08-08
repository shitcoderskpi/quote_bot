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
    capnproto-dev \
    cairo-dev \
    pango-dev

COPY . .

# Patch cppcoro to use ftruncate instead of ftruncate64
RUN sed -i 's/ftruncate64/ftruncate/g' /app/extern/cppcoro/lib/writable_file.cpp

RUN capnp compile -oc++ image.capnp

RUN mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_C_FLAGS="-fno-omit-frame-pointer -g3 -O0 -DDEBUG" .. && \
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
    capnproto
#    coreutils \
#    gdb \
#    musl-dbg

COPY --from=builder /app/build/generator .

#COPY ./entrypoint.sh ./entrypoint.sh

COPY templates/ ./templates/

#RUN chmod +x ./entrypoint.sh

EXPOSE 8080

#ENTRYPOINT ["./entrypoint.sh"]
ENTRYPOINT ["./generator"]
