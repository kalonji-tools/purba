"""PROTOTYPE (#967). Form KLASSCALL: a class per experiment, and each case a typed call of its act. Never merge.

    class SlugOf(Experiment):
        def act(text: str) -> str:
            return slugify(text)

        plain = case(act, text="Hi").expect(product="hi")

`case(act, ...)` takes the act's own parameters (a ParamSpec), so a type checker sees a misspelled or
mistyped input. Expected values go in `.expect(...)`, apart from the inputs.
A step of a chain is an attribute: `after(Opened.ann)`.
"""

import inspect
from typing import Any, Callable, ParamSpec, TypeVar

from engine import REGISTRY, After, Case, Raises, Warns, caller_line, use
from engine import Experiment as _Exp

__all__ = ["Experiment", "after", "case", "use"]
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


def case(act: Callable[P, R], /, *args: P.args, **kwargs: P.kwargs) -> Step:
    try:
        given = dict(inspect.signature(act).bind_partial(*args, **kwargs).arguments)
    except TypeError:  # a misspelled input: purba refuses this experiment, not the module
        given = dict(kwargs)
    return Step(Case("", given, {}, line=caller_line()))


class Experiment:
    def __init_subclass__(cls) -> None:
        act = cls.__dict__["act"]
        e = _Exp(cls.__name__, act, inspect.getsourcefile(act) or "", inspect.getsourcelines(cls)[1])
        for k, v in cls.__dict__.items():
            if inspect.isfunction(v) and k != "act" and not k.startswith("_"):
                e.assays[k] = v
        for k, v in cls.__dict__.items():
            if isinstance(v, Step):
                v.c.name = k
                v.c.exp = e
                e.cases.append(v.c)
        REGISTRY.append(e)


def after(step: Step, input: str | None = None) -> Any:
    return After(step.c, input, caller_line())
