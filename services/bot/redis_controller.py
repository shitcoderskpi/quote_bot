from logging import getLogger
from typing import Any, Optional
from redis.asyncio import Redis
from redis.typing import EncodableT

from config import LOG_LVL, logger_init

class RedisQueue:
    def __init__(self, host: str = "localhost", port: int = 6379, db: int = 0) -> None:
        self._redis = Redis(host=host, port=port, db=db)
        self._logger = getLogger(self.__class__.__name__)
        self._logger.setLevel(LOG_LVL)
        logger_init(self._logger)

    async def enqueue(self, name: str, data: EncodableT) -> int:
        self._logger.debug(f"Enqueuing data to queue {name}")
        return await self._redis.lpush(name, data)

    async def dequeue(self, name: str, timeout: int = 0) -> Optional[Any]:
        self._logger.debug(f"Dequeuing from queue {name}, timeout={timeout}")
        if timeout > 0:
            return await self._redis.brpop(name, timeout=timeout)
        else:
            return await self._redis.rpop(name)

    async def size(self, name: str):
        return await self._redis.llen(name)

