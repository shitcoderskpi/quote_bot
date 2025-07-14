from logging import getLogger
from asyncio import run
from prometheus_client import start_http_server

from config import LOG_LVL, PROMETHEUS_ADDR, logger_init, PROMETHEUS_PORT, LOG_FILE
from bot import bot_

def main():
    logger = getLogger("main")
    logger.setLevel(LOG_LVL)
    logger_init(logger, LOG_FILE)

    logger.info("Logging setup is done.")
    logger.debug(f"Starting Prometheus on 'http://{PROMETHEUS_ADDR}:{PROMETHEUS_PORT}/'")
    start_http_server(PROMETHEUS_PORT, addr=PROMETHEUS_ADDR)

    try:
        run(bot_())
    except Exception as e:
        logger.critical(e)


if __name__ == "__main__":
    main()
