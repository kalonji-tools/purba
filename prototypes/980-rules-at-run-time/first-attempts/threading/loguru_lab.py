"""First attempt: the fixtures test_threading.py uses, taken from loguru's conftest.py. MIT, Delgan/loguru@28349be."""

import io
import sys

import loguru


def reset_logger() -> None:
    loguru.logger.remove()
    loguru.logger.__init__(loguru._logger.Core(), None, 0, False, False, False, False, True, [], {})  # type: ignore[attr-defined]
    loguru._logger.context.set({})  # type: ignore[attr-defined]


class Writer:
    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)

    def clear(self) -> None:
        self.written.clear()


class Capsys:
    """Stands in for pytest's `capsys`: swaps sys.stdout and sys.stderr for buffers."""

    def __init__(self) -> None:
        self.out = io.StringIO()
        self.err = io.StringIO()
        sys.stdout = self.out
        sys.stderr = self.err

    def readouterr(self) -> tuple[str, str]:
        return self.out.getvalue(), self.err.getvalue()
