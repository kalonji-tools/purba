"""ex5: code that runs inside an `except`, with no `try` in the test."""

import logging

from experiment import experiment, handling, use


class Lines(logging.Handler):
    def __init__(self) -> None:
        super().__init__()
        self.lines: list[str] = []

    def emit(self, record: logging.LogRecord) -> None:
        kind = record.exc_info[0] if record.exc_info else None
        self.lines.append(f"{record.getMessage()} | {getattr(kind, '__name__', None)}")


def fresh_log() -> Lines:
    lines = Lines()
    logging.getLogger("ex5").addHandler(lines)
    return lines


@experiment
def logged(error: Exception, lines: Lines = use(fresh_log)) -> list[str]:
    handling(error, logging.getLogger("ex5").exception, "failed")
    return lines.lines


division = logged(error=ZeroDivisionError("division by zero")).expect(product=["failed | ZeroDivisionError"])
