from sys import path
from os import pardir
from os.path import dirname, abspath, join
from unittest import TestCase

curr_dir = dirname(__file__)
parent_dir = abspath(join(curr_dir, pardir))
path.append(pardir)

from redis_controller import RedisQueue


class RedisQueueTests(TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.queue = RedisQueue("localhost:6379", "test")

    async def test_push_pop(self):
        await self.queue.push("test")
        result = await self.queue.pop()
        self.assertEqual(result, "test")

