from logging import getLogger
from typing import Any
from redis.asyncio import Redis
from redis.typing import EncodableT

from config import LOG_LVL, logger_init

class RedisQueue:
    def __init__(self, host: str = "localhost", port: int = 6379, db: int = 0) -> None:
        self._redis = Redis(host=host, port=port, db=db)
        self._logger = getLogger(self.__class__.__name__)
        self._logger.setLevel(LOG_LVL)
        logger_init(self._logger)
        self._closed = False

    async def enqueue(self, name: str, data: EncodableT) -> int:
        self._logger.debug(f"Enqueuing data to queue {name}")
        return await self._redis.lpush(name, data)

    async def dequeue(self, name: str, timeout: int | float = 0) -> Any | None:
        self._logger.debug(f"Dequeuing from queue {name}, timeout={timeout} s")
        return await self._redis.brpop(name, timeout=timeout)

    async def size(self, name: str):
        return await self._redis.llen(name)

    async def delete(self, name: str):
        self._logger.debug(f"Deleting queue {name}")
        await self._redis.delete(name)

    async def close(self):
        if not self._closed:
            self._logger.debug("Closing queue connection")
            self._closed = True
            await self._redis.aclose()
        else:
            self._logger.warning("Attempt to close already closed queue")

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()

