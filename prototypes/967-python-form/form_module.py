"""PROTOTYPE (#967). Form MODULE: the file is the experiment. Never merge.

    # slug_of.py
    def act(text: str) -> str:
        return slugify(text)

    plain = case(text="Hi", product="hi")

`act` is the act. Every other function that the file defines is an assay. Each case is a variable, flat keys.
A step of a chain in another experiment is imported: `from opened import ann`, then `after(ann)`.
"""

import inspect
import sys
from types import ModuleType
from typing import Any

from engine import REGISTRY, After, Case, Experiment, Raises, Warns, caller_line, raises, use, warns

__all__ = ["after", "case", "raises", "use", "warns"]


def case(*, raises: Raises | None = None, warns: Warns | None = None, **values: Any) -> Case:
    c = Case("", values, {}, raises, warns, caller_line())
    c.__dict__["module"] = sys._getframe(1).f_globals["__name__"]
    return c


def after(step: Case, input: str | None = None) -> Any:
    return After(step, input, caller_line())


def collect(mod: ModuleType) -> None:
    act = vars(mod)["act"]
    e = Experiment(mod.__name__, act, inspect.getsourcefile(act) or "", 1)
    names = set(inspect.signature(act).parameters)
    for k, v in vars(mod).items():
        if inspect.isfunction(v) and v.__module__ == mod.__name__ and k != "act" and not k.startswith("_"):
            e.assays[k] = v
    for k in names & set(e.assays):
        e.refused.append(f"`{k}` names both an input and an assay")
    for k, v in vars(mod).items():
        if isinstance(v, Case) and v.__dict__.get("module") == mod.__name__:
            v.name = k
            raw = v.given
            v.given = {x: y for x, y in raw.items() if x in names}
            v.expect = {x: y for x, y in raw.items() if x not in names}
            v.exp = e
            e.cases.append(v)
    REGISTRY.append(e)
