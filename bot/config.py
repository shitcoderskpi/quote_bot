from logging import DEBUG, Logger, StreamHandler, FileHandler
from sys import prefix, base_prefix
from formatter import ColoredFormatter

LOG_FMT = "[%(asctime)s %(levelname)s] %(name)s: %(message)s"
LOG_LVL = DEBUG


def logger_init(logger: Logger, filename: str | None = None, stream = None):
    formatter = ColoredFormatter(LOG_FMT)
    if filename is None:
        handler = StreamHandler(stream)
    else:
        handler = FileHandler(filename)

    handler.setFormatter(formatter)
    logger.addHandler(handler)

def env_check(logger: Logger):
    if prefix == base_prefix:
        logger.warning("You are not running bot in virtual environment, consider activating it.")

