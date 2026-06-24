import math
from jinja2 import Template
import json
from zstandard.backend_cffi import ZstdDecompressor
from config import *
from redis_controller import RedisQueue
from zstandard import ZstdCompressor

import gi
gi.require_version('Pango', '1.0')
gi.require_version('PangoCairo', '1.0')
from gi.repository import Pango, PangoCairo
import cairo

with open("tg_template.tem", "r", encoding="utf-8") as f:
    template_str = f.read().strip()
template = Template(template_str)
redis = RedisQueue(REDIS_HOST)
compressor = ZstdCompressor(9)
decompressor = ZstdDecompressor()

# ---------------------------------------------------------------------------
# Layout geometry — derived from template coordinates
#
#   SVG:    [0 .................... svg_width ]
#   avatar: x=1  w=36  → ends at 37
#   bubble: x=47 w=bubble_width
#   text:   x=56        ← TEXT_X
#
#   TEXT_LEFT_PAD  = TEXT_X(56) - BUBBLE_X(47)         =  9 px
#   TEXT_RIGHT_PAD = extra space right of text in bubble (timestamps, etc.)
#   SVG_RIGHT_MARGIN = gap between bubble right edge and SVG right edge
# ---------------------------------------------------------------------------
BUBBLE_X          = 47
TEXT_X            = 56
TEXT_LEFT_PAD     = TEXT_X - BUBBLE_X   # 9
TEXT_RIGHT_PAD    = 16
SVG_RIGHT_MARGIN  = 53                  # keeps proportions of original 450px design

MIN_WRAP = 100   # ~200 px bubble minimum
MAX_WRAP = 450   # ~450 px SVG maximum  (bubble_x + MAX_WRAP + pads + margin ≈ 475, clamp svg to 450)
MAX_SVG  = 550

# ---------------------------------------------------------------------------
# Parse content block font at startup
# Block format: 1;{byte_len};{x};{y};{wrap};{f1};{f2};{font};{text}
# ---------------------------------------------------------------------------
def _parse_content_block(tmpl_str: str) -> str:
    for raw_block in tmpl_str.split(",\n"):
        raw_block = raw_block.strip().rstrip(",")
        if not raw_block:
            continue
        parts = raw_block.split(";", 8)
        if len(parts) < 9 or parts[0] != "1":
            continue
        if "content" in parts[8]:
            return parts[7].strip()   # font descriptor
    raise ValueError("Could not locate the content block in the template")

_CONTENT_FONT = _parse_content_block(template_str)

# ---------------------------------------------------------------------------
# Reusable offscreen surface for measurement
# ---------------------------------------------------------------------------
_measure_surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, 1, 1)
_measure_cr      = cairo.Context(_measure_surface)

def _make_layout(text: str, font_desc_str: str) -> "Pango.Layout":
    layout = PangoCairo.create_layout(_measure_cr)
    layout.set_text(text, -1)
    layout.set_font_description(Pango.font_description_from_string(font_desc_str))
    return layout

def measure_natural_width(text: str, font_desc_str: str) -> int:
    """Single-line (unwrapped) pixel width — used to decide bubble width."""
    layout = _make_layout(text, font_desc_str)
    layout.set_width(-1)                        # no wrapping
    w, _ = layout.get_pixel_size()
    return w

def measure_wrapped(text: str, font_desc_str: str, wrap_width: int) -> tuple[int, int]:
    """Pixel (width, height) after wrapping at wrap_width."""
    layout = _make_layout(text, font_desc_str)
    layout.set_width(wrap_width * Pango.SCALE)
    layout.set_wrap(Pango.WrapMode.WORD_CHAR)
    return layout.get_pixel_size()

STATUS_RIGHT_MARGIN = 60   # original: 450 - 390 = 60 px from SVG right edge

def compute_dimensions(content: str) -> dict:
    natural_w  = measure_natural_width(content, _CONTENT_FONT)
    wrap_width = max(MIN_WRAP, min(natural_w, MAX_WRAP))

    _, text_h = measure_wrapped(content, _CONTENT_FONT, wrap_width)

    bubble_width  = wrap_width + TEXT_LEFT_PAD + TEXT_RIGHT_PAD
    svg_width     = min(BUBBLE_X + bubble_width + SVG_RIGHT_MARGIN, MAX_SVG)
    bubble_height = 55 + text_h
    svg_height    = 70 + text_h
    status_x      = svg_width - STATUS_RIGHT_MARGIN   # tracks SVG width

    return dict(
        wrap_width    = wrap_width,
        bubble_width  = bubble_width,
        svg_width     = svg_width,
        bubble_height = bubble_height,
        svg_height    = svg_height,
        status_x      = status_x,
    )

# ---------------------------------------------------------------------------
# Pipeline
# ---------------------------------------------------------------------------
async def receive_msg():
    msg = await redis.dequeue("generate:messages")
    message = decompressor.decompress(msg[1])
    return json.loads(message)

async def create_template():
    context_data = await receive_msg()
    content      = context_data.get("content", "")
    user_status  = context_data.get("user_status") or None  # normalize empty string → None

    context_data["user_status"] = user_status
    context_data.update(compute_dimensions(content))

    payload = b""
    for block in template_str.split(",\n"):
        block = block.strip().rstrip(",")
        if not block:
            continue

        parts = block.split(";", 8)

        # Drop the user_status block entirely when status is absent
        if (
                user_status is None
                and len(parts) >= 9
                and parts[0] == "1"
                and "user_status" in parts[8]
        ):
            continue

        parts = block.split(";", 2)
        if len(parts) < 3:
            continue

        block_type    = parts[0].encode("utf-8")
        rendered_data = Template(parts[2]).render(context_data).encode("utf-8")
        block_len     = str(len(rendered_data)).encode("utf-8")
        payload      += block_type + b";" + block_len + b";" + rendered_data + b","

    chat_id_bytes = str(context_data.get("chat-id", 0)).encode("utf-8")
    payload = (
            b"2;" + str(len(chat_id_bytes)).encode("utf-8") + b";" + chat_id_bytes + b","
            + payload
    )

    return await redis.enqueue("generate:jobs", compressor.compress(payload))