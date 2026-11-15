"""FILE REFUSED: `before_each` is no part of the form, and the refusal points to `@before`
(test_before.py). Says "`before_each` is gone from the form: put `@before(fn)` on each test" """

from __future__ import annotations

from cases import before_each  # type: ignore[attr-defined]
from registry import names, register, reset

before_each(reset)


def test_register() -> None:
    register("ann")
    assert names() == ["ann"]
