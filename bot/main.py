from logging import getLogger

from config import LOG_LVL, logger_init, env_check
from redis_controller import RedisController

def main():
    global logger
    logger.info("Logging setup is done.")

def redis_check():
    redis = RedisController()
    redis.publish("test", {"msg": "Hello world"})


if __name__ == "__main__":
    logger = getLogger("main")
    logger.setLevel(LOG_LVL)
    logger_init(logger)
    env_check(logger)
    main()

