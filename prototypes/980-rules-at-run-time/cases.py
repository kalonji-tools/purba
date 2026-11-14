"""What a tester imports: `case`, `raises`, `warns`, `handling`, `use`, `before_each`, and the bases of a stock. Prototype of #980, on the prototype of #979. Never merge.

A test is a plain function. `run.py` imports it with its last asserts rewritten as checks, and runs it as written.
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
E = TypeVar("E", bound=BaseException)
W = TypeVar("W", bound=Warning)


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


# ── what each case of a file runs first ─────────────────────────────────────────────

BEFORE: list[tuple[str, Callable[[], object]]] = []


def before_each(fn: Callable[[], object]) -> None:
    """Run `fn()` at the start of each case of this file, in the case's own process: `before_each(reset_logger)`.

    Nothing runs after: the case's process ends, and what `fn` changed in it ends too.
    """
    BEFORE.append((sys._getframe(1).f_code.co_filename, fn))


# ── an expected raise, an expected warning, code inside an `except` ─────────────────


class Missed(BaseException):
    """An expected raise or warning that did not come, or came with another message."""

    def __init__(self, form: str, text: str) -> None:
        super().__init__(text)
        self.form = form


SEEN: list[str] = []  # what each `raises`, `warns` and `handling` saw; the runner prints it as a check
OWNED = [0]  # above 0 while the form itself changes the warning filters: the runner's guard lets it (#979)
RAISED = [False]  # True once `raises` caught what it expects (#980)


class Caught(Generic[T]):
    """What `with raises(…) as caught:` caught: `caught.value`, for the checks after it."""

    def __init__(self) -> None:
        self._value: T | None = None

    @property
    def value(self) -> T:
        if self._value is None:
            raise LookupError("nothing was caught")
        return self._value


def _want(kind: type[BaseException], match: str | None) -> str:
    return f"{kind.__name__}" + (f" matching {match!r}" if match is not None else "")


@contextmanager
def raises(kind: type[E], match: str | None = None) -> Generator[Caught[E]]:
    """Expect the one statement inside to raise `kind`, with a message that `re.search(match, …)` finds."""
    caught: Caught[E] = Caught()
    try:
        yield caught
    except kind as error:
        if match is not None and not re.search(match, str(error)):
            raise Missed("raises", f"expected {_want(kind, match)}; it raised {type(error).__name__}: {error}") from None
        caught._value = error
        SEEN.append(f"raises {type(error).__name__}: {error}")
        RAISED[0] = True  # #980: a line of the act that runs after this is seen by the runner
        return
    raise Missed("raises", f"expected {_want(kind, match)}; the statement did not raise")


@contextmanager
def warns(kind: type[W], match: str | None = None) -> Generator[Caught[W]]:
    """Expect the one statement inside to warn `kind`. Any other warning goes on to fail the case."""
    caught: Caught[W] = Caught()
    OWNED[0] += 1
    catcher = warnings.catch_warnings(record=True)
    seen = catcher.__enter__()
    warnings.simplefilter("always")
    OWNED[0] -= 1
    try:
        yield caught
    finally:
        OWNED[0] += 1
        catcher.__exit__(None, None, None)
        OWNED[0] -= 1
    assert seen is not None
    hit =next((w for w in seen if issubclass(w.category, kind) and (match is None or re.search(match, str(w.message)))), None)
    for w in seen:
        if w is not hit:
            warnings.warn_explicit(w.message, w.category, w.filename, w.lineno)
    if hit is None:
        raise Missed("warns", f"expected {_want(kind, match)}; " + (", ".join(f"{w.category.__name__}: {w.message}" for w in seen) or "no warning"))
    caught._value = cast(W, hit.message)
    SEEN.append(f"warns {hit.category.__name__}: {hit.message}")


class handling(Generic[P]):
    """Expect the one statement inside to raise `kind`, and run `call(*args)` while that exception is live.

    For code that must run inside an `except`, such as `logger.exception`: a test holds no `try`.
    A class, not a generator, so the traceback that `call` sees holds the test's frame alone, as a `try` gives it.
    """

    def __init__(self, kind: type[BaseException], call: Callable[P, object], /, *args: P.args, **kwargs: P.kwargs) -> None:
        self.kind, self.call, self.args, self.kwargs = kind, call, args, kwargs

    def __enter__(self) -> None:
        return None

    def __exit__(self, _type: object, error: BaseException | None, _tb: object) -> bool:
        if error is None:
            name = getattr(self.call, "__name__", "the call")
            raise Missed("handling", f"expected {self.kind.__name__}, to run `{name}(…)` inside its `except`; the statement did not raise")
        if not isinstance(error, self.kind):
            return False
        self.call(*self.args, **self.kwargs)
        SEEN.append(f"handling {type(error).__name__}: {error}")
        return True


# ── fixtures and stocks ─────────────────────────────────────────────────────────────


class Use(Generic[T]):
    """A fixture: the runner calls `factory()` again for each case, inside the case's own process."""

    def __init__(self, factory: Callable[[], Any]) -> None:
        self.factory = factory

    def __repr__(self) -> str:
        return f"use({getattr(self.factory, '__name__', self.factory)})"


class Stock(ABC, Generic[T]):
    """An input whose state lives outside the process: a database, a file, a server.

    A fork cannot seal it. For the case, its fixture offers a sample (`Samples`) or a checkpoint (`Checkpoints`).
    For each check, it offers a fingerprint (`Fingerprints`) or a checkpoint (#980).
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


class Fingerprints(Stock[T]):
    """A stock whose state the runner can fingerprint. Before and after each check, the runner takes a fingerprint;
    a check whose reading changed the stock is refused (#980). Pure reads need no checkpoint and no copy."""

    @abstractmethod
    def fingerprint(self, value: T, /) -> object: ...


@overload
def use(factory: type[Stock[T]]) -> T: ...
@overload
def use(factory: Callable[[], T]) -> T: ...
def use(factory: Callable[[], Any]) -> Any:
    """A fixture as the default of an input: `db: Db = use(UsersDb)`. A type checker sees the value it gives."""
    return Use(factory)
