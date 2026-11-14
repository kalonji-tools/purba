"""What a tester imports: `experiment`, `use` and `handling`. Prototype 2 (#967). Never merge.

`run.py` reads what these build, and runs it.
"""

from __future__ import annotations

import sys
from collections.abc import Callable
from typing import Any, Generic, ParamSpec, TypeVar, cast

P = ParamSpec("P")
Q = ParamSpec("Q")
R = TypeVar("R")
T = TypeVar("T")


class _Unset:
    def __repr__(self) -> str:
        return "UNSET"


UNSET = _Unset()


class Use(Generic[T]):
    """A fixture: `run.py` calls `factory()` again for each case, inside the case's own process."""

    def __init__(self, factory: Callable[[], T]) -> None:
        self.factory = factory

    def __repr__(self) -> str:
        return f"use({getattr(self.factory, '__name__', self.factory)})"


def use(factory: Callable[[], T]) -> T:
    """Mark a default of the act as a fixture. A type checker sees the value that `factory` builds."""
    return cast(T, Use(factory))


class Step(Generic[T]):
    """The product of an earlier case, taken as an input: a step of a chain."""

    def __init__(self, case: Case[Any]) -> None:
        self.case = case

    def __repr__(self) -> str:
        return f"<product of line {self.case.line}>"


CREATED: list[Case[Any]] = []


class Case(Generic[R]):
    """One row of an experiment: its inputs, and what the tester expects."""

    def __init__(self, experiment: Experiment[Any, R], args: tuple[Any, ...], inputs: dict[str, Any], file: str, line: int) -> None:
        self.experiment = experiment
        self.args = args
        self.inputs = inputs
        self.file = file
        self.line = line
        self.expected: dict[str, object] = {}
        self.raised: tuple[type[BaseException], str | None] | None = None
        self.warned: list[tuple[type[Warning], str | None]] = []
        CREATED.append(self)

    def expect(self, product: R | _Unset = UNSET, **readings: object) -> Case[R]:
        """Expect the act's return value (`product`) and the reading of each named assay."""
        if not isinstance(product, _Unset):
            self.expected["product"] = product
        self.expected.update(readings)
        return self

    def raises(self, kind: type[BaseException], match: str | None = None) -> Case[R]:
        """Expect the act to raise `kind`, with a message that `re.search(match, …)` finds."""
        self.raised = (kind, match)
        return self

    def warns(self, kind: type[Warning], match: str | None = None) -> Case[R]:
        """Expect the act to warn. A warning that no case expects fails the case."""
        self.warned.append((kind, match))
        return self

    @property
    def product(self) -> R:
        """This case's product, as the input of a later case. The later case replays this one first."""
        return cast(R, Step(self))


class Experiment(Generic[P, R]):
    """An act, the assays that read what it made, and the cases that call it."""

    def __init__(self, act: Callable[P, R]) -> None:
        self.act = act
        self.name: str = getattr(act, "__name__", "act")
        self.assays: dict[str, Callable[[Any], Any]] = {}

    def __call__(self, *args: P.args, **kwargs: P.kwargs) -> Case[R]:
        caller = sys._getframe(1)
        return Case(self, args, dict(kwargs), caller.f_code.co_filename, caller.f_lineno)

    def assay(self, read: Callable[[Any], T]) -> Callable[[Any], T]:
        """An assay reads `product`, or one input of the act by its name, and returns a reading."""
        self.assays[getattr(read, "__name__", "assay")] = read
        return read


def experiment(act: Callable[P, R]) -> Experiment[P, R]:
    """Make `act` an experiment. Each call of the result is a case, typed by `act`'s parameters."""
    return Experiment(act)


def handling(exception: BaseException, call: Callable[Q, T], *args: Q.args, **kwargs: Q.kwargs) -> T:
    """Raise `exception`, and call `call(*args, **kwargs)` in its `except`, as code that handles it."""
    try:
        raise exception
    except BaseException:
        return call(*args, **kwargs)
