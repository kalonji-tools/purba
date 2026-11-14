"""The two fixtures of loguru's tests/conftest.py that test_add_option_format.py uses (MIT, Delgan/loguru@28349be), as factories for use(…)."""

from __future__ import annotations

from typing import TYPE_CHECKING

import loguru
from loguru import _logger

if TYPE_CHECKING:
    from loguru import Logger


class Writer:
    """loguru's `writer` fixture: a sink that keeps each message it is given."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


def fresh_logger() -> Logger:
    """loguru's autouse `reset_logger` fixture, run before the act: no handler, default options, an empty context."""
    loguru.logger.remove()
    # loguru.pyi declares no Logger.__init__: the reset calls a private constructor, as the conftest does.
    loguru.logger.__init__(_logger.Core(), None, 0, False, False, False, False, True, [], {})  # type: ignore
    _logger.context.set({})
    return loguru.logger
