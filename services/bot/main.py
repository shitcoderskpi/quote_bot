from logging import getLogger
from asyncio import run

from config import LOG_LVL, logger_init, env_check
from bot import bot_

def main():
    global logger
    logger.info("Logging setup is done.")
    run(bot_())


if __name__ == "__main__":
    logger = getLogger("main")
    logger.setLevel(LOG_LVL)
    logger_init(logger)
    env_check(logger)
    main()