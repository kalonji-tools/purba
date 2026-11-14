"""What a tester imports: `case`, `raises`, `warns`, `use`, and the bases of a stock. Prototype 3 (#967). Never merge.

A test is a plain function. `run.py` reads its source, and runs it as an act followed by its checks.
"""

from __future__ import annotations

import re
import sys
import warnings
from abc import ABC, abstractmethod
from collections.abc import Callable, Generator
from contextlib import contextmanager
from typing import Any, Generic, ParamSpec, TypeVar, cast, overload

P = ParamSpec("P")
R = TypeVar("R")
T = TypeVar("T")
M = TypeVar("M")


# ── cases ───────────────────────────────────────────────────────────────────────────

CREATED: list[Case[Any]] = []


class Case(Generic[R]):
    """One set of inputs for a test. Its name is the variable it is bound to."""

    def __init__(self, test: Callable[..., R], args: tuple[Any, ...], inputs: dict[str, Any], file: str, line: int) -> None:
        self.test = test
        self.args = args
        self.inputs = inputs
        self.file = file
        self.line = line
        CREATED.append(self)

    @property
    def product(self) -> R:
        """What this case's test returns, as the input of a later case. The later case runs this one again first."""
        return cast(R, Step(self))


class Step(Generic[T]):
    """The product of an earlier case, taken as an input: a step of a chain."""

    def __init__(self, case: Case[Any]) -> None:
        self.case = case

    def __repr__(self) -> str:
        return f"<product of the case at line {self.case.line}>"


def case(test: Callable[P, R], /, *args: P.args, **kwargs: P.kwargs) -> Case[R]:
    """A case of `test`: name each input as `test`'s parameters do. A type checker reads the call against `test`."""
    caller = sys._getframe(1)
    return Case(test, args, dict(kwargs), caller.f_code.co_filename, caller.f_lineno)


# ── an expected raise, an expected warning ──────────────────────────────────────────


class Missed(Exception):
    """An expected raise or warning that did not come, or came with another message."""


SEEN: list[str] = []  # what each `raises` and `warns` saw; the runner prints it as a check


@contextmanager
def raises(kind: type[BaseException], match: str | None = None) -> Generator[None]:
    """Expect the one statement inside to raise `kind`, with a message that `re.search(match, …)` finds."""
    want = f"{kind.__name__}" + (f" matching {match!r}" if match is not None else "")
    try:
        yield
    except kind as error:
        if match is not None and not re.search(match, str(error)):
            raise Missed(f"expected {want}; it raised {type(error).__name__}: {error}") from None
        SEEN.append(f"raises {type(error).__name__}: {error}")
        return
    raise Missed(f"expected {want}; the statement did not raise")


@contextmanager
def warns(kind: type[Warning], match: str | None = None) -> Generator[None]:
    """Expect the one statement inside to warn `kind`. Any other warning goes on to fail the case."""
    want = f"{kind.__name__}" + (f" matching {match!r}" if match is not None else "")
    with warnings.catch_warnings(record=True) as seen:
        warnings.simplefilter("always")
        yield
    hit = next((w for w in seen if issubclass(w.category, kind) and (match is None or re.search(match, str(w.message)))), None)
    for w in seen:
        if w is not hit:
            warnings.warn_explicit(w.message, w.category, w.filename, w.lineno)
    if hit is None:
        raise Missed(f"expected {want}; " + (", ".join(f"{w.category.__name__}: {w.message}" for w in seen) or "no warning"))
    SEEN.append(f"warns {hit.category.__name__}: {hit.message}")


# ── fixtures and stocks ─────────────────────────────────────────────────────────────


class Use(Generic[T]):
    """A fixture: the runner calls `factory()` again for each case, inside the case's own process."""

    def __init__(self, factory: Callable[[], Any]) -> None:
        self.factory = factory

    def __repr__(self) -> str:
        return f"use({getattr(self.factory, '__name__', self.factory)})"


class Stock(ABC, Generic[T]):
    """An input whose state lives outside the process: a database, a file, a server.

    A fork cannot seal it, so its fixture offers a sample (`Samples`), a checkpoint (`Checkpoints`), or both.
    """


class Samples(Stock[T]):
    """A stock that draws a sample for one case, and discards it after the case."""

    @abstractmethod
    def sample(self) -> T: ...

    @abstractmethod
    def discard(self, value: T, /) -> None: ...


class Checkpoints(Stock[T], Generic[T, M]):
    """A stock that opens one value, sets a checkpoint in its state, and restores it."""

    @abstractmethod
    def open(self) -> T: ...

    @abstractmethod
    def checkpoint(self, value: T, /) -> M: ...

    @abstractmethod
    def restore(self, value: T, mark: M, /) -> None: ...


@overload
def use(factory: type[Stock[T]]) -> T: ...
@overload
def use(factory: Callable[[], T]) -> T: ...
def use(factory: Callable[[], Any]) -> Any:
    """A fixture as the default of an input: `db: Db = use(UsersDb)`. A type checker sees the value it gives."""
    return Use(factory)
