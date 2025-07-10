from logging import getLogger
from redis import ConnectionPool, Redis
from json import dumps

from config import LOG_LVL, logger_init

# TODO: migrate to aioredis...
class RedisController:
    def __init__(self, pool: ConnectionPool = ConnectionPool(host = "localhost", port=6379, db=0)) -> None:
        self.client = Redis(connection_pool=pool)
        self.logger = getLogger(self.__class__.__name__)
        self.logger.setLevel(LOG_LVL)
        logger_init(self.logger)
        self.logger.debug(f"Connected to redis {pool}")

    def publish(self, channel: str, msg):
        msg_serialized = dumps(msg)
        self.logger.debug(f"Publishing message {msg_serialized}")
        self.client.publish(channel, msg_serialized)
