"""5: code that runs inside an `except`, with the exception that the code itself raises."""

import logging

from cases import case


class Seen(logging.Handler):
    def __init__(self) -> None:
        super().__init__()
        self.lines: list[str] = []

    def emit(self, record: logging.LogRecord) -> None:
        self.lines.append(f"{record.getMessage()} | {record.exc_info[1] if record.exc_info else None}")


def test_logged_inside_except(divisor: int, expected: str) -> None:
    seen = Seen()
    log = logging.getLogger("ex5")
    log.addHandler(seen)
    try:
        _ = 1 / divisor
    except ZeroDivisionError:
        log.exception("Error!")
    assert seen.lines == [expected]


by_zero = case(test_logged_inside_except, divisor=0, expected="Error! | division by zero")
by_one  = case(test_logged_inside_except, divisor=1, expected="Error! | division by zero")  # fails: the body did not raise
