#!/usr/bin/env bash
# Usage: ./build-tba.sh [docker|podman]   (auto-detected if omitted)
set -euo pipefail

if [ $# -ge 1 ]; then
  RT=$1
else
  RT=$(command -v docker || command -v podman || true)
  RT=${RT##*/}
fi
[ -n "$RT" ] && command -v "$RT" >/dev/null \
  || { echo "no container runtime found (need podman or docker)" >&2; exit 1; }
echo "using: $RT"

cd "$(dirname "$0")"
SUB=$(cd ../extern/telegram-bot-api && pwd)

case "$(uname -m)" in
  x86_64|amd64)  ARCH=amd64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

git submodule update --init --recursive

# Private build context: same sources, minus the submodule's .dockerignore
# (and our own output). The submodule itself is never modified.
CTX=$(mktemp -d)
cid=""
cleanup() {
  [ -n "$cid" ] && "$RT" rm "$cid" >/dev/null 2>&1 || true
  rm -rf "$CTX"
}
trap cleanup EXIT

tar -C "$SUB" --exclude=./.dockerignore --exclude=./binaries --exclude=./.git -cf - . \
  | tar -C "$CTX" -xf -

"$RT" build -f tba-builder.Dockerfile -t tba-builder "$CTX"

mkdir -p "$SUB/binaries/$ARCH"
cid=$("$RT" create tba-builder)
"$RT" cp "$cid:/telegram-bot-api" "$SUB/binaries/$ARCH/telegram-bot-api"
chmod +x "$SUB/binaries/$ARCH/telegram-bot-api"
