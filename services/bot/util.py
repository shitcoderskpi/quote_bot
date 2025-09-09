from asyncio import queues, run
import json
from os.path import exists, isdir
from zstandard import ZstdCompressor, ZstdDecompressor
from json import loads
from base64 import b64decode
from term_image.image import KittyImage
from os import remove

from redis_controller import RedisQueue

queue_name = "generate:jobs"
data_file = ".data"
compressor = ZstdCompressor(9)
decompressor = ZstdDecompressor()
queue = RedisQueue()


async def main():
    if exists(data_file) and isdir(data_file):
        raise OSError(f"Path '{data_file}' is a directory")
    if not exists(data_file):
        data = await queue.dequeue(queue_name)
        decompressed = decompressor.decompress(data[1])
        with open(data_file, "w+") as f:
            f.write(decompressed.decode())
        print("Restart util.py.")
        exit(0)
    else:
        data = ""
        with open(data_file) as f:
            data = f.read()

        compressed = compressor.compress(data.encode())
        await queue.enqueue(queue_name, compressed)


if __name__ == "__main__":
    run(main())
