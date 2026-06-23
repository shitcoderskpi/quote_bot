import math

from jinja2 import Template
import json
from zstandard.backend_cffi import ZstdDecompressor
from config import *
from redis_controller import RedisQueue
from zstandard import ZstdCompressor

with open("tg_template.tem", "r", encoding="utf-8") as f:
    template_str = f.read().strip()

template = Template(template_str)
redis = RedisQueue(REDIS_HOST)

compressor = ZstdCompressor(9)
decompressor = ZstdDecompressor()

async def receive_msg():
    msg = await redis.dequeue("generate:messages")
    message = decompressor.decompress(msg[1])
    return json.loads(message)

async def create_template(wrap_width: int = 245, char_width: int = 8, line_height: int = 21):
    context_data = await receive_msg()
    content = context_data.get("content", "")

    max_chars_per_line = wrap_width // char_width
    num_lines = math.ceil(len(content) / max_chars_per_line)

    bubble_height = 55 + num_lines * line_height
    svg_height = 70 + num_lines * line_height

    context_data["wrap_width"] = wrap_width
    context_data["bubble_height"] = bubble_height
    context_data["svg_height"] = svg_height

    payload = b""

    raw_blocks = template_str.split(",\n")

    for block in raw_blocks:
        if not block.strip():
            continue
        if block.endswith(","):
            block = block[:-1]
        parts = block.split(";", 2)
        if len(parts) < 3:
            continue

        block_type = parts[0].encode("utf-8")
        raw_data_template = parts[2]
        rendered_data = Template(raw_data_template).render(context_data).encode("utf-8")
        block_len = str(len(rendered_data)).encode("utf-8")
        payload += block_type + b";" + block_len + b";" + rendered_data + b","
    chat_id_bytes = str(context_data.get("chat-id", 0)).encode("utf-8")
    payload = b"2;" + str(len(chat_id_bytes)).encode("utf-8") + b";" + chat_id_bytes + b"," + payload
    dt = compressor.compress(payload)
    return await redis.enqueue("generate:jobs", dt)