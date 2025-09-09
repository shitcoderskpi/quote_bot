import logging
from typing import Any, Final, Mapping, Literal
from logging import Formatter, LogRecord
from dataclasses import dataclass

@dataclass
class Colors:
    critical: str = "\x1b[41m"
    error: str = "\x1b[31m"
    warn: str = "\x1b[33m"
    info: str = "\x1b[39m"
    debug: str = "\x1b[39m"
    reset: Final[str] = "\x1b[0m"

    def __getitem__(self, key):
        match key:
            case logging.CRITICAL:
                return self.critical
            case logging.ERROR:
                return self.error
            case logging.WARN:
                return self.warn
            case logging.INFO:
                return self.info
            case logging.DEBUG:
                return self.debug
            case _:
                raise KeyError(f"Key {key} was not found")


class ColoredFormatter(Formatter):
    def __init__(self, fmt: str | None = None, datefmt: str | None = None, style: Literal["%", "$", "{"] = "%", validate: bool = True, *, defaults: Mapping[str, Any] | None = None, colors: Colors = Colors()) -> None:
        super().__init__(fmt, datefmt, style, validate, defaults=defaults)
        self.colors = colors

    def _color(self, record: str, level) -> str:
        return self.colors[level] + record + self.colors.reset

    def format(self, record: LogRecord) -> str:
        return self._color(super().format(record), record.levelno)

