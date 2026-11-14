"""Probe of #979: the markers in a setup add no frame to its exit, so `logger.catch()` names the test as its caller."""

from loguru import logger

from cases import case


def test_catch_names_the_test():
    """✓ the record's function is this test, and its line is the `with` line."""
    lines: list[str] = []
    logger.remove()
    logger.add(lines.append, format="{function}:{line}")
    with logger.catch():
        int("x")  # the code raises ValueError at the last statement; catch() swallows it
    assert lines[0].splitlines()[0] == "test_catch_names_the_test:13"


catch_names_the_test = case(test_catch_names_the_test)
