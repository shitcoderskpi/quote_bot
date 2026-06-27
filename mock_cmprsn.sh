#!/bin/bash

# Configuration
QUEUE_NAME="generate:jobs"
REDIS_HOST="127.0.0.1"
REDIS_PORT="6379"

# Dummy payload matching your log schema
PAYLOAD='{"chat_id": 987654321, "template": "<svg xmlns=\"http://www.w3.org/2000/svg\"><text>Test</text></svg>"}'

# Compress and pipe directly into Redis using LPUSH
# (If your C++ code uses BRPOP/RPOP, use LPUSH here)
echo -n "$PAYLOAD" | zstd -c | redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -x LPUSH "$QUEUE_NAME"

echo "Pushed compressed payload to Redis queue: $QUEUE_NAME"
