"""#1010: a logger fixture that later tests log through.

`enqueue=True` starts loguru's worker thread, which is still alive after the act: there the fork
tree must fall back to replay. `enqueue=False` starts no thread.
"""

from __future__ import annotations

from dataclasses import dataclass

from cases import case
from loguru import logger


@dataclass
class Logging:
    lines: list[str]
    handler: int


def test_configure(enqueue: bool) -> Logging:
    logger.remove()
    lines: list[str] = []
    handler = logger.add(lines.append, format="{message}", enqueue=enqueue)
    logger.info("ready")
    logger.complete()
    assert lines == ["ready\n"]
    return Logging(lines, handler)


queued = case(test_configure, enqueue=True)
direct = case(test_configure, enqueue=False)


def test_log(logging: Logging, message: str, lines: list[str]) -> None:
    logger.info(message)
    logger.complete()
    assert logging.lines == lines


queued_hello = case(test_log, logging=queued.product, message="hello", lines=["ready\n", "hello\n"])
queued_bye = case(test_log, logging=queued.product, message="bye", lines=["ready\n", "bye\n"])
direct_hello = case(test_log, logging=direct.product, message="hello", lines=["ready\n", "hello\n"])
direct_bye = case(test_log, logging=direct.product, message="bye", lines=["ready\n", "bye\n"])


def test_removed(logging: Logging) -> None:
    logger.remove(logging.handler)
    logger.info("gone")
    assert logging.lines == ["ready\n"]


queued_removed = case(test_removed, logging=queued.product)
direct_removed = case(test_removed, logging=direct.product)
