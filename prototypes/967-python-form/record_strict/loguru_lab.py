"""The fixtures of the loguru port: loguru's `reset_logger` and `writer` (MIT, Delgan/loguru@28349be), as `use(...)` fixtures.

`fresh_logger` is `reset_logger` with no teardown: each case runs in its own forked process, so nothing leaks.
`sink` hands ONE writer per process, not one per build: a replayed step builds its own inputs, so a writer
built per case would not be the writer that the replayed `logger.add` attached. See LOGURU_NOTES.md.
`division_by_zero` is new: an act cannot `try`, so the raised exception arrives as an input.
"""

from typing import TYPE_CHECKING

import loguru
import loguru._logger

if TYPE_CHECKING:
    from loguru import Logger


def fresh_logger() -> "Logger":
    loguru.logger.remove()
    loguru.logger.__init__(loguru._logger.Core(), None, 0, False, False, False, False, True, [], {})  # type: ignore
    loguru._logger.context.set({})
    return loguru.logger


class Writer:
    """A sink that collects messages: loguru's `writer`, as a class so that a type checker sees `read`."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


SINK = Writer()  # empty in every case: the runner forks each case from a process that never logs


def sink() -> Writer:
    return SINK


def division_by_zero() -> ZeroDivisionError:
    try:
        1 / 0  # noqa: B018
    except ZeroDivisionError as exc:
        return exc
    raise AssertionError("1 / 0 did not raise")
