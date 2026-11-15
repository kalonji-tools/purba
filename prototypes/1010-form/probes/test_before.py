"""`@before(fn)` runs `fn()` at the start of each case of the test: after the earlier stages it
takes, before its fixtures and its act. Several run top to bottom. No other decorator is taken."""

from __future__ import annotations

from cases import before, case, use
from registry import names, register, reset


def add_a() -> None:
    register("a")


def add_b() -> None:
    register("b")


def boom() -> None:
    raise RuntimeError("no registry")


@before(reset)
@before(add_a)
@before(add_b)
def test_order() -> None:
    """PASS: reset, then a, then b, top to bottom, before the act. Says "['a', 'b', 'ann']" """
    register("ann")
    assert names() == ["a", "b", "ann"]


def ann_registered() -> str:
    register("ann")
    return "ann"


@before(reset)
def test_before_with_a_fixture(name: str = use(ann_registered)) -> None:
    """PASS: the reset runs before the fixture, so the fixture's "ann" is kept and "builtin" is
    gone. Says "['ann', 'bob']" """
    register("bob")
    assert names() == ["ann", "bob"]


@before(reset)
def test_ann_joins() -> str:
    """PASS: a stage that registers ann."""
    register("ann")
    assert names() == ["ann"]
    return "ann"


ann = case(test_ann_joins)


@before(reset)
def test_after_a_stage(who: str = use(ann)) -> None:
    """PASS: the earlier stage's act ran before this case started, so the reset empties it.
    Says "['bob']" """
    register("bob")
    assert names() == ["bob"]


@before(boom)
def test_before_that_raises() -> None:
    """FAIL: the reset raised, so the act never runs. Says "@before(boom) raised RuntimeError: no
    registry" """
    register("ann")
    assert names() == ["builtin", "ann"]


def tagged(fn):  # type: ignore[no-untyped-def]
    return fn


@tagged
def test_other_decorator() -> None:
    """REFUSED before the run. Says "`@tagged`: a test takes no decorator but the form's
    `@before(fn)`" """
    register("ann")
    assert names() == ["builtin", "ann"]
