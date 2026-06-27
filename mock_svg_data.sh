#!/bin/bash

QUEUE_NAME="generate:jobs"
REDIS_HOST="127.0.0.1"
REDIS_PORT="6379"

# ── Geometry (matches compute_dimensions output for a ~short message) ────────
SVG_WIDTH=325
SVG_HEIGHT=91
BUBBLE_WIDTH=225
BUBBLE_HEIGHT=76
WRAP_WIDTH=200
STATUS_X=265

# ── Block data ───────────────────────────────────────────────────────────────
CHAT_ID="987654321"

SVG_DATA='<svg xmlns="http://www.w3.org/2000/svg" width="'"$SVG_WIDTH"'"
     height="'"$SVG_HEIGHT"'">
  <defs>
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000" flood-opacity="0.15"/>
    </filter>
    <clipPath id="circle-clip">
      <circle cx="19" cy="19" r="18" />
    </clipPath>
  </defs>
  <circle cx="19" cy="19" r="18" fill="#ddd" stroke="#aaa" stroke-width="1"/>
  <image href="data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/wAARC" x="1" y="1" width="36" height="36" clip-path="url(#circle-clip)"/>
  <g filter="url(#shadow)">
    <rect x="47" y="1" width="'"$BUBBLE_WIDTH"'" height="'"$BUBBLE_HEIGHT"'" rx="16" ry="16" fill="#fff" stroke="#ccc" stroke-width="1"/>
  </g>
</svg>'

USERNAME_DATA='56;23;0;0;sans;15;700;#2a5885;Test User'
STATUS_DATA="${STATUS_X};23;0;2;sans;13;400;#888888;online"
CONTENT_DATA="56;46;${WRAP_WIDTH};0;sans;15;400;#000000;Hello, this is a valgrind test message."

# ── Assemble protocol payload ─────────────────────────────────────────────────
# Format: 2;{len(chat_id)};{chat_id},{type};{len};{data},...
make_block() {
    local type="$1"
    local data="$2"
    local len=${#data}
    printf '%s;%s;%s,' "$type" "$len" "$data"
}

CHAT_ID_LEN=${#CHAT_ID}
PAYLOAD="2;${CHAT_ID_LEN};${CHAT_ID},"
PAYLOAD+=$(make_block 0 "$SVG_DATA")
PAYLOAD+=$(make_block 1 "$USERNAME_DATA")
PAYLOAD+=$(make_block 1 "$STATUS_DATA")
PAYLOAD+=$(make_block 1 "$CONTENT_DATA")

# ── Compress and push ─────────────────────────────────────────────────────────
TMP=$(mktemp)
echo -n "$PAYLOAD" > "$TMP"
zstd -f "$TMP" -o "$TMP.zst"
redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -x LPUSH "$QUEUE_NAME" < "$TMP.zst"
rm "$TMP" "$TMP.zst"

echo "Pushed payload to $QUEUE_NAME"
echo "  svg:     ${SVG_WIDTH}x${SVG_HEIGHT}"
echo "  bubble:  ${BUBBLE_WIDTH}x${BUBBLE_HEIGHT}"
echo "  wrap:    ${WRAP_WIDTH}px"
echo "  status_x: ${STATUS_X}px"
