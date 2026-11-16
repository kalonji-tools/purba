"""#1025, change 4: a raise names where it came from. Below the `at` line of the test, a second line
names the frame that raised, when it is not the test's own line."""

from __future__ import annotations

from lazy import gen, parse


def test_generator_raises() -> None:
    """FAIL: Says "the act raised ValueError: boom" Says "items = list(gen())" Says "raised inside
    the generator gen() at lazy.py:6" """
    items = list(gen())
    assert items == [1]


def test_function_raises() -> None:
    """FAIL: Says "the act raised ValueError: not a number: 'x'" Says "raised in parse() at
    lazy.py:11" """
    n = parse("x")
    assert n == 1


def test_own_line_raises() -> None:
    """FAIL: the raise is on the test's own line, so no second line. Says "the act raised
    ZeroDivisionError" """
    n = parse("1") // 0
    assert n == 1


def test_reading_raises() -> None:
    """FAIL: a reading that raises names the frame too. Says "reading it raised ValueError" Says
    "raised in parse() at lazy.py:11" """
    text = "x"
    assert parse(text) == 1
