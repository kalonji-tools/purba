"""PROTOTYPE (#967). Form CALL: a case is a call of the act, named by its variable. Never merge.

    @experiment
    def slug_of(text: str) -> str:
        return slugify(text)

    plain = slug_of(text="Hi").expect(product="hi")

`slug_of(...)` takes the act's own parameters (a ParamSpec), so a type checker sees a misspelled or
mistyped input. The variable names the case. A step of a chain is that variable: `after(plain)`.
"""

import inspect
from types import ModuleType
from typing import Any, Callable, Generic, ParamSpec, TypeVar

from engine import REGISTRY, After, Case, Experiment, Raises, Warns, caller_line, use

__all__ = ["after", "experiment", "use"]
P = ParamSpec("P")
R = TypeVar("R")


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


class Exp(Generic[P, R]):
    def __init__(self, fn: Callable[P, R]) -> None:
        self.e = Experiment(fn.__name__, fn, inspect.getsourcefile(fn) or "", fn.__code__.co_firstlineno)
        self.steps: list[Step] = []
        REGISTRY.append(self.e)

    def __call__(self, *args: P.args, **kwargs: P.kwargs) -> Step:
        try:
            given = dict(inspect.signature(self.e.act).bind_partial(*args, **kwargs).arguments)
        except TypeError:  # a misspelled input: purba refuses this experiment, not the module
            given = dict(kwargs)
        s = Step(Case("", dict(given), {}, line=caller_line()))
        s.c.exp = self.e
        self.steps.append(s)
        return s

    def assay(self, fn: Callable[..., Any]) -> Callable[..., Any]:
        self.e.assays[fn.__name__] = fn
        return fn


def experiment(fn: Callable[P, R]) -> Exp[P, R]:
    return Exp(fn)


def after(step: Step, input: str | None = None) -> Any:
    return After(step.c, input, caller_line())


def collect(mod: ModuleType) -> None:
    names = {id(v): k for k, v in vars(mod).items() if isinstance(v, Step)}
    for exp in [v for v in vars(mod).values() if isinstance(v, Exp)]:
        for s in exp.steps:
            if id(s) in names:
                s.c.name = names[id(s)]
                exp.e.cases.append(s.c)
            else:
                exp.e.refused.append(f"the case at line {s.c.line} has no name: assign it to a variable")
