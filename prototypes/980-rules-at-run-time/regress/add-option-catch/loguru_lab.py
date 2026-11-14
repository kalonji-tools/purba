"""Fixtures for the add-option-catch port, from loguru's conftest.py.

loguru is MIT, Delgan/loguru@28349be; see LICENSE.loguru.
"""

import contextlib
import io
import sys
import threading
import traceback
from collections.abc import Iterator

import loguru
import loguru._logger


def reset_logger() -> None:
    """The autouse fixture's reset, run before each case."""
    loguru.logger.remove()
    # loguru's stub gives Logger no __init__ arguments; the reset passes the ones its Core needs.
    loguru.logger.__init__(
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}  # pyright: ignore[reportCallIssue]  # ty: ignore[too-many-positional-arguments]
    )
    loguru._logger.context.set({})


class Writer:
    """The `writer` fixture: a sink that keeps every message."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)

    def clear(self) -> None:
        self.written.clear()


class Capture:
    """Stands in for pytest's `capsys`: swaps sys.stdout and sys.stderr."""

    def __init__(self) -> None:
        self.out = io.StringIO()
        self.err = io.StringIO()
        sys.stdout = self.out
        sys.stderr = self.err

    def readouterr(self) -> tuple[str, str]:
        out, err = self.out.getvalue(), self.err.getvalue()
        self.out.seek(0)
        self.out.truncate()
        self.err.seek(0)
        self.err.truncate()
        return out, err


def _print_thread_exception(args: threading.ExceptHookArgs) -> None:
    print("Exception in thread:", file=sys.stderr, flush=True)
    traceback.print_exception(
        args.exc_type, args.exc_value, args.exc_traceback, file=sys.stderr
    )


@contextlib.contextmanager
def default_threading_excepthook() -> Iterator[None]:
    """The conftest's `default_threading_excepthook()`: swaps threading.excepthook for the statements inside, and puts it back."""
    old = threading.excepthook
    threading.excepthook = _print_thread_exception
    try:
        yield
    finally:
        threading.excepthook = old
