from asyncio import run
import json
from zstandard import ZstdCompressor, ZstdDecompressor
from json import loads
from base64 import b64decode
from term_image.image import KittyImage
from os import remove

from redis_controller import RedisQueue

compressor = ZstdCompressor(9)
decompressor = ZstdDecompressor()
data = r"""{
  "chat_id": -89574657,
  "template": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"360\" height=\"110\" font-family=\"sans-serif\">\n  <defs>\n    <filter id=\"shadow\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\">\n      <feDropShadow dx=\"0\" dy=\"2\" stdDeviation=\"2\" flood-color=\"#000\" flood-opacity=\"0.15\"/>\n    </filter>\n  </defs>\n\n  <circle cx=\"19\" cy=\"19\" r=\"18\" fill=\"#ddd\" stroke=\"#aaa\" stroke-width=\"1\"/>\n\n  <g filter=\"url(#shadow)\">\n    <rect x=\"47\" y=\"0\" width=\"260\" height=\"70\" rx=\"16\" ry=\"16\" fill=\"#fff\" stroke=\"#ccc\" stroke-width=\"1\"/>\n  </g>\n\n  <text x=\"56\" y=\"45\" font-size=\"13px\" fill=\"#000\">Привет! Это пример сообщения</text>\n  <text x=\"56\" y=\"23\" font-size=\"14px\" font-weight=\"bold\" fill=\"#2a5885\">Александр</text>\n  <text x=\"300\" y=\"23\" font-size=\"12px\" fill=\"#888\" text-anchor=\"end\">админ</text>\n</svg>"
}"""


async def main():
    compressed_data = compressor.compress(data.encode("utf-8"))

    print(f"Original size: {len(data)} bytes")
    print(f"Compressed size: {len(compressed_data)} bytes")
    print(f"Compression ratio: {len(compressed_data)/len(data):.2f}")

    queue = RedisQueue()
    await queue.enqueue("generate:jobs", compressed_data)
    compressed = await queue.dequeue("generate:results")

    decompressed = decompressor.decompress(compressed[1])

    res = json.loads(decompressed)

    with open("output.webp", "wb") as f:
        f.write(b64decode(res.get("image")))
    img = KittyImage.from_file("output.webp")
    img.draw()
    remove("output.webp")


if __name__ == "__main__":
    run(main())
