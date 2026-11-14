"""The fixtures test_threading.py uses, from loguru's tests/conftest.py. MIT, Delgan/loguru@28349be; see LICENSE.loguru.

`reset_logger` goes to `before_each`; `Writer` and `Capsys` go to `use(...)`.
"""

import io
import sys
from typing import Any, cast

import loguru
import loguru._logger


def reset_logger() -> None:
    """loguru's autouse `reset_logger`, the reset before the case. Nothing runs after: the case's process ends."""
    loguru.logger.remove()
    cast(Any, loguru.logger).__init__(loguru._logger.Core(), None, 0, False, False, False, False, True, [], {})
    loguru._logger.context.set({})


class Writer:
    """loguru's `writer` fixture: a sink that keeps each message."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)

    def clear(self) -> None:
        self.written.clear()


class Capsys:
    """Stands in for pytest's `capsys`: building it swaps sys.stdout and sys.stderr for buffers.

    Build it only through `use(Capsys)`: built at module level, it swaps the runner's own streams.
    """

    def __init__(self) -> None:
        self.out = io.StringIO()
        self.err = io.StringIO()
        sys.stdout = self.out
        sys.stderr = self.err

    def readouterr(self) -> tuple[str, str]:
        return self.out.getvalue(), self.err.getvalue()
