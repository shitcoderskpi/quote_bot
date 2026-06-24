from template_generator import create_template
from asyncio import sleep, run
from logging import getLogger
from config import LOG_LVL, PROMETHEUS_ADDR, logger_init, PROMETHEUS_PORT, LOG_FILE

async def main_loop():
    logger = getLogger("template_engine")
    logger_init(logger, filename=LOG_FILE)
    logger_init(logger, level=LOG_LVL)

    while True:
        try:
            await create_template()
        except Exception as e:
            logger.exception(f"Error in create_template: {e}")
        await sleep(0.1)


if __name__ == '__main__':
    run(main_loop())