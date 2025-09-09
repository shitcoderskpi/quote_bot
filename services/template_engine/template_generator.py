from jinja2 import Template
import json
import time
import math
from zstandard.backend_cffi import ZstdDecompressor
from config import *
from redis_controller import RedisQueue
from zstandard import ZstdCompressor

with open("tg_template.svg", "r", encoding="utf-8") as f:
    template_str = f.read()

template = Template(template_str)
redis = RedisQueue(REDIS_HOST)

compressor = ZstdCompressor(9)
decompressor = ZstdDecompressor()

async def receive_msg():
    msg = await redis.dequeue("generate:messages")
    message = decompressor.decompress(msg[1])
    return json.loads(message)

async def send_msg(template, chatid):
    data = { "chat_id" : chatid, "template" : template }
    dt = compressor.compress(json.dumps(data).encode("utf-8")) #
    return await redis.enqueue("generate:jobs", dt)


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

    rendered_svg = template.render(context_data)

    return rendered_svg, context_data.get("chat-id")