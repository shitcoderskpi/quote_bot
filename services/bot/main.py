from logging import getLogger
from asyncio import run
from prometheus_client import start_http_server

from config import LOG_LVL, PROMETHEUS_ADDR, logger_init, env_check, PROMETHEUS_PORT
from bot import bot_

def main():
    global logger
    logger.info("Logging setup is done.")
    logger.debug(f"Starting Prometheus on 'http://{PROMETHEUS_ADDR}:{PROMETHEUS_PORT}/'")
    start_http_server(PROMETHEUS_PORT, addr=PROMETHEUS_ADDR)
    run(bot_())


if __name__ == "__main__":
    logger = getLogger("main")
    logger.setLevel(LOG_LVL)
    logger_init(logger)
    env_check(logger)
    main()
