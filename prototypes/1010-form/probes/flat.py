"""Code under test that sits in the tests' folder, as in a flat layout. It is not a test module, so
the watch does not cover it."""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import contextmanager


class Box:
    def __init__(self, v: int | None) -> None:
        if v is None:
            v = 0
        self.v = v


@contextmanager
def opened_box(v: int | None) -> Iterator[Box]:
    if v is None:
        v = 0
    yield Box(v)
