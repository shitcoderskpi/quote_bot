from asyncio import run
from logging import getLogger
from typing import Any
from aioredis import from_url

from config import LOG_LVL, logger_init

# TODO: get rid of aioredis, and find alternative
class RedisQueue:
    def __init__(self, redis_url: str, queue_name: str):
        self.redis_url = redis_url
        self.queue_name = queue_name
        self._logger = getLogger(self.__class__.__name__)
        self._logger.setLevel(LOG_LVL)
        logger_init(self._logger)
        self._pool = run(self._connect(redis_url))

    @staticmethod
    async def _connect(redis_url: str):
        return await from_url(redis_url)

    async def push(self, message: Any):
        await self._pool.lpush(self.queue_name, message)

    async def pop(self) -> Any:
        # BRPOP returns tuple (queue_name, message)
        res = await self._pool.brpop(self.queue_name)
        return res

    async def close(self):
        await self._pool.close()
