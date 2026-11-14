"""5: code that runs inside an `except`, with the exception that the code itself raises; checks on a caught exception."""

import logging

from app import check_age
from cases import case, handling, raises


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
    with handling(ZeroDivisionError, log.exception, "Error!"):
        _ = 1 / divisor
    assert seen.lines == [expected]


by_zero = case(test_logged_inside_except, divisor=0, expected="Error! | division by zero")
by_one  = case(test_logged_inside_except, divisor=1, expected="Error! | division by zero")  # fails: the statement did not raise


def test_too_young(age: int) -> None:
    with raises(ValueError) as caught:
        check_age(age)
    assert str(caught.value) == "too young"
    assert caught.value.args == ("too young",)


twelve = case(test_too_young, age=12)
