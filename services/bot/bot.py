from dataclasses import dataclass
from io import BytesIO
from json import dumps
from logging import getLogger
from re import compile
from aiogram import F, Bot, Dispatcher, html
from aiogram.client.default import DefaultBotProperties
from aiogram.enums import ParseMode
from aiogram.filters import Command, CommandStart
from aiogram.types import Message
from asyncio import run
from base64 import b64encode

from config import LOG_LVL, TOKEN, logger_init
from redis_controller import RedisQueue
from config import REDIS_HOST

logger = getLogger("bot")
logger.setLevel(LOG_LVL)
logger_init(logger)
if TOKEN is None:
    logger.critical("token is not set in environment")
    raise KeyError()

dp = Dispatcher()
bot = Bot(token=TOKEN, default=DefaultBotProperties(parse_mode=ParseMode.HTML))
redis = RedisQueue(REDIS_HOST)

@dataclass
class SerializableMessage:
    username: str
    user_status: str | None
    content: str
    image: BytesIO
    chat_id: int

    def to_json(self):
        return dumps({
            "chat-id": self.chat_id,
            "username": self.username,
            "user_status": self.user_status,
            "content": self.content,
            "image": b64encode(self.image.read()).decode("utf-8")
            }, ensure_ascii=False).encode("utf-8")


@dp.message(CommandStart())
async def command_start_handler(message: Message) -> None:
    await message.answer(f"Hello, {html.bold(message.from_user.full_name)}!")

async def get_member(msg: Message):
    logger.debug("Getting chat member ...")
    return await bot.get_chat_member(msg.chat.id, msg.from_user.id)

async def get_photos(msg: Message, limit: int = 1):
    logger.debug(f"Getting {msg.from_user.full_name}'s photos ...")
    return await bot.get_user_profile_photos(msg.from_user.id, limit=limit)

@dp.message(Command(compile("q(oute)?")), F.chat.type.in_({"group", "supergroup"}))
async def command_quote_handler(message: Message) -> None:
    reply = message.reply_to_message

    if reply is None:
        await message.answer("Please specify a message for quote generation")
        return

    member = await get_member(reply)

    avatar = BytesIO()
    photos = await get_photos(reply)
    if photos.total_count:
        logger.debug("Downloading image ...")
        await bot.download(photos.photos[0][0], avatar)
    else:
        logger.warning(f"User {reply.from_user.id} does not have any photos.")

    msg = SerializableMessage(reply.from_user.full_name, getattr(member, "custom_title", None), reply.text, avatar, reply.chat.id)
    await redis.enqueue("generate:jobs", msg.to_json())


async def bot_() -> None:
    await dp.start_polling(bot)
    await redis.close()


if __name__ == "__main__":
    logger.warning("Run bot through main.py.")
    run(bot_())