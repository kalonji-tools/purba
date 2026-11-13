"""PROTOTYPE (#967). Form BELOW: the act first, then one call for each case below it. Never merge.

    @experiment
    def slug_of(text: str) -> str:
        return slugify(text)

    slug_of.case("plain", text="Hi").expect(product="hi")

A case gives its inputs, then `.expect(...)` its assays, or `.raises(...)`. An assay is `@slug_of.assay`.
A case is a value, so a step of a chain is a variable: `ann = opened.case(...)`, then `after(ann)`.
"""

import inspect
from typing import Any, Callable

from engine import REGISTRY, After, Case, Experiment, Raises, Warns, caller_line, use

__all__ = ["after", "experiment", "use"]


class Step:
    def __init__(self, c: Case) -> None:
        self.c = c

    def expect(self, **values: Any) -> "Step":
        self.c.expect.update(values)
        return self

    def raises(self, type_: type[BaseException], message: str) -> "Step":
        self.c.raises = Raises(type_, message)
        return self

    def warns(self, type_: type[Warning], message: str) -> "Step":
        self.c.warns = Warns(type_, message)
        return self


class Exp:
    def __init__(self, fn: Callable[..., Any]) -> None:
        self.e = Experiment(fn.__name__, fn, inspect.getsourcefile(fn) or "", fn.__code__.co_firstlineno)
        REGISTRY.append(self.e)

    def case(self, name: str, /, **given: Any) -> Step:
        c = Case(name, given, {}, line=caller_line())
        c.exp = self.e
        self.e.cases.append(c)
        return Step(c)

    def assay(self, fn: Callable[..., Any]) -> Callable[..., Any]:
        self.e.assays[fn.__name__] = fn
        return fn


def experiment(fn: Callable[..., Any]) -> Exp:
    return Exp(fn)


def after(step: Step, input: str | None = None) -> Any:
    return After(step.c, input, caller_line())
