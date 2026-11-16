"""Stands in for the code under test in the probes. A function of the test module that it calls
back is the test's value, not the test's act."""

from __future__ import annotations

from collections.abc import Callable


def emit(sink: Callable[[str], object], message: str) -> None:
    sink(message)


def wrap(stream: object) -> bool:
    """Like colorama's `should_wrap`: it flushes the stream it is asked about."""
    stream.flush()  # type: ignore[attr-defined]
    return True


def parse(text: str) -> int:
    return int(text)
