import logging
from os import getenv
from logging import DEBUG, Logger, StreamHandler, FileHandler
from formatter import ColoredFormatter

LOG_FMT = "[%(asctime)s %(levelname)s] %(name)s: %(message)s"
LOG_LVL = DEBUG
LOG_PATH = getenv("LOG_PATH")
LOG_FILE = f"{LOG_PATH}bot-service.log"

TOKEN = getenv("BOT_TOKEN")

REDIS_HOST = getenv("REDIS_HOST")

PROMETHEUS_PORT = 8000
PROMETHEUS_ADDR = "0.0.0.0"


def logger_init(logger: Logger,
                filename: str | None = None,
                stream = None,
                level = logging.DEBUG):
    formatter = ColoredFormatter(LOG_FMT)
    if filename is None:
        handler = StreamHandler(stream)
    else:
        handler = FileHandler(filename)

    handler.setFormatter(formatter)
    handler.setLevel(level)
    logger.addHandler(handler)

def env_check(logger: Logger):
    logger.info("Starting environment check...")

    if TOKEN is None:
        logger.critical("token is not set in environment")
        raise KeyError()

    if REDIS_HOST is None:
        logger.critical("redis host is not set in environment")
        raise KeyError()

    logger.info("Environment check passed.")
