from sys import path
from os import pardir
from os.path import dirname, abspath, join
from unittest import TestCase, main

curr_dir = dirname(__file__)
parent_dir = abspath(join(curr_dir, pardir))
path.append(pardir)

from redis_controller import RedisQueue


class RedisQueueTests(TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.queue = RedisQueue()

    async def test_publish(self) -> None:
        task_id = await self.queue.publish("test", "testpayload")
        self.assertIsNotNone(task_id)

    async def test_get_result(self) -> None:
        task_id = await self.queue.publish("test", "payload")

        self.assertIsNotNone(await self.queue.result(task_id))


if __name__ == "__main__":
    main()

