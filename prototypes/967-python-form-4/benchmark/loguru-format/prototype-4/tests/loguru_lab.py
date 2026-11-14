"""The writer and the logger reset from loguru's tests/conftest.py (MIT, Delgan/loguru@28349be), ported to this form."""

import loguru
import loguru._logger


class Writer:
    """A sink that keeps each message loguru gives it. It has no `write` method, so loguru calls it as a function."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


def reset_logger() -> None:
    """Remove every handler, the stderr one too, and give the logger a new core and an empty context."""
    loguru.logger.remove()
    loguru.logger.__init__(loguru._logger.Core(), None, 0, False, False, False, False, True, [], {})  # pyright: ignore[reportCallIssue]  # ty: ignore[too-many-positional-arguments]
    loguru._logger.context.set({})
