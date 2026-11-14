"""Helpers for the loguru port: the two fixtures of loguru's tests/conftest.py, and one format function.

The fixtures come from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be; see LICENSE.loguru).
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import loguru
import loguru._logger

if TYPE_CHECKING:
    from loguru import Logger, Record


class Writer:
    """A sink that keeps each message it is given: loguru's `writer` fixture, as a class."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


def fresh_logger() -> Logger:
    """loguru's autouse `reset_logger`, as a fixture that gives the logger: no handler, no extra, an empty context.

    The original also resets after each test. Here each case runs in a fork of its own, so nothing is left to reset.
    """
    loguru.logger.remove()
    # loguru's stub declares no Logger.__init__; the conftest calls the private one.
    loguru.logger.__init__(loguru._logger.Core(), None, 0, False, False, False, False, True, [], {})  # pyright: ignore[reportCallIssue]  # ty: ignore[too-many-positional-arguments]
    loguru._logger.context.set({})
    return loguru.logger


def progressive_formatter(record: Record) -> str:
    """The format of test_progressive_format: no newline while the record's extra holds "noend".

    It lives here because a function defined inside a test may hold no `if`.
    """
    fmt = "[{level.name}] {message}"
    if "noend" not in record["extra"]:
        fmt += "\n"
    return fmt
