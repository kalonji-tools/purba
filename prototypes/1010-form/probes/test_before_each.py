"""FILE REFUSED: `before_each` is no part of the form; a test names its reset as a fixture
(test_reset.py). Says "cannot import name 'before_each'" """

from __future__ import annotations

from cases import before_each  # type: ignore[attr-defined]
from registry import names, register, reset

before_each(reset)


def test_register() -> None:
    register("ann")
    assert names() == ["ann"]
