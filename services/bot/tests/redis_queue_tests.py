from sys import path
from os import pardir
from os.path import dirname, abspath, join
from unittest import main
from unittest.async_case import IsolatedAsyncioTestCase
from time import perf_counter

curr_dir = dirname(__file__)
parent_dir = abspath(join(curr_dir, pardir))

if parent_dir not in path:
    path.append(parent_dir)

from redis_controller import RedisQueue


class RedisQueueTests(IsolatedAsyncioTestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)

    async def test_enqueue(self) -> None:
        async with RedisQueue() as queue:
            task_id = await queue.enqueue("test", "payload")
            size = await queue.size("test")
        self.assertIsNotNone(task_id)
        self.assertEqual(task_id, 1)
        self.assertEqual(size, 1)

    async def test_dequeue(self) -> None:
        async with RedisQueue() as queue:
            if not await queue.size("test"):
                await queue.enqueue("test", "payload")

            res = await queue.dequeue("test")
            size = await queue.size("test")

        self.assertIsNotNone(res)
        _, item = res
        self.assertEqual(item.decode(), "payload")
        self.assertEqual(size, 0)

    @staticmethod
    async def measure(queue: RedisQueue):
        start = perf_counter()
        result = await queue.dequeue("test", timeout=1)
        elapsed = perf_counter() - start
        return result, elapsed * 1000

    async def test_timeout(self) -> None:
        async with RedisQueue() as queue:
            if await queue.size("test"):
                _ = await queue.dequeue("test")
            result, time = await self.measure(queue)

        self.assertIsNone(result)
        self.assertAlmostEqual(time, 1000, delta=100)


if __name__ == "__main__":
    main(failfast=True)

