"""PROTOTYPE (#967). Form STACKED: one `@case` per case, stacked above the act, as `parametrize` is. Never merge.

    @case("plain", input=dict(text="Hi"), assay=dict(product="hi"))
    def slug_of(text: str) -> str:
        return slugify(text)

A case names its inputs and its expected values in two dicts. An assay is a function under `@assay(experiment)`.
A step of a chain is named by text: `after(opened, "ann")`.
"""

import inspect
from typing import Any, Callable, TypeVar

from engine import REGISTRY, After, Case, Experiment, Raises, Warns, caller_line, raises, use, warns

__all__ = ["after", "assay", "case", "raises", "use", "warns"]
F = TypeVar("F", bound=Callable[..., Any])


def case(name: str, *, input: dict[str, Any] | None = None, assay: dict[str, Any] | None = None,
         raises: Raises | None = None, warns: Warns | None = None) -> Callable[[F], F]:
    line = caller_line()

    def deco(fn: F) -> F:
        e = fn.__dict__.get("__purba__")
        if e is None:
            e = Experiment(fn.__name__, fn, inspect.getsourcefile(fn) or "", fn.__code__.co_firstlineno)
            fn.__dict__["__purba__"] = e
            REGISTRY.append(e)
        e.cases.insert(0, Case(name, dict(input or {}), dict(assay or {}), raises, warns, line))  # decorators apply bottom up
        return fn

    return deco


def assay(of: Callable[..., Any]) -> Callable[[F], F]:
    def deco(fn: F) -> F:
        of.__dict__["__purba__"].assays[fn.__name__] = fn
        return fn

    return deco


def after(step: Callable[..., Any] | str, case: str, input: str | None = None) -> Any:
    return After((step, case), input, caller_line())
