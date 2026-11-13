"""PROTOTYPE (#967). Form KLASS: one class per experiment. Never merge.

    class SlugOf(Experiment):
        def act(text: str) -> str:
            return slugify(text)

        plain = case(text="Hi", product="hi")

`act` is the act. Every other function in the class is an assay, named by its name.
Each case is a class attribute; its keys name an input or an assay, flat.
A step of a chain is an attribute: `after(Opened.ann)`.
"""

import inspect
from typing import Any

from engine import REGISTRY, After, Case, Raises, Warns, caller_line, raises, use, warns
from engine import Experiment as _Exp

__all__ = ["Experiment", "after", "case", "raises", "use", "warns"]


def case(*, raises: Raises | None = None, warns: Warns | None = None, **values: Any) -> Case:
    return Case("", values, {}, raises, warns, caller_line())


class Experiment:
    def __init_subclass__(cls) -> None:
        act = cls.__dict__["act"]
        e = _Exp(cls.__name__, act, inspect.getsourcefile(act) or "", inspect.getsourcelines(cls)[1])
        names = set(inspect.signature(act).parameters)
        for k, v in cls.__dict__.items():
            if inspect.isfunction(v) and k != "act" and not k.startswith("_"):
                e.assays[k] = v
        for k in names & set(e.assays):
            e.refused.append(f"`{k}` names both an input and an assay")
        for k, v in cls.__dict__.items():
            if isinstance(v, Case):
                v.name = k
                raw = v.given
                v.given = {x: y for x, y in raw.items() if x in names}
                v.expect = {x: y for x, y in raw.items() if x not in names}
                e.cases.append(v)
        REGISTRY.append(e)


def after(step: Case, input: str | None = None) -> Any:
    return After(step, input, caller_line())
